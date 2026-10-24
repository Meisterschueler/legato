# Feature-Spezifikation: Performance-Wurm

**Status**: Entwurf | **Erstellt**: 2026-10-23 | **Constitution**: [1.0.0](../../.specify/memory/constitution.md) | **Abhängig von**: [001-legato-mvp](../001-legato-mvp/spec.md)

**Produktübersicht**: [produkt.md](../../docs/produkt.md) | **Glossar**: [glossar.md](../../docs/glossar.md)

## Überblick

Erweitert das Üben um den Performance-Wurm als zweite Ansicht neben dem Notenblatt.

## Umfang

Diese Spec umfasst User Story 8 (Priorität P3).

## Nutzungsszenarien und Tests

### User Story 8 – Performance-Wurm (Priorität: P3)

Statt des Notenblatts kann der Pianist eine Ansicht wählen, die Tempo und Lautstärke als „Wurm“ in einem Koordinatensystem zeigt: horizontal Tempo (in M.M.), vertikal Lautstärke. Der Kopf zeigt den aktuellen Zustand, der Schwanz die vergangenen Noten.

**Warum diese Priorität**: Der Wurm ist die zentrale, nicht ablenkende Rückmeldung zur Gleichmäßigkeit während des Spiels.

**Unabhängiger Test**: Referenz-MIDI-Dateien mit gleichmäßigem bzw. ungleichmäßigem Spiel einspeisen und die Form des Kopfes prüfen.

**Akzeptanzszenarien**:

1. **Angenommen** der Wurm wird angezeigt, **wenn** der Pianist mit vollkommen gleichmäßigem Tempo und gleichmäßiger Lautstärke spielt, **dann** ist der Kopf ein kleiner Kreis, der seine Position nicht ändert.
2. **Angenommen** der Wurm wird angezeigt, **wenn** das Tempo von Note zu Note schwankt (zeitliche Ungleichmäßigkeit), **dann** wird der Kopf zu einer horizontalen Ellipse. Je stärker die Schwankung, desto breiter die Ellipse.
3. **Angenommen** der Wurm wird angezeigt, **wenn** die Lautstärke von Note zu Note schwankt (dynamische Ungleichmäßigkeit), **dann** wird der Kopf zu einer vertikalen Ellipse. Je stärker die Schwankung, desto höher die Ellipse.
4. **Angenommen** der Wurm wird angezeigt, **wenn** der Pianist über längere Zeit schneller oder lauter wird (Agogik/Dynamik), **dann** wandert der Kopf, wächst aber nicht.
5. **Angenommen** der Wurm wird angezeigt, **wenn** er sich den Grenzen des sichtbaren Bereichs nähert und ihn verlassen könnte, **dann** passen sich die Achsengrenzen fließend an, sodass der Wurm immer vollständig sichtbar bleibt.
6. **Angenommen** der Wurm wird angezeigt, **wenn** der Pianist spielt, **dann** ist der Kopf rot gefärbt, und je weiter eine Note zurückliegt, desto schwächer ist ihre Färbung im Schwanz.
7. **Angenommen** der Wurm wird angezeigt, **wenn** der Pianist in den Einstellungen den betrachteten Zeitraum für den Kopf oder die Länge des Schwanzes ändert, **dann** übernimmt der Wurm die neuen Werte, sodass der Pianist verschiedene Einstellungen ausprobieren kann.

---

### Randfälle

| Randfall | Verhalten |
|---|---|
| Der Wurm nähert sich den Grenzen des sichtbaren Bereichs. | Die Achsengrenzen passen sich fließend an (User Story 8). |

## Anforderungen

Die Schlüsselwörter MUSS, SOLL, DARF NICHT usw. sind im [Glossar](../../docs/glossar.md#schlüsselwörter) definiert.

### Funktionale Anforderungen

**Darstellung**

- **FR-046**: Die App MUSS die Ansicht „Performance-Wurm“ als Alternative zum Notenblatt anbieten (FR-022). Der Pianist MUSS den betrachteten Zeitraum für den Kopf des Wurms und die Länge des Schwanzes einstellen können.

### Nicht-funktionale Anforderungen

Es gelten die nicht-funktionalen Anforderungen aus [001](../001-legato-mvp/spec.md#nicht-funktionale-anforderungen). Nach NFR-002 MUSS der Performance-Wurm ohne Onlineverbindung funktionieren.

### Wesentliche Entitäten

Keine neuen.

## Erfolgskriterien

Keine eigenen; maßgeblich sind die Akzeptanzszenarien von User Story 8.

## Annahmen

- Sinnvolle Standardwerte für Zeitraum und Länge des Wurms werden durch Ausprobieren ermittelt.
