# Feature-Spezifikation: Bewertung

**Status**: Entwurf | **Erstellt**: 2026-10-23 | **Constitution**: [1.0.0](../../.specify/memory/constitution.md) | **Abhängig von**: [001-legato-mvp](../001-legato-mvp/spec.md)

**Produktübersicht**: [produkt.md](../../docs/produkt.md) | **Glossar**: [glossar.md](../../docs/glossar.md)

## Überblick

Nach einem Durchlauf bewertet die App das Spiel und zeigt nach dem Aufhören eine kurze Rückmeldung. Alles funktioniert ohne Anmeldung und offline.

## Umfang

Diese Spec umfasst User Story 5 (Priorität P2).

## Nutzungsszenarien und Tests

### User Story 5 – Bewertung nach dem Durchlauf (Priorität: P2)

Nach jedem Durchlauf kann der Pianist eine Bewertung abrufen. Sie besteht aus einer Zahl und den Stellen, die zu Abzügen geführt haben. Abzüge entstehen durch Auslassungen, falsche Töne, Zusatztöne sowie zeitliche, dynamische und artikulatorische Ungleichmäßigkeit. Das Tempo bestimmt der Pianist selbst (Constitution II). Es fließt nicht in die Bewertung ein, sondern wird mit ihr angezeigt und in der langfristigen Entwicklung über den Schwierigkeitsgrad berücksichtigt (User Story 14 in [007](../007-konto-backend/spec.md)).

[NEEDS CLARIFICATION: Gewichtung der einzelnen Kriterien in der Bewertung.]

[NEEDS CLARIFICATION: Geht eine längere, aber noch zulässige Pause an einer notierten Pause in die Bewertung ein?]

**Warum diese Priorität**: Die Bewertung ist die Grundlage für jede weitere Analyse und für die langfristige Entwicklung.

**Unabhängiger Test**: Referenz-MIDI-Dateien mit bekannten Abweichungen einspeisen und Bewertung sowie zugeordnete Stellen prüfen.

**Akzeptanzszenarien**:

1. **Angenommen** ein Durchlauf ist abgeschlossen, **wenn** der Pianist die Bewertung abruft, **dann** sieht er eine positive Zahl (größer ist besser), das Tempo des Durchlaufs und die Stellen, die die Bewertung verschlechtert haben (Note, Hand und, sofern der Notentext einen Fingersatz enthält, Finger).
2. **Angenommen** zwei Durchläufe unterscheiden sich nur durch Agogik, Dynamik oder allmähliche Änderungen der Artikulation, **wenn** beide bewertet werden, **dann** erhalten sie dieselbe Bewertung.
3. **Angenommen** zwei Durchläufe unterscheiden sich nur im Tempo, und ihre Ungleichmäßigkeiten sind im Verhältnis zum jeweiligen Tempo gleich groß, **wenn** beide bewertet werden, **dann** erhalten sie dieselbe Bewertung.
4. **Angenommen** zwei Durchläufe unterscheiden sich nur durch Auslassungen, falsche Töne, Zusatztöne oder Ungleichmäßigkeiten, **wenn** beide bewertet werden, **dann** erhält der mit weniger Abweichungen die höhere Bewertung.
5. **Angenommen** zwei Durchläufe unterscheiden sich nur darin, dass einzelne Töne einer Hand kürzer oder länger gehalten werden als ihre Nachbarn, **wenn** beide bewertet werden, **dann** erhält der gleichmäßiger artikulierte Durchlauf die höhere Bewertung.
6. **Angenommen** ein Ton hat keinen folgenden Ton derselben Hand (z. B. am Ende des Durchlaufs), **wenn** der Durchlauf bewertet wird, **dann** wird für ihn keine Artikulation bewertet.
7. **Angenommen** ein Durchlauf wurde bewertet, **wenn** er erneut bewertet wird, **dann** ist das Ergebnis auf derselben Plattform identisch und auf einer anderen Plattform im Rahmen der festgelegten Toleranzen identisch (Constitution VIII).
8. **Angenommen** keine Onlineverbindung besteht, **wenn** der Pianist die Bewertung abruft, **dann** ist sie trotzdem verfügbar.
9. **Angenommen** seit der letzten Rückmeldung wurde genau ein Durchlauf abgeschlossen, **wenn** der Pianist aufhört, **dann** zeigt die App fließend eine kurze Rückmeldung mit Bewertung und Tempo dieses Durchlaufs an.
10. **Angenommen** seit der letzten Rückmeldung wurden mehrere Durchläufe abgeschlossen, **wenn** der Pianist aufhört, **dann** nennt die kurze Rückmeldung die gespielten Stücke, die Tonarten und die Anzahl der Durchläufe.
11. **Angenommen** die kurze Rückmeldung wird angezeigt, **wenn** der Pianist sie antippt, **dann** zeigt die App eine Übersicht dieser Durchläufe mit Stück, Tonart, Tempo und Bewertung.
12. **Angenommen** ein Durchlauf ist abgeschlossen, **wenn** der Pianist nahtlos den nächsten Durchlauf spielt, **dann** erscheint keine Rückmeldung.
13. **Angenommen** die kurze Rückmeldung wird angezeigt, **wenn** der Pianist wieder zu spielen beginnt, **dann** verschwindet sie fließend.
14. **Angenommen** seit der letzten Rückmeldung wurden Durchläufe abgebrochen, **wenn** der Pianist aufhört, **dann** werden sie in der Rückmeldung nicht erwähnt, da sie verworfen sind.

---

### Randfälle

