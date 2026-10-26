import 'package:flutter/material.dart';
import 'package:legato/src/debug/debug_screen.dart';
import 'package:legato/src/l10n/generated/app_localizations.dart';
import 'package:legato/src/settings/settings_screen.dart';

/// Expert settings: score follower (FR-032), recognition method (FR-085) and
/// the debug view.
class ExpertSettingsScreen extends StatelessWidget {
  const ExpertSettingsScreen({super.key, required this.settings});

  final SettingsController settings;

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context);
    Widget choice(String key, List<String> options) => RadioGroup<String>(
      groupValue: settings.value[key] as String,
      onChanged: (v) => settings.set(key, v),
      child: Column(
        children: [
          for (final o in options) RadioListTile(value: o, title: Text(o)),
        ],
      ),
    );
    return Scaffold(
      appBar: AppBar(title: Text(l10n.expertSettings)),
      body: ValueListenableBuilder(
        valueListenable: settings,
        builder: (context, _, _) => ListView(
          children: [
            ListTile(title: Text(l10n.scoreFollower)),
            choice('score_follower', ['Pthmm', 'OuterHmm', 'Arzt']),
            ListTile(title: Text(l10n.recognitionMethod)),
            choice('recognition_method', [
              'EditDistance',
              'Fingerprint',
              'Exhaustive',
            ]),
            const Divider(),
            ListTile(
              title: Text(l10n.debugView),
              trailing: const Icon(Icons.chevron_right),
              onTap: () => Navigator.of(context).push(
                MaterialPageRoute<void>(
                  builder: (_) => DebugScreen(engine: settings.engine),
                ),
              ),
            ),
          ],
        ),
      ),
    );
  }
}
