# Feature-Spezifikation: Konto und Backend

**Status**: Entwurf | **Erstellt**: 2026-10-23 | **Constitution**: [1.0.0](../../.specify/memory/constitution.md) | **Werkspezifikationen**: [hanon.md](../../docs/werke/hanon.md) | **Abhängig von**: [001-legato-mvp](../001-legato-mvp/spec.md), [002-bewertung](../002-bewertung/spec.md), [003-speichern](../003-speichern/spec.md)

**Produktübersicht**: [produkt.md](../../docs/produkt.md) | **Glossar**: [glossar.md](../../docs/glossar.md)

## Überblick

Mit der Anmeldung erschließt Legato den Katalog, überträgt Durchläufe ins Backend und bietet Auswertungen über mehrere Durchläufe: Verlauf, langfristige Entwicklung, eine Highscore-Liste und, sofern [005-schwaechen](../005-schwaechen/spec.md) umgesetzt ist, Schwächen über mehrere Durchläufe.

## Umfang

Diese Spec umfasst die User Stories 11–16 (Priorität P4).

## Nutzungsszenarien und Tests

### User Story 11 – Anmelden und Katalog nutzen (Priorität: P4)

Erst nach der Anmeldung werden dem Pianisten die Stücke des Katalogs (z. B. Hanon-Übungen) und die Auswertungen angeboten. Nach der Synchronisation sind sie auch offline verfügbar. Bei bestehender Onlineverbindung prüft die App, ob es Aktualisierungen des Katalogs gibt.

**Warum diese Priorität**: Außer Hanon Nr. 1 werden mit der App keine Stücke ausgeliefert. Ohne Katalog sind die übrigen Hanon-Übungen nicht verfügbar.

**Unabhängiger Test**: Anmelden, Stück herunterladen, Verbindung trennen und das Stück offline spielen.

**Akzeptanzszenarien**:

1. **Angenommen** der Pianist hat ein Konto bei einem unterstützten Identitätsanbieter, **wenn** er sich anmeldet, **dann** braucht er kein eigenes Passwort für Legato.
2. **Angenommen** der Pianist ist nicht angemeldet, **wenn** er sich anmeldet, **dann** weist die App ihn darauf hin, dass ab jetzt seine Durchläufe per MIDI übertragen, gespeichert und ausgewertet werden und dass er unter seinem Nutzernamen in der Highscore-Liste erscheint. Der Anzeige in der Highscore-Liste kann er jederzeit widersprechen (User Story 15).
3. **Angenommen** der Pianist ist nicht angemeldet, **wenn** er die Stückauswahl öffnet, **dann** werden ihm keine Stücke aus dem Katalog angeboten.
4. **Angenommen** der Pianist ist angemeldet, **wenn** er den Katalog synchronisiert, **dann** sind die Stücke anschließend auch offline verfügbar.
5. **Angenommen** der Pianist ist angemeldet und online, **wenn** eine Aktualisierung des Katalogs vorliegt, **dann** wird außerhalb des Spiels ein unaufdringlicher Hinweis mit der Möglichkeit zur Synchronisation angezeigt. Während eines Durchlaufs erscheint kein Hinweis.
6. **Angenommen** der Pianist ist nicht angemeldet, **wenn** er sich anmeldet, **dann** schlägt die App ein Land vor, das er jederzeit ändern kann.
7. **Angenommen** der Pianist meldet sich zum ersten Mal an, **wenn** die Anmeldung abgeschlossen ist, **dann** wählt er einen von mehreren zufälligen, eindeutigen Nutzernamen oder lässt neue vorschlagen und kann den Namen jederzeit ändern (User Story 15).

---

### User Story 12 – Durchläufe ins Backend übertragen (Priorität: P4)

Mit der Anmeldung werden alle ab diesem Zeitpunkt per MIDI entstandenen Durchläufe ins Backend übertragen, um erweiterte Analysen, Statistiken und den Vergleich mit anderen Nutzern zu ermöglichen.

**Warum diese Priorität**: Die Anmeldung (User Story 11) schließt die Übertragung ein. Außerdem Voraussetzung für die langfristige Entwicklung (User Story 14) und für geräteübergreifende Nutzung.

