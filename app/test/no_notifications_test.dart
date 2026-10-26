import 'dart:io';

import 'package:flutter_test/flutter_test.dart';

/// SC-005, FR-026: the app neither requests nor sends notifications.
void main() {
  test('no notification permission or capability', () {
    final android = File(
      'android/app/src/main/AndroidManifest.xml',
    ).readAsStringSync();
    expect(android, isNot(contains('POST_NOTIFICATIONS')));
    expect(android, isNot(contains('c2dm')));

    for (final plist in ['ios/Runner/Info.plist', 'macos/Runner/Info.plist']) {
      expect(
        File(plist).readAsStringSync(),
        isNot(contains('remote-notification')),
        reason: plist,
      );
    }
    final entitlements = [
      ...Directory('ios/Runner').listSync(),
      ...Directory('macos/Runner').listSync(),
    ].whereType<File>().where((f) => f.path.endsWith('.entitlements'));
    for (final file in entitlements) {
      expect(
        file.readAsStringSync(),
        isNot(contains('aps-environment')),
        reason: file.path,
      );
    }
  });

  test('no notification packages', () {
    final pubspec = File('pubspec.yaml').readAsStringSync();
    expect(pubspec, isNot(contains('notification')));
    expect(pubspec, isNot(contains('firebase')));
  });
}
