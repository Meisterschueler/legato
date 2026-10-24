# Feature-Spezifikation: Playlist

**Status**: Entwurf | **Erstellt**: 2026-10-24 | **Constitution**: [1.0.0](../../.specify/memory/constitution.md) | **Werkspezifikationen**: [hanon.md](../../docs/werke/hanon.md) | **Abhängig von**: [001-legato-mvp](../001-legato-mvp/spec.md), für Audio-Eingabe [006-audio-eingabe](../006-audio-eingabe/spec.md), für Stücke aus dem Katalog [007-konto-backend](../007-konto-backend/spec.md)

**Produktübersicht**: [produkt.md](../../docs/produkt.md) | **Glossar**: [glossar.md](../../docs/glossar.md)

## Überblick

Neben dem Erkennungsmodus, in dem die App erkennt, was der Pianist spielt, gibt es den Playlist-Modus: Der Pianist stellt vorab eine Playlist aus Stücken seines Repertoires mit je einer Tonart zusammen. Beim Üben ist die Zuordnung fest auf das aktuelle Stück eingestellt. Spielt der Pianist gerade nicht, wechselt er mit dem rechten Pedal zum nächsten, mit dem linken zum vorigen Stück, ohne die App zu bedienen; über den Bildschirm kann er jederzeit wechseln. Nach jedem Wechsel deutet die App die Figur des neuen Stücks nach einer kurzen Verzögerung ganz leise am Instrument an.

## Umfang

Diese Spec umfasst die User Stories 18–19 (Priorität P3).

## Nutzungsszenarien und Tests

### User Story 18 – Playlist zusammenstellen (Priorität: P3)

Der Pianist stellt in einer Tabelle eine Playlist aus Stücken seines Repertoires zusammen. Jede Zeile enthält ein Stück und eine Tonart. Er kann Zeilen hinzufügen, löschen und umordnen und die Tonarten einzeln oder für alle Stücke nach einer Regel setzen.

**Warum diese Priorität**: Voraussetzung für den Playlist-Modus (User Story 19).

**Unabhängiger Test**: Eine Playlist mit mehreren Stücken anlegen, umordnen, Zeilen löschen, Tonarten nach jeder Regel setzen, App neu starten und prüfen, dass die Playlist unverändert ist.

**Akzeptanzszenarien**:

1. **Angenommen** die Playlist ist geöffnet, **wenn** der Pianist ein Stück aus dem Repertoire hinzufügt (z. B. per Ziehen in die Tabelle), **dann** erscheint es an der gewählten Stelle in der notierten Tonart.
2. **Angenommen** ein Stück ist bereits in der Playlist, **wenn** der Pianist es erneut hinzufügt, **dann** erscheint es ein weiteres Mal, sodass dasselbe Stück z. B. in mehreren Tonarten geübt werden kann.
3. **Angenommen** die Playlist enthält mehrere Zeilen, **wenn** der Pianist eine Zeile an eine andere Stelle verschiebt oder zwei Zeilen tauscht, **dann** ändert sich die Reihenfolge entsprechend; Stück und Tonart der Zeilen bleiben erhalten.
4. **Angenommen** die Playlist enthält eine Zeile, **wenn** der Pianist sie löscht, **dann** ist sie entfernt und die übrigen Zeilen rücken nach.
5. **Angenommen** die Playlist enthält ein Stück, **wenn** der Pianist dessen Tonart ändert, **dann** stehen nur die Tonarten zur Wahl, die die Werkspezifikation zulässt.
6. **Angenommen** der Pianist setzt die Tonart des ersten Stücks, **wenn** er für die weiteren Stücke „gleiche Tonart“ wählt, **dann** erhalten alle Stücke diese Tonart.
7. **Angenommen** der Pianist setzt die Tonart des ersten Stücks, **wenn** er für die weiteren Stücke „chromatisch aufwärts“ wählt, **dann** liegt jedes weitere Stück einen Halbton über dem vorigen (z. B. C, Des, D, Es …).
8. **Angenommen** der Pianist setzt die Tonart des ersten Stücks, **wenn** er für die weiteren Stücke „Quintenzirkel aufwärts“ wählt, **dann** liegt jedes weitere Stück eine Quinte über dem vorigen (z. B. C, G, D, A …).
9. **Angenommen** der Pianist setzt die Tonart des ersten Stücks, **wenn** er für die weiteren Stücke „unverändert“ wählt, **dann** behalten die weiteren Stücke ihre Tonart.
10. **Angenommen** die Playlist enthält ein Stück, dessen Werkspezifikation die nach einer Regel ermittelte Tonart nicht zulässt, **wenn** die Regel angewendet wird, **dann** behält dieses Stück seine Tonart und zählt für die Regel nicht mit: Das nächste Stück erhält die Tonart, die dieses Stück erhalten hätte (z. B. „chromatisch aufwärts“ ab C: C, Des, *unverändert*, D, Es …).
11. **Angenommen** eine Regel wurde angewendet, **wenn** der Pianist danach Zeilen umordnet, **dann** behalten die Zeilen ihre Tonart; die Regel wird nicht erneut angewendet.
12. **Angenommen** eine Playlist wurde zusammengestellt, **wenn** die App beendet und neu gestartet wird, **dann** ist die Playlist unverändert vorhanden, auch offline.
13. **Angenommen** ein Stück der Playlist ist nicht mehr lokal verfügbar (z. B. Datei entfernt oder, sofern [007](../007-konto-backend/spec.md) umgesetzt ist, Katalog nach Abmeldung nicht mehr verfügbar), **wenn** der Pianist die Playlist öffnet, **dann** ist die Zeile als nicht verfügbar gekennzeichnet.

