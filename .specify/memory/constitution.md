# Legato – Constitution

Diese Constitution legt die unverhandelbaren Grundprinzipien von Legato fest. Jede Spezifikation, jeder Plan und jede Implementierung muss mit ihr vereinbar sein. Bei Konflikten hat die Constitution Vorrang vor allen anderen Dokumenten.

## Leitbild

Legato ist eine stille Jury: Sie hört fortgeschrittenen Pianisten konzentriert zu, erkennt systematische Schwächen und gibt auf Nachfrage hilfreiches Feedback. Sie ist kein nach Aufmerksamkeit heischender, gnadenlos-mechanischer Lehrer.

## Grundprinzipien

### I. Zurückhaltung (stille Jury)

- Die App soll den Pianisten möglichst wenig ablenken. Er soll Klavierspielen können und auch zwischen Stücken wechseln können, ohne dass er mit der App interagieren muss.
- Ausführliches Feedback wird nur nach dem Durchlauf oder auf Nachfrage angezeigt.
- Visuelle Änderungen erfolgen immer fließend („organisch“), nie sprung- oder ruckhaft.
- Es gibt keine manipulativen Mechanismen zur Bindung des Nutzers (z. B. Push-Benachrichtigungen zum Üben).

### II. Freiheit der Interpretation

- Musikalische Gestaltung wird nicht bestraft. Das gilt insbesondere für niederfrequente Änderungen von Tempo (Agogik) und Lautstärke (Dynamik).
- Bewertet werden nur objektive Abweichungen: Auslassungen, falsche Töne, Zusatztöne und hochfrequente Ungleichmäßigkeiten (Abweichungen einer Note gegenüber ihren unmittelbaren Nachbarn).
- Der Pianist bestimmt Tempo, Tonart und die Zahl der Wiederholungen selbst, soweit das Werk es zulässt.

### III. Offline zuerst

- Die Grundfunktionalität (Noten laden, anzeigen, spielen, zuordnen, bewerten, speichern) funktioniert in der App vollständig ohne Onlineverbindung. Eine Web-App darf eine Onlineverbindung voraussetzen.
- Das Backend erweitert die App, ist aber nie Voraussetzung für das Üben.
- Daten, die offline entstehen, gehen nicht verloren.

### IV. Für Fortgeschrittene

- Zielgruppe sind fortgeschrittene Pianisten. Die App ersetzt keinen Klavierunterricht.
- Inhalte sind technische Klassiker (z. B. Hanon, Czerny, Cortot, Bach).
- Die Oberfläche setzt musikalisches Fachwissen voraus und erklärt keine Grundlagen.

### V. Präzision

- MIDI ist der bevorzugte Eingabeweg. Audio über Mikrofon ist nur eine Notlösung.
- Die Latenz zwischen Anschlag und Anzeige muss so gering sein, dass der Pianist sie nicht als störend wahrnimmt.

### VI. Nachvollziehbare Bewertung

- Bewertungen sind deterministisch: Dieselbe Eingabe ergibt dieselbe Bewertung.
- Jede Bewertung und jede erkannte Schwäche muss sich auf konkrete Stellen im Spiel zurückführen lassen (welche Note, welche Hand und, sofern der Notentext einen Fingersatz enthält, welcher Finger).

### VII. Datensparsamkeit und Kontrolle

- Der Nutzer besitzt seine Daten. Er kann sie jederzeit exportieren und löschen.
- Daten werden nur nach Authentifizierung und mit Wissen des Nutzers ins Backend übertragen.
- Öffentliche Sichtbarkeit (z. B. Highscore-Liste) ist nur pseudonym zulässig. Der Nutzer wird vorher darauf hingewiesen und kann jederzeit widersprechen (Opt-out).
- Die Anforderungen der DSGVO werden eingehalten.

### VIII. Plattformübergreifende Konsistenz

- Legato läuft auf iOS, Android, Windows, macOS und Linux sowie als Web-App im Browser.
- Kernverhalten und Bewertung sind auf allen Plattformen identisch. Plattformunterschiede betreffen nur die Bedienung, nicht die Ergebnisse.

## Entwicklungsprinzipien

### Spezifikation zuerst

- Ohne Spezifikation gibt es keine Implementierung. Die Reihenfolge ist: Constitution → Spec → Plan → Tasks → Implementierung.
- Eine **Spec** beschreibt *was* und *warum*: User Stories, Akzeptanzkriterien, Randfälle. Sie enthält keine Technologieentscheidungen.
- Ein **Plan** beschreibt *wie*: Architektur, Technologien, Algorithmen, Datenformate.
- Offene Punkte werden in der Spec als `[NEEDS CLARIFICATION]` markiert und vor der Planung geklärt.
- Hinweise für die Planung dürfen in der Spec in einem als nicht normativ gekennzeichneten Abschnitt gesammelt werden, bis der Plan existiert. Dann werden sie in den Plan übernommen und aus der Spec entfernt.

### Testbarkeit

- Jede Anforderung hat ein messbares, prüfbares Akzeptanzkriterium (Given/When/Then).
- Zuordnung (Score Following) und Bewertung werden mit reproduzierbaren Referenzdaten (z. B. aufgezeichnete MIDI-Durchläufe) getestet.
- Tests werden vor oder zusammen mit der Implementierung geschrieben.

### Inkrementelle Lieferung

- User Stories werden priorisiert (P1, P2, P3, …). Jede kann für sich umgesetzt, getestet und ausgeliefert werden, sobald die User Stories umgesetzt sind, die sie voraussetzt.
- Zuerst wird ein lauffähiges MVP ausgeliefert, danach wird es erweitert.

### Einfachheit

- Abstraktionen, Abhängigkeiten und Features werden nur eingeführt, wenn eine Spec sie erfordert.
- Fremdbibliotheken müssen lizenzrechtlich mit dem Projekt vereinbar sein.

## Governance

- Änderungen an der Constitution erfordern eine Begründung und eine Prüfung, welche Specs und Pläne davon betroffen sind.
- Die Versionierung folgt Semantic Versioning:
  - MAJOR: Ein Prinzip wird entfernt oder grundlegend geändert.
  - MINOR: Ein Prinzip wird neu aufgenommen oder wesentlich erweitert.
  - PATCH: Präzisierungen und Formulierungen.
- Jeder Plan enthält einen Abschnitt „Constitution Check“, der die Vereinbarkeit mit allen Grundprinzipien bestätigt oder Abweichungen begründet.

**Version**: 1.0.0 | **Ratifiziert**: 2026-10-24
