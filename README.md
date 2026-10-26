# Legato

Legato ist eine stille Jury für fortgeschrittene Pianisten: Sie hört beim Üben technischer Klassiker konzentriert zu, erkennt systematische Schwächen und gibt auf Nachfrage hilfreiches Feedback.

Das Projekt wird nach Spec-Driven Development entwickelt: Constitution → Spec → Plan → Tasks → Implementierung.

## Dokumente

- [Constitution](.specify/memory/constitution.md): Grundprinzipien, die für alle Specs und Pläne gelten
- [Produktübersicht](docs/produkt.md): Überblick, Abgrenzung und Aufteilung in Features
- [Glossar](docs/glossar.md): Begriffe und Schlüsselwörter für alle Specs
- [Literatur](docs/literatur.md)
- [Specs](specs/): Feature-Spezifikationen, fortlaufend nummeriert
  - [001-legato-mvp](specs/001-legato-mvp/spec.md)
  - [002-bewertung](specs/002-bewertung/spec.md)
  - [003-speichern](specs/003-speichern/spec.md)
  - [004-performance-wurm](specs/004-performance-wurm/spec.md)
  - [005-schwaechen](specs/005-schwaechen/spec.md)
  - [006-audio-eingabe](specs/006-audio-eingabe/spec.md)
  - [007-konto-backend](specs/007-konto-backend/spec.md)
  - [008-web-app](specs/008-web-app/spec.md)
  - [009-playlist](specs/009-playlist/spec.md)
- [Werke](docs/werke/): Fachliche Beschreibung der unterstützten Werke
  - [Hanon](docs/werke/hanon.md)
  - [Brahms](docs/werke/brahms.md)

## Aufbau

| Verzeichnis | Inhalt |
|---|---|
| `core/` | Fachlogik in Rust (`legato-core`): MusicXML, Werkregeln, Erkennung, Folgeverfahren, Durchläufe |
| `engine/` | Laufzeit um den Kern (`legato-engine`): Sitzung, Eingabe, Signale, Einstellungen, Repertoire, Desktop-MIDI, C-Schnittstelle |
| `app/` | Flutter-App als dünne Hülle; Brücke zur Engine in `app/rust/` (`flutter_rust_bridge`) |
| `score-view-web/` | Notenblatt im WebView (TypeScript, Vite) |
| `native/` | MIDI-Schicht für Android (Kotlin) und Apple (Swift) |
| `tools/` | Werkzeuge: MusicXML für Übungen erzeugen, Matchmaker-Referenz |
| `testdata/` | Referenzdaten für die Tests |
| `spikes/` | Messungen aus den Spikes |

## Build und Tests

Voraussetzungen und manuelle Abnahme: [quickstart.md](specs/001-legato-mvp/quickstart.md).

```powershell
cargo test --workspace                     # Kern und Engine
npm --prefix score-view-web test           # Notenblatt
npm --prefix score-view-web run build      # Bundle nach app/assets
cd app; flutter test                       # Bildschirme, Adapter
flutter run -d windows                     # bzw. macos, linux, android, ipad
```

## Lizenz

Lizenziert wahlweise unter der [MIT-Lizenz](LICENSE-MIT) oder der [Apache-Lizenz 2.0](LICENSE-APACHE) (`MIT OR Apache-2.0`). Herkunftshinweise für übernommenen Code stehen in `NOTICE`.