---

### User Story 19 – Üben im Playlist-Modus (Priorität: P3)

Der Pianist wählt den Playlist-Modus und beginnt mit dem ersten Stück der Playlist. Die Zuordnung ist fest auf Stück und Tonart der aktuellen Zeile eingestellt. Mit dem rechten Pedal wechselt er zum nächsten, mit dem linken Pedal zum vorigen Stück, ohne die App zu bedienen (Constitution I). Während des Spiels haben die Pedale keine Wechselfunktion, damit sie im Stück selbst benutzt werden können: Pianisten treten das Pedal häufig unbewusst, ein versehentlicher Wechsel mitten im Stück wäre fatal. Über den Bildschirm wechselt dagegen niemand versehentlich; dort ist der Wechsel daher jederzeit möglich und bricht einen laufenden Durchlauf ab. Nach jedem Wechsel deutet die App die Figur des neuen Stücks nach einer kurzen Verzögerung in der eingestellten Tonart ganz leise am Instrument an, sodass der Pianist ohne Blick auf das Display weiß, was als Nächstes kommt.

**Warum diese Priorität**: Ermöglicht strukturiertes Üben einer festen Folge von Stücken und Tonarten ohne Interaktion mit der App. Setzt Eingabe, Zuordnung und Signale (User Stories 1, 3 und 4) voraus.

**Unabhängiger Test**: Eine Playlist mit drei Stücken in verschiedenen Tonarten anlegen, den Playlist-Modus starten, Referenz-Durchläufe und Pedal-Ereignisse einspeisen und prüfen, dass Zuordnung, Wechsel und Andeutung wie beschrieben erfolgen.

**Akzeptanzszenarien**:

