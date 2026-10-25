import 'dart:async';

import 'package:flutter/material.dart';
import 'package:legato/src/engine/engine_client.dart';
import 'package:legato/src/input/input_picker.dart';
import 'package:legato/src/input/input_symbol.dart';
import 'package:legato/src/l10n/generated/app_localizations.dart';
import 'package:legato/src/platform/lifecycle_adapter.dart';
import 'package:legato/src/platform/wakelock_adapter.dart';
import 'package:legato/src/repertoire/repertoire_screen.dart';
import 'package:legato/src/score_view/score_view.dart';

/// Practice screen with the score and the input symbol.
class PracticeScreen extends StatefulWidget {
  const PracticeScreen({super.key, this.engine});

  final EngineClient? engine;

  @override
  State<PracticeScreen> createState() => _PracticeScreenState();
}

class _PracticeScreenState extends State<PracticeScreen> {
  String? _musicxml;

  /// Piece and transposition of the shown score.
  (String, int)? _shown;
  final _input = ValueNotifier<Map<String, dynamic>>({'state': 'Disconnected'});
  final _scoreMessages = StreamController<Map<String, dynamic>>.broadcast();
  StreamSubscription<Map<String, dynamic>>? _events;
  StreamSubscription<Map<String, dynamic>>? _wakelock;
  LifecycleAdapter? _lifecycle;

  @override
  void initState() {
    super.initState();
    final engine = widget.engine;
    if (engine != null) {
      _events = engine.events().listen(_handle);
      _wakelock = keepScreenOn(engine);
      _lifecycle = LifecycleAdapter(engine, onDiscarded: _backgroundNotice);
    }
    // Until the first performance is recognized, the first piece is shown.
    widget.engine?.repertoire().then((pieces) {
      if (pieces.isNotEmpty) _show(pieces.first['id'] as String);
    });
  }

  @override
  void dispose() {
    _events?.cancel();
    _wakelock?.cancel();
    _lifecycle?.dispose();
    _input.dispose();
    _scoreMessages.close();
    super.dispose();
  }

  /// Engine events for the input symbol and the score (US 3 scenario 18).
  void _handle(Map<String, dynamic> event) {
    switch (event['type']) {
      case 'InputStateChanged':
        _input.value = event;
      case 'PerformanceRecognized':
        final piece = event['piece'] as String;
        final transposition = event['transposition'] as int;
        if (_shown == (piece, transposition)) {
          _scoreMessages.add({'type': 'reset'});
        } else {
          _show(piece, transposition: transposition);
        }
      case 'Position':
        _scoreMessages.add({
          'type': 'position',
          'note_ids': event['note_ids'],
          'driver_time_us': event['driver_time_us'],
        });
      // Additions have no note to mark.
      case 'DeviationDetected' when event['note_id'] != null:
        final missing = event['kind'] == 'Omission';
        _scoreMessages.add({
          'type': 'mark',
          'note_id': event['note_id'],
          'kind': missing ? 'missing' : 'wrong',
        });
    }
  }

  void _backgroundNotice() {
    final text = AppLocalizations.of(context).performanceDiscarded;
    ScaffoldMessenger.of(context).showSnackBar(SnackBar(content: Text(text)));
  }

  Future<void> _show(String piece, {int transposition = 0}) async {
    _shown = (piece, transposition);
    final musicxml = await widget.engine!.renderScore(
      piece,
      transposition: transposition,
    );
    if (mounted) setState(() => _musicxml = musicxml);
  }

  Future<void> _openRepertoire() async {
    final piece = await Navigator.of(context).push<String>(
      MaterialPageRoute(
        builder: (_) => RepertoireScreen(engine: widget.engine!),
      ),
    );
    if (piece != null) await _show(piece);
  }

  void _openInputPicker() => showDialog<void>(
    context: context,
    builder: (_) => InputPicker(engine: widget.engine!, status: _input),
  );

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context);
    final musicxml = _musicxml;
    return Scaffold(
      appBar: AppBar(
        title: Text(l10n.appTitle),
        actions: [
          IconButton(
            tooltip: l10n.repertoire,
            icon: const Icon(Icons.library_music),
            onPressed: widget.engine == null ? null : _openRepertoire,
          ),
          ValueListenableBuilder(
            valueListenable: _input,
            builder: (context, input, _) => InputSymbol(
              state: input['state'] as String,
              onTap: widget.engine == null ? null : _openInputPicker,
            ),
          ),
        ],
      ),
      body: musicxml == null
          ? Center(child: Text(l10n.scorePlaceholder))
          : ScoreView(musicxml: musicxml, messages: _scoreMessages.stream),
    );
  }
}
