# Contract: Native MIDI-Schicht ↔ Engine

C-Schnittstelle zwischen der nativen MIDI-Schicht (Swift-Package `LegatoMidi`, Kotlin-Bibliothek `legato-midi`) und `legato-engine` (`engine/src/ffi.rs`, Header mit `cbindgen`). Die Schicht hängt nicht von Flutter ab; bei einem Wechsel zu Tauri bleibt sie unverändert.

## Richtung native Schicht → Engine

```c
// Treiber-Zeitstempel in Mikrosekunden (monoton, Uhr der Plattform)
void legato_midi_nachricht(uint32_t geraet, uint64_t zeit_us, const uint8_t* daten, size_t laenge);
void legato_midi_geraet_hinzu(uint32_t geraet, const char* name, uint8_t art); // 0 = USB, 1 = Bluetooth
void legato_midi_geraet_weg(uint32_t geraet);
void legato_midi_bluetooth_gefunden(const char* kennung, const char* name);
```

- `zeit_us`: iOS/macOS aus `MIDITimeStamp` (bei Bluetooth inkl. Instrument-Zeitstempel), Android aus dem Zeitstempel von `MidiReceiver.onSend`.
- Aufrufe dürfen aus beliebigen Threads kommen; die Engine reiht sie ein und kehrt sofort zurück (keine Blockade des MIDI-Threads).

## Richtung Engine → native Schicht

Die Schicht registriert beim Start Rückruffunktionen:

```c
typedef struct {
    void (*verbinden)(uint32_t geraet);
    void (*trennen)(uint32_t geraet);
    void (*senden)(uint32_t geraet, const uint8_t* daten, size_t laenge);
    void (*bluetooth_suche)(bool an);
    void (*bluetooth_verbinden)(const char* kennung); // dauerhafte Verbindungsanfrage
    void (*audio_session_vorbereiten)(void);         // iOS: playback + mixWithOthers
} LegatoMidiPlattform;

void legato_midi_registrieren(const LegatoMidiPlattform* plattform);
```

## Verhalten (Pflicht der Schicht)

| Fall | Verhalten | Anforderung |
|---|---|---|
| USB-Gerät verschwindet | `legato_midi_geraet_weg` sofort nach Meldung des Betriebssystems | FR-005, SC-003 |
| Bluetooth-Verbindung bricht ab | `legato_midi_geraet_weg`; die Engine fordert danach per `bluetooth_verbinden` eine dauerhafte Verbindungsanfrage an | FR-006 |
| Gerät kehrt zurück | `legato_midi_geraet_hinzu`; die Engine verbindet und sendet das Verbindungssignal | FR-006, FR-027 |
| Wiederverbinden nicht möglich | Schicht meldet nichts; die Oberfläche bietet den manuellen Weg über das Symbol | FR-006, US 1/15 |

## Desktop

Windows und Linux verwenden `midir` innerhalb der Engine; die C-Schnittstelle wird dort nicht benötigt. macOS nutzt das Swift-Package (Bluetooth über CoreBluetooth).
