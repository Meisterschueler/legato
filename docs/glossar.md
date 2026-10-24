# Glossar

Begriffe und Schlüsselwörter, die in allen Specs, Plänen und Werkspezifikationen gelten.

## Schlüsselwörter

Die Schlüsselwörter sind in Anlehnung an RFC 2119 zu verstehen: **MUSS/MÜSSEN** ist zwingend. **SOLL/SOLLEN** wird empfohlen; eine Abweichung ist im Plan zu begründen. **DARF NICHT/DÜRFEN NICHT** bzw. **DARF KEINE/DÜRFEN KEINE** ist verboten. Die Verneinung wird mit dem Schlüsselwort großgeschrieben.

## Noten und Inhalte

| Begriff | Bedeutung |
|---|---|
| **Werk** | Eine Sammlung von Stücken eines Komponisten, z. B. Hanon, *Der Klaviervirtuose in 60 Übungen*. |
| **Stück** | Eine einzelne spielbare Nummer eines Werks, z. B. Hanon Nr. 1. |
| **Notentext** | Der notierte Inhalt eines Stücks: Noten, Fingersatz, Wiederholungen. |
| **Figur** | Eine kurze Tonfolge, die sich innerhalb eines Stücks wiederholt, meist versetzt, z. B. 8 Töne je Hand bei Hanon Teil 1. Nicht jedes Stück hat eine Figur: Hanon besteht vollständig aus Figuren, Bach kaum. |
| **Fingersatz** | Die Angabe, mit welchem Finger welcher Hand eine Note gespielt wird (1 = Daumen bis 5 = kleiner Finger). |
| **Transposition** | Spielen eines Stücks in einer anderen Tonart als notiert. Welche Tonarten zulässig sind und in welcher Lage sie gespielt werden, legt die Werkspezifikation fest. |
| **Variation** | Abwandlung eines Stücks. Für alle Stücke gibt es neben der notierten Grundform „beidhändig“ die Variationen „nur linke Hand“ und „nur rechte Hand“. Weitere, werkspezifische Variationen legt die Werkspezifikation fest; sie sind nicht im Umfang. |
| **Katalog** | Die vom Backend bereitgestellte Sammlung fertiger Stücke. |
| **Repertoire** | Alle Stücke, die dem Pianisten lokal zur Verfügung stehen: das mit der App ausgelieferte Stück, aus Dateien geladene Stücke und synchronisierte Stücke des Katalogs. |

## Üben

| Begriff | Bedeutung |
|---|---|
| **Durchlauf** | Der Abschnitt des Spiels, für den eine Bewertung erstellt wird (engl. *performance*). Die Abgrenzung legt die Werkspezifikation fest (für Hanon siehe [hanon.md](werke/hanon.md)). Stück, Variation und Tonart stehen zu Beginn eines Durchlaufs fest und ändern sich bis zu seinem Ende nicht. |
| **Abbruch** | Ein Durchlauf endet, bevor sein Ende erreicht ist, z. B. weil der Pianist mitten im Durchlauf aufhört zu spielen, weil ein Zuordnungsverlust eintritt oder weil die Eingabe getrennt wird. Ein abgebrochener Durchlauf wird verworfen: Er wird weder bewertet noch gespeichert noch übertragen. Ein Durchlauf beginnt erst, wenn Stück, Variation und Tonart eindeutig erkannt sind; vorher kann es daher keinen Abbruch geben. |
| **Übeeinheit** | Zusammenhängende Zeit, in der der Pianist mit der App übt (engl. *practice session*, vgl. [Ericsson et al., 1993](literatur.md)). Eine Übeeinheit kann mehrere Stücke und Durchläufe enthalten. Sie endet, wenn die App beendet oder in den Hintergrund geschickt wird oder der Pianist länger als eine Ruhezeit nicht spielt. |
| **Aufhören** | Nach dem Ende eines Durchlaufs folgt eine Stille, die länger ist als die Grenzzeit für Pausen. Töne, die keinen neuen Durchlauf beginnen (z. B. der Schlusston bei Hanon), zählen nicht als Weiterspielen. Spielt der Pianist dagegen nahtlos den nächsten Durchlauf, hat er nicht aufgehört. |
| **Übezeit** | Zeit, die ein Pianist mit der App geübt hat. |
| **Tempo** | Geschwindigkeit des Spiels in M.M., bezogen auf die Schlageinheit des Notentexts. Das **Tempo eines Durchlaufs** ist eine einzige Zahl für die typische Geschwindigkeit des gesamten Durchlaufs, unabhängig von Agogik. Für den Vergleich zwischen Stücken wird es als Anschlagrate (Anschläge pro Sekunde und Hand) ausgedrückt. Tempounterschiede werden im Verhältnis wahrgenommen, nicht als Differenz ([Drake & Botte, 1993](literatur.md)). |
| **Ansicht** | Eine Darstellungsform während des Spiels (Notenblatt, Performance-Wurm). |
| **Erkennungsmodus** | Modus, in dem die App Stück, Variation und Tonart jedes Durchlaufs selbständig unter dem Repertoire erkennt (Standard). |
| **Playlist** | Vom Pianisten zusammengestellte, geordnete Folge von Stücken aus dem Repertoire mit je einer Tonart (siehe [009](../specs/009-playlist/spec.md)). |
| **Playlist-Modus** | Modus, in dem die Zuordnung fest auf Stück und Tonart des aktuellen Playlist-Eintrags eingestellt ist. Der Pianist wechselt außerhalb des Spiels per Pedal oder jederzeit über den Bildschirm zum nächsten bzw. vorigen Eintrag. |
| **Andeutung** | Sehr leises Vorspielen der Figur eines Stücks über MIDI beim Start des Playlist-Modus und nach jedem Wechsel. |
| **Performance-Wurm** | Ansicht, die Tempo und Lautstärke als Wurm in einem Koordinatensystem zeigt (siehe User Story 8 in [004](../specs/004-performance-wurm/spec.md)). Eigenname, daher unabhängig vom Begriff „Durchlauf“. Geht auf die Darstellung im Tempo-Lautstärke-Raum zurück ([Langner & Goebl, 2003](literatur.md)). |

