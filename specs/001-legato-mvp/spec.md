# Feature-Spezifikation: Legato MVP

**Status**: Entwurf | **Erstellt**: 2026-10-22 | **Constitution**: [1.0.0](../../.specify/memory/constitution.md) | **Werkspezifikationen**: [hanon.md](../../docs/werke/hanon.md)

**Produktübersicht**: [produkt.md](../../docs/produkt.md) | **Glossar**: [glossar.md](../../docs/glossar.md)

## Überblick

Das MVP macht Legato zum Üben nutzbar: Der Pianist wählt eine Eingabe und lädt Noten. Beim Spielen erkennt die App Stück, Variation und Tonart, folgt dem Spiel im Notenblatt und meldet das Ende jedes Durchlaufs akustisch. Begriffe sind im [Glossar](../../docs/glossar.md) definiert.

## Umfang

Diese Spec umfasst die User Stories 1–4 (Priorität P1).

## Nutzungsszenarien und Tests

### User Story 1 – Eingabe wählen und MIDI-Verbindung überwachen (Priorität: P1)

Spielt der Pianist lange, ohne dass die MIDI-Verbindung besteht, geht sein gesamtes Spiel verloren, ohne dass er es bemerkt, weil er nicht auf das Display schaut. Die App muss deshalb eine Trennung sofort auf einem Weg melden, der ohne MIDI funktioniert, und die Verbindung selbständig wiederherstellen.

**Warum diese Priorität**: Ein unbemerkter Verbindungsverlust entwertet eine ganze Übeeinheit. Ohne zuverlässige Verbindung funktioniert keine andere Funktion.

**Unabhängiger Test**: Während eines Durchlaufs und zwischen zwei Durchläufen die MIDI-Verbindung trennen und wiederherstellen; Warnung, automatische Wiederverbindung und Statusanzeige prüfen.

**Akzeptanzszenarien**:

1. **Angenommen** eine MIDI-Verbindung besteht, **wenn** sie getrennt wird, **dann** gibt die App sofort eine Warnung über den Lautsprecher des Geräts aus und zeigt den Verbindungsverlust deutlich an.
2. **Angenommen** die Verbindung ist getrennt, **wenn** sie nicht wiederhergestellt wird, **dann** wiederholt die App die Warnung in Abständen, bis die Verbindung wiederhergestellt ist oder der Pianist die Warnung bestätigt.
3. **Angenommen** die Verbindung ist getrennt, **wenn** das MIDI-Gerät wieder verfügbar ist, **dann** verbindet sich die App selbständig, ohne dass der Pianist mit der App interagieren muss, und sendet das Verbindungssignal (User Story 4).
4. **Angenommen** ein Durchlauf läuft, **wenn** die Verbindung getrennt wird, **dann** wird dieser Durchlauf verworfen. Bereits abgeschlossene Durchläufe bleiben erhalten.
5. **Angenommen** die App läuft, **wenn** eine Ansicht (Notenblatt oder, sofern User Story 8 aus [004](../004-performance-wurm/spec.md) umgesetzt ist, Performance-Wurm) geöffnet ist, **dann** zeigt ein Symbol den Zustand der Eingabe: nicht verbunden, MIDI verbunden, Audio verbunden (sofern User Story 10 aus [006](../006-audio-eingabe/spec.md) umgesetzt ist) oder Verbindung abgebrochen.
6. **Angenommen** MIDI ist verbunden, **wenn** die Verbindung getrennt wird, **dann** wechselt die App nicht selbständig zur Audio-Eingabe.
7. **Angenommen** eine Übeeinheit läuft, **wenn** der Pianist das Gerät nicht bedient, **dann** sperrt sich der Bildschirm nicht, und die App bleibt im Vordergrund aktiv.
8. **Angenommen** ein Durchlauf läuft, **wenn** das Betriebssystem oder der Pianist die App in den Hintergrund schickt, **dann** wird der Durchlauf verworfen, und die App zeigt beim Zurückkehren einen Hinweis an.
9. **Angenommen** der Pianist hat keine Eingabeart gewählt, **wenn** er spielt, **dann** findet keine Zuordnung statt, und die App warnt nicht akustisch; das Symbol zeigt „nicht verbunden“.
10. **Angenommen** eine Ansicht ist geöffnet, **wenn** die Eingabe verbunden ist, **dann** ist das Symbol klein und unauffällig. Ist sie nicht verbunden oder abgebrochen, ist es deutlich sichtbar. Die Zustände unterscheiden sich in der Form, nicht nur in der Farbe, und das Symbol steht in allen Ansichten an derselben Stelle.
11. **Angenommen** das Symbol wird angezeigt, **wenn** der Pianist es antippt, **dann** öffnet sich eine kleine Auswahl mit den verfügbaren MIDI-Geräten und, sofern User Story 10 umgesetzt ist, dem Mikrofon, der aktuellen Wahl, einer Anzeige, ob Eingaben ankommen, und bei abgebrochener Verbindung der Möglichkeit, die Warnung zu bestätigen und die Verbindung neu aufzubauen.
12. **Angenommen** der Pianist hat einmal eine Eingabe gewählt, **wenn** er die App erneut startet, **dann** stellt die App diese Eingabe selbständig wieder her (dasselbe MIDI-Gerät bzw. das Mikrofon).
13. **Angenommen** ein Durchlauf läuft, **wenn** der Pianist die Eingabe wechselt, **dann** wird der Durchlauf verworfen.
14. **Angenommen** mehrere MIDI-Geräte sind verfügbar, **wenn** der Pianist eines davon wählt, **dann** gehen auch die akustischen Signale (User Story 4) standardmäßig an dieses Gerät.
15. **Angenommen** die Verbindung ist getrennt und lässt sich auf der Plattform nicht ohne Zutun des Pianisten wiederherstellen (z. B. Bluetooth-MIDI), **wenn** das MIDI-Gerät wieder verfügbar ist, **dann** kann der Pianist die Verbindung über das Symbol mit möglichst wenigen Schritten neu aufbauen.

