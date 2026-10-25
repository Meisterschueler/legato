import 'dart:async';

import 'package:legato/src/engine/engine_client.dart';
import 'package:wakelock_plus/wakelock_plus.dart';

/// Keeps the screen on while a practice session runs (NFR-005).
StreamSubscription<Map<String, dynamic>> keepScreenOn(EngineClient engine) =>
    engine
        .events()
        .where((e) => e['type'] == 'PracticeSessionChanged')
        .listen((e) => WakelockPlus.toggle(enable: e['active'] == true));
