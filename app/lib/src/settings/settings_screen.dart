import 'package:flutter/material.dart';
import 'package:legato/src/engine/engine_client.dart';
import 'package:legato/src/l10n/generated/app_localizations.dart';
import 'package:legato/src/settings/expert_settings_screen.dart';
import 'package:legato/src/settings/signal_settings.dart';

/// The engine's `Settings` as JSON; each change is saved at once and applies
/// from the next performance on (data-model.md).
class SettingsController extends ValueNotifier<Map<String, dynamic>> {
  SettingsController(this._engine, super.value);

  final EngineClient _engine;

  Future<void> set(String key, Object? setting) async {
    value = {...value, key: setting};
    value = await _engine.setSettings(value);
  }
}

/// Settings (FR-018, FR-020, FR-025); signals (US 4) and expert settings.
class SettingsScreen extends StatefulWidget {
  const SettingsScreen({super.key, required this.engine});

  final EngineClient engine;

  @override
  State<SettingsScreen> createState() => _SettingsScreenState();
}

class _SettingsScreenState extends State<SettingsScreen> {
  SettingsController? _settings;
  List<Map<String, dynamic>> _pieces = const [];

  @override
  void initState() {
    super.initState();
    _load();
  }

  @override
  void dispose() {
    _settings?.dispose();
    super.dispose();
  }

  Future<void> _load() async {
    final settings = await widget.engine.getSettings();
    final pieces = await widget.engine.repertoire();
    if (!mounted) return;
    setState(() {
      _settings = SettingsController(widget.engine, settings);
      _pieces = pieces;
    });
  }

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context);
    final controller = _settings;
    return Scaffold(
      appBar: AppBar(title: Text(l10n.settings)),
      body: controller == null
          ? const Center(child: CircularProgressIndicator())
          : ValueListenableBuilder(
              valueListenable: controller,
              builder: (context, settings, _) => ListView(
                children: [
                  _Header(l10n.timeLimits),
                  _SecondsSlider(
                    label: l10n.pauseLimit,
                    ms: settings['pause_limit_ms'] as int,
                    min: 2,
                    max: 10,
                    onChanged: (ms) => controller.set('pause_limit_ms', ms),
                  ),
                  _SecondsSlider(
                    label: l10n.restLimit,
                    ms: settings['rest_limit_ms'] as int,
                    min: 4,
                    max: 20,
                    onChanged: (ms) => controller.set('rest_limit_ms', ms),
                  ),
                  _Header(l10n.errorMarkingDuring),
                  ..._marking(controller, 'error_marking_during', l10n),
                  _Header(l10n.errorMarkingAfter),
                  ..._marking(controller, 'error_marking_after', l10n),
                  _Header(l10n.recognizeOnly),
                  _filter(controller, l10n),
                  ..._display(controller, l10n),
                  _Header(l10n.signals),
                  SignalSettings(settings: controller),
                  const Divider(),
                  ListTile(
                    title: Text(l10n.expertSettings),
                    trailing: const Icon(Icons.chevron_right),
                    onTap: () => Navigator.of(context).push(
                      MaterialPageRoute<void>(
                        builder: (_) =>
                            ExpertSettingsScreen(settings: controller),
                      ),
                    ),
                  ),
                ],
              ),
            ),
    );
  }

  List<Widget> _marking(
    SettingsController controller,
    String key,
    AppLocalizations l10n,
  ) {
    final marking = controller.value[key] as Map<String, dynamic>;
    return [
      for (final (field, label) in [
        ('omission', l10n.omissions),
        ('wrong_note', l10n.wrongNotes),
        ('addition', l10n.additions),
      ])
        SwitchListTile(
          title: Text(label),
          value: marking[field] as bool,
          onChanged: (on) => controller.set(key, {...marking, field: on}),
        ),
    ];
  }

  /// Display variant per work that offers more than one (FR-011).
  List<Widget> _display(SettingsController controller, AppLocalizations l10n) {
    final works = {
      for (final p in _pieces)
        if ((p['display_variants'] as List).length > 1) p['work'] as String: p,
    };
    if (works.isEmpty) return const [];
    final chosen = controller.value['display_variant'] as Map<String, dynamic>;
    return [
      _Header(l10n.display),
      for (final MapEntry(key: work, value: piece) in works.entries)
        ListTile(
          title: Text(piece['title'] as String),
          subtitle: DropdownButton<String>(
            isExpanded: true,
            value: chosen[work] as String? ?? 'Complete',
            items: [
              for (final v in piece['display_variants'] as List)
                DropdownMenuItem(
                  value: v as String,
                  child: Text(displayVariantName(l10n, v)),
                ),
            ],
            onChanged: (v) =>
                controller.set('display_variant', {...chosen, work: v}),
          ),
        ),
    ];
  }

  /// FR-018; empty lists allow everything.
  Widget _filter(SettingsController controller, AppLocalizations l10n) {
    final filter =
        controller.value['recognition_filter'] as Map<String, dynamic>;
    Widget chips<T>(String field, List<(T, String)> options) {
      final chosen = (filter[field] as List).cast<T>();
      return Wrap(
        spacing: 8,
        children: [
          for (final (option, label) in options)
            FilterChip(
              label: Text(label),
              selected: chosen.contains(option),
              onSelected: (on) => controller.set('recognition_filter', {
                ...filter,
                field: on
                    ? [...chosen, option]
                    : chosen.where((o) => o != option).toList(),
              }),
            ),
        ],
      );
    }

    final keys = l10n.keyNames.split(',');
    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 16),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(l10n.nothingSelectedMeansAll),
          chips('pieces', [
            for (final p in _pieces)
              (
                p['id'] as String,
                p['number'] == null
                    ? p['title'] as String
                    : l10n.pieceWithNumber(
                        p['title'] as String,
                        p['number'] as String,
                      ),
              ),
          ]),
          chips('variations', [
            ('BothHands', l10n.bothHands),
            ('RightHandOnly', l10n.rightHand),
            ('LeftHandOnly', l10n.leftHand),
          ]),
          chips('transpositions', [
            for (var t = 0; t < keys.length; t++) (t, keys[t]),
          ]),
        ],
      ),
    );
  }
}

/// Name of a `DisplayVariant` (FR-011).
String displayVariantName(AppLocalizations l10n, String variant) {
  return switch (variant) {
    'Abbreviated' => l10n.displayAbbreviated,
    'FirstMeasures' => l10n.displayFirstMeasures,
    'FirstMeasure' => l10n.displayFirstMeasure,
    _ => l10n.displayComplete,
  };
}

class _Header extends StatelessWidget {
  const _Header(this.text);

  final String text;

  @override
  Widget build(BuildContext context) => Padding(
    padding: const EdgeInsets.fromLTRB(16, 16, 16, 4),
    child: Text(text, style: Theme.of(context).textTheme.titleSmall),
  );
}

class _SecondsSlider extends StatelessWidget {
  const _SecondsSlider({
    required this.label,
    required this.ms,
    required this.min,
    required this.max,
    required this.onChanged,
  });

  final String label;
  final int ms;
  final int min;
  final int max;
  final ValueChanged<int> onChanged;

  @override
  Widget build(BuildContext context) {
    final seconds = (ms / 1000).clamp(min, max).toDouble();
    final text = AppLocalizations.of(context).seconds(seconds.round());
    return ListTile(
      title: Text(label),
      trailing: Text(text),
      subtitle: Slider(
        value: seconds,
        min: min.toDouble(),
        max: max.toDouble(),
        divisions: max - min,
        label: text,
        onChanged: (s) => onChanged(s.round() * 1000),
      ),
    );
  }
}
