# Feature-Spezifikation: Schwächen

**Status**: Entwurf | **Erstellt**: 2026-10-23 | **Constitution**: [1.0.0](../../.specify/memory/constitution.md) | **Abhängig von**: [001-legato-mvp](../001-legato-mvp/spec.md), [002-bewertung](../002-bewertung/spec.md), für Schwächen über mehrere Durchläufe [007-konto-backend](../007-konto-backend/spec.md)

**Produktübersicht**: [produkt.md](../../docs/produkt.md) | **Glossar**: [glossar.md](../../docs/glossar.md)

## Überblick

Die App erkennt systematische Schwächen innerhalb eines Durchlaufs und, nach der Anmeldung, über mehrere Durchläufe hinweg.

## Umfang

Diese Spec umfasst User Story 9 (Priorität P3).

## Nutzungsszenarien und Tests

### User Story 9 – Systematische Schwächen erkennen (Priorität: P3)

Die App erkennt systematische Abweichungen innerhalb eines Durchlaufs oder über mehrere Durchläufe hinweg und zeigt sie auf Nachfrage an, z. B. „Der 4. Finger der linken Hand schlägt durchgehend zu spät an“. Schwächen innerhalb eines Durchlaufs gehören zur Grundfunktionalität und sind auch ohne Anmeldung und offline verfügbar. Schwächen über mehrere Durchläufe sind eine Auswertung und setzen die Anmeldung voraus (User Story 11 in [007](../007-konto-backend/spec.md)).

**Warum diese Priorität**: Das ist der wesentliche Mehrwert gegenüber anderen Apps, setzt aber zuverlässige Zuordnung und Bewertung (User Stories 3 und 5) voraus.

**Unabhängiger Test**: Referenz-Durchläufe mit einer eingebauten systematischen Abweichung einspeisen und prüfen, dass genau diese erkannt wird. Für Tonart-Schwächen mehrere Referenz-Durchläufe in unterschiedlichen Tonarten einspeisen.

**Akzeptanzszenarien**:

1. **Angenommen** innerhalb eines Durchlaufs tritt bei einem bestimmten Finger, einem bestimmten Fingerübergang oder einer bestimmten Hand wiederholt dieselbe Art von Abweichung auf, **wenn** der Pianist die Analyse aufruft, **dann** wird diese Schwäche benannt und mit Beispielstellen belegt.
2. **Angenommen** der Pianist ist angemeldet, hat ein Stück in mindestens zwei unterschiedlichen Tonarten gespielt, und eine Tonart fällt systematisch stärker ab, als ihr Schwierigkeitsgrad erwarten lässt (User Story 14 in [007](../007-konto-backend/spec.md)), **wenn** er die Analyse aufruft, **dann** wird diese Tonart als Schwäche benannt und mit den betroffenen Durchläufen belegt.
3. **Angenommen** der Pianist hat ein Stück nur in einer Tonart gespielt, **wenn** er die Analyse aufruft, **dann** wird für dieses Stück keine Tonart-Schwäche gemeldet.
4. **Angenommen** Abweichungen treten nur zufällig auf (Tagesform), **wenn** der Pianist die Analyse aufruft, **dann** wird keine Schwäche gemeldet.
5. **Angenommen** eine Schwäche wurde erkannt, **wenn** der Pianist spielt, **dann** wird sie nicht unaufgefordert angezeigt.
6. **Angenommen** ein Durchlauf wurde mit nur einer Hand gespielt, **wenn** der Pianist die Analyse aufruft, **dann** werden für ihn keine Schwächen gemeldet, die beide Hände betreffen.
7. **Angenommen** der Pianist ist nicht angemeldet, **wenn** er die Analyse aufruft, **dann** werden Schwächen innerhalb eines Durchlaufs gemeldet, aber keine Schwächen über mehrere Durchläufe.
8. **Angenommen** der Notentext enthält keinen Fingersatz, **wenn** der Pianist die Analyse aufruft, **dann** werden Schwächen der Hände und Tonarten gemeldet, Schwächen einzelner Finger und Fingerübergänge dagegen nicht; das wird als solches gekennzeichnet.

Folgende Schwächen werden erkannt:

| Schwäche | Erforderliche Durchläufe | Fingersatz erforderlich |
|---|---|---|
| Ein einzelner Finger schlägt systematisch zu früh, zu spät, zu laut oder zu leise an | einer | ja |
| Eine Hand hinkt der anderen nach | einer | nein |
| Ungleichheit der Lautstärke zwischen den Händen | einer | nein |
| Ein einzelner Finger lässt die Taste systematisch zu früh oder zu spät los | einer | ja |
| Ein bestimmter Fingerübergang (z. B. 4 → 5) ist systematisch ungleichmäßig artikuliert | einer | ja |
| Die Hände artikulieren unterschiedlich | einer | nein |
| Probleme in bestimmten Tonarten | mehrere, mit demselben Stück in mindestens zwei unterschiedlichen Tonarten (nur nach Anmeldung) | nein |

---

### Randfälle

| Randfall | Verhalten |
|---|---|
| Der Pianist hat ein Stück nur in einer Tonart gespielt. | Keine Tonart-Schwäche (User Story 9). |
| Ein Durchlauf wurde mit nur einer Hand gespielt. | Keine Schwächen, die beide Hände betreffen (User Story 9). |
| Der Notentext enthält keinen Fingersatz. | Nur Schwächen der Hände und Tonarten (User Story 9). |

## Anforderungen

Die Schlüsselwörter MUSS, SOLL, DARF NICHT usw. sind im [Glossar](../../docs/glossar.md#schlüsselwörter) definiert.

### Funktionale Anforderungen

**Analyse**

- **FR-047**: Das System MUSS systematische Schwächen innerhalb eines Durchlaufs oder über mehrere Durchläufe hinweg erkennen und mit Beispielstellen belegen. Schwächen innerhalb eines Durchlaufs MUSS die App auch ohne Anmeldung erkennen; Schwächen über mehrere Durchläufe DÜRFEN NUR angemeldeten Pianisten angezeigt werden. Die zu erkennenden Schwächen, die dafür erforderliche Anzahl an Durchläufen und ob sie einen Fingersatz voraussetzen, sind in User Story 9 aufgeführt.

### Nicht-funktionale Anforderungen

Es gelten die nicht-funktionalen Anforderungen aus [001](../001-legato-mvp/spec.md#nicht-funktionale-anforderungen). Nach NFR-002 MÜSSEN Schwächen innerhalb eines Durchlaufs ohne Onlineverbindung erkannt werden.

### Wesentliche Entitäten

- **Schwäche**: Systematisches Muster aus Abweichungen innerhalb eines oder über mehrere Durchläufe.

## Erfolgskriterien

- **SC-009**: In Referenz-Durchläufen mit eingebauten systematischen Abweichungen wird jede dieser Abweichungen als Schwäche erkannt, ohne zusätzliche falsche Schwächen zu melden. Finger- und Hand-Schwächen werden bereits in einem einzelnen Durchlauf erkannt, Tonart-Schwächen ab zwei Tonarten desselben Stücks.

## Annahmen

Es gelten die Annahmen aus [001](../001-legato-mvp/spec.md#annahmen).
