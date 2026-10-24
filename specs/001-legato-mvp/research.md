# Research: Legato MVP

**Plan**: [plan.md](plan.md) | **Spec**: [spec.md](spec.md) | **Constitution**: [1.1.0](../../.specify/memory/constitution.md)

Jeder Abschnitt folgt dem Schema Entscheidung · Begründung · Verworfene Alternativen. Punkte, die vor der Implementierung mit einem kurzen Prototyp (Spike) bestätigt werden müssen, sind als **Spike** gekennzeichnet; sie stehen am Anfang von `tasks.md`.

**Prioritäten**: iOS (insbesondere iPad) vor Android vor Desktop. MIDI per USB auf allen Plattformen vollständig, Bluetooth-MIDI vollständig auf iOS, macOS und Android, auf Windows und Linux nach Möglichkeit.

## R1 – App-Framework

**Entscheidung**: Flutter für Oberfläche und App-Hülle; die gesamte Fachlogik in Rust (R2), angebunden über [`flutter_rust_bridge`](https://github.com/fzyzcjy/flutter_rust_bridge) (MIT). Die Architektur hält einen späteren Wechsel zu Tauri offen (R3).

**Begründung**:
- Flutter ist auf iOS und Android seit Jahren in Produktion erprobt; geringes Ökosystemrisiko für die wichtigste Plattform.
- Eigene Rendering-Engine: fließende Animationen (FR-023) und identisches Erscheinungsbild auf allen Plattformen.
- Desktop (Windows, macOS, Linux) wird ebenfalls unterstützt (NFR-001).
- Lizenzen: Flutter BSD-3-Clause, `flutter_rust_bridge` MIT.

**Verworfene Alternativen**:
- *Tauri 2*: gemeinsame Web-Oberfläche mit der Web-App (008) und fertige Notendarstellung im Webview; Tauri Mobile ist aber jung (stabil seit Ende 2024). Bleibt Ausweichoption (R3).
- *Kern in Dart*: einfacher, aber Dart wird für die Web-App nach JavaScript übersetzt (abweichende Mathematik, 53-Bit-Ganzzahlen), und das Backend soll dieselben Algorithmen ausführen (R2).
- *Python-Matchmaker in der App*: Python, numpy, partitura und Cython-Teile müssten für iOS gebaut werden; App-Größe, Startzeit und Latenz ungewiss.
- *Kotlin/Compose Multiplatform*: `javax.sound.midi` erkennt auf dem Desktop Geräteänderungen nicht zuverlässig; kein Notenrenderer.
- *.NET MAUI*: kein Linux. *Reine Web-App*: kein Web MIDI in Safari/iOS.

## R2 – Gemeinsamer Kern in Rust

**Entscheidung**: Zuordnung, Erkennung, Werkregeln, Notentext und Durchlauf-Steuerung liegen in einem Rust-Crate ohne I/O (`legato-core`). Dasselbe Crate läuft in der App (nativ), im Backend (Rust, bindet das Crate direkt ein; siehe [007](../007-konto-backend/spec.md)) und in der Web-App (WebAssembly).

**Konsistenz über Plattformen** (Constitution VI, VIII): Bitgenauigkeit wird nicht verlangt. Große Abweichungen werden konstruktiv verhindert, kleine numerische Unterschiede durch Toleranzen aufgefangen:
- kein Zufall, keine Uhr im Kern; Zeit nur aus Ereignis-Zeitstempeln;
- keine Iteration über `HashMap`/`HashSet`, sondern `Vec` oder `BTreeMap`;
- Summen in fester Reihenfolge, keine parallele Reduktion;
- feste Regel bei Gleichstand: die kleinste Position gewinnt;
- jede gespeicherte Auswertung trägt die Version des Kerns.

**Toleranzen**: Zwischenwerte (Wahrscheinlichkeiten) relativ 1e-6; Zuordnung: mindestens 99,9 % der Töne plattformübergreifend identisch zugeordnet, keine Abweichung um mehr als eine Position; Durchlauf-Ergebnis (abgeschlossen/abgebrochen, erkannter Kandidat) identisch. Geprüft durch denselben Testsatz auf iOS-Simulator, Android-Emulator, Linux und WebAssembly (R13).

## R3 – Austauschbarkeit des Frameworks (Flutter → Tauri)

**Entscheidung**: Das Framework ist nur eine dünne Hülle. Alles, was bei einem Wechsel erhalten bleiben muss, liegt außerhalb von Flutter:

| Baustein | Ort | Bei Wechsel zu Tauri |
|---|---|---|
| Kern (`legato-core`) | Rust | unverändert |
| Engine (`legato-engine`): Sitzung, Eingabezustand, Signale, Warnung, Einstellungen, Repertoire, Desktop-MIDI | Rust | unverändert |
| Native MIDI-Schicht (iOS/macOS, Android) | Swift-Package, Kotlin-Bibliothek; ruft die Engine direkt über eine C-Schnittstelle auf ([contracts/native-midi.md](contracts/native-midi.md)) | unverändert; nur das Starten wird neu verdrahtet |
| Notenblatt (OSMD, Markierungen, Scrollen) | eigenständiges Web-Bundle (`score-view-web/`, TypeScript), in Flutter per WebView eingebettet | unverändert, läuft direkt im Tauri-Webview |
| Texte (i18n) | ARB-Dateien (JSON) | weiterverwendbar |
| Flutter-Oberfläche (Dart) | nur Darstellung und Bedienung | neu zu schreiben |
| Adapter: `flutter_rust_bridge`, Bildschirmsperre, Lebenszyklus | Flutter | neu zu schreiben (klein) |

**Regeln**:
- Keine Fachlogik in Dart. Dart hält nur Darstellungszustand und ruft die Engine-Schnittstelle ([contracts/engine-schnittstelle.md](contracts/engine-schnittstelle.md)) auf.
- Die Engine-Schnittstelle ist framework-neutral (Rust-Typen, serialisierbar); `flutter_rust_bridge` bzw. später Tauri-Befehle sind reine Adapter.
- Persistenz erledigt die Engine (Dateien in einem Verzeichnis, das die Hülle übergibt), nicht `shared_preferences`.
- MIDI-Ereignisse laufen nie über Dart.

**Begründung**: Der framework-spezifische Anteil beschränkt sich auf die Bildschirme und wenige Adapter; der Wechsel bleibt aufwendig, aber nicht unmöglich.

## R4 – MIDI-Ein- und -Ausgabe

**Entscheidung**: Eigene, schmale native MIDI-Schicht je Plattform; die Ereignisse gehen mit Zeitstempel des Treibers direkt an die Engine.

| Plattform | USB | Bluetooth-MIDI | Wiederverbinden (FR-006) |
|---|---|---|---|
| iOS, macOS | CoreMIDI | CoreBluetooth: Suche nach dem BLE-MIDI-Dienst, Verbinden; danach stellt CoreMIDI das Gerät als Endpunkt bereit | Kennung des Geräts speichern; dauerhafte Verbindungsanfrage über CoreBluetooth (läuft nicht ab), USB über CoreMIDI-Benachrichtigungen |
| Android | `android.media.midi`, `MidiManager.DeviceCallback` | eigene BLE-Suche, `MidiManager.openBluetoothDevice()`; Berechtigungen `BLUETOOTH_SCAN`/`BLUETOOTH_CONNECT` (ab Android 12) | gespeicherte Adresse erneut öffnen |
| Windows, Linux | `midir` (MIT) in der Engine; Abfrage der Portliste alle 250 ms | nach Möglichkeit (Windows: WinRT bzw. Windows MIDI Services; Linux: BlueZ mit MIDI-Unterstützung) | USB automatisch, Bluetooth über das Symbol |

**Bekannte Hindernisse**:
- Bluetooth-Trennung ohne Abmeldung (Strom weg, Reichweite) erkennt das Betriebssystem erst nach dem Überwachungs-Timeout (iOS bis ca. 6 s). SC-003 zählt ab der Meldung des Betriebssystems.
- Android: herstellerabhängiges Verhalten des Bluetooth-Stacks; Tests auf mehreren Geräten nötig.
- Bluetooth-Adapter (z. B. Yamaha MD-BT01, CME WIDI) verhalten sich teils eigen.
- Bluetooth-MIDI hat 10–30 ms zusätzliche Latenz und Jitter von einigen ms. Für NFR-003 unkritisch; für die Messung der Gleichmäßigkeit ([002](../002-bewertung/spec.md)) müssen die Zeitstempel des Instruments erhalten bleiben (CoreMIDI: ja; Android: zu prüfen).

**Spike S1** (vorgezogen, ohne Framework): Minimale native iOS-App mit der MIDI-Schicht und einer Rust-Testbibliothek. Messen: Erkennung von Trennung und Wiederkehr (USB, Bluetooth), automatisches Wiederverbinden, Latenz, Jitter, Weitergabe der Instrument-Zeitstempel. Danach dasselbe für Android.

**Verworfene Alternativen**: `flutter_midi_command` (bindet MIDI an Flutter und Dart; Wiederverbinden nicht steuerbar); `midir` auf iOS (kein Bluetooth-Verbindungsaufbau); Web MIDI (nicht in Safari/WKWebView).

## R5 – Warnung, Bildschirmsperre, Hintergrund

**Entscheidung**:
- Warnung über den Lautsprecher (FR-005): in der Engine mit [`rodio`](https://github.com/RustAudio/rodio) (MIT/Apache-2.0), mitgelieferte kurze Audiodatei. Unter iOS setzt die native Schicht die Audio-Session auf `playback` mit `mixWithOthers`. Kein Nutzer-Tippen nötig, framework-neutral.
- Bildschirmsperre (NFR-005): Flutter-Adapter (`wakelock_plus`), aktiv solange eine Übeeinheit läuft. Die Übeeinheit beginnt mit dem ersten Ton und endet nach einer Ruhezeit ohne Ton (Startwert 10 min, einstellbar), beim Beenden der App oder im Hintergrund (Glossar „Übeeinheit“). Bei Tauri: `keepawake` bzw. kleines Plugin.
- Hintergrund: Flutter `AppLifecycleListener` meldet an die Engine; ein laufender Durchlauf wird verworfen, beim Zurückkehren erscheint ein Hinweis.

## R6 – Notendateien

**Entscheidung**: MusicXML 4.0 (`.musicxml`, `.xml`) und komprimiert (`.mxl`). Eigener Parser im Kern auf Basis von [`quick-xml`](https://github.com/tafia/quick-xml) (MIT) und `zip` (MIT), beschränkt auf die benötigte Teilmenge ([contracts/notendatei.md](contracts/notendatei.md)). Der Parser wird gegen die `note_array` von partitura getestet (R13).

**Verworfene Alternativen**: MIDI-Dateien (kein Fingersatz, keine Wiederholungen); MEI (wenig verbreitet).

## R7 – Notendarstellung

**Entscheidung**: Eigenständiges Web-Bundle `score-view-web/` (TypeScript) mit [OpenSheetMusicDisplay](https://opensheetmusicdisplay.org) (BSD-3-Clause), in Flutter über `webview_flutter` eingebettet (iOS, Android, macOS).
- Der Kern erzeugt für jede Kombination aus Tonart und Anzeigevariante ein MusicXML mit stabilen Noten-IDs; OSMD rendert nur.
- Markierungen als SVG-Ebene über dem Notenbild; Positionsmarke und Scrollen mit `requestAnimationFrame`, Farbwechsel mit CSS-Übergängen (FR-023).
- Kommunikation Dart ↔ Web-Bundle über eine kleine Nachrichtenschnittstelle (`postMessage`); dieselbe Schnittstelle nutzt später Tauri.

**Begründung**: Für Flutter gibt es keinen ausgereiften MusicXML-Renderer; das Web-Bundle ist zugleich der wichtigste wiederverwendbare Baustein für Tauri und die Web-App.

**Verworfene Alternativen**: Verovio (LGPL-3.0; statisches Linken auf iOS problematisch); eigene Darstellung in Flutter (zu aufwendig über Hanon hinaus).

**Spike S2**: Hanon Nr. 1 in allen vier Anzeigevarianten im WebView auf iPad und Android-Mittelklasse; flüssiges Scrollen mit 60 fps; Latenz der Nachrichten Dart → WebView. Windows/Linux: geeignetes WebView-Paket prüfen (niedrige Priorität).

## R8 – Zuordnung während des Spiels: Portierung aus Matchmaker

**Entscheidung**: Die wichtigsten MIDI-Verfahren aus [Matchmaker](https://github.com/pymatchmaker/matchmaker) ([Park et al., 2025](../../docs/literatur.md); Apache-2.0) werden nach Rust portiert, gegen einen festgelegten Matchmaker-Commit:

| Verfahren | Matchmaker | Quelle | Rolle |
|---|---|---|---|
| Pitch-HMM | `pthmm` | Matchmaker-Standard für MIDI | Kandidat für Standard |
| Outer-Product-HMM | `outerhmm` | [Nakamura et al., 2014](../../docs/literatur.md) | Kandidat für Standard (robust bei Akkorden, Fehlern) |
| Online Time Warping | `arzt` | Arzt/Widmer, Brazier/Widmer | Experteneinstellung; Grundlage für parallel verfolgte Kandidaten (`arzt_multi_fold`) |

Weitere Verfahren (`hmm`, `dixon`) bei Bedarf später. Das Standardverfahren wird nach Spike S3 festgelegt (FR-032).

**Pipeline** wie bei Matchmaker: Ereignisstrom → Vorverarbeitung (`Preprocessor`, bei Matchmaker `Processor`) → Folgeverfahren (Trait `ScoreFollower`, bei Matchmaker `OnlineAlignment`); Verfahren austauschbar.

**Kompatibler Modus und Erweiterungen**: Jedes portierte Verfahren hat einen kompatiblen Modus, der Matchmaker nachbildet (Wiederholungen aufgelöst, eine Partitur). Darauf bauen die Legato-Erweiterungen auf:
- Erkennung über parallel verfolgte Kandidaten aus Stück, Variation und Tonart (FR-016, FR-018); Hanon Nr. 1: 36 Kandidaten;
- Wiederholungen als Kanten im Zustandsgraph statt Auflösung (FR-014);
- Einordnung in Auslassung, falschen Ton und Zusatzton (FR-012, FR-013);
- Verwerfen von Pedal (FR-003) und Echo eigener Signaltöne (FR-030, 300-ms-Fenster);
- Abbruch über Grenzzeiten und Zuordnungsverlust (FR-020, FR-021).

**Startwerte** (laut Spec durch Ausprobieren anzupassen): Grenzzeit für Pausen 4 s, Obergrenze an notierten Pausen 10 s, Zuordnungsverlust unter 30 % der letzten *n* Töne. *n* ist die Erkennungslänge über alle gespielten Hände: bei Hanon 8 Töne je Hand, also beidhändig 16 und einhändig 8.

**Lizenz**: Portierter Code trägt einen Herkunftshinweis; `NOTICE` mit Verweis auf Matchmaker (Apache-2.0).

**Spike S3**: `pthmm` und `outerhmm` im kompatiblen Modus portieren, gegen die Matchmaker-Referenz prüfen (R13), dann mit Erweiterungen an 10 aufgezeichneten Durchläufen von Hanon Nr. 1 gegen SC-001 (≥ 99 %) und SC-002 messen.

## R9 – Latenz (NFR-003)

**Budget** (95. Perzentil, Ziel ≤ 50 ms):

| Abschnitt | Budget |
|---|---|
| MIDI-Treiber bis Engine (USB, native Schicht, C-Aufruf) | ≤ 5 ms |
| Kern: Vorverarbeitung und Zuordnung | ≤ 2 ms |
| Engine → Dart (`flutter_rust_bridge`-Stream) | ≤ 3 ms |
| Dart → WebView (`postMessage`) | ≤ 5 ms |
| Nächstes Bild (60 Hz) | ≤ 17 ms |
| Reserve (Bluetooth: +10–30 ms) | ≈ 18 ms |

**Messung**: In Entwicklungs-Builds trägt jedes Ereignis den Zeitstempel des Treibers; das Web-Bundle meldet den Zeitpunkt des ersten Bildes mit der neuen Position.

## R10 – Werkregeln (FR-009)

**Entscheidung**: Werkregeln sind Rust-Code im Kern (Trait `WorkRules`), je Werkspezifikation ein Modul (`works::hanon`). Das Werk ergibt sich aus den MusicXML-Metadaten (`<creator type="composer">`, `<work-title>`, `<movement-number>`). Unbekannte Werke erhalten allgemeine Regeln: keine Transposition, Durchlauf = gesamtes Stück, Erkennungslänge 8 Töne je Hand, nur Anzeigevariante „komplett“.

## R11 – Persistenz

**Entscheidung**: Die Engine speichert Einstellungen als JSON und kopiert geladene Notendateien in ein Verzeichnis, das die Hülle übergibt (Flutter: `path_provider`). Hanon Nr. 1 wird als Ressource mitgeliefert (FR-010). Durchläufe werden in 001 nicht dauerhaft gespeichert (folgt in [003](../003-speichern/spec.md)).

## R12 – Mehrsprachigkeit (NFR-006)

**Entscheidung**: ARB-Dateien (`app_en.arb`, `app_de.arb`) mit Flutter `gen-l10n`; Sprache aus der Regionseinstellung des Geräts, Englisch als Fallback für nicht unterstützte Sprachen und einzelne fehlende Texte. Das Notenblatt-Bundle enthält nur wenige Texte (z. B. *simile*, das nicht übersetzt wird).

## R13 – Tests

**Matchmaker-Referenz (Verfahren 3)**:
- Testdaten aus Matchmaker übernehmen (`EXAMPLE_MATCH`, `EXAMPLE_PIECES`: u. a. `simple_mozart`, `bach_fugue`; Apache-2.0, mit Herkunftsangabe).
- Ein Python-Werkzeug (`tools/matchmaker-reference/`, festgelegter Matchmaker-Commit, `uv`) erzeugt Erwartungsdateien: `note_array` von partitura, Zwischenwerte (Beobachtungswahrscheinlichkeiten, Übergangsmatrix) und Position je Eingabeereignis ([contracts/matchmaker-referenz.md](contracts/matchmaker-referenz.md)).
- Die MIDI-Tests von Matchmaker (`test_prob_hmm.py`, `test_prob_outer_product_hmm.py`, `test_oltw_arzt.py`) werden 1:1 nach Rust übertragen (gleiche Namen und Prüfungen).
- Rust-Tests lesen die `note_array` aus der Erwartungsdatei, damit Unterschiede im MusicXML-Parser den Vergleich nicht stören; der Parser wird separat gegen partitura getestet.
- Vergleich im kompatiblen Modus: Zwischenwerte relativ 1e-6; Positionen ≥ 99 % identisch, Abweichung höchstens eine Position.
- CI benötigt kein Python; die Erwartungsdateien werden nur beim Aktualisieren des Matchmaker-Commits neu erzeugt.

**Eigene Referenzdaten**: Legato-Erweiterungen werden gegen eigene Referenz-Durchläufe getestet ([contracts/referenzdaten.md](contracts/referenzdaten.md)); daraus SC-001 und SC-002.

**Plattformübergreifend**: Derselbe Testsatz läuft auf iOS-Simulator, Android-Emulator, Linux und WebAssembly; Vergleich mit den Toleranzen aus R2.

**Weitere Ebenen**: Engine mit simuliertem MIDI-Port (Trennen, Wiederverbinden, Echo, SC-003); `flutter test` für Widgets; Vitest für `score-view-web`; manuelle Abnahme laut [quickstart.md](quickstart.md).

## R14 – Repositories

**Entscheidung**: Vorerst ein Monorepo (Cargo-Workspace, Flutter-App, Web-Bundle, Werkzeuge). Eine Aufteilung wird mit dem Backend ([007](../007-konto-backend/spec.md)) neu bewertet.