**Unabhängiger Test**: Durchläufe vor und nach der Anmeldung spielen, teils offline, und prüfen, welche übertragen werden.

**Akzeptanzszenarien**:

1. **Angenommen** der Pianist ist angemeldet, **wenn** ein Durchlauf per MIDI abgeschlossen ist, **dann** wird er ins Backend übertragen.
2. **Angenommen** Durchläufe sind nach der Anmeldung offline entstanden, **wenn** wieder eine Verbindung besteht, **dann** werden sie nachträglich übertragen, ohne verloren zu gehen.
3. **Angenommen** eine Übertragung läuft, **wenn** die Verbindung abbricht, **dann** gilt die Übertragung als nicht erfolgt und wird bei der nächsten Verbindung wiederholt, ohne dass Durchläufe im Backend doppelt oder unvollständig vorliegen.
4. **Angenommen** Durchläufe sind vor der Anmeldung entstanden, **wenn** der Pianist sich anmeldet, **dann** werden sie nicht übertragen.
5. **Angenommen** der Pianist ist nicht angemeldet, **wenn** ein Durchlauf abgeschlossen ist, **dann** verlässt er das Gerät nicht.
6. **Angenommen** der Pianist ist angemeldet, **wenn** ein Durchlauf per Audio-Eingabe abgeschlossen ist, **dann** wird er nie übertragen.
7. **Angenommen** nicht übertragene Durchläufe liegen vor, **wenn** der Pianist sich in der App abmeldet, **dann** bleiben sie lokal gespeichert und werden nach der nächsten Anmeldung mit demselben Konto übertragen. Durchläufe, die während der Abmeldung entstehen, werden nicht übertragen.

---

### User Story 13 – Verlauf und Übersicht der Durchläufe (Priorität: P4)

Ein angemeldeter Pianist sieht seine Durchläufe im zeitlichen Verlauf: als Diagramm, das standardmäßig den gesamten Zeitraum zeigt und sich zoomen lässt, und als Tabelle. Er kann nach Stück, Variation und Tonart filtern und so die Entwicklung einzelner Stücke verfolgen. Grundlage sind die ins Backend übertragenen Durchläufe aller seiner Geräte.

**Warum diese Priorität**: Macht die Entwicklung über Wochen und Monate sichtbar, setzt aber Anmeldung und Übertragung (User Stories 11 und 12) voraus.

**Unabhängiger Test**: Durchläufe verschiedener Stücke an mehreren Tagen auf zwei Geräten spielen und übertragen, den Verlauf aufrufen, zoomen und filtern und prüfen, dass Diagramm und Tabelle dieselben Durchläufe zeigen.

**Akzeptanzszenarien**:

1. **Angenommen** der Pianist ist nicht angemeldet, **wenn** er den Verlauf aufrufen will, **dann** ist er nicht verfügbar, und ein Hinweis nennt die Anmeldung als Voraussetzung.
2. **Angenommen** der Pianist ist angemeldet, **wenn** er den Verlauf öffnet, **dann** zeigt das Diagramm den gesamten Zeitraum mit Bewertung und Tempo der Durchläufe.
3. **Angenommen** das Diagramm wird angezeigt, **wenn** der Pianist in einen Zeitraum hineinzoomt, **dann** zeigt es diesen Ausschnitt mit allen Durchläufen darin, und mit einer einzigen Aktion kehrt er zum gesamten Zeitraum zurück.
4. **Angenommen** der Pianist hat mehrere Stücke gewählt, **wenn** das Diagramm angezeigt wird, **dann** ist für jedes Stück ein eigener Verlauf erkennbar. Die Unterscheidung beruht nicht nur auf der Farbe.
5. **Angenommen** der Pianist filtert nach Stück, Variation oder Tonart, **wenn** er zwischen Diagramm und Tabelle wechselt, **dann** gilt der Filter für beide.
6. **Angenommen** die Tabelle wird angezeigt, **wenn** der Pianist nach einer Spalte sortiert, **dann** sind die Durchläufe danach geordnet. Die Tabelle zeigt mindestens Datum, Stück, Variation, Tonart, Tempo und Bewertung.
7. **Angenommen** Diagramm oder Tabelle werden angezeigt, **wenn** der Pianist einen Durchlauf antippt, **dann** öffnet sich dessen Bewertung mit den Stellen, die zu Abzügen geführt haben (User Story 5 in [002](../002-bewertung/spec.md)).
8. **Angenommen** der Pianist hat auf einem anderen Gerät gespielt, **wenn** diese Durchläufe übertragen sind, **dann** erscheinen sie im Verlauf.
9. **Angenommen** Durchläufe sind vor der Anmeldung oder per Audio-Eingabe entstanden, **wenn** der Verlauf angezeigt wird, **dann** erscheinen sie nicht, da sie nicht übertragen werden (User Story 12).
10. **Angenommen** User Story 14 ist umgesetzt, **wenn** der Pianist den Verlauf öffnet, **dann** kann er statt der Bewertungen sein Spielniveau oder seine Vertrautheit mit den gewählten Stücken anzeigen.
11. **Angenommen** der Verlauf wird angezeigt, **wenn** ein Durchlauf besonders gut oder schlecht war, **dann** hebt die App ihn nicht als Rekord hervor und zeigt weder Serien noch Aufforderungen zum Üben.

