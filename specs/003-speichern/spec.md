# Feature-Spezifikation: Speichern und lokale Daten

**Status**: Entwurf | **Erstellt**: 2026-10-23 | **Constitution**: [1.0.0](../../.specify/memory/constitution.md) | **Abhängig von**: [001-legato-mvp](../001-legato-mvp/spec.md)

**Produktübersicht**: [produkt.md](../../docs/produkt.md) | **Glossar**: [glossar.md](../../docs/glossar.md)

## Überblick

Die App speichert jeden abgeschlossenen Durchlauf lokal. Der Pianist kann Durchläufe exportieren sowie alle lokalen Daten exportieren und löschen. Alles funktioniert ohne Anmeldung und offline.

## Umfang

Diese Spec umfasst die User Stories 6–7 (Priorität P2).

## Nutzungsszenarien und Tests

### User Story 6 – Durchläufe speichern (Priorität: P2)

Am Ende eines Durchlaufs wird das Spiel lokal gespeichert, damit der Pianist es später anhören, erneut auswerten oder exportieren kann.

**Warum diese Priorität**: Ohne gespeicherte Durchläufe sind weder spätere Analysen noch Export möglich, und offline entstandene Daten würden verloren gehen.

**Unabhängiger Test**: Einen Durchlauf spielen, App beenden und neu starten, Durchlauf wiederfinden und abspielen.

**Akzeptanzszenarien**:

1. **Angenommen** ein Durchlauf läuft, **wenn** er abgeschlossen ist, **dann** ist er automatisch und dauerhaft lokal gespeichert, ohne dass der Pianist etwas tun muss. Das gilt für alle Durchläufe.
2. **Angenommen** ein Durchlauf ist gespeichert, **wenn** der Pianist ihn exportiert, **dann** erhält er eine Datei in einem gängigen Format, das andere Programme abspielen können.
3. **Angenommen** eine Übeeinheit läuft, **wenn** die App unerwartet beendet wird, **dann** sind alle bereits abgeschlossenen Durchläufe erhalten.
4. **Angenommen** ein Durchlauf wurde abgebrochen (siehe User Story 3 in [001](../001-legato-mvp/spec.md)), **wenn** er endet, **dann** wird er nicht gespeichert.

---

### User Story 7 – Lokale Daten exportieren und löschen (Priorität: P2)

Der Pianist kann alle lokal gespeicherten Daten exportieren und löschen, auch offline. Export und Löschen im Backend regelt User Story 16 in [007](../007-konto-backend/spec.md).

**Warum diese Priorität**: Vorgabe der Constitution (VII) und der DSGVO.

**Unabhängiger Test**: Daten anlegen, exportieren, Vollständigkeit prüfen, lokale Daten löschen und prüfen, dass auf dem Gerät keine Daten verbleiben.

**Akzeptanzszenarien**:

1. **Angenommen** lokale Daten sind vorhanden, **wenn** der Pianist einen Export anfordert, auch offline, **dann** erhält er alle lokal gespeicherten Daten in maschinenlesbarer Form.
2. **Angenommen** lokale Daten sind vorhanden, **wenn** der Pianist sie löscht, **dann** sind auf dem Gerät keine seiner Durchläufe und Einstellungen mehr vorhanden.

---

### Randfälle

| Randfall | Verhalten |
|---|---|
| Die App wird während einer Übeeinheit unerwartet beendet. | Alle bereits abgeschlossenen Durchläufe sind erhalten (User Story 6). |

## Anforderungen

Die Schlüsselwörter MUSS, SOLL, DARF NICHT usw. sind im [Glossar](../../docs/glossar.md#schlüsselwörter) definiert.

### Funktionale Anforderungen

**Speichern**

- **FR-042**: Die App MUSS jeden abgeschlossenen Durchlauf automatisch und dauerhaft lokal speichern.
- **FR-043**: Die App MUSS Durchläufe in einem gängigen, abspielbaren Format exportieren können.
- **FR-044**: Verworfene Durchläufe DÜRFEN NICHT gespeichert oder übertragen werden.

**Datenschutz**

- **FR-045**: Der Pianist MUSS jederzeit, auch offline, alle lokal gespeicherten Daten exportieren und löschen können.

### Nicht-funktionale Anforderungen

Es gelten die nicht-funktionalen Anforderungen aus [001](../001-legato-mvp/spec.md#nicht-funktionale-anforderungen). Nach NFR-002 MÜSSEN alle Anforderungen dieser Spec ohne Onlineverbindung funktionieren.

### Wesentliche Entitäten

- **Durchlauf** (ergänzt [001](../001-legato-mvp/spec.md#wesentliche-entitäten)): wird lokal gespeichert, gegebenenfalls mit Tempo und Bewertung ([002](../002-bewertung/spec.md)).

## Erfolgskriterien

- **SC-008**: Nach einem Absturz ist kein bereits abgeschlossener Durchlauf verloren.

## Annahmen

Es gelten die Annahmen aus [001](../001-legato-mvp/spec.md#annahmen).

## Hinweise für die Planung (nicht normativ)

Die folgenden Hinweise sind Entscheidungen über das *Wie* und gehören in den Plan, nicht in die Spec. Sobald der Plan existiert, werden sie dorthin übernommen und hier entfernt:

- **Speicherformat**: .midi für Durchläufe.
