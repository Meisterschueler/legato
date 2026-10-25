import 'dart:convert';

import 'package:flutter/material.dart';

import 'score_web_view.dart';

/// Embeds score-view-web and sends it the MusicXML from `render_score`
/// (contracts/engine-schnittstelle.md#nachrichten-an-score-view-web).
class ScoreView extends StatefulWidget {
  const ScoreView({super.key, required this.musicxml});

  final String musicxml;

  @override
  State<ScoreView> createState() => _ScoreViewState();
}

class _ScoreViewState extends State<ScoreView> {
  late final ScoreWebView _view;
  bool _pageLoaded = false;

  @override
  void initState() {
    super.initState();
    _view = ScoreWebView(
      onPageLoaded: () {
        _pageLoaded = true;
        _load();
      },
    );
  }

  @override
  void dispose() {
    _view.dispose();
    super.dispose();
  }

  @override
  void didUpdateWidget(ScoreView oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (_pageLoaded && oldWidget.musicxml != widget.musicxml) _load();
  }

  Future<void> _load() =>
      _view.send(jsonEncode({'type': 'load', 'musicxml': widget.musicxml}));

  @override
  Widget build(BuildContext context) => _view.build();
}