[NEEDS CLARIFICATION: Ist der Verlauf offline mit dem zuletzt geladenen Stand verfügbar?]

---

### User Story 14 – Langfristige Entwicklung (Priorität: P4)

Die App zeigt dem Pianisten auf Nachfrage, wie sich sein Spielniveau und seine Vertrautheit mit einzelnen Stücken über die Zeit entwickeln. Dabei werden Schwierigkeitsgrad der Stücke (abhängig von Variation, Tonart und gespieltem Tempo), Spielniveau, Vertrautheit und Tagesform berücksichtigt. Wer ein Stück schneller ebenso sauber spielt, zeigt ein höheres Spielniveau. Langsames, sauberes Üben wird nicht bestraft. Die langfristige Entwicklung ist eine Auswertung und steht nur angemeldeten Pianisten zur Verfügung; der Verlauf erscheint in der Übersicht aus User Story 13.

**Warum diese Priorität**: Hoher Wert für die Motivation, benötigt aber viele Durchläufe (auch vieler Nutzer, um Schwierigkeitsgrade zu schätzen).

**Unabhängiger Test**: Simulierte Reihen von Durchläufen mit bekanntem Verlauf und bekanntem Tempo einspeisen und den ermittelten Verlauf prüfen.

**Akzeptanzszenarien**:

1. **Angenommen** ein Stück ist für einen Pianisten neu, **wenn** er es wiederholt mit steigender Qualität spielt, **dann** steigt seine Vertrautheit mit diesem Stück, während sein Spielniveau sich nur langsam ändert.
2. **Angenommen** ein Pianist ist mit einem Stück vertraut, **wenn** er es lange nicht spielt, **dann** sinkt seine geschätzte Vertrautheit mit diesem Stück.
3. **Angenommen** das Spielniveau eines Pianisten ist geschätzt, **wenn** ein einzelner Durchlauf deutlich schlechter als üblich ist, **dann** ändert sich das Spielniveau nur geringfügig (Tagesform).
4. **Angenommen** viele Pianisten spielen ein Stück, **wenn** ihre Durchläufe verarbeitet werden, **dann** wird der konstante Anteil seines Schwierigkeitsgrads anhand aller Durchläufe angepasst.
5. **Angenommen** zwei Durchläufe desselben Stücks erhalten dieselbe Bewertung, aber einer wurde in höherem Tempo gespielt, **wenn** beide verarbeitet werden, **dann** steigt das Spielniveau durch den schnelleren Durchlauf stärker.
6. **Angenommen** das Spielniveau eines Pianisten ist geschätzt, **wenn** er ein Stück langsam und mit hoher Bewertung spielt, **dann** sinkt sein Spielniveau nicht.
7. **Angenommen** viele Pianisten spielen Stücke in unterschiedlichen Tempi, **wenn** ihre Durchläufe verarbeitet werden, **dann** wird anhand aller Durchläufe geschätzt, wie stark der Schwierigkeitsgrad mit dem Tempo wächst.
8. **Angenommen** eine Folge von Durchläufen wurde verarbeitet, **wenn** sie erneut verarbeitet wird, **dann** ergeben sich identische Werte.
9. **Angenommen** der Pianist ist nicht angemeldet, **wenn** er die langfristige Entwicklung aufrufen will, **dann** ist sie nicht verfügbar, und ein Hinweis nennt die Anmeldung als Voraussetzung.
10. **Angenommen** viele Pianisten spielen Stücke in unterschiedlichen Tonarten, **wenn** ihre Durchläufe verarbeitet werden, **dann** wird anhand aller Durchläufe geschätzt, wie stark jede Tonart den Schwierigkeitsgrad erhöht.
11. **Angenommen** zwei Durchläufe desselben Stücks im selben Tempo erhalten dieselbe Bewertung, aber einer wurde in einer schwierigeren Tonart gespielt, **wenn** beide verarbeitet werden, **dann** steigt das Spielniveau durch den Durchlauf in der schwierigeren Tonart stärker.

