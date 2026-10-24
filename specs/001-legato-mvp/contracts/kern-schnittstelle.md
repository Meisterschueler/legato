# Contract: Schnittstelle zwischen Backend und Oberfläche

Die Oberfläche ruft Befehle des Rust-Backends auf (Tauri `invoke`) und abonniert Ereignisse (Tauri `listen`). Alle Nutzdaten sind JSON. Die Web-App ([008](../../008-web-app/spec.md)) implementiert später dieselbe Schnittstelle gegen den Kern als WebAssembly.

## Befehle (Oberfläche → Backend)

| Befehl | Parameter | Antwort | Anforderung |
|---|---|---|---|
| `eingaben_auflisten` | – | `{ midi: [{ name, verfuegbar }], gewaehlt?: name }` | FR-007 |
| `eingabe_waehlen` | `{ midi: name }` \| `{ keine: true }` | `Eingabezustand` | FR-002; verwirft laufenden Durchlauf |
| `warnung_bestaetigen` | – | – | FR-005 |
| `neu_verbinden` | – | `Eingabezustand` | FR-006 |
| `stueck_laden` | `{ pfad }` | `StueckInfo` \| Fehler `NotendateiUngueltig { grund }` | FR-008 |
| `repertoire` | – | `[StueckInfo]` | US 2 |
| `notenbild` | `{ stueck, transposition, variation, anzeigevariante }` | MusicXML (String) mit Noten-IDs | FR-011, R6 |
| `einstellungen_lesen` / `einstellungen_schreiben` | `Einstellungen` | `Einstellungen` | data-model |
| `lebenszyklus` | `{ vordergrund: bool }` | – | NFR-005 |

`StueckInfo`: `{ id, werk, titel, nummer?, hat_fingersatz, anzeigevarianten: [..] }`

## Ereignisse (Backend → Oberfläche)

| Ereignis | Nutzdaten | Anforderung |
|---|---|---|
| `eingabezustand` | `{ zustand: NichtVerbunden \| MidiVerbunden \| Abgebrochen, geraet?, eingaben_kommen_an: bool }` | FR-007 |
| `durchlauf_erkannt` | `{ stueck, variation, transposition }` | FR-016 |
| `position` | `{ noten_ids: [..], zeit_treiber_us }` | US 3; `zeit_treiber_us` für Latenzmessung (R8) |
| `abweichung` | `{ art, noten_id?, tonhoehe? }` | FR-025 (nur, wenn Markierung während des Spiels an) |
| `durchlauf_beendet` | `{ ergebnis: Abgeschlossen \| Abgebrochen, grund?, abweichungen: [..] }` | FR-025, US 4 |
| `hintergrund_hinweis` | `{ durchlauf_verworfen: bool }` | US 1, Szenario 8 |

## Regeln

- Ereignisse werden in der Reihenfolge ihres Entstehens zugestellt.
- Die Oberfläche zeigt während eines Durchlaufs nichts außer den in FR-024 genannten Ausnahmen an; sie erhält dafür auch keine weiteren Ereignisse.
