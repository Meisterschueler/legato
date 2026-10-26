import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:legato/main.dart';
import 'package:legato/src/engine/engine_client.dart';
import 'package:legato/src/practice/practice_screen.dart';

/// Stands in for the Rust engine, which is not loaded in widget tests.
class _Engine implements EngineClient {
  final engineEvents = StreamController<Map<String, dynamic>>.broadcast();

  @override
  Stream<Map<String, dynamic>> events() => engineEvents.stream;

  @override
  Future<List<Map<String, dynamic>>> repertoire() async => [];

  /// Never completes, so no WebView is created in the test.
  @override
  Future<String> renderScore(
    String piece, {
    int transposition = 0,
    String variation = 'BothHands',
    String displayVariant = 'Complete',
  }) => Completer<String>().future;

  @override
  dynamic noSuchMethod(Invocation invocation) => super.noSuchMethod(invocation);
}

/// SC-004, FR-024: a performance brings no messages, dialogs or animations; the
/// score itself and the error marking are allowed.
void main() {
  testWidgets('a performance brings no interruptions', (tester) async {
    final engine = _Engine();
    await tester.pumpWidget(LegatoApp(home: PracticeScreen(engine: engine)));
    await tester.pumpAndSettle();
    final performance = [
      {'type': 'PracticeSessionChanged', 'active': true},
      {
        'type': 'PerformanceRecognized',
        'piece': 'h1',
        'variation': 'BothHands',
        'transposition': 0,
      },
      for (var i = 0; i < 20; i++)
        {
          'type': 'Position',
          'note_ids': ['$i/0/0/60'],
          'driver_time_us': i * 250000,
        },
      {
        'type': 'DeviationDetected',
        'kind': 'Substitution',
        'note_id': '3/0/0/60',
        'pitch': 61,
      },
      {
        'type': 'PerformanceEnded',
        'outcome': 'Completed',
        'reason': null,
        'deviations': <Object>[],
      },
    ];
    for (final event in performance) {
      engine.engineEvents.add(event);
      await tester.pump();
      expect(find.byType(SnackBar), findsNothing, reason: '$event');
      expect(find.byType(Dialog), findsNothing, reason: '$event');
      expect(find.byType(BottomSheet), findsNothing, reason: '$event');
      expect(tester.hasRunningAnimations, isFalse, reason: '$event');
    }
  });
}
