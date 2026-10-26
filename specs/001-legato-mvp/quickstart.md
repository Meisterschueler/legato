# Quickstart: Legato MVP validieren

**Plan**: [plan.md](plan.md) | **Spec**: [spec.md](spec.md)

## Voraussetzungen

- Rust (stable) mit `cargo`, Ziele für iOS und Android (`rustup target add aarch64-apple-ios aarch64-apple-ios-sim aarch64-linux-android`)
- Flutter (stable) mit `flutter_rust_bridge_codegen`
- Node.js (LTS) für `score-view-web`
- Xcode (iOS, macOS; nur auf macOS), Android Studio mit NDK
- Windows: Visual Studio mit C++-Workload und Windows-SDK, WebView2-Runtime (in Windows 11 enthalten), Entwicklermodus (Flutter-Plugins brauchen symbolische Links)
- Nur zum Neuerzeugen der Matchmaker-Erwartungsdateien: `uv`
- Für manuelle Abnahme: Digitalpiano mit MIDI-Ein- und -Ausgang (USB, optional Bluetooth), Windows-Rechner, iPad

## Automatische Prüfungen

```powershell
cargo test -p legato-core                  # Matchmaker-Vergleich, Werkregeln, Referenzdaten (SC-001, SC-002)
cargo test -p legato-engine                # simulierter MIDI-Port: Trennen, Wiederverbinden, Echo, Signale (SC-003)
npm --prefix score-view-web test           # Notenblatt (Vitest)
flutter test app                           # Bildschirme, Adapter
```

Erwartet: Alle Tests grün; der Bericht von `legato-core` weist für SC-001 ≥ 99 % aus. Plattformübergreifend läuft derselbe Kern-Testsatz in der CI auf iOS-Simulator, Android-Emulator, Linux, Windows und WebAssembly (Toleranzen laut [research.md R2](research.md#r2--gemeinsamer-kern-in-rust)).

Matchmaker-Erwartungsdateien neu erzeugen (nur nach Änderung des Commits):

```powershell
uv run --project tools/matchmaker-reference tools/matchmaker-reference/generate.py
```

## App starten

```powershell
npm --prefix score-view-web run build      # Bundle nach app/assets
cd app
flutter run -d ipad                        # iOS (nur auf macOS)
flutter run -d android
flutter run -d windows                     # bzw. macos, linux
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
| 14 | Latenzprotokoll (Entwicklungs-Build) auswerten; unter Windows nur per USB (research.md R4) | 95 % ≤ 50 ms, Maximum ≤ 100 ms | NFR-003 |
| 15 | iPad mit Bluetooth-MIDI verbinden, Piano aus- und wieder einschalten | Warnung; nach dem Einschalten automatische Verbindung und Verbindungssignal ohne Interaktion | US 1/1–3, FR-006 |
| 16 | Wie 15 auf Android (mind. 2 Hersteller) | wie 15, sonst manueller Weg über das Symbol in höchstens 2 Tippaktionen | US 1/15 |
| 17 | Gerät im Flugmodus: App starten, Datei laden, Hanon Nr. 1 spielen | alles funktioniert ohne Einschränkung | NFR-002, US 3/16 |
| 18 | „Bewegung reduzieren“ am Gerät einschalten, spielen | Positionsmarke und Scrollen überblenden statt sich zu bewegen | FR-023 |
| 19 | Verbindung trennen und nicht wiederherstellen | Warnung sofort, nach 10 s und nach 30 s (iPhone/Android zusätzlich Vibration), danach nur noch das Symbol | FR-005 |
| 20 | Wie 15 unter Windows mit einem in Windows gekoppelten Bluetooth-MIDI-Gerät | wie 15, sonst manueller Weg über das Symbol in höchstens 2 Tippaktionen | US 1/15, FR-006 |
| 21 | In den Experteneinstellungen nacheinander jedes Verfahren für die Erkennung wählen und Hanon Nr. 1 in Es-Dur spielen | jeweils nach höchstens 8 Tönen je Hand erkannt; Wechsel gilt ab der nächsten Erkennung | US 3/15, FR-085 |