| Randfall | Verhalten |
|---|---|
| Der Pianist spielt extrem schnell. | Abweichungen unterhalb der Messgenauigkeit der Eingabe werden nicht bestraft (FR-040). |
| Ein Ton hat keinen folgenden Ton derselben Hand. | Für ihn wird keine Artikulation bewertet (User Story 5). |
| Der Pianist wartet an einer notierten Pause länger, aber noch zulässig. | [NEEDS CLARIFICATION] (User Story 5) |

## Anforderungen

Die Schlüsselwörter MUSS, SOLL, DARF NICHT usw. sind im [Glossar](../../docs/glossar.md#schlüsselwörter) definiert.

### Funktionale Anforderungen

**Bewertung**

- **FR-033**: Die App MUSS jeden abgeschlossenen Durchlauf offline bewerten. Das Ergebnis ist eine positive Zahl, größer ist besser. Verworfene Durchläufe DÜRFEN NICHT bewertet werden.
- **FR-034**: Die Bewertung DARF Agogik, Dynamik und allmähliche Änderungen der Artikulation NICHT bestrafen.
- **FR-035**: Die Bewertung MUSS Auslassungen, falsche Töne, Zusatztöne sowie zeitliche, dynamische und artikulatorische Ungleichmäßigkeit bestrafen.
- **FR-036**: Die Bewertung MUSS deterministisch und auf allen Plattformen im Rahmen der festgelegten Toleranzen identisch sein (Constitution VIII). Sie MUSS auf der nachträglichen Zuordnung des abgeschlossenen Durchlaufs beruhen und DARF NICHT vom gewählten Verfahren für die Zuordnung während des Spiels (FR-032) abhängen.
- **FR-037**: Jeder Abzug MUSS sich auf konkrete Stellen zurückführen lassen (Note, Hand und, sofern der Notentext einen Fingersatz enthält, Finger).
- **FR-038**: Die Bewertung MUSS nur auf Nachfrage oder nach dem Durchlauf angezeigt werden. Ohne Nachfrage zeigt die App nach dem Aufhören eine kurze Rückmeldung an: bei einem Durchlauf Bewertung und Tempo, bei mehreren die Stücke, Tonarten und die Anzahl der Durchläufe. Auf Antippen MUSS sie eine Übersicht dieser Durchläufe zeigen. Die Rückmeldung verschwindet, sobald der Pianist wieder spielt. Spielt er nahtlos weiter, erscheint keine Rückmeldung. Abgebrochene Durchläufe werden nicht erwähnt.
- **FR-039**: Die Artikulation MUSS je Hand zwischen einem Ton und dem folgenden Ton derselben Hand ermittelt werden. Abzüge für artikulatorische Ungleichmäßigkeit MUSS die App beiden beteiligten Tönen (und, sofern der Notentext einen Fingersatz enthält, Fingern) zuordnen.
- **FR-040**: Das Tempo DARF NICHT in die Bewertung einfließen. Ungleichmäßigkeiten MÜSSEN im Verhältnis zum Tempo gemessen werden. Abweichungen unterhalb der Messgenauigkeit der Eingabe DÜRFEN NICHT bestraft werden.
- **FR-041**: Die App MUSS das Tempo jedes Durchlaufs ermitteln und zusammen mit der Bewertung anzeigen.

### Nicht-funktionale Anforderungen

Es gelten die nicht-funktionalen Anforderungen aus [001](../001-legato-mvp/spec.md#nicht-funktionale-anforderungen). Nach NFR-002 MÜSSEN alle Anforderungen dieser Spec ohne Onlineverbindung funktionieren.

### Wesentliche Entitäten

- **Durchlauf** (ergänzt [001](../001-legato-mvp/spec.md#wesentliche-entitäten)): zusätzlich Tempo und Bewertung.
- **Abweichung** (ergänzt [001](../001-legato-mvp/spec.md#wesentliche-entitäten)): zusätzlich Ungleichmäßigkeit (zeitlich, dynamisch, artikulatorisch).

## Erfolgskriterien

- **SC-006**: Referenz-Durchläufe, die sich nur durch Agogik, Dynamik oder allmähliche Änderungen der Artikulation unterscheiden, erhalten identische Bewertungen.
- **SC-007**: Derselbe Durchlauf ergibt auf allen fünf Plattformen dieselbe Bewertung.

## Annahmen

Es gelten die Annahmen aus [001](../001-legato-mvp/spec.md#annahmen).

## Hinweise für die Planung (nicht normativ)

Die folgenden Hinweise sind Entscheidungen über das *Wie* und gehören in den Plan, nicht in die Spec. Sobald der Plan existiert, werden sie dorthin übernommen und hier entfernt:

- **Nachträgliche Zuordnung**: Offline-Verfahren (*score alignment*) auf dem abgeschlossenen Durchlauf, innerhalb der online ermittelten Abgrenzung und mit dem online erkannten Stück, Variation und Tonart. Für die Erkennung von Auslassungen, falschen Tönen und Zusatztönen vgl. [Nakamura et al. (2017)](../../docs/literatur.md). Als Referenzimplementierungen eignen sich die Offline-Matcher von [parangonar](https://github.com/sildater/parangonar).
- **Bewertung**: Notenhöhen bzw. Anschlagstärken in ein Spektrum transformieren und eine gewichtete spektrale Energie berechnen. Niedrige Frequenzen (Agogik, Dynamik) werden schwach, hohe Frequenzen (Ungleichmäßigkeit) stark gewichtet. Für die Artikulation analog mit dem relativen Maß *key overlap ratio* (KOR = KOT / IOI) bzw. *key detached ratio* (KDR = KDT / IOI), vgl. [Bresin & Battel (2000)](../../docs/literatur.md).
