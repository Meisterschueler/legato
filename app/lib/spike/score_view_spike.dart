import 'dart:async';
import 'dart:convert';
import 'dart:math';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:webview_flutter/webview_flutter.dart';

/// Spike S2 (research.md R7): score-view-web in the WebView with synthetic
/// positions at 16 notes per second; measures frame rate and message latency.
class ScoreViewSpike extends StatefulWidget {
  const ScoreViewSpike({super.key});

  @override
  State<ScoreViewSpike> createState() => _ScoreViewSpikeState();
}

class _ScoreViewSpikeState extends State<ScoreViewSpike> {
  static const _notesPerSecond = 16;

  final _clock = Stopwatch()..start();
  late final WebViewController _controller;
  List<List<String>> _onsets = [];
  Timer? _timer;
  int _index = 0;
  final List<double> _latenciesMs = [];
  final List<double> _fps = [];
  double _maxFrameMs = 0;
  String _status = 'Lade Notenblatt …';

  @override
  void initState() {
    super.initState();
    _controller = WebViewController()
      ..setJavaScriptMode(JavaScriptMode.unrestricted)
      ..addJavaScriptChannel('LegatoHost', onMessageReceived: _onMessage)
      ..setNavigationDelegate(NavigationDelegate(onPageFinished: (_) => _load()))
      ..loadFlutterAsset('assets/score-view-web/index.html');
  }

  @override
  void dispose() {
    _timer?.cancel();
    super.dispose();
  }

  Future<void> _load() async {
    final musicxml = await rootBundle.loadString('assets/pieces/hanon-01.musicxml');
    await _send({'type': 'load', 'musicxml': musicxml});
  }

  Future<void> _send(Map<String, Object?> message) {
    return _controller.runJavaScript('legatoReceive(${jsonEncode(jsonEncode(message))})');
  }

  void _onMessage(JavaScriptMessage message) {
    final data = jsonDecode(message.message) as Map<String, dynamic>;
    switch (data['type']) {
      case 'ready':
        _onsets = _groupByOnset((data['note_ids'] as List).cast<String>());
        setState(() => _status = '${_onsets.length} Anschläge geladen');
      case 'frame_shown':
        final sent = (data['driver_time_us'] as num).toInt();
        _latenciesMs.add((_clock.elapsedMicroseconds - sent) / 1000);
      case 'frame_stats':
        if (_timer != null) {
          _fps.add((data['fps'] as num).toDouble());
          _maxFrameMs = max(_maxFrameMs, (data['max_frame_ms'] as num).toDouble());
          setState(() {});
        }
      case 'error':
        setState(() => _status = 'Fehler: ${data['message']}');
    }
  }

  /// Note ids are `measure/staff/onset/pitch`; both hands at one onset form one position.
  static List<List<String>> _groupByOnset(List<String> ids) {
    final groups = <String, List<String>>{};
    for (final id in ids) {
      final parts = id.split('/');
      groups.putIfAbsent('${parts[0]}/${parts[2]}', () => []).add(id);
    }
    return groups.values.toList();
  }

  void _toggle() {
    if (_timer != null) {
      _timer!.cancel();
      setState(() => _timer = null);
      return;
    }
    _latenciesMs.clear();
    _fps.clear();
    _maxFrameMs = 0;
    _index = 0;
    _send({'type': 'reset'});
    setState(() {
      _timer = Timer.periodic(
        const Duration(microseconds: 1000000 ~/ _notesPerSecond),
        (_) => _step(),
      );
    });
  }

  void _step() {
    if (_onsets.isEmpty) return;
    _send({
      'type': 'position',
      'note_ids': _onsets[_index % _onsets.length],
      'driver_time_us': _clock.elapsedMicroseconds,
    });
    _index++;
  }

  String get _summary {
    final latency = [..._latenciesMs]..sort();
    String ms(double v) => v.toStringAsFixed(1);
    final mean = latency.isEmpty ? 0.0 : latency.reduce((a, b) => a + b) / latency.length;
    final p95 = latency.isEmpty ? 0.0 : latency[min(latency.length - 1, (latency.length * 0.95).floor())];
    final fps = _fps.isEmpty ? 0.0 : _fps.reduce((a, b) => a + b) / _fps.length;
    final minFps = _fps.isEmpty ? 0.0 : _fps.reduce(min);
    return 'Positionen: ${latency.length}\n'
        'Latenz Dart → Bild (ms): Mittel ${ms(mean)}, p95 ${ms(p95)}, '
        'max ${ms(latency.isEmpty ? 0 : latency.last)}\n'
        'Bildrate (fps): Mittel ${ms(fps)}, min ${ms(minFps)}; längstes Bild ${ms(_maxFrameMs)} ms';
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('Spike S2'),
        actions: [
          IconButton(
            tooltip: 'Messwerte kopieren',
            icon: const Icon(Icons.copy),
            onPressed: () => Clipboard.setData(ClipboardData(text: 'Legato Spike S2\n$_summary')),
          ),
        ],
      ),
      body: Column(
        children: [
          Expanded(child: WebViewWidget(controller: _controller)),
          Padding(
            padding: const EdgeInsets.all(8),
            child: Text('$_status\n$_summary', style: const TextStyle(fontFamily: 'monospace')),
          ),
        ],
      ),
      floatingActionButton: FloatingActionButton(
        onPressed: _onsets.isEmpty ? null : _toggle,
        child: Icon(_timer == null ? Icons.play_arrow : Icons.stop),
      ),
    );
  }
}
