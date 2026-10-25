//! Work rules (research.md R10): one module per work specification, general rules otherwise.

pub mod detect;
pub mod generic;
pub mod hanon;

use crate::score::model::{Candidate, Hand, Score};

/// Display variants of the score (FR-011).
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum DisplayVariant {
    Complete,
    /// First and last two measures of each direction, gaps marked *simile*.
    Abbreviated,
    /// First measure of each direction, in separate systems.
    FirstMeasures,
    /// First measure of the ascending part only.
    FirstMeasure,
}

/// Onset indices, inclusive.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Range {
    pub start: u32,
    pub end: u32,
}

pub trait WorkRules {
    /// Work id, e.g. `hanon`; `generic` for the general rules.
    fn work_id(&self) -> &'static str;

    /// Rejects files that contradict the work specification (contracts/notendatei.md).
    fn check(&self, _score: &Score) -> Result<(), String> {
        Ok(())
    }

    /// Permissible candidates for the recognition (before the RecognitionFilter, FR-018).
    fn candidates(&self, piece: &str) -> Vec<Candidate>;

    /// Tones per hand needed for the recognition.
    fn recognition_length(&self) -> usize {
        8
    }

    /// Onsets that form one performance.
    fn performance_range(&self, score: &Score) -> Range;

    fn display_variants(&self) -> Vec<DisplayVariant>;

    /// Hand of one-handed playing, derived from the first played pitch; `None` if not unique.
    fn hand_from_position(&self, _score: &Score, _first_pitch: u8) -> Option<Hand> {
        None
    }
}

/// Hand of one-handed playing: a restriction set by the pianist (FR-018) takes precedence.
pub fn hand_for(
    rules: &dyn WorkRules,
    score: &Score,
    first_pitch: u8,
    restriction: Option<Hand>,
) -> Option<Hand> {
    restriction.or_else(|| rules.hand_from_position(score, first_pitch))
}
