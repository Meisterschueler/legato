# Quickstart: Legato MVP validieren

**Plan**: [plan.md](plan.md) | **Spec**: [spec.md](spec.md)

## Voraussetzungen

- Rust (stable) mit `cargo`
- Node.js (LTS) mit `npm`
- Systemvoraussetzungen von Tauri 2 je Plattform (Windows: WebView2 und MSVC Build Tools; macOS/iOS: Xcode; Android: Android Studio mit NDK; Linux: WebKitGTK und ALSA-Entwicklungspakete)
- Für manuelle Abnahme: Digitalpiano mit MIDI-Ein- und -Ausgang (USB, optional Bluetooth)

## Automatische Prüfungen

```powershell
cargo test -p legato-core          # Zuordnung, Erkennung, Werkregeln, Referenzdaten (SC-001, SC-002)
cargo test -p legato-app           # Verbindungsüberwachung mit simuliertem MIDI-Port (SC-003), Signale, Echo
npm --prefix app test              # Oberfläche (Vitest)
npm --prefix app run test:e2e      # Ansichten und Eingabe-Symbol (Playwright, simuliertes Backend)
```

Erwartet: Alle Tests grün; der Bericht von `legato-core` weist für SC-001 ≥ 99 % aus.

## App starten

```powershell
npm --prefix app install
npm --prefix app run tauri dev                  # Desktop
npm --prefix app run tauri android dev          # Android-Gerät/Emulator
npm --prefix app run tauri ios dev              # iOS (nur auf macOS)
```

## Manuelle Abnahme

| # | Szenario | Erwartet | Bezug |
|---|---|---|---|
| 1 | App neu installieren, Repertoire öffnen | genau Hanon Nr. 1 | US 2/4 |
| 2 | Symbol antippen, Digitalpiano wählen | Verbindungssignal am Piano; Symbol klein | US 1/9, US 4/9 |
| 3 | Hanon Nr. 1 in D-Dur ohne Auswahl spielen | nach höchstens 8 Tönen je Hand erkannt; Markierung folgt ruhig | US 3/1, SC-002 |
| 4 | Drei Durchläufe nahtlos, dann aufhören | genau ein OK-Signal nach dem Aufhören | US 4/2 |
| 5 | Mitten im Durchlauf 6 s pausieren | Abbruch-Signal | US 3/12, US 4/4 |
| 6 | Nur rechte Hand spielen | Variation „nur rechte Hand“; keine Auslassungen links | US 3/5 |
| 7 | USB-Kabel während des Durchlaufs ziehen | Warnton binnen 2 s, wiederholt; Symbol deutlich; Durchlauf verworfen | US 1/1,2,4, SC-003 |
| 8 | Kabel wieder einstecken | automatische Verbindung, Verbindungssignal | US 1/3 |
| 9 | App während eines Durchlaufs in den Hintergrund | Durchlauf verworfen, Hinweis beim Zurückkehren | US 1/8 |
| 10 | Ungültige Datei laden | verständliche Fehlermeldung, App bedienbar | US 2/2 |
| 11 | Anzeigevariante während des Spiels wechseln | fließender Wechsel, Markierung folgt weiter | US 3/18 |
| 12 | Gerätesprache `de-AT` bzw. `fr-FR` | Oberfläche Deutsch bzw. Englisch | NFR-006 |
| 13 | 10 Minuten spielen ohne Gerät zu berühren | keine Bildschirmsperre | NFR-005 |
| 14 | Latenzprotokoll (Entwicklungs-Build) auswerten | 95 % ≤ 50 ms, Maximum ≤ 100 ms | NFR-003 |