## Zuordnung und Abweichungen

| Begriff | Bedeutung |
|---|---|
| **Zuordnung** | Zuweisung jedes gespielten Tons zu einer Note des Notentexts. **Während des Spiels** (online, engl. *score following*) dient sie der Erkennung, der Positionsmarkierung, der Fehlermarkierung und dem Abbruch ([Nakamura et al., 2014](literatur.md); [Park et al., 2025](literatur.md)). **Nachträglich** (offline, engl. *score alignment*) wird der abgeschlossene Durchlauf genauer neu zugeordnet; sie ist die Grundlage der Bewertung ([Nakamura et al., 2017](literatur.md)). |
| **Erkennungslänge** | Anzahl gespielter Töne, nach der Stück, Variation und Tonart erkannt sein sollen und über die ein Zuordnungsverlust festgestellt wird. Legt die Werkspezifikation fest, z. B. die Länge der Figur bei Hanon. |
| **Zuordnungsverlust** | Unter den zuletzt gespielten Tönen, so vielen wie die Erkennungslänge, entspricht nur noch ein geringer Anteil dem Notentext; falsche Töne und Zusatztöne überwiegen deutlich. Einzelne oder kurze Folgen von Abweichungen führen daher nicht zum Zuordnungsverlust. |
| **Abweichung** | Oberbegriff für alles, was an einer Stelle vom Notentext oder vom gleichmäßigen Spiel abweicht: Auslassung, falscher Ton, Zusatzton oder Ungleichmäßigkeit. |
| **Auslassung** | Eine Note des Notentexts, die nicht gespielt wurde (engl. *omission*). |
| **Falscher Ton** | Ein Ton, der anstelle einer Note des Notentexts mit falscher Tonhöhe gespielt wurde (engl. *substitution*). |
| **Zusatzton** | Ein gespielter Ton, der zusätzlich zu den Noten gespielt wurde und keiner Note des Notentexts entspricht (engl. *addition*). |
| **Ungleichmäßigkeit** | Hochfrequente Abweichung einer Note gegenüber ihren unmittelbaren Nachbarn. **Zeitliche Ungleichmäßigkeit** betrifft den Tonbeginn, **dynamische Ungleichmäßigkeit** die Anschlagstärke, **artikulatorische Ungleichmäßigkeit** die Artikulation. Das Gegenteil ist Gleichmäßigkeit (engl. *evenness*). Zur Messung der zeitlichen und dynamischen Gleichmäßigkeit mit MIDI vgl. [Jabusch et al., 2004](literatur.md). |
| **Artikulation** | Verhältnis zwischen dem Loslassen einer Taste und dem Anschlag des folgenden Tons derselben Hand, bezogen auf den Abstand der beiden Anschläge (engl. *inter-onset interval*, IOI). Bei Legato überlappen sich die Töne (engl. *key overlap time*, KOT), bei Non-Legato und Staccato entsteht eine Lücke (engl. *key detached time*, KDT) ([Bresin & Battel, 2000](literatur.md)). Allmähliche Änderungen der Artikulation gelten als Interpretation und werden nicht bestraft. |
| **Agogik / Dynamik** | Niederfrequente (langfristige) Änderungen von Tempo bzw. Lautstärke. Sie gelten als Interpretation und werden nicht bestraft. |
| **Schwäche** | Eine systematische Abweichung (Fehlermuster), die innerhalb eines Durchlaufs oder über mehrere Durchläufe wiederkehrt. |

## Bewertung und Entwicklung

