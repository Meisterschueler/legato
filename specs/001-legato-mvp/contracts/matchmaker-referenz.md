# Contract: Matchmaker-Referenz

Erwartungsdateien, mit denen die Rust-Portierung gegen [Matchmaker](https://github.com/pymatchmaker/matchmaker) (Apache-2.0) geprüft wird ([research.md R13](../research.md#r13--tests)).

## Werkzeug

```text
tools/matchmaker-reference/
├── pyproject.toml        # pymatchmaker @ git+…@<commit>, partitura; mit uv
├── generate.py           # schreibt testdata/matchmaker/expected/*.json
└── README.md             # Commit, Aufruf, Herkunft der Daten
```

Aufruf: `uv run generate.py`. Der Matchmaker-Commit ist fest eingetragen; eine Aktualisierung erzeugt alle Erwartungsdateien neu und ist ein eigener Commit.

## Daten

```text
testdata/matchmaker/
├── PROVENANCE.md         # Quelle, Commit, Lizenz (Apache-2.0)
├── pieces/               # aus Matchmaker übernommen: Partitur, MIDI, .match
└── expected/
    └── {piece}.{method}.json
```

## Erwartungsdatei

```json
{
  "matchmaker_commit": "bd0985c…",
  "piece": "simple_mozart",
  "method": "outerhmm",
  "parameters": { "…": "Standardwerte von Matchmaker" },
  "note_array": [ { "onset_beat": 0.0, "pitch": 69, "id": "n1" } ],
  "intermediates": {
    "transition_matrix": [[…]],
    "observation_probabilities": [[…]]
  },
  "steps": [
    { "perf_time": 0.512, "input": [69], "onset_index": 0, "onset_beat": 0.0 }
  ]
}
```

- `note_array`: Ausgabe von partitura; die Rust-Tests lesen diese statt die Partitur selbst zu parsen.
- `intermediates`: nur, wo das Verfahren sie bereitstellt; für große Matrizen genügt eine Stichprobe mit Indizes.
- `steps`: ein Eintrag je Ausgabe des Verfahrens.

## Vergleich (kompatibler Modus)

| Größe | Toleranz |
|---|---|
| `intermediates` | relativ 1e-6 |
| `onset_index` | ≥ 99 % der Schritte identisch, keine Abweichung > 1 |

## Übertragene Tests

Die MIDI-Tests aus `tests/test_prob_hmm.py`, `tests/test_prob_outer_product_hmm.py` und `tests/test_oltw_arzt.py` werden 1:1 nach `core/tests/matchmaker/` übertragen (gleiche Testnamen, gleiche Prüfungen, gleiche Daten).