---

### User Story 2 – Noten aus einer Datei laden (Priorität: P1)

Der Pianist lädt ein Stück aus einer lokalen Datei, um es ohne Backend üben zu können.

**Warum diese Priorität**: Mit der App wird nur Hanon Nr. 1 ausgeliefert. Ohne Anmeldung sind lokale Dateien die einzige Quelle für weitere Stücke.

**Unabhängiger Test**: Eine gültige und eine ungültige Notendatei laden und Anzeige bzw. Fehlermeldung prüfen.

**Akzeptanzszenarien**:

1. **Angenommen** eine gültige Notendatei liegt vor, **wenn** der Pianist sie lädt, **dann** wird das Stück angezeigt und kann gespielt werden.
2. **Angenommen** eine Datei ist ungültig oder wird nicht unterstützt, **wenn** der Pianist sie lädt, **dann** erhält der Pianist eine verständliche Fehlermeldung und die App bleibt bedienbar.
3. **Angenommen** der Notentext einer Datei enthält keinen Fingersatz, **wenn** der Pianist sie lädt, **dann** wird das Stück trotzdem angezeigt. Analysen, die einen Fingersatz voraussetzen, sind dann nicht verfügbar und werden als solche gekennzeichnet.
4. **Angenommen** die App ist neu installiert und der Pianist ist nicht angemeldet, **wenn** er die verfügbaren Stücke ansieht, **dann** ist genau Hanon Nr. 1 verfügbar.

---

### User Story 3 – Üben mit MIDI und Notenblatt (Priorität: P1)

Ein Pianist verbindet sein Digitalpiano per MIDI und beginnt zu spielen. Die App erkennt zu Beginn jedes Durchlaufs selbständig Stück, Variation und Tonart, zeigt den Notentext und markiert ruhig die aktuelle Position. Wenn die Noten nicht auf den Bildschirm passen, scrollen sie so, dass der Blick am Zeilenende problemlos zur nächsten Zeile springen kann. Pausiert der Pianist zu lange oder spielt er etwas anderes, gilt der Durchlauf als abgebrochen.

**Warum diese Priorität**: Ohne Eingabe, Notenanzeige und Zuordnung gibt es keine der weiteren Funktionen.

**Unabhängiger Test**: Ein Stück laden, mit MIDI-Gerät spielen (oder eine aufgezeichnete MIDI-Datei einspeisen) und prüfen, dass die Positionsmarkierung dem Spiel folgt. Durchläufe mit langen Pausen und mit Zuordnungsverlust einspeisen und den Abbruch prüfen.

**Akzeptanzszenarien**:

1. **Angenommen** ein MIDI-Gerät ist verbunden und das Stück ist lokal verfügbar, **wenn** der Pianist es ohne vorherige Auswahl in einem beliebigen Tempo und in einer beliebigen zulässigen Tonart zu spielen beginnt, **dann** erkennt die App Stück, Variation und Tonart, zeigt die Noten an und markiert die aktuelle Stelle.
2. **Angenommen** ein Durchlauf ist beendet, **wenn** der Pianist ohne Interaktion mit der App ein anderes lokal verfügbares Stück, eine andere Variation oder eine andere Tonart beginnt, **dann** erkennt die App Stück, Variation und Tonart neu und zeigt das Stück fließend an.
3. **Angenommen** ein Durchlauf läuft, **wenn** der Pianist kurzzeitig in eine andere Tonart wechselt, **dann** bleibt die erkannte Tonart bestehen und die abweichenden Töne gelten als Abweichungen. Hält der Wechsel bis zu einem Zuordnungsverlust an, gilt der Durchlauf als abgebrochen (Szenario 13).
4. **Angenommen** der Pianist hat die Erkennung eingeschränkt (z. B. nur Hanon Nr. 1, nur rechte Hand, Tonart frei), **wenn** er zu spielen beginnt, **dann** berücksichtigt die Erkennung nur die zugelassenen Stücke, Variationen und Tonarten.
5. **Angenommen** der Pianist spielt nur mit einer Hand, **wenn** die App das Stück erkennt, **dann** ordnet sie den Durchlauf der Variation für diese Hand zu, und die Noten der anderen Hand gelten nicht als Auslassungen. Welche Hand spielt, ergibt sich aus einer Einschränkung (Szenario 4) oder, falls keine besteht, aus der Werkspezifikation.
6. **Angenommen** der Pianist spielt, **wenn** er das Ende der sichtbaren Zeile erreicht, **dann** ist die nächste Zeile bereits sichtbar und die Ansicht scrollt fließend weiter.
7. **Angenommen** der Pianist spielt, **wenn** eine Auslassung, ein falscher Ton oder ein Zusatzton passiert, **dann** erscheint keine Meldung, kein Ton und keine Animation, die vom Spiel ablenkt.
8. **Angenommen** der Pianist hat die Fehlermarkierung während des Spiels eingeschaltet, **wenn** eine Auslassung, ein falscher Ton oder ein Zusatzton passiert, **dann** wird die Stelle im Notenblatt dezent markiert. Die Markierung beruht auf der Zuordnung während des Spiels und kann daher von der Bewertung abweichen.
9. **Angenommen** der Pianist hat die Fehlermarkierung nach dem Spiel eingeschaltet, **wenn** der Durchlauf abgeschlossen ist, **dann** werden alle Auslassungen, falschen Töne und Zusatztöne im Notenblatt dezent markiert.
10. **Angenommen** der Pianist hat beide Fehlermarkierungen ausgeschaltet, **wenn** eine Auslassung, ein falscher Ton oder ein Zusatzton passiert, **dann** werden keine Fehler im Notenblatt markiert.
11. **Angenommen** ein Durchlauf läuft, **wenn** der Pianist an einer notierten Pause länger wartet, **dann** wartet die App bis zu einer Obergrenze auf das Weiterspielen. Erst danach gilt der Durchlauf als abgebrochen.
12. **Angenommen** ein Durchlauf läuft, **wenn** der Pianist außerhalb einer notierten Pause länger als eine Grenzzeit nicht spielt, **dann** gilt der Durchlauf als abgebrochen.
13. **Angenommen** ein Durchlauf läuft, **wenn** der Pianist nahtlos etwas anderes spielt, bis ein Zuordnungsverlust eintritt, **dann** gilt der Durchlauf als abgebrochen, und die App beginnt mit der Erkennung eines neuen Durchlaufs.
14. **Angenommen** der Pianist hat in den Einstellungen die Grenzzeiten geändert, **wenn** der nächste Durchlauf beginnt, **dann** gelten die neuen Werte.
15. **Angenommen** der Pianist hat in den Experteneinstellungen ein anderes Verfahren für die Zuordnung während des Spiels gewählt, **wenn** der nächste Durchlauf beginnt, **dann** verwendet die App dieses Verfahren.
16. **Angenommen** keine Onlineverbindung besteht, **wenn** der Pianist ein lokal verfügbares Stück spielt, **dann** funktionieren alle Schritte dieser Story uneingeschränkt.
17. **Angenommen** der Pianist benutzt das Sustain-Pedal, **wenn** er spielt, **dann** wird das Pedal für Zuordnung und Abweichungen nicht ausgewertet. Zuordnung und Abweichungen beziehen sich nur auf die Tasten.
18. **Angenommen** die Werkspezifikation legt mehrere Anzeigevarianten fest (z. B. Hanon), **wenn** der Pianist eine andere Variante wählt, **dann** zeigt das Notenblatt das Stück fließend in dieser Variante an, und die Positionsmarkierung folgt dem Spiel weiterhin.

