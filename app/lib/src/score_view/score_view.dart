import 'dart:async';
import 'dart:convert';

import 'package:flutter/material.dart';

import 'score_web_view.dart';

/// Embeds score-view-web, sends it the MusicXML from `render_score` and passes
/// [messages] on (contracts/engine-schnittstelle.md#nachrichten-an-score-view-web).
class ScoreView extends StatefulWidget {
  const ScoreView({super.key, required this.musicxml, this.messages});

  final String musicxml;

  /// `position` and `reset` messages; dropped until the page is loaded.
  final Stream<Map<String, dynamic>>? messages;

  @override
  State<ScoreView> createState() => _ScoreViewState();
}

class _ScoreViewState extends State<ScoreView> {
  late final ScoreWebView _view;
  StreamSubscription<Map<String, dynamic>>? _messages;
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
    _messages = widget.messages?.listen((message) {
      if (_pageLoaded) _view.send(jsonEncode(message));
    });
  }

  @override
  void dispose() {
    _messages?.cancel();
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
