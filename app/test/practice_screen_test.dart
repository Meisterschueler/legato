import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:legato/main.dart';
import 'package:legato/src/engine/engine_client.dart';
import 'package:legato/src/practice/practice_screen.dart';

/// Stands in for the Rust engine, which is not loaded in widget tests.
class _Engine implements EngineClient {
  _Engine({this.displayVariant = const {}});

  final Map<String, dynamic> displayVariant;
  final engineEvents = StreamController<Map<String, dynamic>>.broadcast();
  final rendered = <(String, int, String)>[];

  @override
  Stream<Map<String, dynamic>> events() => engineEvents.stream;

  @override
  Future<List<Map<String, dynamic>>> repertoire() async => [
    {
      'id': 'h1',
      'work': 'hanon',
      'display_variants': ['Complete', 'Abbreviated'],
    },
  ];

  @override
  Future<Map<String, dynamic>> getSettings() async => {
    'display_variant': displayVariant,
  };

  /// Never completes, so no WebView is created in the test.
  @override
  Future<String> renderScore(
    String piece, {
    int transposition = 0,
    String variation = 'BothHands',
    String displayVariant = 'Complete',
  }) {
    rendered.add((piece, transposition, displayVariant));
    return Completer<String>().future;
  }

  @override
  dynamic noSuchMethod(Invocation invocation) => super.noSuchMethod(invocation);
}

void main() {
  testWidgets('a recognized performance shows its piece in its key', (
    tester,
  ) async {
    final engine = _Engine();
    await tester.pumpWidget(LegatoApp(home: PracticeScreen(engine: engine)));
    await tester.pumpAndSettle();
    Map<String, dynamic> recognized(int transposition) => {
      'type': 'PerformanceRecognized',
      'piece': 'h1',
      'variation': 'BothHands',
      'transposition': transposition,
    };

    engine.engineEvents.add(recognized(3));
    await tester.pump();
    engine.engineEvents.add(recognized(3));
    await tester.pump();
    engine.engineEvents.add(recognized(5));
    await tester.pump();
    expect(engine.rendered, [
      ('h1', 0, 'Complete'),
      ('h1', 3, 'Complete'),
      ('h1', 5, 'Complete'),
    ]);
  });

  testWidgets('the score shows the display variant chosen for the work', (
    tester,
  ) async {
    final engine = _Engine(displayVariant: {'hanon': 'Abbreviated'});
    await tester.pumpWidget(LegatoApp(home: PracticeScreen(engine: engine)));
    await tester.pumpAndSettle();
    expect(engine.rendered, [('h1', 0, 'Abbreviated')]);
  });
}
