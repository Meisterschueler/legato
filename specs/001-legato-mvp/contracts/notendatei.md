# Contract: Notendatei

## Formate

| Endung | Inhalt |
|---|---|
| `.musicxml`, `.xml` | MusicXML 4.0 (ältere Versionen ab 3.0 werden akzeptiert), `score-partwise` |
| `.mxl` | komprimiertes MusicXML (ZIP mit `META-INF/container.xml`) |

## Ausgewertete Teilmenge

| Element | Verwendung |
|---|---|
| `<work-title>`, `<movement-number>`, `<creator type="composer">` | Werk und Stücknummer (Werkregel) |
| `<part>` mit 2 `<staves>` oder 2 Parts | Hand: bei 2 Parts oberer Part = rechte Hand; bei 2 `<staves>` aus `<voice>` (1–4 = rechte, 5–8 = linke Hand, wie Notenprogramme die Stimmen zählen), sodass Noten ins andere System wechseln dürfen; nutzt der Part keine Stimme 5–8, aus dem System (oberes = rechte, unteres = linke Hand) |
| `<note>` mit `<pitch>` (`step`, `alter`, `octave`), `<duration>`, `<chord/>`, `<staff>`, `<voice>` | Positionen und Noten |
| `<rest/>` | notierte Pausen (FR-020) |
| `<tie type="stop"/>` | übergebundene Noten erzeugen keinen neuen Anschlag |
| `<fingering>` | Fingersatz |
| `<barline>` mit `<repeat direction="forward\|backward"/>` | Wiederholungsabschnitte |
| `<barline>` mit `<bar-style>light-light</bar-style>` am Taktende | Doppelstrich, z. B. Wendepunkt bei Hanon Teil 1 (Werkregel) |

Alles andere (Dynamik, Artikulation, Text, Layout) wird ignoriert, aber bei der Anzeige von OSMD dargestellt.

## Ablehnung (US 2, Szenario 2)

Eine Datei wird mit verständlicher Meldung abgelehnt, wenn:
- sie kein gültiges XML bzw. ZIP ist,
- sie kein `score-partwise` enthält (`score-timewise` wird nicht unterstützt),
- sie keine Note mit Tonhöhe enthält,
- Wiederholungszeichen nicht paarig sind,
- eine Werkregel sie als unvereinbar mit der Werkspezifikation erkennt (z. B. Hanon Teil 1 ohne Wiederholungsabschnitt).

Fehlender Fingersatz ist kein Ablehnungsgrund (US 2, Szenario 3).

## Erzeugtes MusicXML für die Anzeige

Der Kern erzeugt aus dem Notentext je Tonart und Anzeigevariante ein eigenes MusicXML und dazu die Zuordnung `NoteId` → Anzeige-ID ([engine-schnittstelle.md](engine-schnittstelle.md#nachrichten-an-score-view-web)), über die die Oberfläche Markierungen setzt. Gekürzte Anzeigevarianten enthalten an den Lücken die Anweisung `<words>simile</words>`.
