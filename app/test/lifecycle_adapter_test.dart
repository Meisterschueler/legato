import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:legato/main.dart';
import 'package:legato/src/engine/engine_client.dart';
import 'package:legato/src/practice/practice_screen.dart';

/// Stands in for the Rust engine, which is not loaded in widget tests.
class _Engine implements EngineClient {
  final engineEvents = StreamController<Map<String, dynamic>>.broadcast();
  final foreground = <bool>[];

  @override
  Stream<Map<String, dynamic>> events() => engineEvents.stream;

  @override
  void setForeground(bool foreground) => this.foreground.add(foreground);

  @override
  Future<List<Map<String, dynamic>>> repertoire() async => [];

  @override
  dynamic noSuchMethod(Invocation invocation) => super.noSuchMethod(invocation);
}

void main() {
  testWidgets('background is reported and the notice shows on return', (
    tester,
  ) async {
    tester.platformDispatcher.localesTestValue = [const Locale('en', 'US')];
    addTearDown(tester.platformDispatcher.clearLocalesTestValue);
    final engine = _Engine();
    await tester.pumpWidget(LegatoApp(home: PracticeScreen(engine: engine)));
    void lifecycle(AppLifecycleState state) =>
        tester.binding.handleAppLifecycleStateChanged(state);

    lifecycle(AppLifecycleState.inactive);
    expect(engine.foreground, isEmpty);
    lifecycle(AppLifecycleState.hidden);
    expect(engine.foreground, [false]);

    engine.engineEvents.add({
      'type': 'BackgroundNotice',
      'performance_discarded': true,
    });
    await tester.pump();
    lifecycle(AppLifecycleState.inactive);
    lifecycle(AppLifecycleState.resumed);
    await tester.pumpAndSettle();
    expect(engine.foreground, [false, true]);
    expect(
      find.text(
        'The performance was discarded because Legato was in the background.',
      ),
      findsOneWidget,
    );
  });
}
