import 'package:flutter/material.dart';
import 'package:legato/spike/score_view_spike.dart';
import 'package:legato/src/rust/frb_generated.dart';

/// `--dart-define=SPIKE_S2=true` starts the WebView spike instead of the app.
const _spikeS2 = bool.fromEnvironment('SPIKE_S2');

Future<void> main() async {
  await RustLib.init();
  runApp(const LegatoApp());
}

class LegatoApp extends StatelessWidget {
  const LegatoApp({super.key});

  @override
  Widget build(BuildContext context) {
    return const MaterialApp(
      title: 'Legato',
      home: _spikeS2 ? ScoreViewSpike() : Scaffold(body: Center(child: Text('Legato'))),
    );
  }
}