---

### User Story 15 – Highscore-Liste (Priorität: P4)

Angemeldete Pianisten erscheinen unter ihrem Nutzernamen mit Länderflagge, Übezeit und Spielniveau in einer Highscore-Liste für Tag, Monat, Jahr und gesamt. Darauf weist die App bei der Anmeldung hin (User Story 11). Der Pianist kann der Anzeige jederzeit widersprechen; seine Auswertungen bleiben davon unberührt.

**Warum diese Priorität**: Optionale soziale Funktion ohne Einfluss auf das Üben.

**Unabhängiger Test**: Ein angemeldetes, ein angemeldetes mit Widerspruch und ein nicht angemeldetes Konto. Nur der angemeldete Pianist ohne Widerspruch erscheint in der Liste, und zwar unter seinem Nutzernamen.

**Akzeptanzszenarien**:

1. **Angenommen** der Pianist ist nicht angemeldet, **wenn** eine Highscore-Liste abgerufen wird, **dann** erscheint er nicht darin.
2. **Angenommen** der Pianist ist angemeldet und hat nicht widersprochen, **wenn** eine Highscore-Liste abgerufen wird, **dann** erscheint er mit seinem Nutzernamen, seinem Land, seiner Übezeit und seinem Spielniveau in den Listen für Tag, Monat, Jahr und gesamt. Klarname und Angaben des Identitätsanbieters erscheinen nicht.
3. **Angenommen** der Pianist erscheint in den Listen, **wenn** er sein Konto löscht, **dann** verschwindet er aus allen Listen.
4. **Angenommen** der Pianist erscheint in den Listen, **wenn** er in den Einstellungen der Anzeige widerspricht, **dann** verschwindet er aus allen Listen. Übertragung und Auswertungen bleiben unverändert.
5. **Angenommen** der Pianist hat widersprochen, **wenn** er die Anzeige wieder einschaltet, **dann** erscheint er wieder in den Listen.
6. **Angenommen** der Pianist ändert seinen Nutzernamen, **wenn** der neue Name frei und zulässig ist, **dann** erscheint er in allen Listen unter dem neuen Namen. Vor der Änderung weist die App darauf hin, dass der Name öffentlich sichtbar ist.
7. **Angenommen** ein Nutzername ist vergeben, nach einer Änderung oder Kontolöschung noch gesperrt oder nicht zulässig (z. B. beleidigend oder irreführend), **wenn** der Pianist ihn wählt, **dann** lehnt die App ihn ab und nennt den Grund.
8. **Angenommen** der Nutzername „Allegro-Chopin-42“ ist vergeben, **wenn** ein anderer Pianist „allegro-chopin-42“ oder eine verwechselbare Schreibweise (z. B. mit kyrillischem „а“ statt „a“) wählt, **dann** lehnt die App den Namen ab.
9. **Angenommen** ein Pianist hat seinen Nutzernamen geändert, **wenn** ein anderer Pianist den alten Namen wählt, **dann** lehnt die App ihn bis zum Ende der Sperrfrist ab und lässt ihn danach zu.

[NEEDS CLARIFICATION: Wie wird die Übezeit gemessen (Summe der Durchläufe oder gesamte Zeit am Instrument)? Nach welcher der beiden Größen wird sortiert?]

