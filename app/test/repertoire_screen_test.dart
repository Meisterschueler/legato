import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:legato/main.dart';
import 'package:legato/src/engine/engine_client.dart';
import 'package:legato/src/repertoire/repertoire_screen.dart';

/// Stands in for the Rust engine, which is not loaded in widget tests.
class _Engine implements EngineClient {
  @override
  Future<List<Map<String, dynamic>>> repertoire() async => [
    {
      'id': 'a',
      'title': 'The Virtuoso Pianist',
      'number': '1',
      'has_fingering': true,
    },
    {'id': 'b', 'title': 'Etude', 'number': null, 'has_fingering': false},
  ];

  @override
  Future<Map<String, dynamic>> loadPiece(String path) async =>
      throw const InvalidScoreFile('not_partwise');

  @override
  dynamic noSuchMethod(Invocation invocation) => super.noSuchMethod(invocation);
}

void _english(WidgetTester tester) {
  tester.platformDispatcher.localesTestValue = [const Locale('en', 'US')];
  addTearDown(tester.platformDispatcher.clearLocalesTestValue);
}

void main() {
  testWidgets('pieces without fingering are marked', (tester) async {
    _english(tester);
    await tester.pumpWidget(
      LegatoApp(home: RepertoireScreen(engine: _Engine())),
    );
    await tester.pumpAndSettle();

    expect(find.text('The Virtuoso Pianist No. 1'), findsOneWidget);
    expect(find.text('Etude'), findsOneWidget);
    expect(
      find.text('No fingering: analyses that need fingering are not available'),
      findsOneWidget,
    );
  });

  testWidgets('rejected file shows a clear message and the app stays usable', (
    tester,
  ) async {
    _english(tester);
    var picks = 0;
    await tester.pumpWidget(
      LegatoApp(
        home: RepertoireScreen(
          engine: _Engine(),
          pickFile: () async {
            picks++;
            return 'timewise.musicxml';
          },
        ),
      ),
    );
    await tester.pumpAndSettle();

    await tester.tap(find.text('Load file'));
    await tester.pumpAndSettle();
    expect(
      find.text(
        'This MusicXML layout (score-timewise) is not supported. '
        'Please export the piece as score-partwise.',
      ),
      findsOneWidget,
    );
    expect(find.text('Etude'), findsOneWidget);

    await tester.tap(find.text('Load file'));
    await tester.pumpAndSettle();
    expect(picks, 2);
  });
}
