# Matchmaker-Referenz

Erzeugt die Erwartungsdateien, gegen die die Rust-Portierung der Folgeverfahren getestet wird (Verfahren 3, [research.md R13](../../specs/001-legato-mvp/research.md#r13--tests), Format laut [contracts/matchmaker-referenz.md](../../specs/001-legato-mvp/contracts/matchmaker-referenz.md)).

- Matchmaker ist in [pyproject.toml](pyproject.toml) auf einen festen Commit festgelegt.
- Die normale CI braucht kein Python. Die Dateien unter `testdata/matchmaker/` werden nur neu erzeugt, wenn der Commit aktualisiert wird.

## Aufruf

Auf GitHub: Workflow „Matchmaker reference“ manuell starten; er committet das Ergebnis.

Lokal (Linux oder macOS; Matchmaker baut eine Cython-Erweiterung):

```sh
cd tools/matchmaker-reference
uv run --python 3.12 generate.py
```

`generate.py` kopiert die Beispielstücke nach `testdata/matchmaker/pieces/` und schreibt `testdata/matchmaker/expected/{piece}.{method}.json` für `pthmm`, `outerhmm` und `arzt` sowie die `note_array` von partitura für Hanon Nr. 1.

## Commit aktualisieren

1. Commit in `pyproject.toml` ändern.
2. Erwartungsdateien neu erzeugen und den Commit in [testdata/matchmaker/PROVENANCE.md](../../testdata/matchmaker/PROVENANCE.md) eintragen.
3. `cargo test -p legato-core` ausführen; Abweichungen prüfen.
