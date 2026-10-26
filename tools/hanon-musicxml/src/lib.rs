//! MusicXML for Hanon exercises from a compact description: a figure per section, transposed
//! diatonically in C major step by step (docs/werke/hanon-teil1.toml, format in its header).

use std::fmt::Write;

use serde::Deserialize;

const LETTERS: [char; 7] = ['C', 'D', 'E', 'F', 'G', 'A', 'B'];
/// Right hand above the left hand, in diatonic steps.
const OCTAVE: i32 = 7;
/// Divisions per quarter note: a sixteenth lasts 1.
const DIVISIONS: usize = 4;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Book {
    pub exercise: Vec<Exercise>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Exercise {
    pub number: u32,
    /// Keynote of the left hand for "Ende".
    pub end: String,
    /// An open question about the source; such exercises are not generated.
    pub check: Option<String>,
    pub up: Vec<Section>,
    pub down: Vec<Section>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Section {
    /// Left hand, e.g. "C2 E2 F2 G2".
    pub figure: String,
    pub left: Vec<u8>,
    pub right: Vec<u8>,
    pub steps: Steps,
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

pub fn parse(toml: &str) -> Result<Book, String> {
    toml::from_str(toml).map_err(|e| e.to_string())
}

/// `hanon-01.musicxml` for exercise 1.
pub fn file_name(number: u32) -> String {
    format!("hanon-{number:02}.musicxml")
}

/// Diatonic index of a natural note: octave · 7 + letter (C = 0).
pub fn natural(name: &str) -> Result<i32, String> {
    let mut chars = name.chars();
    let letter = chars
        .next()
        .and_then(|c| LETTERS.iter().position(|&l| l == c))
        .ok_or_else(|| format!("unknown note {name:?}"))?;
    let octave: i32 = chars
        .as_str()
        .parse()
        .map_err(|_| format!("octave missing in {name:?}"))?;
    Ok(octave * OCTAVE + letter as i32)
}

/// MIDI pitch of a diatonic index (C4 = 60).
pub fn midi(natural: i32) -> i32 {
    const SEMITONES: [i32; 7] = [0, 2, 4, 5, 7, 9, 11];
    12 * (natural.div_euclid(OCTAVE) + 1) + SEMITONES[natural.rem_euclid(OCTAVE) as usize]
}

/// One measure of the left hand with fingering.
struct Measure {
    notes: Vec<i32>,
    left: Vec<u8>,
    right: Vec<u8>,
}

fn measures(sections: &[Section], len: usize) -> Result<Vec<Measure>, String> {
    let mut result = Vec::new();
    for section in sections {
        let figure = section
            .figure
            .split_whitespace()
            .map(natural)
            .collect::<Result<Vec<_>, _>>()?;
        if figure.len() != len || section.left.len() != len || section.right.len() != len {
            return Err(format!(
                "figure {:?}: {len} notes and fingers per hand expected",
                section.figure
            ));
        }
        for step in section.steps.iter() {
            result.push(Measure {
                notes: figure.iter().map(|n| n + step).collect(),
                left: section.left.clone(),
                right: section.right.clone(),
            });
        }
    }
    Ok(result)
}

#[derive(Clone, Copy)]
enum Clef {
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
}

/// Staff of one hand per measure: 1 (treble clef) or 2 (bass clef). A hand starts on staff 2 and
/// changes to the other staff at a measure that needs fewer ledger lines there, but not for a
/// single measure.
pub fn staves(measures: &[Vec<i32>]) -> Vec<u8> {
    let preferred: Vec<u8> = measures
        .iter()
        .map(|notes| {
            if Clef::Treble.ledger_lines(notes) < Clef::Bass.ledger_lines(notes) {
                1
            } else {
                2
            }
        })
        .collect();
    let mut staff = 2;
    (0..preferred.len())
        .map(|i| {
            if preferred[i] != staff && preferred.get(i + 1) == Some(&preferred[i]) {
                staff = preferred[i];
            }
            staff
        })
        .collect()
}

/// MusicXML of `exercise` in the layout of app/assets/pieces/hanon-01.musicxml.
pub fn generate(exercise: &Exercise) -> Result<String, String> {
    let number = exercise.number;
    if let Some(check) = &exercise.check {
        return Err(format!("Nr. {number} is not checked yet: {check}"));
    }
    let len = exercise
        .up
        .first()
        .ok_or_else(|| format!("Nr. {number}: no section up"))?
        .figure
        .split_whitespace()
        .count();
    let end_type = match len {
        8 => "half",
        16 => "whole",
        _ => return Err(format!("Nr. {number}: figures of 8 or 16 notes expected")),
    };
    let up = measures(&exercise.up, len).map_err(|e| format!("Nr. {number}: {e}"))?;
    let down = measures(&exercise.down, len).map_err(|e| format!("Nr. {number}: {e}"))?;
    let end = natural(&exercise.end).map_err(|e| format!("Nr. {number}: {e}"))?;
    let hand = |shift: i32| -> Vec<Vec<i32>> {
        up.iter()
            .chain(&down)
            .map(|m| m.notes.iter().map(|n| n + shift).collect())
            .chain([vec![end + shift]])
            .collect()
    };
    let right = staves(&hand(OCTAVE));
    let left = staves(&hand(0));

    let mut xml = String::new();
    xml.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    xml.push_str("<!DOCTYPE score-partwise PUBLIC \"-//Recordare//DTD MusicXML 4.0 Partwise//EN\" \"http://www.musicxml.org/dtds/partwise.dtd\">\n");
    xml.push_str("<score-partwise version=\"4.0\">\n");
    xml.push_str("  <work><work-title>The Virtuoso Pianist</work-title></work>\n");
    let _ = writeln!(xml, "  <movement-number>{number}</movement-number>");
    let _ = writeln!(xml, "  <movement-title>Hanon No. {number}</movement-title>");
    xml.push_str("  <identification><creator type=\"composer\">Hanon</creator><rights>Public domain</rights></identification>\n");
    xml.push_str("  <part-list><score-part id=\"P1\"><part-name>Piano</part-name></score-part></part-list>\n");
    xml.push_str("  <part id=\"P1\">\n");

    let last = up.len() + down.len();
    for (i, measure) in up.iter().chain(&down).enumerate() {
        let _ = write!(xml, "    <measure number=\"{}\">", i + 1);
        if i == 0 {
            let _ = write!(
                xml,
                "<attributes><divisions>{DIVISIONS}</divisions><key><fifths>0</fifths></key><time><beats>{}</beats><beat-type>4</beat-type></time><staves>2</staves><clef number=\"1\"><sign>G</sign><line>2</line></clef><clef number=\"2\"><sign>F</sign><line>4</line></clef></attributes>",
                len / DIVISIONS
            );
            xml.push_str("<barline location=\"left\"><bar-style>heavy-light</bar-style><repeat direction=\"forward\"/></barline>");
        }
        for (j, (&note, &finger)) in measure.notes.iter().zip(&measure.right).enumerate() {
            sixteenth(&mut xml, note + OCTAVE, Hand::Right, right[i], j, finger);
        }
        let _ = write!(xml, "<backup><duration>{len}</duration></backup>");
        for (j, (&note, &finger)) in measure.notes.iter().zip(&measure.left).enumerate() {
            sixteenth(&mut xml, note, Hand::Left, left[i], j, finger);
        }
        if i + 1 == up.len() {
            // Turning point: a double barline, as in the original.
            xml.push_str(
                "<barline location=\"right\"><bar-style>light-light</bar-style></barline>",
            );
        }
        if i + 1 == last {
            xml.push_str("<barline location=\"right\"><bar-style>light-heavy</bar-style><repeat direction=\"backward\"/></barline>");
        }
        xml.push_str("</measure>\n");
    }

    let _ = write!(xml, "    <measure number=\"{}\">", last + 1);
    long(
        &mut xml,
        end + OCTAVE,
        Hand::Right,
        right[last],
        len,
        end_type,
        1,
    );
    let _ = write!(xml, "<backup><duration>{len}</duration></backup>");
    long(&mut xml, end, Hand::Left, left[last], len, end_type, 5);
    xml.push_str(
        "<barline location=\"right\"><bar-style>light-heavy</bar-style></barline></measure>\n",
    );
    xml.push_str("  </part>\n</score-partwise>\n");
    Ok(xml)
}

#[derive(Clone, Copy)]
enum Hand {
    Right,
    Left,
}

impl Hand {
    /// Voice 1 with stem up or voice 5 with stem down on whichever staff; the voice gives the hand
    /// (contracts/notendatei.md).
    fn voice(self) -> (u8, &'static str) {
        match self {
            Hand::Right => (1, "up"),
            Hand::Left => (5, "down"),
        }
    }
}

fn pitch(xml: &mut String, natural: i32) {
    let _ = write!(
        xml,
        "<pitch><step>{}</step><octave>{}</octave></pitch>",
        LETTERS[natural.rem_euclid(OCTAVE) as usize],
        natural.div_euclid(OCTAVE)
    );
}

/// The `index`-th sixteenth of a measure, beamed in groups of four.
fn sixteenth(xml: &mut String, natural: i32, hand: Hand, staff: u8, index: usize, finger: u8) {
    let (voice, stem) = hand.voice();
    let beam = match index % 4 {
        0 => "begin",
        3 => "end",
        _ => "continue",
    };
    xml.push_str("<note>");
    pitch(xml, natural);
    let _ = write!(
        xml,
        "<duration>1</duration><voice>{voice}</voice><type>16th</type><stem>{stem}</stem><staff>{staff}</staff><beam number=\"1\">{beam}</beam><beam number=\"2\">{beam}</beam><notations><technical><fingering>{finger}</fingering></technical></notations></note>"
    );
}

/// A note over the whole measure; whole notes have no stem.
fn long(
    xml: &mut String,
    natural: i32,
    hand: Hand,
    staff: u8,
    duration: usize,
    kind: &str,
    finger: u8,
) {
    let (voice, stem) = hand.voice();
    xml.push_str("<note>");
    pitch(xml, natural);
    let _ = write!(
        xml,
        "<duration>{duration}</duration><voice>{voice}</voice><type>{kind}</type>"
    );
    if kind != "whole" {
        let _ = write!(xml, "<stem>{stem}</stem>");
    }
    let _ = write!(
        xml,
        "<staff>{staff}</staff><notations><technical><fingering>{finger}</fingering></technical></notations></note>"
    );
}
