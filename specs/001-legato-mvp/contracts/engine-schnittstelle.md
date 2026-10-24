# Contract: Engine-Schnittstelle

Framework-neutrale Schnittstelle von `legato-engine` (`engine/src/api.rs`). Flutter bindet sie über `flutter_rust_bridge` an; ein späterer Tauri-Adapter bildet dieselben Funktionen auf Tauri-Befehle und -Ereignisse ab. Alle Typen sind serialisierbar (serde). Die Hülle enthält keine Fachlogik.

## Lebenszyklus

| Funktion | Parameter | Anforderung |
|---|---|---|
| `start` | `{ datenverzeichnis, ressourcenverzeichnis, sprache }` | Einstellungen laden, zuletzt gewählte Eingabe wiederherstellen (FR-002) |
| `lebenszyklus` | `{ vordergrund: bool }` | NFR-005: verwirft laufenden Durchlauf beim Wechsel in den Hintergrund |
| `ereignisse` | – → Strom von `Ereignis` | siehe unten |

## Befehle

| Funktion | Parameter | Antwort | Anforderung |
|---|---|---|---|
| `eingaben_auflisten` | – | `{ midi: [{ id, name, art: Usb \| Bluetooth, verfuegbar }], gewaehlt? }` | FR-007 |
| `eingabe_waehlen` | `{ midi: id }` \| `Keine` | `Eingabezustand` | FR-002; verwirft laufenden Durchlauf |
| `bluetooth_suchen` | `{ an: bool }` | – (Funde als Ereignis) | FR-006, US 1/15 |
| `warnung_bestaetigen` | – | – | FR-005 |
| `neu_verbinden` | – | `Eingabezustand` | FR-006 |
| `stueck_laden` | `{ pfad }` | `StueckInfo` \| `NotendateiUngueltig { grund }` | FR-008 |
| `repertoire` | – | `[StueckInfo]` | US 2 |
| `notenbild` | `{ stueck, transposition, variation, anzeigevariante }` | MusicXML (String) mit Noten-IDs | FR-011 |
| `einstellungen_lesen` / `einstellungen_schreiben` | `Einstellungen` | `Einstellungen` | [data-model.md](../data-model.md#einstellungen-persistiert) |

`StueckInfo`: `{ id, werk, titel, nummer?, hat_fingersatz, anzeigevarianten: [..] }`

## Ereignisse

| Ereignis | Nutzdaten | Anforderung |
|---|---|---|
| `Eingabezustand` | `{ zustand: NichtVerbunden \| MidiVerbunden \| Abgebrochen, geraet?, eingaben_kommen_an, warnung_aktiv }` | FR-005, FR-007 |
| `BluetoothGefunden` | `{ id, name }` | FR-006 |
| `DurchlaufErkannt` | `{ stueck, variation, transposition }` | FR-016 |
| `Position` | `{ noten_ids: [..], zeit_treiber_us }` | US 3; Latenzmessung (R9) |
| `Abweichung` | `{ art, noten_id?, tonhoehe? }` | FR-025, nur wenn Markierung während des Spiels an |
| `DurchlaufBeendet` | `{ ergebnis: Abgeschlossen \| Abgebrochen, grund?, abweichungen: [..] }` | FR-025, US 4 |
| `HintergrundHinweis` | `{ durchlauf_verworfen }` | US 1/8 |

## Nachrichten an `notenblatt-web`

Die Hülle reicht Notenbild und Ereignisse unverändert als JSON per `postMessage` weiter:

| Nachricht | Inhalt |
|---|---|
| `laden` | `{ musicxml, anzeigevariante }` |
| `position` | `{ noten_ids }` |
| `markieren` | `{ noten_id, art }` |
| `zuruecksetzen` | – |

Rückmeldungen des Bundles: `bereit`, `bild_gezeigt { zeit_treiber_us }` (nur Entwicklungs-Builds, Latenzmessung).

## Regeln

- Ereignisse werden in der Reihenfolge ihres Entstehens zugestellt.
- Während eines Durchlaufs sendet die Engine nur die in FR-024 erlaubten Ereignisse.
