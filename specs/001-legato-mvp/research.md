# Research: Legato MVP

**Plan**: [plan.md](plan.md) | **Spec**: [spec.md](spec.md)

Jeder Abschnitt folgt dem Schema Entscheidung · Begründung · Verworfene Alternativen. Punkte, die vor der Implementierung mit einem kurzen Prototyp (Spike) bestätigt werden müssen, sind als **Spike** gekennzeichnet; sie stehen am Anfang von `tasks.md`.

## R1 – App-Framework

**Entscheidung**: [Tauri 2](https://tauri.app) mit Rust-Backend und Web-Oberfläche (TypeScript, Svelte 5, Vite).

**Begründung**:
- Eine Codebasis für iOS, Android, Windows, macOS und Linux (NFR-001).
- Der fachliche Kern (Zuordnung, Erkennung, später Bewertung) wird in Rust geschrieben und lässt sich für die Web-App ([008](../008-web-app/spec.md)) unverändert nach WebAssembly übersetzen. So ist Kernverhalten auf allen Plattformen identisch (Constitution VIII).
- Die Oberfläche (Svelte) kann die Web-App später wiederverwenden.
- Rust liefert deterministische, schnelle Verarbeitung ohne Garbage-Collector-Pausen (NFR-003).
- Lizenzen: Tauri MIT/Apache-2.0, Svelte MIT.

**Verworfene Alternativen**:
- *Flutter*: gute Plattformabdeckung, aber kein ausgereifter MusicXML-Renderer in Dart; der Kern müsste für die Web-App doppelt oder über FFI eingebunden werden.
- *Kotlin/Compose Multiplatform*: `javax.sound.midi` erkennt unter Desktop-JVMs neu angeschlossene bzw. getrennte Geräte nicht zuverlässig; kein Notenrenderer.
- *.NET MAUI*: kein Linux.
- *Reine Web-App/PWA*: Web MIDI fehlt in Safari/iOS (Constitution V, NFR-001).

## R2 – MIDI-Ein- und -Ausgabe

**Entscheidung**:
- Windows, macOS, Linux, iOS: Crate [`midir`](https://github.com/Boddlnagg/midir) (MIT) im Rust-Backend (WinMM bzw. WinRT, CoreMIDI, ALSA).
- Android: eigenes Tauri-Plugin in Kotlin mit `android.media.midi`; Ereignisse gelangen per JNI direkt in das Rust-Backend.
- Gemeinsame Schnittstelle `MidiPort` im Backend; der Kern sieht nur Ereignisse mit Zeitstempel.

**Begründung**: Web MIDI steht in den Webviews von iOS/macOS (WKWebView), Linux (WebKitGTK) und Android nicht zuverlässig zur Verfügung; deshalb läuft MIDI nativ. `midir` unterstützt Android nicht.

**Verbindungsüberwachung (FR-005, FR-006, SC-003)**:
- Desktop und iOS: Die Liste der Ports wird alle 250 ms abgefragt. Verschwindet der gewählte Port, gilt die Verbindung als getrennt; erscheint er wieder (gleicher Name), wird automatisch neu verbunden. Damit liegt die Erkennung bei USB deutlich unter 2 s.
- Android: `MidiManager.DeviceCallback` (`onDeviceAdded`/`onDeviceRemoved`) liefert Ereignisse direkt.
- Bluetooth-MIDI: Android kann ein bekanntes Gerät über `openBluetoothDevice` selbständig neu öffnen. Unter iOS muss Bluetooth-MIDI über `CABTMIDICentralViewController` verbunden werden; ob CoreMIDI die Verbindung selbst wiederherstellt, ist unklar. Gelingt es nicht, greift der manuelle Weg über das Symbol (FR-006, Szenario 15).
- **Spike S1**: Erkennung von Trennung und Wiederverbindung je Plattform (USB, unter iOS und Android zusätzlich Bluetooth) messen.

**Verworfene Alternativen**: Web MIDI im Webview (siehe oben); `jni`-Aufrufe aus Rust ohne Kotlin-Plugin (umständlicher, kein Zugriff auf Android-Lebenszyklus).

## R3 – Warnung über den Lautsprecher

**Entscheidung**: Wiedergabe im Rust-Backend mit [`rodio`](https://github.com/RustAudio/rodio) (MIT/Apache-2.0, basiert auf `cpal`; unterstützt Desktop, iOS und Android). Der Warnton ist eine mitgelieferte, kurze Audiodatei. Unter iOS wird die Audio-Session auf `playback` mit `mixWithOthers` gesetzt.

**Begründung**: Webviews spielen Ton auf iOS erst nach einer Nutzerinteraktion; nach dem automatischen Wiederherstellen der Eingabe beim Start (FR-002) gäbe es keine. Natives Abspielen hat dieses Problem nicht.

**Verworfene Alternativen**: Web Audio im Webview (Interaktionspflicht); Systembenachrichtigungen (widerspricht FR-026/SC-005).

## R4 – Bildschirmsperre und Hintergrund (NFR-005)

**Entscheidung**:
- Desktop: Crate `keepawake` (MIT/Apache-2.0): `SetThreadExecutionState` (Windows), IOPMAssertion (macOS), `org.freedesktop.ScreenSaver.Inhibit` (Linux).
- Mobil: kleines Tauri-Plugin: Android `FLAG_KEEP_SCREEN_ON`, iOS `isIdleTimerDisabled`.
- Aktiv, solange eine Eingabe verbunden ist.
- Wechsel in den Hintergrund: `visibilitychange` im Webview sowie die Lebenszyklus-Ereignisse von Tauri werden an den Kern gemeldet; ein laufender Durchlauf wird verworfen, beim Zurückkehren erscheint ein Hinweis.

## R5 – Notendateien

**Entscheidung**: MusicXML 4.0 (`.musicxml`, `.xml`) und komprimiert (`.mxl`). Eigener Parser im Kern auf Basis von [`quick-xml`](https://github.com/tafia/quick-xml) (MIT) und `zip` (MIT), beschränkt auf die benötigte Teilmenge ([contracts/notendatei.md](contracts/notendatei.md)).

**Begründung**: MusicXML ist das verbreitete Austauschformat (MuseScore, Finale, Dorico, Sibelius) und enthält Fingersatz und Wiederholungszeichen. Ein schlanker eigener Parser hält den Kern klein und portabel (auch nach WebAssembly).

**Verworfene Alternativen**: MIDI-Dateien (kein Fingersatz, keine Wiederholungszeichen, keine Notation); MEI (wenig verbreitet).

## R6 – Notendarstellung

**Entscheidung**: [OpenSheetMusicDisplay](https://opensheetmusicdisplay.org) (OSMD, BSD-3-Clause, basiert auf VexFlow, MIT) im Webview.
- Der Kern erzeugt für jede Kombination aus Tonart und Anzeigevariante ein eigenes MusicXML (transponiert bzw. gekürzt mit *simile*); OSMD rendert nur.
- Positions- und Fehlermarkierungen liegen als eigene SVG-Ebene über dem Notenbild. Die Zuordnung von Kern-Noten zu Grafikelementen erfolgt über eine stabile Noten-ID (Takt, Stimme, Index), die der Kern in das erzeugte MusicXML schreibt (`<note id="…">`).
- Fließende Bewegungen (FR-023): Positionsmarke und Scrollen werden mit `requestAnimationFrame` und Interpolation animiert, Farbwechsel mit CSS-Übergängen. Scrollen so, dass beim Erreichen des Zeilenendes die nächste Zeile vollständig sichtbar ist (Szenario 6).
- Wechsel der Anzeigevariante (Szenario 18): Neu rendern im Hintergrund, dann Überblenden.

**Begründung**: OSMD rendert MusicXML direkt und ist lizenzrechtlich unproblematisch.

**Verworfene Alternativen**: Verovio (bessere Notensatzqualität, aber LGPL-3.0; Einbindung als WebAssembly lizenzrechtlich zu prüfen); eigene Darstellung (zu aufwendig).

**Spike S2**: OSMD-Rendering von Hanon Nr. 1 in allen vier Anzeigevarianten und flüssiges Scrollen mit 60 fps auf einem Android-Mittelklassegerät.

## R7 – Zuordnung während des Spiels (FR-012 bis FR-021, FR-032)

**Entscheidung**: Pipeline in Anlehnung an [Matchmaker](https://github.com/pymatchmaker/matchmaker) ([Park et al., 2025](../../docs/literatur.md)): `Ereignisstrom` → `Vorverarbeitung` → `Folgeverfahren`. Folgeverfahren sind austauschbar (Trait `Folgeverfahren`, FR-032). Matchmaker selbst ist Python und dient nur als Vorbild; die Verfahren werden in Rust portiert.

- **Standardverfahren**: Hidden-Markov-Modell über die Anschlagszeitpunkte des Notentexts (Zustand = Position, ein Akkord = Menge gleichzeitig zu spielender Töne beider Hände) nach dem *Outer-Product-HMM* von [Nakamura et al., 2014](../../docs/literatur.md). Fehlermodell für falsche Töne, Zusatztöne und Auslassungen; Sprünge nur entlang der Wiederholungskanten des Notentexts (FR-014). Tempo spielt keine Rolle (FR-017): Übergänge hängen von Tonhöhen ab, nicht von Zeit.
- **Zweites Verfahren**: Online Time Warping auf symbolischer Ebene nach Dixon bzw. Arzt (Experteneinstellung).
- **Sustain-Pedal** (FR-003): Controller 64 wird in der Vorverarbeitung verworfen.
- **Eigene Signaltöne** (FR-030): Die Vorverarbeitung verwirft eingehende Note-On-Ereignisse, deren Tonhöhe die App in den letzten 300 ms selbst gesendet hat.

**Erkennung (FR-016, FR-018, SC-002)**: Für jede zugelassene Kombination aus Stück, Variation und Tonart läuft ein eigener Kandidat (vgl. `arzt_multi_fold` in Matchmaker). Hanon Nr. 1: 12 Tonarten × 3 Variationen = 36 Kandidaten, je ca. 230 Positionen; der Aufwand pro Ton ist gering. Der Durchlauf beginnt, sobald die a-posteriori-Wahrscheinlichkeit des besten Kandidaten einen Schwellenwert übersteigt und deutlich vor dem zweitbesten liegt; spätestens nach der Erkennungslänge wird entschieden. Die bis dahin gespielten Töne gehören zum Durchlauf. Ohne eindeutigen Kandidaten entsteht kein Durchlauf.

**Einhändiges Spiel (FR-019)**: Variationen „nur linke Hand“ und „nur rechte Hand“ sind eigene Kandidaten, deren Notentext nur eine Hand enthält. Ohne Einschränkung bestimmt die Werkregel (Hanon: Lage) die Hand.

**Abbruch (FR-020, FR-021)**: Zeitgeber im Kern über die Zeitstempel der Ereignisse (deterministisch, testbar mit aufgezeichneten Daten). Zuordnungsverlust: Unter den letzten *n* Tönen (*n* = Erkennungslänge) sind weniger als 30 % zugeordnet.

**Startwerte** (laut Spec durch Ausprobieren anzupassen): Grenzzeit für Pausen 4 s, Obergrenze an notierten Pausen 10 s, Zuordnungsverlust unter 30 %.

**Spike S3**: Standardverfahren an 10 aufgezeichneten Durchläufen von Hanon Nr. 1 (mit eingestreuten Fehlern) gegen SC-001 (≥ 99 %) und SC-002 prüfen.

## R8 – Latenz (NFR-003)

**Budget** (95. Perzentil, Ziel ≤ 50 ms):

| Abschnitt | Budget |
|---|---|
| MIDI-Treiber bis Backend (USB) | ≤ 5 ms |
| Kern: Vorverarbeitung und Zuordnung | ≤ 2 ms |
| IPC Backend → Webview (Tauri-Event) | ≤ 5 ms |
| Nächstes Bild (60 Hz) inkl. Animation | ≤ 17 ms |
| Reserve (Bluetooth-MIDI: +10–30 ms) | ≈ 20 ms |

**Messung**: In Entwicklungs-Builds trägt jedes Ereignis den Zeitstempel des MIDI-Treibers; die Oberfläche protokolliert den Zeitpunkt des ersten Bildes, das die neue Position zeigt.

## R9 – Werkregeln (FR-009)

**Entscheidung**: Werkregeln sind Rust-Code im Kern (Trait `Werkregel`), je Werkspezifikation ein Modul (`werke::hanon`). Welches Werk eine Datei ist, ergibt sich aus den MusicXML-Metadaten (`<creator type="composer">`, `<work-title>`, `<movement-number>`). Unbekannte Werke erhalten allgemeine Regeln: keine Transposition, Durchlauf = gesamtes Stück (Wiederholungen gemäß Notentext), Erkennungslänge 8 Töne je Hand, nur die Anzeigevariante „komplett“.

**Begründung**: Die Regeln der Werkspezifikationen sind algorithmisch (Transposition, Lage, Anzeigevarianten) und lassen sich nicht sinnvoll als Daten beschreiben (Einfachheit).

## R10 – Persistenz

**Entscheidung**: Einstellungen als JSON über `tauri-plugin-store` (MIT/Apache-2.0). Geladene Notendateien werden in das App-Datenverzeichnis kopiert und gehören damit zum Repertoire. Hanon Nr. 1 wird als Ressource mitgeliefert (FR-010). Durchläufe werden in 001 nicht dauerhaft gespeichert (folgt in [003](../003-speichern/spec.md)).

## R11 – Mehrsprachigkeit (NFR-006)

**Entscheidung**: Texte in JSON-Ressourcen je Sprache (`en.json`, `de.json`), Bibliothek `svelte-i18n` (MIT). Sprache aus der Regionseinstellung des Geräts (`tauri-plugin-os`, `locale()`), z. B. `de-AT` → Deutsch. Englisch als Fallback für nicht unterstützte Sprachen und für einzelne fehlende Texte.

## R12 – Repositories

**Entscheidung**: Vorerst ein Monorepo mit Cargo- und npm-Workspace. Eine Aufteilung auf mehrere Repositories wird mit dem Backend ([007](../007-konto-backend/spec.md)) neu bewertet.

**Begründung**: Einfachheit; Kern und App ändern sich im MVP gemeinsam.

## R13 – Tests

**Entscheidung**:
- Kern: `cargo test` mit Referenz-Durchläufen (Standard-MIDI-Dateien und von Hand erstellte Referenz-Zuordnung, [contracts/referenzdaten.md](contracts/referenzdaten.md)). Ein Testlauf berechnet die Messgrößen für SC-001 und SC-002.
- Backend: `cargo test` mit simuliertem `MidiPort` (Trennen, Wiederverbinden, Echo), Messung von SC-003.
- Oberfläche: Vitest für Logik, Playwright gegen den Vite-Entwicklungsserver mit simuliertem Backend für Ansichten und Symbol.
- Manuelle Abnahme auf Geräten gemäß [quickstart.md](quickstart.md).
