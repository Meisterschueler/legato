# Contract: Referenzdaten für Tests

Referenzdaten machen Zuordnung und Erkennung reproduzierbar testbar (Constitution, Testbarkeit; SC-001, SC-002). Schlüssel und Werte sind englisch ([Glossar](../../../docs/glossar.md#bezeichner-im-code)).

## Ablage

```text
testdata/legato/
└── hanon-01/
    ├── score.musicxml                 # Notentext (identisch mit der mitgelieferten Ressource)
    ├── c-major-both-hands-3x.mid      # aufgezeichnete Durchläufe (Standard-MIDI-Datei, Typ 0)
    ├── c-major-both-hands-3x.json     # Referenz-Zuordnung
    └── …
```

## Referenz-Zuordnung (JSON)

```json
{
  "piece": "hanon-01",
  "candidate": { "variation": "BothHands", "transposition": 0 },
  "expected": [
    { "performance": 1, "outcome": "Completed" },
    { "performance": 2, "outcome": "Completed" },
    { "performance": 3, "outcome": "Aborted", "reason": "AlignmentLoss" }
  ],
  "played_notes": [
    { "i": 0, "note": "m1-v1-n0" },
    { "i": 1, "note": "m1-v5-n0" },
    { "i": 17, "deviation": "Addition" },
    { "i": 23, "deviation": "Substitution", "note": "m3-v1-n4" }
  ],
  "omissions": ["m5-v5-n2"]
}
```

- `i` ist der Index des Note-On-Ereignisses in der MIDI-Datei (Pedal und andere Controller zählen nicht).
- Jeder gespielte Ton ist genau einmal aufgeführt.

## Messgrößen

- **SC-001**: Anteil der Töne, deren berechnete Zuordnung (Note oder Abweichungsart) mit der Referenz übereinstimmt; Ziel ≥ 99 % über alle Referenzdaten.
- **SC-002**: Für jeden Durchlauf: richtiger Kandidat, erkannt spätestens nach der Erkennungslänge.

## Mindestumfang

Hanon Nr. 1 in mindestens 3 Tonarten, je beidhändig und einhändig, mit fehlerfreien Durchläufen, eingestreuten Abweichungen, nahtlosen Wiederholungen, Abbruch durch Pause, Abbruch durch Zuordnungsverlust, kurzzeitigem Tonartwechsel, Pedalnutzung und Echo eigener Signaltöne.
