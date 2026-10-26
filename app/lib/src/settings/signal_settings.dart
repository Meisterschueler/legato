import 'package:flutter/material.dart';
import 'package:legato/src/l10n/generated/app_localizations.dart';
import 'package:legato/src/settings/settings_screen.dart';

/// Tone sequences to choose from: high, quiet and short (data-model.md).
const _toneChoices = [
  [96, 100],
  [84, 88],
  [72, 76],
  [96],
];

const _noteNames = [
  'C', 'C♯', 'D', 'E♭', 'E', 'F', 'F♯', 'G', 'A♭', 'A', 'B♭', 'B', //
];

String _toneName(int note) => '${_noteNames[note % 12]}${note ~/ 12 - 1}';

/// Signal settings (US 4, FR-027 to FR-029, FR-031).
class SignalSettings extends StatelessWidget {
  const SignalSettings({super.key, required this.settings});

  final SettingsController settings;

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context);
    final s = settings.value;
    final tones = s['signal_tones'] as Map<String, dynamic>;
    final notes = (tones['notes'] as List).cast<int>();
    final velocity = (tones['velocity'] as int).toDouble();
    final choices = [
      ..._toneChoices,
      if (!_toneChoices.any((c) => _same(c, notes))) notes,
    ];
    return Column(
      children: [
        SwitchListTile(
          title: Text(l10n.connectSignal),
          value: s['connect_signal'] == 'On',
          onChanged: (on) => settings.set('connect_signal', on ? 'On' : 'Off'),
        ),
        ListTile(
          title: Text(l10n.okSignal),
          subtitle: SegmentedButton<String>(
            segments: [
              ButtonSegment(
                value: 'EveryPerformance',
                label: Text(l10n.okEveryPerformance),
              ),
              ButtonSegment(
                value: 'WhenStopped',
                label: Text(l10n.okWhenStopped),
              ),
              ButtonSegment(value: 'Off', label: Text(l10n.off)),
            ],
            selected: {s['ok_signal'] as String},
            onSelectionChanged: (v) => settings.set('ok_signal', v.single),
          ),
        ),
        SwitchListTile(
          title: Text(l10n.abortSignal),
          value: s['abort_signal'] == 'Immediately',
          onChanged: (on) =>
              settings.set('abort_signal', on ? 'Immediately' : 'Off'),
        ),
        ListTile(
          title: Text(l10n.signalTones),
          trailing: DropdownButton<List<int>>(
            value: choices.firstWhere((c) => _same(c, notes)),
            items: [
              for (final c in choices)
                DropdownMenuItem(
                  value: c,
                  child: Text(c.map(_toneName).join(' – ')),
                ),
            ],
            onChanged: (c) =>
                settings.set('signal_tones', {...tones, 'notes': c}),
          ),
        ),
        ListTile(
          title: Text(l10n.signalVolume),
          subtitle: Slider(
            value: velocity.clamp(10, 90),
            min: 10,
            max: 90,
            divisions: 8,
            label: '${velocity.round()}',
            onChanged: (v) =>
                settings.set('signal_tones', {...tones, 'velocity': v.round()}),
          ),
        ),
      ],
    );
  }

  static bool _same(List<int> a, List<int> b) =>
      a.length == b.length &&
      Iterable.generate(a.length).every((i) => a[i] == b[i]);
}
