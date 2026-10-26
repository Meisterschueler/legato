//! MusicXML for technical exercises (Hanon, Brahms) from a compact description: per section one
//! figure per hand over one measure, transposed diatonically in the notated key step by step.
//!
//! Format of the descriptions in docs/werke/*.toml:
//! - Book: `work` (MusicXML work title), `composer` (also the prefix of the file names), `change`
//!   (`"hand"`: the hands change between treble and bass staff; `"clef"`: the right hand stays on
//!   the upper, the left hand on the lower staff and the staves change clef), `repeat` (repeat signs
//!   around up and down), `double_bar` (double barline between up and down).
//! - Exercise: `number` (e.g. 1 or "1a"), `fifths` (key signature, default 0), `end` (optional
//!   measure-long note per hand after up and down), `check` (open question about the source; such
//!   exercises are not generated), sections `up` and `down`.
//! - Section: `right`, `left` with `notes` (scientific pitch notation, C4 = MIDI 60, `#`/`b` as in
//!   the key), `rhythm` (notes per quarter note: 1 quarter, 2 eighths, 3 triplet eighths,
//!   4 sixteenths; its length gives the time signature) and optional `fingers`; `steps` (`from`,
//!   `to`): the figure is played once per diatonic step from `from` to `to`.
//! - Staff or clef changes where a measure needs fewer ledger lines, but not for a single measure.
//!   The hand follows from the voice (right 1, left 5), not from the staff.

use std::fmt::Write;

use serde::{Deserialize, Deserializer};

const LETTERS: [char; 7] = ['C', 'D', 'E', 'F', 'G', 'A', 'B'];
const OCTAVE: i32 = 7;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Book {
    pub work: String,
    pub composer: String,
    pub change: Change,
    #[serde(default)]
    pub repeat: bool,
    #[serde(default)]
    pub double_bar: bool,
    pub exercise: Vec<Exercise>,
}