---

### User Story 16 – Kontodaten exportieren und Konto löschen (Priorität: P4)

Ein angemeldeter Pianist kann alle zu ihm gespeicherten Daten exportieren und sein Konto mit allen Daten im Backend löschen. Export und Löschen lokaler Daten regelt User Story 7 in [003](../003-speichern/spec.md).

**Warum diese Priorität**: Vorgabe der Constitution (VII) und der DSGVO. Muss spätestens zusammen mit User Story 12 verfügbar sein.

**Unabhängiger Test**: Anmelden, Durchläufe übertragen, Export anfordern und Vollständigkeit prüfen, Konto löschen und prüfen, dass im Backend keine Daten verbleiben.

**Akzeptanzszenarien**:

1. **Angenommen** der Pianist ist angemeldet, **wenn** er einen Export anfordert, **dann** erhält er alle zu ihm gespeicherten Daten in maschinenlesbarer Form.
2. **Angenommen** der Pianist ist angemeldet, **wenn** er sein Konto löscht, **dann** werden alle seine Daten im Backend gelöscht und er erscheint in keiner Statistik mehr.

---

### Randfälle

| Randfall | Verhalten |
|---|---|
| Die Onlineverbindung bricht während einer Übertragung ab. | Die Übertragung gilt als nicht erfolgt und wird bei der nächsten Verbindung wiederholt (User Story 12). |
| Durchläufe sind vor der Anmeldung entstanden. | Sie werden nicht übertragen (User Story 12). |
| Dieselbe Person nutzt die App auf mehreren Geräten. | [NEEDS CLARIFICATION] |

## Anforderungen

