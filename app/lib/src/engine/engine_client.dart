import 'dart:convert';

import 'package:legato/src/rust/api/engine.dart';

/// Thin wrapper around the Rust engine: decodes JSON, holds no domain logic.
class EngineClient {
  EngineClient._(this._engine);

  final LegatoEngine _engine;

  static Future<EngineClient> start({
    required String dataDir,
    required String resourceDir,
    required String locale,
  }) async {
    final engine = await LegatoEngine.start(
      dataDir: dataDir,
      resourceDir: resourceDir,
      locale: locale,
    );
    return EngineClient._(engine);
  }

  /// Events of contracts/engine-schnittstelle.md; can be listened to once.
  Stream<Map<String, dynamic>> events() =>
      _engine.events().map((json) => jsonDecode(json) as Map<String, dynamic>);

  void setForeground(bool foreground) =>
      _engine.setForeground(foreground: foreground);

  Future<Map<String, dynamic>> getSettings() async =>
      jsonDecode(await _engine.getSettings()) as Map<String, dynamic>;

  Future<Map<String, dynamic>> setSettings(Map<String, dynamic> settings) async =>
      jsonDecode(await _engine.setSettings(json: jsonEncode(settings)))
          as Map<String, dynamic>;
}