1. **Angenommen** eine Playlist ist vorhanden, **wenn** der Pianist den Playlist-Modus startet, **dann** ist das zuletzt aktuelle Stück wieder aktuell (beim ersten Start oder wenn es nicht mehr verfügbar ist, das erste verfügbare Stück der Playlist), das Notenblatt zeigt es in der eingestellten Tonart, und die App deutet nach einer kurzen Verzögerung seine Figur leise an.
2. **Angenommen** der Playlist-Modus ist aktiv, **wenn** der Pianist das aktuelle Stück spielt, **dann** ordnet die App das Spiel ausschließlich diesem Stück in der eingestellten Tonart zu; eine Erkennung anderer Stücke oder Tonarten findet nicht statt. Die Variation („beidhändig“, „nur linke Hand“, „nur rechte Hand“) wird wie im Erkennungsmodus erkannt.
3. **Angenommen** der Playlist-Modus ist aktiv, **wenn** der Pianist das aktuelle Stück mehrfach hintereinander spielt, **dann** wird jeder Durchlauf wie im Erkennungsmodus gewertet; das Stück bleibt aktuell, bis der Pianist wechselt.
4. **Angenommen** der Playlist-Modus ist aktiv und der Pianist spielt nicht, **wenn** er das rechte Pedal tritt, **dann** wird das nächste verfügbare Stück aktuell, das Notenblatt wechselt fließend dorthin, und die App deutet nach einer kurzen Verzögerung dessen Figur leise an.
5. **Angenommen** der Playlist-Modus ist aktiv und der Pianist spielt nicht, **wenn** er das linke Pedal tritt, **dann** wird das vorige verfügbare Stück aktuell, das Notenblatt wechselt fließend dorthin, und die App deutet nach einer kurzen Verzögerung dessen Figur leise an.
6. **Angenommen** das letzte Stück der Playlist ist aktuell, **wenn** der Pianist das rechte Pedal tritt, **dann** bleibt es aktuell, und die App sendet, sofern das Abbruch-Signal eingeschaltet ist, das Abbruch-Signal als Hinweis, dass es kein nächstes Stück gibt. Entsprechendes gilt für das linke Pedal beim ersten Stück.
7. **Angenommen** der Pianist spielt (ein Durchlauf läuft oder eine Taste ist gedrückt, z. B. der mit Pedal gehaltene Schlusston), **wenn** er ein Pedal tritt, **dann** wechselt die App das Stück nicht, und der Durchlauf läuft unverändert weiter. Das Pedal wird für Zuordnung und Abweichungen nicht ausgewertet (FR-003).
8. **Angenommen** die Andeutung ertönt, **wenn** der Pianist zu spielen beginnt, **dann** bricht die Andeutung sofort ab, und ihre Töne werden nicht als Spiel erfasst (FR-030).
9. **Angenommen** der Pianist spielt im Playlist-Modus etwas anderes als das aktuelle Stück, **wenn** ein Zuordnungsverlust eintritt, **dann** gilt der Durchlauf als abgebrochen; das aktuelle Stück bleibt unverändert.
10. **Angenommen** der Playlist-Modus ist aktiv, **wenn** die Ansicht angezeigt wird, **dann** zeigt sie unaufdringlich die Position in der Playlist (z. B. „3 / 8“) sowie Stück und Tonart des aktuellen Eintrags.
11. **Angenommen** der Playlist-Modus ist aktiv, **wenn** der Pianist das Stück auf dem Bildschirm wechselt (z. B. durch Antippen von „weiter“ oder „zurück“ oder einer Zeile der Playlist), **dann** verhält sich die App wie beim Wechsel per Pedal, allerdings auch während des Spiels: Ein laufender Durchlauf gilt als abgebrochen und wird verworfen.
12. **Angenommen** die Audio-Eingabe ist aktiv (sofern User Story 10 aus [006](../006-audio-eingabe/spec.md) umgesetzt ist) oder das Instrument hat kein linkes Pedal, **wenn** der Pianist das Stück wechseln will, **dann** ist das über den Bildschirm möglich (Szenario 11). Bei Audio-Eingabe ertönt keine Andeutung.
13. **Angenommen** der Pianist hat die Andeutung in den Einstellungen ausgeschaltet oder ihre Lautstärke geändert, **wenn** er das Stück wechselt, **dann** verwendet die App die neuen Werte.
14. **Angenommen** der Playlist-Modus ist aktiv, **wenn** der Pianist in den Erkennungsmodus wechselt, **dann** erkennt die App wieder selbständig Stück, Variation und Tonart unter allen lokal verfügbaren Stücken (FR-016).
15. **Angenommen** der Pianist hat die App im Playlist-Modus beendet, **wenn** er sie erneut startet, **dann** ist der Playlist-Modus aktiv und das zuletzt aktuelle Stück wieder aktuell.

---

### Randfälle

