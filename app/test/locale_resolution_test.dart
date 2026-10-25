import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:legato/src/l10n/locale_resolution.dart';

void main() {
  const supported = [Locale('en'), Locale('de')];

  test('de-AT gives German', () {
    expect(
      resolveLocale([const Locale('de', 'AT')], supported),
      const Locale('de'),
    );
  });

  test('fr-FR gives English', () {
    expect(
      resolveLocale([const Locale('fr', 'FR')], supported),
      const Locale('en'),
    );
  });

  test('the first supported preference wins', () {
    expect(
      resolveLocale([const Locale('fr'), const Locale('de', 'CH')], supported),
      const Locale('de'),
    );
  });
}
