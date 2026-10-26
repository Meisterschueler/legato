import 'dart:async';
import 'dart:ui';

import 'package:flutter/material.dart';
import 'package:legato/src/engine/engine_client.dart';
import 'package:legato/src/l10n/generated/app_localizations.dart';

/// Values kept per graph.
const _points = 200;

/// Tempo in quarter notes per minute from two positions; `null` across a
/// repeat or within one onset.
double? tempo((double, double) from, (double, double) to) {
  final (t0, beat0) = from;
  final (t1, beat1) = to;
  if (beat1 <= beat0 || t1 <= t0) return null;
  return (beat1 - beat0) / (t1 - t0) * 60;
}

/// Inner state of the engine while playing: recognized piece, key and
/// position, loudness and tempo. Shows what arrives while the view is open.
class DebugScreen extends StatefulWidget {
  const DebugScreen({super.key, required this.engine});

  final EngineClient engine;

  @override
  State<DebugScreen> createState() => _DebugScreenState();
}

class _DebugScreenState extends State<DebugScreen> {
  StreamSubscription<Map<String, dynamic>>? _events;
  Map<String, String> _titles = const {};
  Map<String, dynamic>? _recognized;
  Map<String, dynamic>? _position;
  String? _ended;

  /// Driver time (s) and beat of the first position at the current onset.
  (double, double)? _last;
  final _velocities = <double>[];
  final _tempos = <double>[];

  @override
  void initState() {
    super.initState();
    _events = widget.engine.events().listen(_handle);
    widget.engine.repertoire().then((pieces) {
      if (mounted) {
        setState(
          () => _titles = {
            for (final p in pieces) p['id'] as String: p['title'] as String,
          },
        );
      }
    });
  }

  @override
  void dispose() {
    _events?.cancel();
    super.dispose();
  }

  void _handle(Map<String, dynamic> event) {
    switch (event['type']) {
      case 'PerformanceRecognized':
        _recognized = event;
        _position = null;
        _ended = null;
        _last = null;
      case 'Position':
        _position = event;
        final now = (
          (event['driver_time_us'] as num) / 1e6,
          (event['beat'] as num).toDouble(),
        );
        if (_last?.$2 == now.$2) break;
        if (_last case final last?) {
          if (tempo(last, now) case final t?) _add(_tempos, t);
        }
        _last = now;
      case 'Tone':
        _add(_velocities, (event['velocity'] as num).toDouble());
      case 'PerformanceEnded':
        _ended = [event['outcome'], ?event['reason']].join(' ');
      default:
        return;
    }
    setState(() {});
  }

  void _add(List<double> values, double value) {
    values.add(value);
    if (values.length > _points) values.removeAt(0);
  }

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context);
    final recognized = _recognized;
    final position = _position;
    final keys = l10n.keyNames.split(',');
    final transposition = recognized?['transposition'] as int?;
    final piece = recognized?['piece'] as String?;
    return Scaffold(
      appBar: AppBar(title: Text(l10n.debugView)),
      body: ListView(
        padding: const EdgeInsets.all(16),
        children: [
          _Row(
            l10n.debugPiece,
            piece == null
                ? '–'
                : '${_titles[piece] ?? piece} ($piece), '
                      '${recognized!['variation']}',
          ),
          _Row(
            l10n.debugKey,
            transposition == null
                ? '–'
                : '${keys[transposition % 12]} (+$transposition)',
          ),
          _Row(
            l10n.debugPosition,
            position == null
                ? '–'
                : '${(position['beat'] as num).toStringAsFixed(2)}  '
                      '${(position['note_ids'] as List).join(' ')}',
          ),
          if (_ended case final ended?) _Row('PerformanceEnded', ended),
          const SizedBox(height: 16),
          _Chart(l10n.debugVelocity, _velocities, min: 0, max: 127),
          const SizedBox(height: 16),
          _Chart(l10n.debugTempo, _tempos, min: 0),
        ],
      ),
    );
  }
}

class _Row extends StatelessWidget {
  const _Row(this.label, this.value);

  final String label;
  final String value;

  @override
  Widget build(BuildContext context) => Padding(
    padding: const EdgeInsets.symmetric(vertical: 4),
    child: Text.rich(
      TextSpan(
        children: [
          TextSpan(
            text: '$label: ',
            style: const TextStyle(fontWeight: FontWeight.bold),
          ),
          TextSpan(text: value),
        ],
      ),
    ),
  );
}

/// Line graph of [values] between [min] and [max] (default: the largest
/// value).
class _Chart extends StatelessWidget {
  const _Chart(this.label, this.values, {required this.min, this.max});

  final String label;
  final List<double> values;
  final double min;
  final double? max;

  @override
  Widget build(BuildContext context) {
    final top = max ?? values.fold<double>(min + 1, (a, b) => a > b ? a : b);
    final last = values.isEmpty ? '–' : values.last.toStringAsFixed(0);
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text('$label: $last  (max ${top.toStringAsFixed(0)})'),
        const SizedBox(height: 4),
        SizedBox(
          height: 150,
          width: double.infinity,
          child: CustomPaint(
            painter: _LinePainter(
              [...values],
              min,
              top,
              Theme.of(context).colorScheme.primary,
            ),
          ),
        ),
      ],
    );
  }
}

class _LinePainter extends CustomPainter {
  _LinePainter(this.values, this.min, this.max, this.color);

  final List<double> values;
  final double min;
  final double max;
  final Color color;

  @override
  void paint(Canvas canvas, Size size) {
    canvas.drawRect(
      Offset.zero & size,
      Paint()
        ..style = PaintingStyle.stroke
        ..color = color.withValues(alpha: 0.3),
    );
    if (values.length < 2) return;
    final dx = size.width / (_points - 1);
    final points = [
      for (var i = 0; i < values.length; i++)
        Offset(
          i * dx,
          size.height * (1 - ((values[i] - min) / (max - min)).clamp(0.0, 1.0)),
        ),
    ];
    canvas.drawPoints(
      PointMode.polygon,
      points,
      Paint()
        ..color = color
        ..strokeWidth = 2,
    );
  }

  @override
  bool shouldRepaint(_LinePainter old) => true;
}