| Randfall | Verhalten |
|---|---|
| Der Pianist tritt ein Pedal während eines Durchlaufs oder bei gedrückter Taste. | Kein Wechsel; der Durchlauf läuft weiter (User Story 19). |
| Der Pianist hört mitten in einem Durchlauf auf und tritt sofort ein Pedal. | Kein Wechsel, solange der Durchlauf noch läuft. Erst nach dem Abbruch (Grenzzeit für Pausen, FR-020) wechselt das Pedal das Stück. |
| Der Pianist wechselt während eines Durchlaufs über den Bildschirm. | Der Durchlauf gilt als abgebrochen und wird verworfen; das Stück wechselt (User Story 19). |
| Am Ende bzw. Anfang der Playlist wird weiter- bzw. zurückgeschaltet. | Das Stück bleibt aktuell; Abbruch-Signal als Hinweis, sofern eingeschaltet (User Story 19). |
| Ein Stück der Playlist ist nicht mehr lokal verfügbar. | Die Zeile ist gekennzeichnet und wird beim Wechseln übersprungen (User Stories 18 und 19). |
| Die Playlist ist leer oder enthält kein verfügbares Stück. | Der Playlist-Modus kann nicht gestartet werden; ein Hinweis nennt den Grund. |
| Das Instrument hat nur ein Pedal oder sendet keine Pedal-Ereignisse. | Wechsel über den Bildschirm (User Story 19). |
| Das Instrument meldet die Töne der Andeutung über MIDI zurück. | Die Töne werden nicht als Spiel erfasst (FR-030). |
| Ein Stück hat keine Figur (z. B. Bach). | Angedeutet wird der Anfang des Stücks im Umfang der Erkennungslänge. |
| Eine Regel ermittelt eine Tonart, die das Werk nicht zulässt. | Das Stück behält seine Tonart und zählt für die Regel nicht mit (User Story 18). |

## Anforderungen

