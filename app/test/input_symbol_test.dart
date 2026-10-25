import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:legato/main.dart';
import 'package:legato/src/engine/engine_client.dart';
import 'package:legato/src/input/input_symbol.dart';
import 'package:legato/src/practice/practice_screen.dart';

/// Stands in for the Rust engine, which is not loaded in widget tests.
class _Engine implements EngineClient {
  final stateEvents = StreamController<Map<String, dynamic>>();
  final selected = <int?>[];
  var reconnects = 0;
  var acknowledged = 0;

  @override
  Stream<Map<String, dynamic>> events() => stateEvents.stream;

  @override
  Future<List<Map<String, dynamic>>> repertoire() async => [];

  @override
  Future<Map<String, dynamic>> listInputs() async => {
    'midi': [
      {'id': 7, 'name': 'ATX3', 'transport': 'Bluetooth'},
    ],
    'selected': {'name': 'ATX3', 'transport': 'Bluetooth'},
  };

  @override
  Future<void> selectInput(int? device) async => selected.add(device);

  @override
  Future<void> reconnect() async => reconnects++;

  @override
  Future<void> acknowledgeWarning() async => acknowledged++;

  @override
  dynamic noSuchMethod(Invocation invocation) => super.noSuchMethod(invocation);
}

Map<String, dynamic> _event(String state, {bool warning = false}) => {
  'type': 'InputStateChanged',
  'state': state,
  'device': 'ATX3',
  'receiving': false,
  'warning_active': warning,
};

const _states = ['Disconnected', 'MidiConnected', 'ConnectionLost'];

Widget _alone(Widget w) => LegatoApp(home: Scaffold(body: w));

void _english(WidgetTester tester) {
  tester.platformDispatcher.localesTestValue = [const Locale('en', 'US')];
  addTearDown(tester.platformDispatcher.clearLocalesTestValue);
}

void main() {
  testWidgets('states differ in shape, small only when connected', (
    tester,
  ) async {
    _english(tester);
    final icons = <String, IconData?>{};
    final sizes = <String, double>{};
    for (final state in _states) {
      await tester.pumpWidget(_alone(InputSymbol(state: state)));
      await tester.pumpAndSettle();
      final icon = find.descendant(
        of: find.byType(InputSymbol),
        matching: find.byType(Icon),
      );
      icons[state] = tester.widget<Icon>(icon).icon;
      sizes[state] = tester.getSize(icon).width;
    }
    expect(icons.values.toSet(), hasLength(3));
    expect(sizes['MidiConnected'], lessThan(sizes['Disconnected']!));
    expect(sizes['MidiConnected'], lessThan(sizes['ConnectionLost']!));
  });

  testWidgets('symbol stays in place and tapping opens the selection', (
    tester,
  ) async {
    _english(tester);
    final engine = _Engine();
    await tester.pumpWidget(LegatoApp(home: PracticeScreen(engine: engine)));
    final symbol = find.byType(InputSymbol);
    final positions = <Offset>{};
    for (final state in _states) {
      engine.stateEvents.add(_event(state, warning: true));
      await tester.pumpAndSettle();
      positions.add(tester.getCenter(symbol));
    }
    expect(positions, hasLength(1));

    // US 1 scenario 15: reconnecting takes two taps.
    await tester.tap(symbol);
    await tester.pumpAndSettle();
    expect(find.text('ATX3'), findsOneWidget);
    expect(find.text('Connection lost'), findsOneWidget);
    await tester.tap(find.text('Reconnect'));
    expect(engine.reconnects, 1);

    await tester.tap(find.text('Stop warning'));
    expect(engine.acknowledged, 1);

    await tester.tap(find.text('None'));
    expect(engine.selected, [null]);
  });
}
