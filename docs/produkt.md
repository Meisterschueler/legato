# Produktübersicht: Legato

**Constitution**: [1.0.0](../.specify/memory/constitution.md) | **Glossar**: [glossar.md](glossar.md) | **Literatur**: [literatur.md](literatur.md)

## Überblick

Legato unterstützt fortgeschrittene Pianisten beim Üben technischer Klassiker (z. B. Hanon, Czerny, Cortot, Bach). Die App hört dem Spiel zu, ordnet es dem Notentext zu, zeigt während des Spiels nur ruhige, passive Visualisierungen und liefert nach dem Durchlauf bzw. auf Nachfrage eine Bewertung und Hinweise auf systematische Schwächen. Leitbild und Grundprinzipien sind in der [Constitution](../.specify/memory/constitution.md) festgelegt.

## Bestandteile

- **App**: Schnittstelle zum Klavier. Lädt und zeigt Noten, empfängt das Spiel, ordnet es zu, visualisiert, bewertet und speichert.
- **Backend**: Erweitert die App um den Katalog, die Speicherung von Durchläufen, die langfristige Entwicklung und Statistiken. Nur nach Authentifizierung nutzbar.
- **Webseite**: Informationsseite für den Endbenutzer (z. B. Produktinformation, FAQ, Impressum). Nach der Anmeldung bietet sie dieselbe Funktionalität wie die App (**Web-App**, User Story 17). Anforderungen an die App gelten auch für die Web-App, sofern nicht anders angegeben.

Ohne Anmeldung steht die **Grundfunktionalität** zur Verfügung: Eingabe, Zuordnung, Ansichten, Bewertung einzelner Durchläufe einschließlich der Schwächen innerhalb eines Durchlaufs, Speichern, Signale und das Laden von Notendateien. **Auswertungen** über mehrere Durchläufe setzen die Anmeldung voraus.

## Nicht im Umfang

- Inhalte für Anfänger und Stücke aus dem Bereich Neoklassik
- Ersatz für Klavierunterricht (z. B. Lektionen, Erklärungen musikalischer Grundlagen)
- Variationen von Übungen, die die Werkspezifikation festlegt, z. B. rhythmische oder melodische Abwandlungen (siehe [hanon.md](werke/hanon.md)). Einhändiges Spiel ist dagegen im Umfang.
- Hanon Teil 2 und 3 (folgen in einer späteren Spec)
- Funktionalität der App auf der Webseite ohne Anmeldung

## Features

User Stories und Anforderungs-IDs (FR, NFR, SC) sind über alle Specs lückenlos fortlaufend nummeriert, damit Verweise eindeutig bleiben.

| Spec | User Stories | Priorität | Abhängig von |
|---|---|---|---|
| [001-legato-mvp](../specs/001-legato-mvp/spec.md) | 1 Eingabe und MIDI-Verbindung · 2 Noten aus Datei · 3 Üben mit Notenblatt · 4 Akustisches Signal | P1 (MVP) | – |
| [002-bewertung](../specs/002-bewertung/spec.md) | 5 Bewertung | P2 | 001 |
| [003-speichern](../specs/003-speichern/spec.md) | 6 Speichern · 7 Lokale Daten exportieren und löschen | P2 | 001 |
| [004-performance-wurm](../specs/004-performance-wurm/spec.md) | 8 Performance-Wurm | P3 | 001 |
| [005-schwaechen](../specs/005-schwaechen/spec.md) | 9 Schwächen | P3 | 001, 002 (007 für Schwächen über mehrere Durchläufe) |
| [006-audio-eingabe](../specs/006-audio-eingabe/spec.md) | 10 Audio-Eingabe | P3 | 001 |
| [007-konto-backend](../specs/007-konto-backend/spec.md) | 11 Anmelden und Katalog · 12 Übertragung · 13 Verlauf · 14 Langfristige Entwicklung · 15 Highscore-Liste · 16 Kontodaten exportieren und Konto löschen | P4 | 001, 002, 003 |
| [008-web-app](../specs/008-web-app/spec.md) | 17 Web-App | P4 | 001, 002, 003, 007 |
| [009-playlist](../specs/009-playlist/spec.md) | 18 Playlist zusammenstellen · 19 Üben im Playlist-Modus | P3 | 001 (006 für Audio-Eingabe, 007 für Stücke aus dem Katalog) |
