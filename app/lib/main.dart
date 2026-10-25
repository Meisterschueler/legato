import 'dart:io';
import 'dart:ui';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:legato/spike/score_view_spike.dart';
import 'package:legato/src/engine/engine_client.dart';
import 'package:legato/src/l10n/generated/app_localizations.dart';
import 'package:legato/src/l10n/locale_resolution.dart';
import 'package:legato/src/practice/practice_screen.dart';
import 'package:legato/src/rust/frb_generated.dart';
import 'package:path_provider/path_provider.dart';

/// `--dart-define=SPIKE_S2=true` starts the WebView spike instead of the app.
const _spikeS2 = bool.fromEnvironment('SPIKE_S2');

Future<void> main() async {
  WidgetsFlutterBinding.ensureInitialized();
  await RustLib.init();
  if (_spikeS2) {
    runApp(const LegatoApp(home: ScoreViewSpike()));
    return;
  }
  final dataDir = await getApplicationSupportDirectory();
  final engine = await EngineClient.start(
    dataDir: dataDir.path,
    resourceDir: await _bundledResources(dataDir.path),
    locale: PlatformDispatcher.instance.locale.toLanguageTag(),
  );
  runApp(LegatoApp(home: PracticeScreen(engine: engine)));
}

/// The engine reads bundled pieces from files; assets are not files on every platform.
Future<String> _bundledResources(String dataDir) async {
  final dir = '$dataDir/resources';
  final hanon = File('$dir/pieces/hanon-01.musicxml');
  await hanon.parent.create(recursive: true);
  await hanon.writeAsString(
    await rootBundle.loadString('assets/pieces/hanon-01.musicxml'),
  );
  return dir;
}

class LegatoApp extends StatelessWidget {
  const LegatoApp({super.key, required this.home});

  final Widget home;

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      onGenerateTitle: (context) => AppLocalizations.of(context).appTitle,
      localizationsDelegates: AppLocalizations.localizationsDelegates,
      supportedLocales: AppLocalizations.supportedLocales,
      localeListResolutionCallback: resolveLocale,
      home: home,
    );
  }
}