| Begriff | Bedeutung |
|---|---|
| **Bewertung** | Positive Zahl, die die Qualität eines einzelnen Durchlaufs beschreibt. Größer ist besser. Unabhängig vom Tempo. |
| **Spielniveau** | Langfristige Spielstärke eines Pianisten. Steigt in der Regel langsam an. |
| **Schwierigkeitsgrad** | Anspruch, ein Stück in einer bestimmten Variation, Tonart und einem bestimmten Tempo zu spielen. Setzt sich zusammen aus einem konstanten Anteil von Stück und Variation, einem Anteil der Tonart (z. B. ist Fis-Dur schwerer als C-Dur) und einem Anteil, der mit dem Tempo wächst. |
| **Vertrautheit** | Wie gut ein Pianist ein bestimmtes Stück beherrscht. Steigt durch Üben und kann ohne Üben wieder sinken. |
| **Tagesform** | Zufällige Schwankung der Leistung. Gilt als Rauschen. |
| **Auswertung** | Darstellung oder Analyse über mehrere Durchläufe hinweg, z. B. zeitlicher Verlauf, Schwächen über mehrere Durchläufe, Spielniveau, Vertrautheit. Nur für angemeldete Pianisten. |

## Konto

| Begriff | Bedeutung |
|---|---|
| **Nutzername** | Öffentlicher Name eines angemeldeten Pianisten, unter dem er in der Highscore-Liste erscheint. Wird bei der ersten Anmeldung aus zufälligen Vorschlägen gewählt (z. B. „Allegro-Chopin-42“), ist eindeutig und kann jederzeit geändert werden. Steht in keinem Bezug zum Konto beim Identitätsanbieter. |

## Bezeichner im Code

Bezeichner im Code sind englisch (Constitution, Entwicklungsprinzipien). Für Begriffe dieses Glossars sind die folgenden Bezeichner verbindlich; die Schreibweise folgt der Konvention der jeweiligen Sprache (z. B. `Performance`, `performance`, `PERFORMANCE`). Neue Fachbegriffe werden hier ergänzt, bevor sie im Code verwendet werden.

| Begriff | Code |
|---|---|
| Werk | `Work` |
| Stück | `Piece` |
| Notentext | `Score` |
| Figur | `Figure` |
| Fingersatz | `Fingering` |
| Transposition | `Transposition` |
| Variation („beidhändig“, „nur linke Hand“, „nur rechte Hand“) | `Variation` (`BothHands`, `LeftHandOnly`, `RightHandOnly`) |
| Katalog | `Catalog` |
| Repertoire | `Repertoire` |
| Durchlauf | `Performance` |
| Abbruch | `Abort` |
| Übeeinheit | `PracticeSession` |
| Aufhören | `Stop` |
| Übezeit | `PracticeTime` |
| Tempo | `Tempo` |
| Ansicht / Notenblatt | `View` / `ScoreView` |
| Erkennungsmodus | `RecognitionMode` |
| Playlist / Playlist-Modus | `Playlist` / `PlaylistMode` |
| Andeutung | `Preview` |
| Performance-Wurm | `PerformanceWorm` |
| Zuordnung (während des Spiels / nachträglich) | `Alignment` (`ScoreFollowing` / `OfflineAlignment`) |
| Folgeverfahren | `ScoreFollower` |
| Erkennung | `Recognition` |
| Erkennungslänge | `RecognitionLength` |
| Zuordnungsverlust | `AlignmentLoss` |
| Abweichung | `Deviation` |
| Auslassung | `Omission` |
| Falscher Ton | `Substitution` |
| Zusatzton | `Addition` |
| Ungleichmäßigkeit | `Unevenness` |
| Artikulation | `Articulation` |
| Agogik / Dynamik | `Agogics` / `Dynamics` |
| Schwäche | `Weakness` |
| Bewertung | `Rating` |
| Spielniveau | `SkillLevel` |
| Schwierigkeitsgrad | `Difficulty` |
| Vertrautheit | `Familiarity` |
| Tagesform | `DailyForm` |
| Auswertung | `Analysis` |
| Nutzername | `Username` |
| Werkregel (Umsetzung einer Werkspezifikation) | `WorkRules` |
| Kandidat (Stück, Variation, Tonart) | `Candidate` |
| Anschlagszeitpunkt im Notentext | `Onset` |
| Gespielter Ton | `PlayedNote` |
| Eingabe / Eingabezustand | `Input` / `InputState` |
| Warnung bei Verbindungsverlust | `DisconnectWarning` |
| Verbindungs-, OK-, Abbruch-Signal | `ConnectSignal`, `OkSignal`, `AbortSignal` |
| Grenzzeit für Pausen / Obergrenze an notierten Pausen | `PauseLimit` / `RestLimit` |
| Anzeigevariante | `DisplayVariant` |
| Fehlermarkierung | `ErrorMarking` |
| Einschränkung der Erkennung | `RecognitionFilter` |