Die Schlüsselwörter MUSS, SOLL, DARF NICHT usw. sind im [Glossar](../../docs/glossar.md#schlüsselwörter) definiert.

### Funktionale Anforderungen

**Katalog**

- **FR-052**: Die App MUSS Stücke aus dem Katalog nur angemeldeten Pianisten anbieten.
- **FR-053**: Synchronisierte Stücke MÜSSEN offline verfügbar bleiben.
- **FR-054**: Bei bestehender Onlineverbindung MUSS die App auf Aktualisierungen des Katalogs prüfen und außerhalb eines Durchlaufs unaufdringlich darauf hinweisen.

**Konto**

- **FR-055**: Die Anmeldung MUSS über externe Identitätsanbieter ohne eigenes Legato-Passwort möglich sein.
- **FR-056**: Bei der Anmeldung MUSS die App darauf hinweisen, dass die Durchläufe per MIDI übertragen, gespeichert und ausgewertet werden und dass der Pianist unter seinem Nutzernamen in der Highscore-Liste erscheint, sofern er nicht widerspricht.
- **FR-057**: Bei der Anmeldung MUSS die App ein Land vorschlagen. Der Pianist MUSS es jederzeit ändern können.
- **FR-058**: Bei der ersten Anmeldung MUSS das System mehrere zufällige, eindeutige Nutzernamen ohne Bezug zum Konto beim Identitätsanbieter vorschlagen. Der Pianist MUSS einen davon wählen oder neue Vorschläge anfordern können. Er MUSS seinen Nutzernamen jederzeit in einen freien, zulässigen Namen ändern können. Nutzernamen MÜSSEN ohne Rücksicht auf Groß- und Kleinschreibung eindeutig sein; verwechselbare Schreibweisen bestehender Namen sind nicht zulässig. Nach einer Änderung oder Kontolöschung frei gewordene Namen MÜSSEN eine Zeit lang gesperrt bleiben.

**Übertragung**

- **FR-059**: Nach der Anmeldung MUSS die App alle ab diesem Zeitpunkt per MIDI entstandenen Durchläufe übertragen, offline entstandene nachträglich und ohne Verlust. Eine unterbrochene Übertragung gilt als nicht erfolgt und MUSS bei der nächsten Verbindung wiederholt werden, ohne dass Durchläufe im Backend doppelt oder unvollständig vorliegen. Nicht übertragene Durchläufe MÜSSEN eine Abmeldung in der App überdauern und nach der nächsten Anmeldung mit demselben Konto übertragen werden.
- **FR-060**: Durchläufe, die vor der Anmeldung oder per Audio-Eingabe entstanden sind, DÜRFEN NICHT übertragen werden.

**Auswertungen**

- **FR-061**: Das System MUSS angemeldeten Pianisten den zeitlichen Verlauf ihrer übertragenen Durchläufe (Bewertung, Tempo) als Diagramm und als Tabelle anzeigen (siehe User Story 13). Das Diagramm MUSS standardmäßig den gesamten Zeitraum zeigen und das Zoomen erlauben. Diagramm und Tabelle MÜSSEN sich gemeinsam nach Stück, Variation und Tonart filtern lassen.
- **FR-062**: Das System MUSS das Spielniveau des Pianisten, seine Vertrautheit mit einem Stück und den Schwierigkeitsgrad eines Stücks in Abhängigkeit von Variation, Tonart und Tempo unter Berücksichtigung der Tagesform schätzen (siehe User Story 14). Die Ergebnisse DÜRFEN NUR angemeldeten Pianisten angezeigt werden.
- **FR-063**: Auswertungen DÜRFEN KEINE Rekorde oder Serien hervorheben und KEINE Aufforderungen zum Üben enthalten. Die Highscore-Liste (FR-065) ist keine Auswertung und davon ausgenommen.
- **FR-064**: Bewertet das Backend einen Durchlauf, MUSS das Ergebnis mit der Bewertung in der App im Rahmen der festgelegten Toleranzen identisch sein (FR-036).

**Highscore-Liste**

- **FR-065**: Das Backend MUSS eine Highscore-Liste für Tag, Monat, Jahr und gesamt bereitstellen, die Nutzername, Land, Übezeit und Spielniveau der Pianisten aufführt. Klarnamen und Angaben des Identitätsanbieters DÜRFEN NICHT erscheinen.
- **FR-066**: Nicht angemeldete Pianisten und Pianisten, die widersprochen haben, DÜRFEN NICHT in der Highscore-Liste erscheinen.
- **FR-067**: Der Pianist MUSS der Anzeige in der Highscore-Liste jederzeit widersprechen und sie wieder einschalten können. Ein Widerspruch DARF Übertragung und Auswertungen NICHT beeinflussen.

**Datenschutz**

- **FR-068**: Der Pianist MUSS alle seine Daten exportieren können.
- **FR-069**: Der Pianist MUSS sein Konto und alle zugehörigen Daten im Backend löschen können.
- **FR-070**: Das System MUSS die Anforderungen der DSGVO erfüllen.

### Nicht-funktionale Anforderungen

Es gelten die nicht-funktionalen Anforderungen aus [001](../001-legato-mvp/spec.md#nicht-funktionale-anforderungen).

### Wesentliche Entitäten

- **Pianist** (ergänzt [001](../001-legato-mvp/spec.md#wesentliche-entitäten)): optional mit Konto (inklusive Nutzername und gegebenenfalls Widerspruch gegen die Highscore-Liste), Land und Spielniveau.
- **Stück** (ergänzt [001](../001-legato-mvp/spec.md#wesentliche-entitäten)): je Variation ein konstanter Anteil am Schwierigkeitsgrad.
- **Tonart**: ein Anteil am Schwierigkeitsgrad, der für alle Stücke gilt.
- **Vertrautheit**: Beziehung zwischen einem Pianisten und einem Stück, verändert sich über die Zeit.

## Erfolgskriterien

- **SC-011**: Nach einer Kontolöschung sind keine personenbezogenen Daten des Pianisten mehr im Backend vorhanden.

## Annahmen

- Der Schwierigkeitsgrad hängt nur von Stück, Variation, Tonart und Tempo ab und steigt mit dem Tempo. Eine Verdopplung des Tempos erhöht ihn bei jedem Ausgangstempo um ungefähr gleich viel. Der Anteil der Tonart ist zunächst für alle Stücke gleich.
- Die ersten Inhalte des Katalogs sind die Stücke aus Hanon Teil 1 gemäß [hanon.md](../../docs/werke/hanon.md).

## Hinweise für die Planung (nicht normativ)

Die folgenden Hinweise sind Entscheidungen über das *Wie* und gehören in den Plan, nicht in die Spec. Sobald der Plan existiert, werden sie dorthin übernommen und hier entfernt:

- **Anmeldung**: OAuth, z. B. über Google, Apple und GitHub.
- **Land**: Vorschlag aus der Regionseinstellung des Geräts (z. B. `de-AT` → Österreich). Funktioniert offline und erhebt keine zusätzlichen Daten.
- **Langfristige Entwicklung**: Rating-Verfahren wie Weng-Lin/OpenSkill ([Weng & Lin, 2011](../../docs/literatur.md)). Ein Durchlauf wird modelliert als Team A (Spielniveau des Pianisten + Vertrautheit mit dem Stück) gegen Team B (Schwierigkeitsgrad). Der Schwierigkeitsgrad ist $d_\text{Stück,Variation} + d_\text{Tonart} + \beta \cdot \log(t / t_\text{ref})$ mit $d_\text{C-Dur} = 0$ und $t$ = Median der Anschlagrate der schnelleren Hand. Der Median macht $t$ unabhängig von Agogik, der Logarithmus entspricht der verhältnisbasierten Tempowahrnehmung (Weber-Gesetz, vgl. [Drake & Botte, 1993](../../docs/literatur.md)) und der Mälzel-Skala des Metronoms. $\beta$ wird zunächst global aus allen Durchläufen geschätzt, später ggf. je Stück. Ebenso $d_\text{Tonart}$: zunächst global je Tonart, später ggf. je Stück als Abweichung vom globalen Wert, sobald genügend Durchläufe vorliegen. Damit ist der Zielkonflikt zwischen Geschwindigkeit und Genauigkeit (*speed-accuracy trade-off*) abgebildet.
- **Gemeinsamer Code**: Das Backend wird in Rust geschrieben und bindet denselben Kern (`legato-core`) ein wie App und Web-App (Entscheidung aus dem Plan von [001](../001-legato-mvp/research.md#r2--gemeinsamer-kern-in-rust)). Identische Bewertungen gelten im Rahmen der festgelegten Toleranzen (FR-036, FR-064).
- **Nutzername**: Schema Vortragsbezeichnung-Komponist-Zahl (z. B. „Allegro-Chopin-42“, „Dolce-Satie-7“). Italienische Vortragsbezeichnungen sind sprachunabhängig verständlich; negativ klingende (z. B. „Morendo“, „Lamentoso“) auslassen. Nur gemeinfreie Komponisten mit kurzer ASCII-Schreibweise. Rund 40 Bezeichnungen × 50 Komponisten × 999 Zahlen ergeben rund 2 Millionen Kombinationen; bei Kollision neu würfeln. 3–5 Vorschläge zur Auswahl. Selbst gewählte Namen: 3–24 Zeichen, nur Buchstaben, Ziffern, `-` und `_`; per Sperrliste prüfen, Namen im Muster der Erstvergabe ausschließen, Verwechselbarkeit per Unicode-Skeleton (UTS #39) prüfen. Dauer der Sperre frei gewordener Namen festlegen.
- **Datenschutz**: Rechtsgrundlagen mit einer Datenschutzberatung prüfen: Übertragung, Speicherung und Auswertung als Vertragserfüllung (Art. 6 Abs. 1 lit. b DSGVO), pseudonyme Anzeige in der Highscore-Liste als berechtigtes Interesse mit Widerspruchsrecht (Art. 6 Abs. 1 lit. f, Art. 21 DSGVO). In Ländern mit wenigen Nutzern kann die Kombination aus Land, Übezeit und Spielniveau auf eine Person hindeuten; das Land ggf. erst ab einer Mindestzahl von Nutzern anzeigen.
- **Verlauf**: Übersicht und Ausschnitt (Navigator unter dem Diagramm, Schnellwahl Woche · Monat · Jahr · gesamt, Zoom per Geste). Bei großen Zeiträumen je Tag oder Übeeinheit zusammenfassen (Median und Spanne), erst beim Hineinzoomen einzelne Durchläufe zeigen. Bei vielen Stücken das gewählte Stück hervorheben, die übrigen grau. Spielniveau und Vertrautheit mit Unsicherheitsband.
