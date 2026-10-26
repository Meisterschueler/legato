import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:legato/main.dart';
import 'package:legato/src/engine/engine_client.dart';
import 'package:legato/src/settings/settings_screen.dart';

/// Stands in for the Rust engine; keeps the settings it is given.
class _Engine implements EngineClient {
  Map<String, dynamic> settings = {
    'recognition_filter': {
      'pieces': <String>[],
      'variations': <String>[],
      'transpositions': <int>[],
    },
    'pause_limit_ms': 4000,
    'rest_limit_ms': 10000,
    'score_follower': 'Pthmm',
    'recognition_method': 'EditDistance',
    'error_marking_during': _noMarking,
    'error_marking_after': _noMarking,
    'connect_signal': 'On',
    'ok_signal': 'WhenStopped',
    'abort_signal': 'Immediately',
    'signal_tones': {
      'notes': [96, 100],
      'velocity': 30,
    },
  };

  static const _noMarking = {
    'omission': false,
    'wrong_note': false,
    'addition': false,
  };

  @override
  Future<Map<String, dynamic>> getSettings() async => settings;

  @override
  Future<Map<String, dynamic>> setSettings(Map<String, dynamic> s) async =>
      settings = s;

  @override
  Future<List<Map<String, dynamic>>> repertoire() async => [
    {'id': 'h1', 'title': 'The Virtuoso Pianist', 'number': '1'},
  ];

  @override
  dynamic noSuchMethod(Invocation invocation) => super.noSuchMethod(invocation);
}

Future<_Engine> _open(WidgetTester tester) async {
  tester.platformDispatcher.localesTestValue = [const Locale('en', 'US')];
  addTearDown(tester.platformDispatcher.clearLocalesTestValue);
  final engine = _Engine();
  await tester.pumpWidget(LegatoApp(home: SettingsScreen(engine: engine)));
  await tester.pumpAndSettle();
  return engine;
}

Future<void> _tap(WidgetTester tester, Finder finder) async {
  await tester.scrollUntilVisible(finder, 200);
  await tester.ensureVisible(finder);
  await tester.pumpAndSettle();
  await tester.tap(finder);
  await tester.pumpAndSettle();
}

void main() {
  testWidgets('changes are saved in the engine', (tester) async {
    final engine = await _open(tester);

    await _tap(tester, find.text('Right hand'));
    expect(engine.settings['recognition_filter']['variations'], [
      'RightHandOnly',
    ]);

    await _tap(tester, find.text('After every performance'));
    expect(engine.settings['ok_signal'], 'EveryPerformance');

    await _tap(tester, find.widgetWithText(SwitchListTile, 'Abort signal'));
    expect(engine.settings['abort_signal'], 'Off');
  });

  testWidgets('expert settings choose the recognition method', (tester) async {
    final engine = await _open(tester);

    await _tap(tester, find.text('Expert settings'));
    await _tap(tester, find.text('Fingerprint'));
    expect(engine.settings['recognition_method'], 'Fingerprint');
    expect(
      tester
          .widget<RadioGroup<String>>(find.byType(RadioGroup<String>).last)
          .groupValue,
      'Fingerprint',
    );
  });
}
