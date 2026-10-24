# Contract: Matchmaker-Referenz

Erwartungsdateien, mit denen die Rust-Portierung gegen [Matchmaker](https://github.com/pymatchmaker/matchmaker) (Apache-2.0) geprüft wird ([research.md R13](../research.md#r13--tests)).

## Werkzeug

```text
tools/matchmaker-referenz/
├── pyproject.toml        # pymatchmaker @ git+…@<commit>, partitura; mit uv
├── erzeugen.py           # schreibt testdaten/matchmaker/erwartet/*.json
└── README.md             # Commit, Aufruf, Herkunft der Daten
```

Aufruf: `uv run erzeugen.py`. Der Matchmaker-Commit ist fest eingetragen; eine Aktualisierung erzeugt alle Erwartungsdateien neu und ist ein eigener Commit.

## Daten

```text
testdaten/matchmaker/
├── HERKUNFT.md           # Quelle, Commit, Lizenz (Apache-2.0)
├── stuecke/              # aus Matchmaker übernommen: Partitur, MIDI, .match
└── erwartet/
    └── {stueck}.{verfahren}.json
```

## Erwartungsdatei

```json
{
  "matchmaker_commit": "bd0985c…",
  "stueck": "simple_mozart",
  "verfahren": "outerhmm",
  "parameter": { "…": "Standardwerte von Matchmaker" },
  "note_array": [ { "onset_beat": 0.0, "pitch": 69, "id": "n1" } ],
  "zwischenwerte": {
    "uebergangsmatrix": [[…]],
    "beobachtung": [[…]]
  },
  "schritte": [
    { "perf_time": 0.512, "eingabe": [69], "position_index": 0, "position_beat": 0.0 }
  ]
}
```

- `note_array`: Ausgabe von partitura; die Rust-Tests lesen diese statt die Partitur selbst zu parsen.
- `zwischenwerte`: nur, wo das Verfahren sie bereitstellt; für große Matrizen genügt eine Stichprobe mit Indizes.
- `schritte`: ein Eintrag je Ausgabe des Verfahrens.

## Vergleich (kompatibler Modus)

| Größe | Toleranz |
|---|---|
| Zwischenwerte | relativ 1e-6 |
| `position_index` | ≥ 99 % der Schritte identisch, keine Abweichung > 1 |

## Übertragene Tests

Die MIDI-Tests aus `tests/test_prob_hmm.py`, `tests/test_prob_outer_product_hmm.py` und `tests/test_oltw_arzt.py` werden 1:1 nach `core/tests/matchmaker/` übertragen (gleiche Testnamen, gleiche Prüfungen, gleiche Daten).
