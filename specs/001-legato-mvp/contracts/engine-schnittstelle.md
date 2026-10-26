# Contract: Engine-Schnittstelle

Framework-neutrale Schnittstelle von `legato-engine` (`engine/src/api.rs`). Flutter bindet sie über `flutter_rust_bridge` an; ein späterer Tauri-Adapter bildet dieselben Funktionen auf Tauri-Befehle und -Ereignisse ab. Alle Typen sind serialisierbar (serde). Die Hülle enthält keine Fachlogik. Bezeichner laut [Glossar](../../../docs/glossar.md#bezeichner-im-code).

## Lebenszyklus

| Funktion | Parameter | Anforderung |
|---|---|---|
| `start` | `{ data_dir, resource_dir, locale }` | Einstellungen laden, zuletzt gewählte Eingabe wiederherstellen (FR-002) |
| `set_foreground` | `{ foreground: bool }` | NFR-005: verwirft laufenden Durchlauf beim Wechsel in den Hintergrund |
| `events` | – → Strom von `Event` | siehe unten |

## Befehle

| Funktion | Parameter | Antwort | Anforderung |
|---|---|---|---|
| `list_inputs` | – | `{ midi: [{ id, name, transport: Usb \| Bluetooth }], selected?: { name, transport, bluetooth_id? } }`; `selected` auch, wenn das Gerät gerade fehlt | FR-007 |
| `select_input` | `{ midi: id }` \| `None` | `InputState` | FR-002; verwirft laufenden Durchlauf |
| `scan_bluetooth` | `{ enabled: bool }` | – (Funde als Ereignis) | FR-006, US 1/15 |
| `acknowledge_warning` | – | – | FR-005 |
| `reconnect` | – | `InputState` | FR-006 |
| `load_piece` | `{ path }` | `PieceInfo` \| `InvalidScoreFile { reason }` | FR-008 |
| `repertoire` | – | `[PieceInfo]` | US 2 |
| `render_score` | `{ piece, transposition, variation, display_variant }` | MusicXML (String) | FR-011 |
| `get_settings` / `set_settings` | `Settings` | `Settings` | [data-model.md](../data-model.md#settings-einstellungen-persistiert) |

`PieceInfo`: `{ id, work, title, number?, has_fingering, display_variants: [..] }`

## Ereignisse (`Event`)

| Ereignis | Nutzdaten | Anforderung |
|---|---|---|
| `InputStateChanged` | `{ state: Disconnected \| MidiConnected \| ConnectionLost, device?, receiving, warning_active }` | FR-005, FR-007 |
| `BluetoothDeviceFound` | `{ id, name }` | FR-006 |
| `PerformanceRecognized` | `{ piece, variation, transposition }` | FR-016 |
| `Position` | `{ note_ids: [..], driver_time_us }` | US 3; Latenzmessung (R9) |
| `DeviationDetected` | `{ kind, note_id?, pitch? }` | FR-025, nur wenn Fehlermarkierung während des Spiels an |
| `PerformanceEnded` | `{ outcome: Completed \| Aborted, reason?, deviations: [..] }` | FR-025, US 4 |
| `BackgroundNotice` | `{ performance_discarded }` | US 1/8 |
| `PracticeSessionChanged` | `{ active }` | NFR-005 (Bildschirmsperre) |

## Nachrichten an `score-view-web`

Die Hülle reicht Notenbild und Ereignisse unverändert als JSON per `postMessage` weiter:

| Nachricht | Inhalt |
|---|---|
| `load` | `{ musicxml, display_variant }` |
| `position` | `{ note_ids, driver_time_us? }` |
| `mark` | `{ note_id, kind }` |
| `reset` | – |

Rückmeldungen des Bundles: `ready { note_ids }` (nach jedem `load`, IDs in Notenreihenfolge), `error { message }`, `frame_shown { driver_time_us }` und `frame_stats { fps, max_frame_ms }` (nur Entwicklungs-Builds, Messungen R7/R9).

Transport: Die Hülle ruft `legatoReceive(json)` im WebView auf; das Bundle antwortet über den JavaScript-Kanal `LegatoHost.postMessage(json)`, unter Windows (WebView2) über `window.chrome.webview.postMessage(json)`.

**Anzeige-ID** (inhaltsbasiert, von Engine und Bundle gleich aus dem MusicXML der kompletten Anzeige nach Transposition berechnet; verkürzte Anzeigevarianten erhalten keine `position`- und `mark`-Nachrichten): `Takt/System/Einsatz/Tonhöhe`, z. B. `3/1/1:4/60`: Taktindex ab 0 in Dokumentreihenfolge, Systemindex ab 0 (0 = oben), Einsatz innerhalb des Takts als gekürzter Bruch einer ganzen Note, MIDI-Tonhöhe.

## Regeln

- Ereignisse werden in der Reihenfolge ihres Entstehens zugestellt.
- Während eines Durchlaufs sendet die Engine nur die in FR-024 erlaubten Ereignisse.