Die Schlüsselwörter MUSS, SOLL, DARF NICHT usw. sind im [Glossar](../../docs/glossar.md#schlüsselwörter) definiert.

### Funktionale Anforderungen

**Playlist**

- **FR-075**: Die App MUSS genau eine Playlist als Tabelle anbieten, deren Zeilen je ein Stück aus dem Repertoire und eine Tonart enthalten. Dasselbe Stück DARF mehrfach vorkommen.
- **FR-076**: Der Pianist MUSS Zeilen hinzufügen, löschen, verschieben und tauschen können. Neu hinzugefügte Stücke MÜSSEN in der notierten Tonart erscheinen.
- **FR-077**: Der Pianist MUSS die Tonart jeder Zeile einzeln setzen können, beschränkt auf die Tonarten, die die Werkspezifikation zulässt.
- **FR-078**: Setzt der Pianist die Tonart des ersten Stücks, MUSS er für die weiteren Stücke einmalig wählen können zwischen „gleiche Tonart“, „chromatisch aufwärts“ (je ein Halbton höher als das vorige Stück), „Quintenzirkel aufwärts“ (je eine Quinte höher als das vorige Stück) und „unverändert“. Stücke, deren Werkspezifikation die ermittelte Tonart nicht zulässt, MÜSSEN ihre Tonart behalten und DÜRFEN für die Regel NICHT mitzählen.
- **FR-079**: Die Playlist MUSS lokal gespeichert werden und ohne Onlineverbindung verfügbar sein. Nicht mehr lokal verfügbare Stücke MÜSSEN gekennzeichnet werden.

**Playlist-Modus**

- **FR-080**: Der Pianist MUSS zwischen Erkennungsmodus (Standard) und Playlist-Modus wählen können. Die App MUSS den zuletzt gewählten Modus beim Start wiederherstellen. Beim Start des Playlist-Modus und beim Start der App im Playlist-Modus MUSS das zuletzt aktuelle Stück wieder aktuell sein, sofern es noch verfügbar ist, sonst das erste verfügbare Stück.
- **FR-081**: Im Playlist-Modus MUSS die Zuordnung fest auf Stück und Tonart der aktuellen Zeile eingestellt sein. Die Variation MUSS weiterhin erkannt werden (FR-019). Für Durchläufe, Abbruch, Bewertung und Speichern gelten dieselben Regeln wie im Erkennungsmodus.
- **FR-082**: Im Playlist-Modus MUSS das rechte Pedal zum nächsten und das linke Pedal zum vorigen verfügbaren Stück wechseln, sofern der Pianist nicht spielt. Während eines Durchlaufs oder solange eine Taste gedrückt ist, DARF ein Pedal das Stück NICHT wechseln. Am Ende bzw. Anfang der Playlist MUSS das Stück aktuell bleiben, und die App SOLL das Abbruch-Signal senden, sofern es eingeschaltet ist (FR-029). Derselbe Wechsel MUSS jederzeit auch über den Bildschirm möglich sein, auch während eines Durchlaufs; ein laufender Durchlauf gilt dabei als abgebrochen und wird verworfen.
- **FR-083**: Nach jedem Wechsel und beim Start des Playlist-Modus SOLL die App nach einer kurzen Verzögerung per MIDI die Figur des aktuellen Stücks in der eingestellten Tonart ganz leise andeuten (Andeutung); hat das Stück keine Figur, den Anfang im Umfang der Erkennungslänge. Die Andeutung MUSS abbrechen, sobald der Pianist zu spielen beginnt. Der Pianist MUSS sie ausschalten und ihre Lautstärke einstellen können. Für die Andeutung gelten FR-030 und FR-031 sowie Szenario 8 von User Story 4 entsprechend.
- **FR-084**: Im Playlist-Modus MUSS die Ansicht unaufdringlich die Position in der Playlist sowie Stück und Tonart des aktuellen Eintrags zeigen. Der Wechsel MUSS fließend erfolgen (FR-023).

### Nicht-funktionale Anforderungen

Es gelten die nicht-funktionalen Anforderungen aus [001](../001-legato-mvp/spec.md#nicht-funktionale-anforderungen). Nach NFR-002 MÜSSEN alle Anforderungen dieser Spec ohne Onlineverbindung funktionieren.

### Wesentliche Entitäten

- **Playlist**: Genau eine je Pianist. Geordnete Liste von Einträgen.
- **Playlist-Eintrag**: Position, Stück und Tonart.
- **Durchlauf** (ergänzt [001](../001-legato-mvp/spec.md#wesentliche-entitäten)): zusätzlich der Modus, in dem er entstanden ist.

## Erfolgskriterien

- **SC-013**: Im Playlist-Modus wechselt die App bei jedem eingespeisten Pedal-Ereignis außerhalb des Spiels in Referenzdaten zum richtigen Stück, bei Pedal-Ereignissen während des Spiels nie. Durchläufe des aktuellen Stücks werden genauso zugeordnet wie im Erkennungsmodus mit Einschränkung auf dieses Stück und diese Tonart (FR-018).

## Annahmen

- Digitalpianos mit drei Pedalen senden das rechte Pedal (Dämpfer) und das linke Pedal (Una corda) als MIDI-Ereignisse.
- Sinnvolle Standardwerte für Verzögerung, Lautstärke und Tempo der Andeutung werden durch Ausprobieren ermittelt.

## Hinweise für die Planung (nicht normativ)

Die folgenden Hinweise sind Entscheidungen über das *Wie* und gehören in den Plan, nicht in die Spec. Sobald der Plan existiert, werden sie dorthin übernommen und hier entfernt:

- **Pedale**: Rechtes Pedal = MIDI-CC 64 (Sustain), linkes Pedal = MIDI-CC 67 (Soft). Auslösen beim Überschreiten eines Schwellwerts (z. B. ≥ 64) mit Hysterese, damit Halbpedal und Prellen keinen Doppelwechsel auslösen.
- **Andeutung**: Figur der rechten Hand (bzw. der gespielten Hand) einmal in mäßigem Tempo mit sehr geringer Anschlagstärke senden, auf demselben MIDI-Ausgang wie die Signale.
- **Tonart-Regeln**: Für Hanon entspricht „chromatisch aufwärts“ einer Transposition um $(i-1) \bmod 12$ Halbtöne und „Quintenzirkel aufwärts“ um $7(i-1) \bmod 12$ Halbtöne gegenüber dem ersten Stück ($i$ = Position unter den Stücken, die für die Regel mitzählen), jeweils innerhalb der zulässigen Lage (siehe [hanon.md](../../docs/werke/hanon.md)).
