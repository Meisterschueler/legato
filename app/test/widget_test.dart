import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:legato/main.dart';
import 'package:legato/src/practice/practice_screen.dart';

void main() {
  testWidgets('practice screen in English', (tester) async {
    tester.platformDispatcher.localesTestValue = [const Locale('en', 'US')];
    addTearDown(tester.platformDispatcher.clearLocalesTestValue);
    await tester.pumpWidget(const LegatoApp(home: PracticeScreen()));
    await tester.pumpAndSettle();
    expect(find.text('Legato'), findsOneWidget);
    expect(find.text('The score appears here'), findsOneWidget);
  });

  testWidgets('practice screen in German for de-AT', (tester) async {
    tester.platformDispatcher.localesTestValue = [const Locale('de', 'AT')];
    addTearDown(tester.platformDispatcher.clearLocalesTestValue);
    await tester.pumpWidget(const LegatoApp(home: PracticeScreen()));
    await tester.pumpAndSettle();
    expect(find.text('Hier erscheinen die Noten'), findsOneWidget);
  });
}
