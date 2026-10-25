import 'dart:async';
import 'dart:convert';
import 'dart:io';

import 'package:flutter/material.dart';
import 'package:webview_flutter/webview_flutter.dart';
import 'package:webview_windows/webview_windows.dart' as win;

const _page = 'assets/score-view-web/index.html';

/// score-view-web in the WebView of the platform: `webview_flutter`, on Windows
/// `webview_windows` (research.md R7). Messages are JSON strings both ways
/// (contracts/engine-schnittstelle.md#nachrichten-an-score-view-web).
class ScoreWebView {
  ScoreWebView({
    required void Function() onPageLoaded,
    void Function(String json)? onMessage,
  }) {
    if (Platform.isWindows) {
      final controller = win.WebviewController();
      _windows = controller;
      _subscriptions = [
        controller.loadingState.listen((state) {
          if (state == win.LoadingState.navigationCompleted) onPageLoaded();
        }),
        controller.webMessage.listen((m) => onMessage?.call(m as String)),
      ];
      // The page is a single file (score-view-web/vite.config.ts), so file:// works.
      final url = File(Platform.resolvedExecutable).parent.uri
          .resolve('data/flutter_assets/$_page');
      controller.initialize().then((_) => controller.loadUrl(url.toString()));
    } else {
      final controller = WebViewController()
        ..setJavaScriptMode(JavaScriptMode.unrestricted)
        ..setNavigationDelegate(
          NavigationDelegate(onPageFinished: (_) => onPageLoaded()),
        );
      if (onMessage != null) {
        controller.addJavaScriptChannel(
          'LegatoHost',
          onMessageReceived: (m) => onMessage(m.message),
        );
      }
      controller.loadFlutterAsset(_page);
      _flutter = controller;
    }
  }

  WebViewController? _flutter;
  win.WebviewController? _windows;
  List<StreamSubscription<Object?>> _subscriptions = [];

  /// Calls `legatoReceive` in the page.
  Future<void> send(String json) async {
    final script = 'legatoReceive(${jsonEncode(json)})';
    final windows = _windows;
    if (windows != null) {
      await windows.executeScript(script);
    } else {
      await _flutter!.runJavaScript(script);
    }
  }

  Widget build() {
    final windows = _windows;
    if (windows == null) return WebViewWidget(controller: _flutter!);
    return ValueListenableBuilder(
      valueListenable: windows,
      builder: (_, value, _) =>
          value.isInitialized ? win.Webview(windows) : const SizedBox.expand(),
    );
  }

  void dispose() {
    for (final subscription in _subscriptions) {
      subscription.cancel();
    }
    _windows?.dispose();
  }
}
