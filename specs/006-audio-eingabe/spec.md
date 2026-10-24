# Feature-Spezifikation: Audio-Eingabe

**Status**: Entwurf | **Erstellt**: 2026-10-23 | **Constitution**: [1.0.0](../../.specify/memory/constitution.md) | **Abhängig von**: [001-legato-mvp](../001-legato-mvp/spec.md)

**Produktübersicht**: [produkt.md](../../docs/produkt.md) | **Glossar**: [glossar.md](../../docs/glossar.md)

## Überblick

Pianisten ohne MIDI-Gerät können über das Mikrofon spielen. Die Audio-Eingabe ist nur eine Notlösung (Constitution V).

## Umfang

Diese Spec umfasst User Story 10 (Priorität P3).

## Nutzungsszenarien und Tests

### User Story 10 – Audio-Eingabe als Notlösung (Priorität: P3)

Ein Pianist ohne MIDI-Gerät kann über das Mikrofon spielen. Die App weist dabei auf die geringere Genauigkeit hin.

**Warum diese Priorität**: Erweitert die Zielgruppe, ist aber nur eine Notlösung (Constitution V).

**Unabhängiger Test**: Referenzaufnahmen einspeisen und Zuordnung prüfen. Prüfen, dass der Hinweis auf eingeschränkte Genauigkeit erscheint.

**Akzeptanzszenarien**:

1. **Angenommen** kein MIDI-Gerät ist verbunden, **wenn** der Pianist die Audio-Eingabe ausdrücklich wählt, **dann** wird er vor dem Spiel einmalig auf die Einschränkungen hingewiesen.
2. **Angenommen** MIDI ist verbunden, **wenn** die Verbindung verloren geht, **dann** wechselt die App nicht selbständig zur Audio-Eingabe. Audio ist kein Ersatz für MIDI.
3. **Angenommen** die Audio-Eingabe ist aktiv, **wenn** der Pianist spielt, **dann** wird nur der Beginn eines Tons (Note-On) ausgewertet. Das Ende eines Tons (Note-Off) wird verworfen. Die Artikulation wird daher weder bewertet noch analysiert, und das wird als solches gekennzeichnet.
4. **Angenommen** die Audio-Eingabe ist aktiv, **wenn** ein Durchlauf bewertet oder analysiert wird, **dann** berücksichtigen Bewertung und Analysen die deutlich geringere zeitliche Genauigkeit des Tonbeginns. Ergebnisse, die mit Audio nicht zuverlässig möglich sind, werden nicht angeboten oder als unsicher gekennzeichnet.
5. **Angenommen** die Audio-Eingabe ist aktiv, **wenn** ein Durchlauf abgeschlossen ist, **dann** wird die erkannte Tonfolge wie bei MIDI-Eingabe lokal gespeichert und nie ins Backend übertragen.
6. **Angenommen** der Pianist hat die Audioaufnahme nicht eingeschaltet (Standard), **wenn** ein Durchlauf abgeschlossen ist, **dann** wird keine Audioaufnahme gespeichert.
7. **Angenommen** der Pianist hat die Audioaufnahme ausdrücklich eingeschaltet, **wenn** ein Durchlauf abgeschlossen ist, **dann** wird zusätzlich die Audioaufnahme lokal gespeichert. Sie ist im Export und beim Löschen der lokalen Daten enthalten (User Story 7 in [003](../003-speichern/spec.md)).
8. **Angenommen** der Pianist will die Audioaufnahme einschalten, **wenn** er die Einstellung wählt, **dann** weist die App darauf hin, dass das Mikrofon auch Umgebungsgeräusche und Stimmen aufnimmt.

---

### Randfälle

| Randfall | Verhalten |
|---|---|
| Die MIDI-Verbindung geht verloren. | Kein selbständiger Wechsel zur Audio-Eingabe (User Story 10). |
| Das Mikrofon fällt während eines Durchlaufs weg. | Der Durchlauf wird verworfen (FR-004). |

## Anforderungen

Die Schlüsselwörter MUSS, SOLL, DARF NICHT usw. sind im [Glossar](../../docs/glossar.md#schlüsselwörter) definiert.

### Funktionale Anforderungen

**Eingabe**

- **FR-048**: Die App SOLL Klavierspiel über das Mikrofon empfangen können (Notlösung). Sie MUSS den Pianisten über Einschränkungen der gewählten Eingabe informieren. Für die Wahl der Eingabe gilt FR-002.
- **FR-049**: Bei Audio-Eingabe MUSS die App nur den Tonbeginn (Note-On) auswerten und das Tonende (Note-Off) verwerfen.

**Bewertung**

- **FR-050**: Bei Audio-Eingabe DARF die Artikulation NICHT bewertet und NICHT analysiert werden, auch nicht für Schwächen ([005](../005-schwaechen/spec.md)). Das MUSS als solches gekennzeichnet werden.

**Speichern**

- **FR-051**: Bei Audio-Eingabe MUSS die App die erkannte Tonfolge wie bei MIDI-Eingabe speichern (FR-042). Die Audioaufnahme DARF NUR gespeichert werden, wenn der Pianist es ausdrücklich eingeschaltet hat (Standard: aus); vorher MUSS die App darauf hinweisen, dass das Mikrofon auch Umgebungsgeräusche aufnimmt. Die Audioaufnahme DARF das Gerät NICHT verlassen und MUSS in Export und Löschen der lokalen Daten (FR-045) enthalten sein.

### Nicht-funktionale Anforderungen

Es gelten die nicht-funktionalen Anforderungen aus [001](../001-legato-mvp/spec.md#nicht-funktionale-anforderungen). Nach NFR-002 MUSS die Audio-Eingabe ohne Onlineverbindung funktionieren.

### Wesentliche Entitäten

- **Durchlauf** (ergänzt [001](../001-legato-mvp/spec.md#wesentliche-entitäten)): bei Audio-Eingabe optional mit Audioaufnahme.

## Erfolgskriterien

- **SC-010**: Bei Audio-Eingabe werden mindestens [NEEDS CLARIFICATION: Zielwert] der Töne in Referenzaufnahmen korrekt zugeordnet (vgl. SC-001).

## Annahmen

Es gelten die Annahmen aus [001](../001-legato-mvp/spec.md#annahmen).

## Hinweise für die Planung (nicht normativ)

Die folgenden Hinweise sind Entscheidungen über das *Wie* und gehören in den Plan, nicht in die Spec. Sobald der Plan existiert, werden sie dorthin übernommen und hier entfernt:

- **Speicherformat**: Erkannte Tonfolge als .midi wie bei MIDI-Eingabe ([003](../003-speichern/spec.md)). Audioaufnahme verlustfrei (z. B. FLAC), damit eine spätere Neuauswertung nicht unter Kompressionsartefakten leidet; verlustbehaftete Formate wie MP3 taugen nur zum Anhören.
