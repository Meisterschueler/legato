# Hinweise für Agenten

## Commit-Nachrichten

[Conventional Commits](https://www.conventionalcommits.org/de/v1.0.0/): `<typ>(<bereich>): <beschreibung>`

- **Typ**: `feat`, `fix`, `test`, `refactor`, `docs`, `ci`, `build`, `chore`
- **Bereich**: `core`, `engine`, `app`, `web` (score-view-web), `native`, `spike`, `specs`, `tools`; entfällt, wenn mehrere betroffen sind
- **Beschreibung**: deutsch, knapp (Kopfzeile ≤ 72 Zeichen), ohne Punkt; Task-IDs am Ende in Klammern
- Ein Thema je Commit; Begründung bei Bedarf im Text darunter

Beispiele:

```
feat(core): MusicXML lesen und transponiert schreiben (T025–T028)
fix(ci): ALSA-Header im generate-Job installieren
docs(specs): Anzeige-ID statt id-Attribut im MusicXML
```
