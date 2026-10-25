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
  final _input = ValueNotifier<Map<String, dynamic>>({'state': 'Disconnected'});
  StreamSubscription<Map<String, dynamic>>? _events;
  StreamSubscription<Map<String, dynamic>>? _wakelock;
  LifecycleAdapter? _lifecycle;

  @override
  void initState() {
    super.initState();
    final engine = widget.engine;
    if (engine != null) {
      _events = engine.events().listen((event) {
        if (event['type'] == 'InputStateChanged') _input.value = event;
      });
      _wakelock = keepScreenOn(engine);
      _lifecycle = LifecycleAdapter(engine, onDiscarded: _backgroundNotice);
    }
    // shortcut: shows the first piece until the recognition (US 3) chooses it.
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
    super.dispose();
  }

  void _backgroundNotice() {
    final text = AppLocalizations.of(context).performanceDiscarded;
    ScaffoldMessenger.of(context).showSnackBar(SnackBar(content: Text(text)));
  }

  Future<void> _show(String piece) async {
    final musicxml = await widget.engine!.renderScore(piece);
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
          : ScoreView(musicxml: musicxml),
    );
  }
}
