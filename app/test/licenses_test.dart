import 'package:flutter_test/flutter_test.dart';
import 'package:legato/src/about/licenses.dart';

void main() {
  test('Rust licenses are read per license text with their crates', () {
    final line = '-' * 80;
    final text =
        'serde\r\nanyhow\r\n\r\nMIT License\r\n\r\nCopyright\r\n'
        '$line\r\nzlib-rs\n\nzlib License\n$line\n';
    final entries = rustLicenses(text).toList();
    expect(entries.map((e) => e.packages), [
      ['serde', 'anyhow'],
      ['zlib-rs'],
    ]);
    expect(entries.first.paragraphs.map((p) => p.text), [
      'MIT License',
      'Copyright',
    ]);
  });
}
