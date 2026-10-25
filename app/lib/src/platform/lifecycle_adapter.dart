import 'dart:async';

import 'package:flutter/widgets.dart';
import 'package:legato/src/engine/engine_client.dart';

/// Reports foreground changes to the engine and calls [onDiscarded] on return
/// if a performance was discarded meanwhile (US 1 scenario 8).
class LifecycleAdapter {
  LifecycleAdapter(this._engine, {required this.onDiscarded}) {
    _events = _engine.events().listen((e) {
      if (e['type'] == 'BackgroundNotice' &&
          e['performance_discarded'] == true) {
        _discarded = true;
      }
    });
    _listener = AppLifecycleListener(onStateChange: _changed);
  }

  final EngineClient _engine;
  final VoidCallback onDiscarded;
  late final StreamSubscription<Map<String, dynamic>> _events;
  late final AppLifecycleListener _listener;
  var _foreground = true;
  var _discarded = false;

  void _changed(AppLifecycleState state) {
    // Losing focus (inactive) keeps the performance; hidden or paused does not.
    final foreground =
        state == AppLifecycleState.resumed ||
        state == AppLifecycleState.inactive;
    if (foreground && _discarded) {
      _discarded = false;
      onDiscarded();
    }
    if (foreground == _foreground) return;
    _foreground = foreground;
    _engine.setForeground(foreground);
  }

  void dispose() {
    _events.cancel();
    _listener.dispose();
  }
}
