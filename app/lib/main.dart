import 'package:flutter/material.dart';

void main() {
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
