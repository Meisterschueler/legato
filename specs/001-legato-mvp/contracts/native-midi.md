# Contract: Native MIDI-Schicht ↔ Engine

C-Schnittstelle zwischen der nativen MIDI-Schicht (Swift-Package `LegatoMidi`, Kotlin-Bibliothek `legato-midi`) und `legato-engine` (`engine/src/ffi.rs`, Header mit `cbindgen`). Die Schicht hängt nicht von Flutter ab; bei einem Wechsel zu Tauri bleibt sie unverändert.

## Richtung native Schicht → Engine

```c
// Driver timestamp in microseconds (monotonic platform clock)
void legato_midi_message(uint32_t device, uint64_t time_us, const uint8_t* data, size_t len);
void legato_midi_device_added(uint32_t device, const char* name, uint8_t transport); // 0 = USB, 1 = Bluetooth
void legato_midi_device_removed(uint32_t device);
void legato_midi_bluetooth_found(const char* identifier, const char* name);
```

- `time_us`: iOS/macOS aus `MIDITimeStamp` (bei Bluetooth inkl. Instrument-Zeitstempel), Android aus dem Zeitstempel von `MidiReceiver.onSend`.
- Aufrufe dürfen aus beliebigen Threads kommen; die Engine reiht sie ein und kehrt sofort zurück (keine Blockade des MIDI-Threads).

## Richtung Engine → native Schicht

Die Schicht registriert beim Start Rückruffunktionen:

```c
typedef struct {
    void (*connect)(uint32_t device);
    void (*disconnect)(uint32_t device);
    void (*send)(uint32_t device, const uint8_t* data, size_t len);
    void (*bluetooth_scan)(bool enabled);
    void (*bluetooth_connect)(const char* identifier); // pending connection request, never times out
    void (*prepare_audio_session)(void);              // iOS: playback + mixWithOthers
    void (*vibrate)(void);                            // no-op without vibration motor (e.g. iPad)
} LegatoMidiPlatform;

void legato_midi_register(const LegatoMidiPlatform* platform);
```

## Verhalten (Pflicht der Schicht)

| Fall | Verhalten | Anforderung |
|---|---|---|
| USB-Gerät verschwindet | `legato_midi_device_removed` sofort nach Meldung des Betriebssystems | FR-005, SC-003 |
| Bluetooth-Verbindung bricht ab | `legato_midi_device_removed`; die Engine fordert danach per `bluetooth_connect` eine dauerhafte Verbindungsanfrage an | FR-006 |
| Gerät kehrt zurück | `legato_midi_device_added`; die Engine verbindet und sendet das `ConnectSignal` | FR-006, FR-027 |
| Wiederverbinden nicht möglich | Schicht meldet nichts; die Oberfläche bietet den manuellen Weg über das Symbol | FR-006, US 1/15 |

## Desktop

Windows und Linux verwenden `midir` innerhalb der Engine; die C-Schnittstelle wird dort nicht benötigt. macOS nutzt das Swift-Package (Bluetooth über CoreBluetooth).
