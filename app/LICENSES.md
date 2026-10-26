# Lizenzen der Abhängigkeiten

Jede Abhängigkeit wird vor der Aufnahme geprüft (Constitution, Abschnitt Lizenzen).

Die App zeigt alle Lizenzen unter Einstellungen → Lizenzen (T098): Dart-Pakete sammelt Flutter selbst, die Rust-Crates erzeugt `cargo about` (`app/rust/about.toml`, im CI-Job `generate` nach `assets/licenses/`), die im Notenblatt gebündelten npm-Pakete Vite (`build.license`).

| Paket | Lizenz | Zweck |
|---|---|---|
| Flutter SDK (`flutter`, `flutter_localizations`, `flutter_test`) | BSD-3-Clause | App-Hülle, Mehrsprachigkeit |
| `intl` | BSD-3-Clause | Mehrsprachigkeit (`gen-l10n`) |
| `webview_flutter` | BSD-3-Clause | Notenblatt (`score-view-web`) |
| `webview_windows` | BSD-3-Clause | Notenblatt unter Windows (WebView2) |
| `wakelock_plus` | BSD-3-Clause | Bildschirmsperre während der Übeeinheit |
| `path_provider` | BSD-3-Clause | App-Verzeichnis für die Engine |
| `file_selector` | BSD-3-Clause | Laden von Notendateien |
| `flutter_lints` | BSD-3-Clause | Linting (nur Entwicklung) |
| `flutter_rust_bridge` | MIT | Anbindung der Rust-Engine |
