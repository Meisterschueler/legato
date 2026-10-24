# Feature-Spezifikation: Web-App

**Status**: Entwurf | **Erstellt**: 2026-10-23 | **Constitution**: [1.0.0](../../.specify/memory/constitution.md) | **Abhängig von**: [001-legato-mvp](../001-legato-mvp/spec.md), [002-bewertung](../002-bewertung/spec.md), [003-speichern](../003-speichern/spec.md), [007-konto-backend](../007-konto-backend/spec.md)

**Produktübersicht**: [produkt.md](../../docs/produkt.md) | **Glossar**: [glossar.md](../../docs/glossar.md)

## Überblick

Die Webseite bietet angemeldeten Pianisten dieselbe Funktionalität wie die App im Browser. Ohne Anmeldung zeigt sie nur Informationsinhalte.

## Umfang

Diese Spec umfasst User Story 17 (Priorität P4).

## Nutzungsszenarien und Tests

### User Story 17 – Web-App im Browser (Priorität: P4)

Ein angemeldeter Pianist kann Legato auch ohne Installation im Browser nutzen. Nach der Anmeldung auf der Webseite steht ihm dieselbe Funktionalität wie in der App zur Verfügung. MIDI empfängt und sendet der Browser.

**Warum diese Priorität**: Senkt die Einstiegshürde und erlaubt die Nutzung an Rechnern ohne installierte App, setzt aber Anmeldung und Übertragung (User Stories 11 und 12 in [007](../007-konto-backend/spec.md)) voraus.

**Unabhängiger Test**: Auf der Webseite anmelden, MIDI-Gerät im Browser wählen, einen Durchlauf spielen und prüfen, dass Zuordnung, Ansichten, Bewertung, Speicherung und Übertragung wie in der App funktionieren. Denselben Durchlauf in App und Web-App bewerten und vergleichen.

**Akzeptanzszenarien**:

1. **Angenommen** der Pianist ist auf der Webseite nicht angemeldet, **wenn** er sie aufruft, **dann** sieht er nur Informationsinhalte und keine Funktionen der App.
2. **Angenommen** der Pianist meldet sich auf der Webseite an, **wenn** die Anmeldung abgeschlossen ist, **dann** stehen ihm die Funktionen der App einschließlich Katalog zur Verfügung. Für die Anmeldung gelten dieselben Bedingungen wie in der App (User Story 11 in [007](../007-konto-backend/spec.md)).
3. **Angenommen** der Browser unterstützt MIDI, **wenn** der Pianist ein MIDI-Gerät wählt, **dann** werden Spiel und akustische Signale wie in der App über MIDI empfangen bzw. gesendet.
4. **Angenommen** der Browser unterstützt kein MIDI oder der Pianist hat den Zugriff verweigert, **wenn** er die Eingabe wählen will, **dann** werden keine MIDI-Geräte angeboten, und ein Hinweis nennt den Grund. Die Audio-Eingabe bleibt verfügbar, sofern User Story 10 aus [006](../006-audio-eingabe/spec.md) umgesetzt ist.
5. **Angenommen** ein Durchlauf läuft, **wenn** der Pianist den Tab wechselt, das Browserfenster minimiert oder die Seite neu lädt, **dann** wird der Durchlauf verworfen, und beim Zurückkehren erscheint ein Hinweis (analog User Story 1 in [001](../001-legato-mvp/spec.md)).
6. **Angenommen** ein Durchlauf wurde in der Web-App bewertet, **wenn** derselbe Durchlauf in der App bewertet wird, **dann** ist das Ergebnis identisch.
7. **Angenommen** der Pianist ist in der Web-App angemeldet, **wenn** ein Durchlauf per MIDI abgeschlossen ist, **dann** wird er gespeichert und ins Backend übertragen (User Story 12 in [007](../007-konto-backend/spec.md)).
8. **Angenommen** der Pianist ist in der Web-App angemeldet, **wenn** er sich abmeldet, **dann** werden alle im Browser gespeicherten Daten gelöscht. Liegen noch nicht übertragene Durchläufe vor (z. B. per Audio-Eingabe), wird er vorher gewarnt.
9. **Angenommen** keine Onlineverbindung besteht, **wenn** der Pianist die Web-App aufruft, **dann** muss sie nicht nutzbar sein.

