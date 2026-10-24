# Matchmaker-Referenz

Erzeugt die Erwartungsdateien, gegen die die Rust-Portierung der Folgeverfahren getestet wird (Verfahren 3, [research.md R13](../../specs/001-legato-mvp/research.md#r13--tests), Format laut [contracts/matchmaker-referenz.md](../../specs/001-legato-mvp/contracts/matchmaker-referenz.md)).

- Matchmaker ist in [pyproject.toml](pyproject.toml) auf einen festen Commit festgelegt.
- Die CI braucht kein Python. Die Erwartungsdateien unter `testdata/matchmaker/` werden nur neu erzeugt, wenn der Commit aktualisiert wird.

## Aufruf

```sh
cd tools/matchmaker-reference
uv sync
uv run python generate.py --out ../../testdata/matchmaker
```

`generate.py` entsteht mit T033.

## Commit aktualisieren

1. Commit in `pyproject.toml` ändern.
2. Erwartungsdateien neu erzeugen und den Commit in [testdata/matchmaker/PROVENANCE.md](../../testdata/matchmaker/PROVENANCE.md) eintragen.
3. `cargo test -p legato-core` ausführen; Abweichungen prüfen.
