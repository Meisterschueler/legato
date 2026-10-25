import 'package:flutter/material.dart';
import 'package:legato/src/l10n/generated/app_localizations.dart';

/// Input state (US 1 scenario 10): small while connected, prominent otherwise;
/// each state has its own shape.
class InputSymbol extends StatelessWidget {
  const InputSymbol({super.key, required this.state, this.onTap});

  /// `state` of `InputStateChanged`: `Disconnected`, `MidiConnected` or `ConnectionLost`.
  final String state;
  final VoidCallback? onTap;

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context);
    final colors = Theme.of(context).colorScheme;
    final (icon, label, size, color) = switch (state) {
      'MidiConnected' => (
        Icons.piano,
        l10n.inputMidiConnected,
        18.0,
        colors.outline,
      ),
      'ConnectionLost' => (
        Icons.warning_amber,
        l10n.inputConnectionLost,
        32.0,
        colors.error,
      ),
      _ => (Icons.piano_off, l10n.inputDisconnected, 32.0, colors.onSurface),
    };
    // The fixed 48 px button keeps the symbol in place across states.
    return IconButton(
      tooltip: label,
      onPressed: onTap,
      icon: AnimatedSwitcher(
        duration: const Duration(milliseconds: 300),
        child: Icon(icon, key: ValueKey(icon), size: size, color: color),
      ),
    );
  }
}
