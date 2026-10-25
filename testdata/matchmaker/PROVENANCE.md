# Herkunft der Matchmaker-Daten

Daten in diesem Verzeichnis stammen aus [Matchmaker](https://github.com/pymatchmaker/matchmaker) (Apache-2.0) oder wurden mit [tools/matchmaker-reference](../../tools/matchmaker-reference/README.md) daraus erzeugt.

| Angabe | Wert |
|---|---|
| Repository | https://github.com/pymatchmaker/matchmaker |
| Commit | `6c09490ba7e4b9125f6839cb534d6e150dd14d80` |
| Lizenz | Apache-2.0 |

## Übernommene Dateien

| Datei | Quelle in Matchmaker | Änderungen |
|---|---|---|
| `pieces/simple_mozart_k265_var1.{musicxml,mid,match}` | `matchmaker/assets/` (`EXAMPLE_PIECES["simple_mozart"]`) | keine |
| `pieces/mozart_k265_var1.{musicxml,mid,match}` | `matchmaker/assets/` (`EXAMPLE_MATCH` u. a.) | keine |
| `pieces/Bach-fugue_bwv_858.{musicxml,mid,match}` | `matchmaker/assets/` (`EXAMPLE_PIECES["bach_fugue"]`) | keine |

## Erzeugte Dateien

`expected/{piece}.{method}.json` und `expected/hanon-01.note_array.json` schreibt `generate.py` (Workflow „Matchmaker reference“).