---

### User Story 4 – Akustisches Signal am Ende eines Durchlaufs (Priorität: P1)

Der Pianist soll nicht auf das Display schauen müssen, um zu wissen, ob die Eingabe verbunden ist und ob ein Durchlauf gewertet wurde. Deshalb sendet die App per MIDI kurze, unauffällige Signale an das Instrument: ein Verbindungssignal, sobald die MIDI-Verbindung (wieder) hergestellt ist, ein OK-Signal für abgeschlossene und ein Abbruch-Signal für abgebrochene Durchläufe. Die Signale sind getrennt einstellbar:

| Einstellung | Optionen | Standard |
|---|---|---|
| Verbindungssignal | an · aus | an |
| OK-Signal | nach jedem Durchlauf · wenn der Pianist aufgehört hat · aus | wenn der Pianist aufgehört hat |
| Abbruch-Signal | sofort · aus | sofort |
| Tonfolgen, Lautstärke | einstellbar | je höchstens 2 Töne, leise, in hoher Lage; aufsteigend für OK, absteigend für Abbruch, gleichbleibend für Verbindung |

„Aufgehört“ ist im [Glossar](../../docs/glossar.md#üben) unter „Aufhören“ definiert. So stört das OK-Signal nicht, wenn mehrere Durchläufe nahtlos aufeinander folgen, wie bei den Wiederholungen von Hanon. In der Standardeinstellung ertönt das OK-Signal zum selben Zeitpunkt, zu dem die kurze Rückmeldung erscheint (User Story 5 in [002](../002-bewertung/spec.md)).

**Warum diese Priorität**: Unterstützt das Üben ohne Blick auf die App (Constitution I).

**Unabhängiger Test**: Referenz-Durchläufe einspeisen, die abgeschlossen bzw. abgebrochen werden, teils nahtlos aufeinander folgend, und die an den MIDI-Ausgang gesendeten Töne prüfen.

**Akzeptanzszenarien**:

1. **Angenommen** das OK-Signal steht auf „nach jedem Durchlauf“, **wenn** ein Durchlauf abgeschlossen ist, **dann** sendet die App sofort das OK-Signal.
2. **Angenommen** das OK-Signal steht auf „wenn der Pianist aufgehört hat“, **wenn** der Pianist fünf Durchläufe nahtlos nacheinander spielt und dann aufhört, **dann** sendet die App das OK-Signal genau einmal, nach dem Aufhören.
3. **Angenommen** das OK-Signal steht auf „wenn der Pianist aufgehört hat“, **wenn** seit dem letzten Signal kein Durchlauf abgeschlossen wurde, **dann** sendet die App kein OK-Signal.
4. **Angenommen** das Abbruch-Signal steht auf „sofort“, **wenn** ein Durchlauf abgebrochen wird, **dann** sendet die App sofort das Abbruch-Signal.
5. **Angenommen** der Pianist spielt nach dem letzten Durchlauf noch einige Töne, ohne dass ein neuer Durchlauf erkannt wird, **wenn** er aufhört, **dann** gilt das nicht als Abbruch, und es ertönt kein Abbruch-Signal.
6. **Angenommen** das Instrument meldet empfangene Töne über MIDI zurück, **wenn** die App ein Signal sendet, **dann** werden dessen Töne nicht als Spiel des Pianisten erfasst.
7. **Angenommen** der Pianist hat in den Einstellungen die Signale geändert, **wenn** der nächste Durchlauf beginnt, **dann** verwendet die App die neuen Werte.
8. **Angenommen** das Instrument kann kein MIDI empfangen, die MIDI-Verbindung ist getrennt oder die Audio-Eingabe ist aktiv, **wenn** ein Signal fällig ist, **dann** wird kein MIDI-Signal gesendet.
9. **Angenommen** das Verbindungssignal steht auf „an“, **wenn** die MIDI-Verbindung hergestellt (auch durch Wahl des Geräts oder Wiederherstellen beim Start) oder wiederhergestellt ist, **dann** sendet die App sofort das Verbindungssignal.
10. **Angenommen** zwei Durchläufe unterscheiden sich in ihrer Qualität, **wenn** für beide das OK-Signal ertönt, **dann** ist es jeweils identisch (eingestellte Tonfolge und Lautstärke) und drückt keine Belohnung oder Wertung aus.

---

### Randfälle

| Randfall | Verhalten |
|---|---|
| Das MIDI-Gerät wird während eines Durchlaufs getrennt. | Der Durchlauf wird verworfen. Die App warnt sofort über den Lautsprecher des Geräts und verbindet sich selbständig neu (User Story 1). |
| Der Pianist spielt nach dem letzten Durchlauf noch einige Töne und hört dann auf. | Ohne eindeutige Erkennung beginnt kein Durchlauf; es gibt daher keinen Abbruch (User Story 4). |
| Die App gelangt während eines Durchlaufs in den Hintergrund (z. B. durch einen Anruf). | Der Durchlauf wird verworfen; beim Zurückkehren erscheint ein Hinweis (User Story 1). |
| Der Pianist spielt nur mit einer Hand. | Gültiger Durchlauf in der Variation „nur linke Hand“ bzw. „nur rechte Hand“ (User Story 3). |
| Der Pianist pausiert lange oder hört mitten in einem Durchlauf auf. | An einer notierten Pause wartet die App bis zu einer Obergrenze, sonst gilt der Durchlauf nach einer Grenzzeit als abgebrochen. Beide Werte sind einstellbar (User Story 3). |
| Der Pianist spielt mitten in einem Durchlauf nahtlos etwas anderes. | Sobald ein Zuordnungsverlust eintritt, gilt der Durchlauf als abgebrochen; die Erkennung beginnt neu (User Story 3). |
| Das Instrument meldet die Töne des akustischen Signals über MIDI zurück. | Die Töne werden nicht als Spiel erfasst (User Story 4). |
| Das Sustain-Pedal wird benutzt. | Das Pedal wird für Zuordnung und Abweichungen nicht ausgewertet. Die Artikulation bezieht sich auf die Tasten, nicht auf den Klang. |
| Der Pianist spielt extrem langsam. | Ist der Abstand zwischen zwei Anschlägen größer als die Grenzzeit für Pausen, gilt die Pausenregel. Eine weitere Untergrenze gibt es nicht. |
| Der Pianist spielt extrem schnell. | Keine Obergrenze. |
| Mehrere Stücke beginnen gleich, sodass die Erkennung erst nach einigen Noten eindeutig ist. | Der Pianist kann die Erkennung auf bestimmte Stücke, Variationen und Tonarten einschränken (User Story 3). |
| Der Pianist wechselt während eines Durchlaufs die Tonart. | Nicht möglich: Die erkannte Tonart bleibt bis zum Ende des Durchlaufs bestehen, abweichende Töne gelten als Abweichungen. Hält der Wechsel an, führt er zu einem Zuordnungsverlust und damit zum Abbruch. Der nächste Durchlauf kann in einer beliebigen Tonart beginnen (User Story 3). |
| Der Pianist wiederholt einen Abschnitt beliebig oft oder gar nicht. | Gemäß Werkspezifikation (siehe [hanon.md](../../docs/werke/hanon.md)). |
| Der Pianist spielt in einer anderen Lage als notiert. | Gemäß Werkspezifikation (siehe [hanon.md](../../docs/werke/hanon.md)). |
| Der Pianist spielt ein Stück, das nicht lokal verfügbar ist. | Ohne eindeutige Erkennung beginnt kein Durchlauf (FR-016). |

## Anforderungen

Die Schlüsselwörter MUSS, SOLL, DARF NICHT usw. sind im [Glossar](../../docs/glossar.md#schlüsselwörter) definiert.

### Funktionale Anforderungen

**Eingabe**

- **FR-001**: Die App MUSS Klavierspiel über MIDI empfangen.
- **FR-002**: Die Eingabeart und das MIDI-Gerät MUSS der Pianist ausdrücklich wählen; die App DARF NICHT selbständig zwischen MIDI und Audio wechseln. Die App MUSS die zuletzt gewählte Eingabe beim Start wiederherstellen. Ein Wechsel der Eingabe während eines Durchlaufs verwirft diesen.
- **FR-003**: Das Sustain-Pedal DARF NICHT für Zuordnung und Abweichungen ausgewertet werden. Die Steuerung per Pedal im Playlist-Modus regelt FR-082 in [009](../009-playlist/spec.md).
- **FR-004**: Wird die Eingabe (MIDI-Gerät oder Mikrofon) während eines Durchlaufs getrennt, MUSS die App den Durchlauf verwerfen.
- **FR-005**: Wird eine bestehende MIDI-Verbindung getrennt, MUSS die App sofort über den Lautsprecher des Geräts warnen und die Warnung in Abständen wiederholen, bis die Verbindung wiederhergestellt oder die Warnung bestätigt ist.
- **FR-006**: Die App MUSS eine getrennte MIDI-Verbindung selbständig wiederherstellen, sobald das Gerät wieder verfügbar ist. Ist das auf einer Plattform nicht möglich, MUSS der Pianist die Verbindung über das Symbol (FR-007) mit möglichst wenigen Schritten neu aufbauen können.
- **FR-007**: Die App MUSS in jeder Ansicht (FR-022) per Symbol anzeigen, ob die Eingabe nicht verbunden, per MIDI verbunden, per Audio verbunden (sofern User Story 10 umgesetzt ist) oder abgebrochen ist. Das Symbol MUSS unauffällig sein, solange die Eingabe verbunden ist, und sich in der Form je Zustand unterscheiden. Antippen MUSS eine Auswahl der Eingabe öffnen, in der der Pianist auch die Warnung bei Verbindungsverlust bestätigen kann. Ohne gewählte Eingabeart findet keine Zuordnung statt.

**Noten**

- **FR-008**: Die App MUSS Stücke aus lokalen Dateien laden können.
- **FR-009**: Werkspezifische Regeln (z. B. Wiederholungen, Transpositionen, Fingersatz, Anzeigevarianten) MÜSSEN gemäß der jeweiligen Werkspezifikation umgesetzt werden (z. B. [hanon.md](../../docs/werke/hanon.md)).
- **FR-010**: Mit der App MUSS Hanon Nr. 1 ausgeliefert werden. Weitere Stücke DÜRFEN NICHT ausgeliefert werden.
- **FR-011**: Die App MUSS die Anzeigevarianten anbieten, die die Werkspezifikation festlegt. Der Pianist MUSS zwischen ihnen wählen können.

**Zuordnung**

- **FR-012**: Die App MUSS jeden gespielten Ton einer Note des Notentexts zuordnen oder als falschen Ton bzw. Zusatzton erkennen.
- **FR-013**: Die App MUSS Auslassungen erkennen.
- **FR-014**: Die App MUSS beliebig viele Wiederholungen eines wiederholbaren Abschnitts zuordnen können.
- **FR-015**: Die App MUSS transponiertes Spiel zuordnen können, soweit das Werk es zulässt.
- **FR-016**: Die App MUSS zu Beginn jedes Durchlaufs Stück, Variation und Tonart unter den lokal verfügbaren Stücken selbständig erkennen, ohne dass der Pianist sie vorher auswählt. Während eines Durchlaufs DÜRFEN sie sich NICHT ändern; abweichende Töne gelten als Abweichungen. Ein Durchlauf beginnt, sobald die Erkennung eindeutig ist; die bis dahin gespielten Töne gehören dazu. Ohne eindeutige Erkennung entsteht kein Durchlauf.
- **FR-017**: Die App MUSS Spiel in jedem vom Pianisten gewählten Tempo zuordnen, ohne dass ein Tempo vorgegeben wird.
- **FR-018**: Der Pianist MUSS die Erkennung optional auf bestimmte Stücke, Variationen („beidhändig“, „nur linke Hand“, „nur rechte Hand“) und Tonarten einschränken können.
- **FR-019**: Die App MUSS einhändiges Spiel als Variation „nur linke Hand“ bzw. „nur rechte Hand“ erkennen. Die Noten der anderen Hand DÜRFEN dann NICHT als Auslassungen gelten. Welche Hand spielt, ergibt sich aus einer Einschränkung (FR-018) oder, falls keine besteht, aus der Werkspezifikation.
- **FR-020**: Die App MUSS einen Durchlauf als abgebrochen werten und verwerfen, wenn der Pianist außerhalb einer notierten Pause länger als eine Grenzzeit nicht spielt. An einer notierten Pause MUSS sie bis zu einer Obergrenze auf das Weiterspielen warten. Der Pianist MUSS beide Werte einstellen können.
- **FR-021**: Die App MUSS einen Durchlauf bei einem Zuordnungsverlust als abgebrochen werten und verwerfen. Danach MUSS sie mit der Erkennung eines neuen Durchlaufs beginnen.

**Darstellung**

- **FR-022**: Die App MUSS die Ansicht „Notenblatt“ anbieten. Gibt es weitere Ansichten (Performance-Wurm, FR-046), MUSS der Pianist zwischen ihnen wählen können.
- **FR-023**: Alle Bewegungen und Farbwechsel MÜSSEN fließend erfolgen, nie sprung- oder ruckhaft.
- **FR-024**: Während eines Durchlaufs DARF die App KEINE Meldungen, Töne, Belohnungsanimationen oder Aufforderungen ausgeben. Ausgenommen sind die Ansichten selbst (FR-022), die dezente Fehlermarkierung (FR-025), die akustischen Signale (FR-027 bis FR-029) und die Warnung bei Verbindungsverlust (FR-005).
- **FR-025**: Der Pianist MUSS getrennt einstellen können, ob Auslassungen, falsche Töne und Zusatztöne im Notenblatt während und/oder nach dem Durchlauf dezent markiert werden. Die Fehlermarkierung MUSS auf der Zuordnung während des Spiels beruhen.
- **FR-026**: Die App DARF KEINE Mechanismen zur Bindung des Nutzers einsetzen, insbesondere keine Push-Benachrichtigungen zum Üben.

**Signale**

- **FR-027**: Die App SOLL per MIDI ein Verbindungssignal senden, sobald eine MIDI-Verbindung hergestellt oder wiederhergestellt ist. Der Pianist MUSS wählen können zwischen „an“ (Standard) und „aus“.
- **FR-028**: Die App SOLL per MIDI ein OK-Signal für abgeschlossene Durchläufe senden. Der Pianist MUSS wählen können zwischen „nach jedem Durchlauf“, „wenn der Pianist aufgehört hat“ (Standard; höchstens ein Signal, sofern seit dem letzten Signal mindestens ein Durchlauf abgeschlossen wurde) und „aus“.
- **FR-029**: Die App SOLL per MIDI ein Abbruch-Signal senden, sobald ein Durchlauf abgebrochen wird. Der Pianist MUSS wählen können zwischen „sofort“ (Standard) und „aus“.
- **FR-030**: Töne, die die App selbst sendet, DÜRFEN NICHT als Spiel des Pianisten erfasst werden.
- **FR-031**: Alle Signale (FR-027 bis FR-029) MÜSSEN neutral sein (keine Belohnung). Tonfolge und Lautstärke MÜSSEN einstellbar sein.

**Experteneinstellungen**

- **FR-032**: Die App MUSS in einem eigenen Bereich für Experteneinstellungen mehrere Verfahren für die Zuordnung während des Spiels zur Wahl anbieten. Ein Standardverfahren MUSS voreingestellt sein. Eine Änderung gilt ab dem nächsten Durchlauf.

### Nicht-funktionale Anforderungen

- **NFR-001**: Die App MUSS auf iOS, Android, Windows, macOS und Linux laufen.
- **NFR-002**: Die Grundfunktionalität (siehe [Produktübersicht](../../docs/produkt.md#bestandteile)) MUSS ohne Onlineverbindung funktionieren, in dieser Spec also alle Anforderungen. Das gilt nicht für die Web-App ([008](../008-web-app/spec.md)).
- **NFR-003**: Die Latenz zwischen Anschlag und Anzeige DARF NICHT als störend wahrnehmbar sein. [NEEDS CLARIFICATION: Zielwert in ms]
- **NFR-004**: Die Oberfläche setzt musikalisches Fachwissen voraus und erklärt keine Grundlagen.
- **NFR-005**: Während einer Übeeinheit MUSS die App im Vordergrund aktiv bleiben und die automatische Bildschirmsperre verhindern. Gelangt sie trotzdem in den Hintergrund, MUSS ein laufender Durchlauf verworfen werden.
- **NFR-006**: Die App MUSS mehrsprachig sein. Die Sprache MUSS sich nach der Regionseinstellung des Geräts richten. Wird die dort eingestellte Sprache nicht unterstützt, MUSS Englisch verwendet werden. Zunächst MÜSSEN Englisch und Deutsch unterstützt werden. Akzeptanz: **Angenommen** die Regionseinstellung des Geräts ist `de-AT` bzw. `fr-FR`, **wenn** der Pianist die App startet, **dann** erscheint die Oberfläche auf Deutsch bzw. Englisch.

### Wesentliche Entitäten

- **Pianist**: Nutzer der App. Konto, Land und Spielniveau folgen in [007](../007-konto-backend/spec.md).
- **Werk**: Sammlung von Stücken mit Komponist und Titel. Verweist auf eine Werkspezifikation mit werkspezifischen Regeln.
- **Stück**: Gehört zu einem Werk. Hat einen Notentext mit optionalem Fingersatz, wiederholbaren Abschnitten und zulässigen Transpositionen.
- **Übeeinheit**: Gehört zu einem Pianisten. Enthält einen oder mehrere Durchläufe, auch verschiedener Stücke.
- **Durchlauf**: Aufgezeichnetes Spiel eines abgegrenzten Abschnitts mit Eingabeart, Variation, Tonart, Zuordnung und Abweichungen.
- **Abweichung**: Auslassung, falscher Ton oder Zusatzton an einer bestimmten Stelle (Note, Hand und, sofern der Notentext einen Fingersatz enthält, Finger). Ungleichmäßigkeiten folgen in [002](../002-bewertung/spec.md).

## Erfolgskriterien

- **SC-001**: Bei MIDI-Eingabe werden mindestens [NEEDS CLARIFICATION: Zielwert, z. B. 99 %] der Töne in Referenz-Durchläufen korrekt zugeordnet.
- **SC-002**: Bei MIDI-Eingabe erkennt die App Stück, Variation und Tonart in Referenz-Durchläufen zu Beginn jedes Durchlaufs innerhalb der Erkennungslänge korrekt (z. B. 8 Töne je Hand bei Hanon Nr. 1), sofern sich die zugelassenen Stücke, Variationen und Tonarten innerhalb dieser Länge unterscheiden.
- **SC-003**: Nach der Trennung einer MIDI-Verbindung ertönt die Warnung innerhalb von [NEEDS CLARIFICATION: Zielwert, z. B. 2 s].
- **SC-004**: Während eines Durchlaufs erscheint keinerlei unaufgeforderte Meldung.
- **SC-005**: Die App versendet keine Benachrichtigungen außerhalb der laufenden App.

## Annahmen

- Pianisten, die MIDI nutzen, besitzen ein Digitalpiano oder ein akustisches Klavier mit MIDI-Ausgang.
- Sinnvolle Standardwerte für die Grenzzeiten bei Pausen, für den Grenzwert des Zuordnungsverlusts, für die Ruhezeit, nach der eine Übeeinheit endet, und für das akustische Signal werden durch Ausprobieren ermittelt.
- Digitalpianos mit MIDI-Eingang geben empfangene Töne mit ihrem eigenen Klang wieder.
- Das Gerät, auf dem die App läuft, hat einen Lautsprecher.

## Hinweise für die Planung (nicht normativ)

Die folgenden Hinweise sind Entscheidungen über das *Wie* und gehören in den Plan, nicht in die Spec. Sobald der Plan existiert, werden sie dorthin übernommen und hier entfernt:

- **Notendateien**: z. B. MusicXML.
- **Sprache**: Aus der Regionseinstellung des Geräts (z. B. `de-AT` → Deutsch, `fr-CH` → Französisch). Texte in Ressourcendateien je Sprache, Englisch als Fallback auch für einzelne fehlende Texte.
- **Zuordnung während des Spiels**: Online-Verfahren (*score following*), an [Matchmaker](https://github.com/pymatchmaker/matchmaker) orientiert ([Park et al., 2025](../../docs/literatur.md)): Pipeline `Stream` → `Processor` → `OnlineAlignment`, Verfahren als austauschbare Plugins (FR-032). Kandidaten für MIDI: Pitch-HMM, HMM nach Cancino-Chacón et al., Outer-Product-HMM ([Nakamura et al., 2014](../../docs/literatur.md)), Online Time Warping nach Dixon bzw. Arzt. Matchmaker ist Python und folgt einer bekannten Partitur; für die App ist es daher nur Vorbild (Portierung nötig), und die Erkennung (FR-016) erfordert mehrere parallel verfolgte Kandidaten (Stück, Variation, Tonart), vgl. `arzt_multi_fold`.
- **Bluetooth-MIDI**: Prüfen, ob sich Bluetooth-MIDI-Verbindungen (insbesondere unter iOS) ohne Zutun des Pianisten wiederherstellen lassen (FR-006).
- **Repositories**: Aufteilung des Projekts auf mehrere GitHub-Repositories.
