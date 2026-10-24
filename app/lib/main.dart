import 'package:flutter/material.dart';
import 'package:legato/src/rust/frb_generated.dart';

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
      home: Scaffold(body: Center(child: Text('Legato'))),
    );
  }
}
