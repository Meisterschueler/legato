import 'dart:convert';

import 'package:flutter/foundation.dart';
import 'package:flutter/services.dart';

final _separator = '\n${'-' * 80}\n';

/// Licenses Flutter does not collect itself (T098): Legato's NOTICE, the Rust
/// crates (cargo-about, `app/rust/about.hbs`) and the packages bundled into
/// the score view (Vite `build.license`).
Stream<LicenseEntry> bundledLicenses() async* {
  yield LicenseEntryWithLineBreaks([
    'Legato',
  ], await rootBundle.loadString('assets/licenses/NOTICE'));
  yield* Stream.fromIterable(
    rustLicenses(await rootBundle.loadString('assets/licenses/rust.txt')),
  );
  final web = await rootBundle.loadString(
    'assets/score-view-web/licenses.json',
  );
  for (final p in (jsonDecode(web) as List).cast<Map<String, dynamic>>()) {
    final text = p['text'] as String? ?? p['identifier'] as String;
    yield LicenseEntryWithLineBreaks([p['name'] as String], text);
  }
}

/// Entries of `about.hbs`: crate names, a blank line, the license text.
Iterable<LicenseEntry> rustLicenses(String text) sync* {
  for (final entry in text.replaceAll('\r\n', '\n').split(_separator)) {
    final blank = entry.indexOf('\n\n');
    if (blank < 0) continue;
    final crates = entry.substring(0, blank).trim().split('\n');
    yield LicenseEntryWithLineBreaks(crates, entry.substring(blank + 2));
  }
}