#[derive(Debug, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Change {
    Hand,
    Clef,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Exercise {
    #[serde(deserialize_with = "number")]
    pub number: String,
    #[serde(default)]
    pub fifths: i32,
    pub end: Option<End>,
    pub check: Option<String>,
    pub up: Vec<Section>,
    pub down: Vec<Section>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct End {
    pub right: Tone,
    pub left: Tone,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Tone {
    pub note: String,
    pub finger: Option<u8>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Section {
    pub right: Figure,
    pub left: Figure,
    pub steps: Steps,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Figure {
    pub notes: String,
    pub rhythm: Vec<usize>,
    pub fingers: Option<Vec<u8>>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Steps {
    pub from: i32,
    pub to: i32,
}

impl Steps {
    fn iter(&self) -> impl Iterator<Item = i32> + use<> {
        let (from, to) = (self.from, self.to);
        let step = if to >= from { 1 } else { -1 };
        (0..=(to - from).abs()).map(move |i| from + step * i)
    }
}

/// Exercise numbers are integers (Hanon) or text ("1a").
fn number<'de, D: Deserializer<'de>>(deserializer: D) -> Result<String, D::Error> {
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Number {
        Integer(u32),
        Text(String),
    }
    Ok(match Number::deserialize(deserializer)? {
        Number::Integer(n) => n.to_string(),
        Number::Text(s) => s,
    })
}

pub fn parse(toml: &str) -> Result<Book, String> {
    toml::from_str(toml).map_err(|e| e.to_string())
}

/// Directory and file name prefix of a book, e.g. `hanon`.
pub fn prefix(book: &Book) -> String {
    book.composer.to_lowercase()
}

/// `hanon-01.musicxml`, `brahms-1a.musicxml`.
pub fn file_name(book: &Book, exercise: &Exercise) -> String {
    format!("{}-{:0>2}.musicxml", prefix(book), exercise.number)
}

/// Alteration of a letter (C = 0) in the key with `fifths`.
fn key_alter(letter: usize, fifths: i32) -> i32 {
    const SHARPS: [usize; 7] = [3, 0, 4, 1, 5, 2, 6];
    let position = SHARPS.iter().position(|&l| l == letter).unwrap() as i32;
    if position < fifths {
        1
    } else if 6 - position < -fifths {
        -1
    } else {
        0
    }
}

/// Diatonic index (octave · 7 + letter, C = 0) of a note that belongs to the key.
pub fn natural(name: &str, fifths: i32) -> Result<i32, String> {
    let mut chars = name.chars();
    let letter = chars
        .next()
        .and_then(|c| LETTERS.iter().position(|&l| l == c))
        .ok_or_else(|| format!("unknown note {name:?}"))?;
    let rest = chars.as_str();
    let digits = rest.trim_start_matches(['#', 'b']);
    let accidentals = &rest[..rest.len() - digits.len()];
    let alter = accidentals.matches('#').count() as i32 - accidentals.matches('b').count() as i32;
    let octave: i32 = digits
        .parse()
        .map_err(|_| format!("octave missing in {name:?}"))?;
    if alter != key_alter(letter, fifths) {
        return Err(format!("{name:?} is not in the key"));
    }
    Ok(octave * OCTAVE + letter as i32)
}

/// The notes of one hand in one measure; without rhythm a single note over the whole measure.
struct Voice<'a> {
    notes: Vec<i32>,
    rhythm: &'a [usize],
    fingers: Option<&'a [u8]>,
}

struct Measure<'a> {
    right: Voice<'a>,
    left: Voice<'a>,
}

fn figure(figure: &Figure, fifths: i32, beats: usize) -> Result<Vec<i32>, String> {
    let notes = figure
        .notes
        .split_whitespace()
        .map(|n| natural(n, fifths))
        .collect::<Result<Vec<_>, _>>()?;
    if figure.rhythm.len() != beats
        || figure.rhythm.iter().any(|n| !(1..=4).contains(n))
        || figure.rhythm.iter().sum::<usize>() != notes.len()
    {
        return Err(format!(
            "figure {:?}: rhythm {:?} does not fit {beats} quarter notes",
            figure.notes, figure.rhythm
        ));
    }
    if let Some(fingers) = &figure.fingers
        && (fingers.len() != notes.len() || fingers.iter().any(|f| !(1..=5).contains(f)))
    {
        return Err(format!(
            "figure {:?}: one finger 1–5 per note expected",
            figure.notes
        ));
    }
    Ok(notes)
}

fn measures(sections: &[Section], fifths: i32, beats: usize) -> Result<Vec<Measure<'_>>, String> {
    let mut result = Vec::new();
    for section in sections {
        let right = figure(&section.right, fifths, beats)?;
        let left = figure(&section.left, fifths, beats)?;
        for step in section.steps.iter() {
            result.push(Measure {
                right: Voice::shifted(&right, &section.right, step),
                left: Voice::shifted(&left, &section.left, step),
            });
        }
    }
    Ok(result)
}

impl<'a> Voice<'a> {
    fn shifted(notes: &[i32], figure: &'a Figure, step: i32) -> Self {
        Voice {
            notes: notes.iter().map(|n| n + step).collect(),
            rhythm: &figure.rhythm,
            fingers: figure.fingers.as_deref(),
        }
    }

    fn long(tone: &'a Tone, fifths: i32) -> Result<Self, String> {
        Ok(Voice {
            notes: vec![natural(&tone.note, fifths)?],
            rhythm: &[],
            fingers: tone.finger.as_ref().map(std::slice::from_ref),
        })
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Clef {
    Treble,
    Bass,
}

impl Clef {
    /// Ledger lines of the note farthest outside the staff.
    fn ledger_lines(self, notes: &[i32]) -> i32 {
        // Diatonic index of the bottom line: E4 or G2.
        let bottom = match self {
            Clef::Treble => 4 * OCTAVE + 2,
            Clef::Bass => 2 * OCTAVE + 4,
        };
        let top = bottom + 8;
        notes
            .iter()
            .map(|&n| (bottom - n).max(n - top).max(0) / 2)
            .max()
            .unwrap_or(0)
    }

    fn xml(self, staff: u8) -> String {
        let (sign, line) = match self {
            Clef::Treble => ('G', 2),
            Clef::Bass => ('F', 4),
        };
        format!("<clef number=\"{staff}\"><sign>{sign}</sign><line>{line}</line></clef>")
    }

    /// Staff with this clef if the hands change staff: treble above, bass below.
    fn staff(self) -> u8 {
        match self {
            Clef::Treble => 1,
            Clef::Bass => 2,
        }
    }
}

/// Clef per measure: starts with `start` and changes at a measure that needs fewer ledger lines in
/// the other clef (bass if equal), but not for a single measure.
pub fn clefs(measures: &[Vec<i32>], start: Clef) -> Vec<Clef> {
    let preferred: Vec<Clef> = measures
        .iter()
        .map(|notes| {
            if Clef::Treble.ledger_lines(notes) < Clef::Bass.ledger_lines(notes) {
                Clef::Treble
            } else {
                Clef::Bass
            }
        })
        .collect();
    let mut clef = start;
    (0..preferred.len())
        .map(|i| {
            if preferred[i] != clef && preferred.get(i + 1) == Some(&preferred[i]) {
                clef = preferred[i];
            }
            clef
        })
        .collect()
}

fn lcm(a: usize, b: usize) -> usize {
    fn gcd(a: usize, b: usize) -> usize {
        if b == 0 { a } else { gcd(b, a % b) }
    }
    a / gcd(a, b) * b
}

/// MusicXML of `exercise`.
pub fn generate(book: &Book, exercise: &Exercise) -> Result<String, String> {
    let number = &exercise.number;
    let context = |e: String| format!("Nr. {number}: {e}");
    if let Some(check) = &exercise.check {
        return Err(context(format!("not checked yet: {check}")));
    }
    let fifths = exercise.fifths;
    let beats = exercise
        .up
        .first()
        .ok_or_else(|| context("no section up".into()))?
        .right
        .rhythm
        .len();
    let mut all = measures(&exercise.up, fifths, beats).map_err(context)?;
    let turn = all.len();
    all.extend(measures(&exercise.down, fifths, beats).map_err(context)?);
    let music = all.len();
    if let Some(end) = &exercise.end {
        all.push(Measure {
            right: Voice::long(&end.right, fifths).map_err(context)?,
            left: Voice::long(&end.left, fifths).map_err(context)?,
        });
    }
    let divisions = all
        .iter()
        .flat_map(|m| m.right.rhythm.iter().chain(m.left.rhythm))
        .fold(1, |d, &n| lcm(d, n));

    let right: Vec<Vec<i32>> = all.iter().map(|m| m.right.notes.clone()).collect();
    let left: Vec<Vec<i32>> = all.iter().map(|m| m.left.notes.clone()).collect();
    let (right_staff, left_staff, upper, lower): (Vec<u8>, Vec<u8>, Vec<Clef>, Vec<Clef>) =
        match book.change {
            Change::Hand => {
                let staves = |notes| clefs(notes, Clef::Bass).iter().map(|c| c.staff()).collect();
                (
                    staves(&right),
                    staves(&left),
                    vec![Clef::Treble; all.len()],
                    vec![Clef::Bass; all.len()],
                )
            }
            Change::Clef => (
                vec![1; all.len()],
                vec![2; all.len()],
                clefs(&right, Clef::Treble),
                clefs(&left, Clef::Bass),
            ),
        };

    let mut xml = String::new();
    xml.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    xml.push_str("<!DOCTYPE score-partwise PUBLIC \"-//Recordare//DTD MusicXML 4.0 Partwise//EN\" \"http://www.musicxml.org/dtds/partwise.dtd\">\n");
    xml.push_str("<score-partwise version=\"4.0\">\n");
    let _ = writeln!(xml, "  <work><work-title>{}</work-title></work>", book.work);
    let _ = writeln!(xml, "  <movement-number>{number}</movement-number>");
    let _ = writeln!(
        xml,
        "  <movement-title>{} No. {number}</movement-title>",
        book.composer
    );
    let _ = writeln!(
        xml,
        "  <identification><creator type=\"composer\">{}</creator><rights>Public domain</rights></identification>",
        book.composer
    );
    xml.push_str("  <part-list><score-part id=\"P1\"><part-name>Piano</part-name></score-part></part-list>\n");
    xml.push_str("  <part id=\"P1\">\n");

    let duration = beats * divisions;
    for (i, measure) in all.iter().enumerate() {
        let _ = write!(xml, "    <measure number=\"{}\">", i + 1);
        if i == 0 {
            let _ = write!(
                xml,
                "<attributes><divisions>{divisions}</divisions><key><fifths>{fifths}</fifths></key><time><beats>{beats}</beats><beat-type>4</beat-type></time><staves>2</staves>{}{}</attributes>",
                upper[0].xml(1),
                lower[0].xml(2)
            );
            if book.repeat {
                xml.push_str("<barline location=\"left\"><bar-style>heavy-light</bar-style><repeat direction=\"forward\"/></barline>");
            }
        } else if upper[i] != upper[i - 1] || lower[i] != lower[i - 1] {
            xml.push_str("<attributes>");
            if upper[i] != upper[i - 1] {
                xml.push_str(&upper[i].xml(1));
            }
            if lower[i] != lower[i - 1] {
                xml.push_str(&lower[i].xml(2));
            }
            xml.push_str("</attributes>");
        }
        let time = (divisions, beats);
        voice(
            &mut xml,
            &measure.right,
            Hand::Right,
            right_staff[i],
            fifths,
            time,
        );
        let _ = write!(xml, "<backup><duration>{duration}</duration></backup>");
        voice(
            &mut xml,
            &measure.left,
            Hand::Left,
            left_staff[i],
            fifths,
            time,
        );
        if i + 1 == turn && book.double_bar {
            xml.push_str(
                "<barline location=\"right\"><bar-style>light-light</bar-style></barline>",
            );
        }
        if i + 1 == music && book.repeat {
            xml.push_str("<barline location=\"right\"><bar-style>light-heavy</bar-style><repeat direction=\"backward\"/></barline>");
        } else if i + 1 == all.len() {
            xml.push_str(
                "<barline location=\"right\"><bar-style>light-heavy</bar-style></barline>",
            );
        }
        xml.push_str("</measure>\n");
    }
    xml.push_str("  </part>\n</score-partwise>\n");
    Ok(xml)
}

#[derive(Clone, Copy)]
enum Hand {
    Right,
    Left,
}

/// The notes of one hand in a measure, beamed per quarter note. `time`: divisions per quarter
/// note and quarter notes per measure.
fn voice(
    xml: &mut String,
    voice: &Voice,
    hand: Hand,
    staff: u8,
    fifths: i32,
    time: (usize, usize),
) {
    let (divisions, beats) = time;
    // The voice gives the hand (contracts/notendatei.md).
    let (number, stem) = match hand {
        Hand::Right => (1, "up"),
        Hand::Left => (5, "down"),
    };
    let finger = |i: usize| voice.fingers.map(|f| f[i]);
    if voice.rhythm.is_empty() {
        let (kind, dot) = match beats {
            1 => ("quarter", false),
            2 => ("half", false),
            3 => ("half", true),
            _ => ("whole", false),
        };
        note(xml, voice.notes[0], fifths, finger(0), "", |xml| {
            let _ = write!(
                xml,
                "<duration>{}</duration><voice>{number}</voice><type>{kind}</type>",
                beats * divisions
            );
            if dot {
                xml.push_str("<dot/>");
            }
            if kind != "whole" {
                let _ = write!(xml, "<stem>{stem}</stem>");
            }
            let _ = write!(xml, "<staff>{staff}</staff>");
        });
        return;
    }
    let mut index = 0;
    for &count in voice.rhythm {
        let kind = match count {
            1 => "quarter",
            2 | 3 => "eighth",
            _ => "16th",
        };
        for j in 0..count {
            let tuplet = match (count, j) {
                (3, 0) => "<tuplet type=\"start\" bracket=\"no\"/>",
                (3, 2) => "<tuplet type=\"stop\"/>",
                _ => "",
            };
            note(
                xml,
                voice.notes[index],
                fifths,
                finger(index),
                tuplet,
                |xml| {
                    let _ = write!(
                        xml,
                        "<duration>{}</duration><voice>{number}</voice><type>{kind}</type>",
                        divisions / count
                    );
                    if count == 3 {
                        xml.push_str("<time-modification><actual-notes>3</actual-notes><normal-notes>2</normal-notes></time-modification>");
                    }
                    let _ = write!(xml, "<stem>{stem}</stem><staff>{staff}</staff>");
                    if count > 1 {
                        let beam = match j {
                            0 => "begin",
                            j if j + 1 == count => "end",
                            _ => "continue",
                        };
                        let _ = write!(xml, "<beam number=\"1\">{beam}</beam>");
                        if count == 4 {
                            let _ = write!(xml, "<beam number=\"2\">{beam}</beam>");
                        }
                    }
                },
            );
            index += 1;
        }
    }
}

/// `<note>` with pitch, the elements written by `body`, the fingering and `tuplet`.
fn note(
    xml: &mut String,
    natural: i32,
    fifths: i32,
    finger: Option<u8>,
    tuplet: &str,
    body: impl FnOnce(&mut String),
) {
    let letter = natural.rem_euclid(OCTAVE) as usize;
    let _ = write!(xml, "<note><pitch><step>{}</step>", LETTERS[letter]);
    let alter = key_alter(letter, fifths);
    if alter != 0 {
        let _ = write!(xml, "<alter>{alter}</alter>");
    }
    let _ = write!(
        xml,
        "<octave>{}</octave></pitch>",
        natural.div_euclid(OCTAVE)
    );
    body(xml);
    if finger.is_some() || !tuplet.is_empty() {
        xml.push_str("<notations>");
        if let Some(finger) = finger {
            let _ = write!(
                xml,
                "<technical><fingering>{finger}</fingering></technical>"
            );
        }
        let _ = write!(xml, "{tuplet}</notations>");
    }
    xml.push_str("</note>");
}
