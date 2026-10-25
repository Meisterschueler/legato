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

  Future<Map<String, dynamic>> setSettings(
    Map<String, dynamic> settings,
  ) async =>
      jsonDecode(await _engine.setSettings(json: jsonEncode(settings)))
          as Map<String, dynamic>;

  /// `{ midi: [{ id, name, transport }], selected?: { name, transport, bluetooth_id? } }`.
  Future<Map<String, dynamic>> listInputs() async =>
      jsonDecode(await _engine.listInputs()) as Map<String, dynamic>;

  /// `null` selects no input; the new state arrives as `InputStateChanged`.
  Future<void> selectInput(int? device) => _engine.selectInput(device: device);

  Future<void> acknowledgeWarning() => _engine.acknowledgeWarning();

  Future<void> reconnect() => _engine.reconnect();

  /// `[PieceInfo]`: `{ id, work, title, number?, has_fingering, display_variants }`.
  Future<List<Map<String, dynamic>>> repertoire() async =>
      (jsonDecode(await _engine.repertoire()) as List)
          .cast<Map<String, dynamic>>();

  /// Returns the `PieceInfo`; throws [InvalidScoreFile] if the file is rejected.
  Future<Map<String, dynamic>> loadPiece(String path) async {
    try {
      return jsonDecode(await _engine.loadPiece(path: path))
          as Map<String, dynamic>;
    } on String catch (reason) {
      throw InvalidScoreFile(reason);
    }
  }

  Future<String> renderScore(
    String piece, {
    int transposition = 0,
    String variation = 'BothHands',
    String displayVariant = 'Complete',
  }) => _engine.renderScore(
    piece: piece,
    transposition: transposition,
    variation: variation,
    displayVariant: displayVariant,
  );
}

/// `reason`: `unreadable`, `not_partwise`, `no_pitched_notes`, `unpaired_repeats`
/// or `incompatible_with_work`.
class InvalidScoreFile implements Exception {
  const InvalidScoreFile(this.reason);

  final String reason;

  @override
  String toString() => 'InvalidScoreFile($reason)';
}
