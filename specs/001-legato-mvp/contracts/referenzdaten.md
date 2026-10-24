# Contract: Referenzdaten für Tests

Referenzdaten machen Zuordnung und Erkennung reproduzierbar testbar (Constitution, Testbarkeit; SC-001, SC-002).

## Ablage

```text
tests/referenz/
└── hanon-01/
    ├── stueck.musicxml              # Notentext (identisch mit der mitgelieferten Ressource)
    ├── c-dur-beidhaendig-3x.mid     # aufgezeichneter Durchlauf (Standard-MIDI-Datei, Typ 0)
    ├── c-dur-beidhaendig-3x.json    # Referenz-Zuordnung
    └── …
```

## Referenz-Zuordnung (JSON)

```json
{
  "stueck": "hanon-01",
  "kandidat": { "variation": "Beidhaendig", "transposition": 0 },
  "erwartet": [
    { "durchlauf": 1, "ergebnis": "Abgeschlossen" },
    { "durchlauf": 2, "ergebnis": "Abgeschlossen" },
    { "durchlauf": 3, "ergebnis": "Abgebrochen", "grund": "Zuordnungsverlust" }
  ],
  "toene": [
    { "i": 0, "note": "t1-v1-n0" },
    { "i": 1, "note": "t1-v5-n0" },
    { "i": 17, "abweichung": "Zusatzton" },
    { "i": 23, "abweichung": "FalscherTon", "note": "t3-v1-n4" }
  ],
  "auslassungen": ["t5-v5-n2"]
}
```

- `i` ist der Index des Note-On-Ereignisses in der MIDI-Datei (Pedal und andere Controller zählen nicht).
- Jeder Ton ist genau einmal aufgeführt.

## Messgrößen

- **SC-001**: Anteil der Töne, deren berechnete Zuordnung (Note oder Abweichungsart) mit der Referenz übereinstimmt; Ziel ≥ 99 % über alle Referenzdaten.
- **SC-002**: Für jeden Durchlauf: richtiger Kandidat, erkannt spätestens nach der Erkennungslänge.

## Mindestumfang

Hanon Nr. 1 in mindestens 3 Tonarten, je beidhändig und einhändig, mit fehlerfreien Durchläufen, eingestreuten Abweichungen, nahtlosen Wiederholungen, Abbruch durch Pause, Abbruch durch Zuordnungsverlust, kurzzeitigem Tonartwechsel, Pedalnutzung und Echo eigener Signaltöne.
