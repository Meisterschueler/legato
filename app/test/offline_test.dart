import 'dart:io';

import 'package:flutter_test/flutter_test.dart';

/// Dart and Rust sources below `dir` (relative to `app/`).
Iterable<File> _sources(String dir, String extension) =>
    Directory(dir)
        .listSync(recursive: true)
        .whereType<File>()
        .where((f) => f.path.endsWith(extension));

/// NFR-002: everything works offline; nothing in the app goes to the network.
void main() {
  test('release manifest without internet permission', () {
    final android = File('android/app/src/main/AndroidManifest.xml')
        .readAsStringSync();
    expect(android, isNot(contains('android.permission.INTERNET')));
    expect(
      File('macos/Runner/Release.entitlements').readAsStringSync(),
      isNot(contains('network')),
    );
  });

  test('no network access in app/lib', () {
    final network = RegExp(
      r'HttpClient|package:http/|WebSocket|Socket\.connect|Uri\.https?\(',
    );
    for (final file in _sources('lib', '.dart')) {
      expect(
        network.hasMatch(file.readAsStringSync()),
        isFalse,
        reason: file.path,
      );
    }
  });

  test('no network access in core and engine', () {
    final network = RegExp(r'std::net|reqwest|ureq|hyper|TcpStream|UdpSocket');
    for (final file in [
      ..._sources('../core/src', '.rs'),
      ..._sources('../engine/src', '.rs'),
      File('../core/Cargo.toml'),
      File('../engine/Cargo.toml'),
    ]) {
      expect(
        network.hasMatch(file.readAsStringSync()),
        isFalse,
        reason: file.path,
      );
    }
  });
}
