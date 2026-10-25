import 'package:flutter/widgets.dart';

/// NFR-006: the first preferred language that is supported, regardless of region;
/// English otherwise.
Locale resolveLocale(List<Locale>? preferred, Iterable<Locale> supported) {
  for (final locale in preferred ?? const <Locale>[]) {
    for (final candidate in supported) {
      if (candidate.languageCode == locale.languageCode) return candidate;
    }
  }
  return const Locale('en');
}