---

### Randfälle

| Randfall | Verhalten |
|---|---|
| Der Browser unterstützt kein MIDI (z. B. Safari). | Keine Verbindung per MIDI möglich; ein Hinweis nennt den Grund (User Story 17). |

## Anforderungen

Die Schlüsselwörter MUSS, SOLL, DARF NICHT usw. sind im [Glossar](../../docs/glossar.md#schlüsselwörter) definiert.

### Funktionale Anforderungen

**Web-App**

- **FR-071**: Die Webseite MUSS angemeldeten Pianisten dieselbe Funktionalität wie die App bieten. Nicht angemeldeten Pianisten DARF sie NUR Informationsinhalte anbieten.
- **FR-072**: Die Web-App MUSS MIDI über den Browser empfangen und senden, sofern der Browser es unterstützt. Andernfalls oder wenn der Zugriff verweigert wird, MUSS sie den Grund nennen.
- **FR-073**: Verliert die Seite während eines Durchlaufs die Sichtbarkeit (z. B. Tabwechsel, Minimieren, Neuladen), MUSS die Web-App den Durchlauf verwerfen.
- **FR-074**: Beim Abmelden MUSS die Web-App alle im Browser gespeicherten Daten des Pianisten löschen. Liegen nicht übertragene Durchläufe vor, MUSS sie vorher warnen.

### Nicht-funktionale Anforderungen

Es gelten die nicht-funktionalen Anforderungen aus [001](../001-legato-mvp/spec.md#nicht-funktionale-anforderungen), ergänzt um:

- **NFR-007**: Die Web-App MUSS in aktuellen Browsern laufen. [NEEDS CLARIFICATION: Welche Browser werden unterstützt? Safari bietet derzeit keinen MIDI-Zugriff.]
- **NFR-008**: Die Web-App setzt eine Onlineverbindung voraus (Ausnahme von NFR-002).
- **NFR-009**: Die Web-App MUSS wie die App mehrsprachig sein (NFR-006). Sprache und Ländervorschlag (FR-057) MÜSSEN sich nach der Regionseinstellung des Geräts bzw. Browsers richten.

## Erfolgskriterien

- **SC-012**: Derselbe Durchlauf ergibt in der Web-App dieselbe Bewertung wie in der App (vgl. SC-007).

## Annahmen

Es gelten die Annahmen aus [001](../001-legato-mvp/spec.md#annahmen).

## Hinweise für die Planung (nicht normativ)

Die folgenden Hinweise sind Entscheidungen über das *Wie* und gehören in den Plan, nicht in die Spec. Sobald der Plan existiert, werden sie dorthin übernommen und hier entfernt:

- **Sprache im Browser**: über `navigator.languages`.
- **Web-App**: MIDI über die Web MIDI API (`navigator.requestMIDIAccess`, nur in sicherem Kontext/HTTPS; derzeit Chromium-Browser und Firefox, nicht Safari). Bildschirmsperre über die Screen Wake Lock API verhindern, Hintergrund über die Page Visibility API erkennen, lokale Daten in IndexedDB, beim Abmelden z. B. per `Clear-Site-Data`-Header löschen. Audio-Eingabe über `getUserMedia` und Web Audio API (AudioWorklet), auch in Safari verfügbar. Für identische Bewertungen siehe „Gemeinsamer Code“ in [007](../007-konto-backend/spec.md#hinweise-für-die-planung-nicht-normativ); der gemeinsame Code gilt auch für die Web-App.
