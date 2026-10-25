import 'dart:io';

import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:legato/src/engine/engine_client.dart';
import 'package:legato/src/l10n/generated/app_localizations.dart';

/// Input selection opened from the input symbol (US 1 scenarios 11, 15).
class InputPicker extends StatefulWidget {
  const InputPicker({super.key, required this.engine, required this.status});

  final EngineClient engine;

  /// Latest `InputStateChanged` event.
  final ValueListenable<Map<String, dynamic>> status;

  @override
  State<InputPicker> createState() => _InputPickerState();
}

class _InputPickerState extends State<InputPicker> {
  Map<String, dynamic>? _inputs;

  @override
  void initState() {
    super.initState();
    // shortcut: devices appearing without a state change show up on the next state change or reopen.
    widget.status.addListener(_load);
    _load();
  }

  @override
  void dispose() {
    widget.status.removeListener(_load);
    super.dispose();
  }

  Future<void> _load() async {
    final inputs = await widget.engine.listInputs();
    if (mounted) setState(() => _inputs = inputs);
  }

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context);
    return ValueListenableBuilder(
      valueListenable: widget.status,
      builder: (context, status, _) {
        final state = status['state'];
        final selected = _inputs?['selected'] as Map<String, dynamic>?;
        final devices = (_inputs?['midi'] as List? ?? const [])
            .cast<Map<String, dynamic>>();
        bool isSelected(Map<String, dynamic> d) =>
            selected != null &&
            d['name'] == selected['name'] &&
            d['transport'] == selected['transport'];
        Widget tile(String name, bool chosen, VoidCallback? onTap) => ListTile(
          selected: chosen,
          enabled: onTap != null,
          leading: Icon(
            chosen ? Icons.radio_button_checked : Icons.radio_button_unchecked,
          ),
          title: Text(name),
          subtitle: onTap == null ? Text(l10n.inputAbsent) : null,
          onTap: onTap,
        );
        return SimpleDialog(
          title: Text(l10n.midiInput),
          children: [
            ListTile(
              leading: Icon(
                status['receiving'] == true ? Icons.music_note : Icons.remove,
              ),
              title: Text(switch (state) {
                'MidiConnected' => l10n.inputMidiConnected,
                'ConnectionLost' => l10n.inputConnectionLost,
                _ => l10n.inputDisconnected,
              }),
              subtitle: Text(
                status['receiving'] == true
                    ? l10n.inputReceiving
                    : l10n.inputSilent,
              ),
            ),
            const Divider(),
            tile(
              l10n.inputNone,
              selected == null,
              () => widget.engine.selectInput(null),
            ),
            for (final d in devices)
              tile(
                d['name'] as String,
                isSelected(d),
                () => widget.engine.selectInput(d['id'] as int),
              ),
            if (selected != null && !devices.any(isSelected))
              tile(selected['name'] as String, true, null),
            if (status['warning_active'] == true)
              TextButton(
                onPressed: widget.engine.acknowledgeWarning,
                child: Text(l10n.acknowledgeWarning),
              ),
            if (state == 'ConnectionLost')
              FilledButton(
                onPressed: widget.engine.reconnect,
                child: Text(l10n.reconnect),
              ),
            // Windows pairs Bluetooth MIDI devices itself (research.md R4).
            if (Platform.isWindows)
              TextButton.icon(
                icon: const Icon(Icons.bluetooth),
                label: Text(l10n.bluetoothSettings),
                onPressed: () =>
                    Process.start('explorer.exe', ['ms-settings:bluetooth']),
              ),
          ],
        );
      },
    );
  }
}
