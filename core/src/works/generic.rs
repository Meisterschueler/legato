//! General rules for works without specification: no transposition, performance = whole piece,
//! recognition length 8 tones per hand, display variant "complete" only.

use super::{DisplayVariant, Range, WorkRules};
use crate::score::model::{Candidate, Score, Variation};

pub const VARIATIONS: [Variation; 3] = [
    Variation::BothHands,
    Variation::LeftHandOnly,
    Variation::RightHandOnly,
];

pub struct GenericRules;

impl WorkRules for GenericRules {
    fn work_id(&self) -> &'static str {
        "generic"
    }

    fn candidates(&self, piece: &str) -> Vec<Candidate> {
        VARIATIONS
            .iter()
            .map(|&variation| Candidate {
                piece: piece.to_owned(),
                variation,
                transposition: 0,
            })
            .collect()
    }

    fn performance_range(&self, score: &Score) -> Range {
        Range {
            start: 0,
            end: score.onsets.len().saturating_sub(1) as u32,
        }
    }

    fn display_variants(&self) -> Vec<DisplayVariant> {
        vec![DisplayVariant::Complete]
    }
}
