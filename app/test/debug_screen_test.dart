import 'package:flutter_test/flutter_test.dart';
import 'package:legato/src/debug/debug_screen.dart';

void main() {
  test('tempo in quarter notes per minute between two positions', () {
    expect(tempo((1.0, 2.0), (1.5, 3.0)), 120);
    // Within one onset and back to the start of a repeat.
    expect(tempo((1.0, 2.0), (1.01, 2.0)), isNull);
    expect(tempo((1.0, 8.0), (1.5, 0.0)), isNull);
  });
}
