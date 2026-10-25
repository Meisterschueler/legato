//! Score model (data-model.md#noten).

use std::collections::BTreeSet;

/// Exact musical time as a reduced fraction.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub struct Fraction {
    pub numerator: i64,
    pub denominator: i64,
}

impl Fraction {
    pub const ZERO: Fraction = Fraction {
        numerator: 0,
        denominator: 1,
    };

    pub fn new(numerator: i64, denominator: i64) -> Self {
        assert!(denominator != 0, "denominator must not be zero");
        let sign = denominator.signum();
        let divisor = gcd(numerator, denominator).max(1);
        Fraction {
            numerator: sign * numerator / divisor,
            denominator: sign * denominator / divisor,
        }
    }
}

impl std::ops::Add for Fraction {
    type Output = Fraction;

    fn add(self, other: Fraction) -> Fraction {
        Fraction::new(
            self.numerator * other.denominator + other.numerator * self.denominator,
            self.denominator * other.denominator,
        )
    }
}

impl PartialOrd for Fraction {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Fraction {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        (self.numerator * other.denominator).cmp(&(other.numerator * self.denominator))
    }
}

fn gcd(a: i64, b: i64) -> i64 {
    if b == 0 { a.abs() } else { gcd(b, a % b) }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Hand {
    Left,
    Right,
}

/// `(measure, voice, index)`; stable, internal only. The interface addresses notes
/// by their display id (contracts/engine-schnittstelle.md).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NoteId {
    pub measure: u32,
    pub voice: u8,
    /// Position of the note within the measure across all parts, in document order.
    pub index: u32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Note {
    pub id: NoteId,
    /// MIDI note number, in C major as notated.
    pub pitch: u8,
    /// From the staff (lower = left).
    pub hand: Hand,
    /// 1–5.
    pub finger: Option<u8>,
    /// System index from the top (0 = upper staff), for the display id.
    pub staff: u8,
    /// Onset within the measure in whole notes, for the display id.
    pub onset_in_measure: Fraction,
}

impl Note {
    /// Display id `measure/staff/onset/pitch` of this note after `transposition` half tones.
    pub fn display_id(&self, transposition: i8) -> String {
        let onset = self.onset_in_measure;
        format!(
            "{}/{}/{}:{}/{}",
            self.id.measure,
            self.staff,
            onset.numerator,
            onset.denominator,
            i16::from(self.pitch) + i16::from(transposition),
        )
    }
}

/// All notes to be played at the same time.
#[derive(Clone, Debug, PartialEq)]
pub struct Onset {
    pub index: u32,
    /// Absolute time in quarter notes since the start of the piece.
    pub time: Fraction,
    pub notes: Vec<Note>,
    /// Normally `index + 1`; at the end of a repeat section additionally its start (FR-014).
    pub successors: Vec<u32>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SectionKind {
    Repeat,
    Ending,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Section {
    pub kind: SectionKind,
    /// Onset indices, inclusive.
    pub start: u32,
    pub end: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Measure {
    /// `number` attribute of the MusicXML measure.
    pub number: String,
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct Score {
    pub onsets: Vec<Onset>,
    pub sections: Vec<Section>,
    /// Onsets preceded by a notated rest (FR-020).
    pub rests_before: BTreeSet<u32>,
    pub measures: Vec<Measure>,
}

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct Metadata {
    pub composer: Option<String>,
    pub work_title: Option<String>,
    pub movement_number: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Variation {
    BothHands,
    LeftHandOnly,
    RightHandOnly,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PieceSource {
    Bundled,
    File,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Work {
    pub id: String,
    pub composer: String,
    pub title: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Piece {
    /// Hash of the file content (stable across restarts).
    pub id: String,
    pub work: String,
    pub number: Option<String>,
    pub source: PieceSource,
    pub score: Score,
    /// `true` if every note has a fingering.
    pub has_fingering: bool,
}

/// A permissible combination for the recognition.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Candidate {
    pub piece: String,
    pub variation: Variation,
    /// Half tones; Hanon 0–11 (FR-015).
    pub transposition: i8,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fractions_are_reduced_and_ordered() {
        assert_eq!(Fraction::new(2, 8), Fraction::new(1, 4));
        assert_eq!(Fraction::new(0, 4), Fraction::ZERO);
        assert_eq!(
            Fraction::new(1, 4) + Fraction::new(1, 8),
            Fraction::new(3, 8)
        );
        assert!(Fraction::new(1, 3) < Fraction::new(1, 2));
    }
}
