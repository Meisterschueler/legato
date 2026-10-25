import 'package:flutter/material.dart';
import 'package:legato/src/engine/engine_client.dart';
import 'package:legato/src/l10n/generated/app_localizations.dart';

/// Practice screen: score view and input symbol follow with US 1 and US 2.
class PracticeScreen extends StatelessWidget {
  const PracticeScreen({super.key, this.engine});

  final EngineClient? engine;

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context);
    return Scaffold(
      appBar: AppBar(
        title: Text(l10n.appTitle),
        actions: [
          IconButton(
            tooltip: l10n.midiInput,
            icon: const Icon(Icons.piano),
            onPressed: null,
          ),
        ],
      ),
      body: Center(child: Text(l10n.scorePlaceholder)),
    );
  }
}
