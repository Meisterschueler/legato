import 'package:flutter_test/flutter_test.dart';
import 'package:legato/src/practice/practice_screen.dart';

void main() {
  test('marks after the performance follow error_marking_after (FR-025)', () {
    final deviations = [
      {'kind': 'Omission', 'note_id': '0/0/0:1/48', 'pitch': null},
      {'kind': 'Substitution', 'note_id': '0/0/1:8/52', 'pitch': 53},
      {'kind': 'Addition', 'note_id': null, 'pitch': 70},
    ];
    expect(
      marksAfter(deviations, {
        'omission': true,
        'wrong_note': false,
        'addition': true,
      }),
      [
        {'type': 'mark', 'note_id': '0/0/0:1/48', 'kind': 'missing'},
      ],
    );
    expect(marksAfter(deviations, {'wrong_note': true}), [
      {'type': 'mark', 'note_id': '0/0/1:8/52', 'kind': 'wrong'},
    ]);
    expect(marksAfter(deviations, {}), isEmpty);
  });
}
