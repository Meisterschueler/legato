import 'package:flutter_test/flutter_test.dart';
import 'package:legato/main.dart';

void main() {
  testWidgets('app starts', (tester) async {
    await tester.pumpWidget(const LegatoApp());
    expect(find.text('Legato'), findsOneWidget);
  });
}
