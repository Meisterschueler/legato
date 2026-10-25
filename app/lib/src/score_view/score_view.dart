import 'dart:convert';

import 'package:flutter/material.dart';
import 'package:webview_flutter/webview_flutter.dart';

/// Embeds score-view-web and sends it the MusicXML from `render_score`
/// (contracts/engine-schnittstelle.md#nachrichten-an-score-view-web).
class ScoreView extends StatefulWidget {
  const ScoreView({super.key, required this.musicxml});

  final String musicxml;

  @override
  State<ScoreView> createState() => _ScoreViewState();
}

class _ScoreViewState extends State<ScoreView> {
  late final WebViewController _controller;
  bool _pageLoaded = false;

  @override
  void initState() {
    super.initState();
    _controller = WebViewController()
      ..setJavaScriptMode(JavaScriptMode.unrestricted)
      ..setNavigationDelegate(
        NavigationDelegate(
          onPageFinished: (_) {
            _pageLoaded = true;
            _load();
          },
        ),
      )
      ..loadFlutterAsset('assets/score-view-web/index.html');
  }

  @override
  void didUpdateWidget(ScoreView oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (_pageLoaded && oldWidget.musicxml != widget.musicxml) _load();
  }

  Future<void> _load() {
    final message = jsonEncode({'type': 'load', 'musicxml': widget.musicxml});
    return _controller.runJavaScript('legatoReceive(${jsonEncode(message)})');
  }

  @override
  Widget build(BuildContext context) => WebViewWidget(controller: _controller);
}
