//! Hanon, part 1 (docs/werke/hanon.md).

use super::generic::VARIATIONS;
use super::{DisplayVariant, Range, WorkRules};
use crate::score::model::{Candidate, Hand, Score};
use crate::score::musicxml_write::Shown;

pub struct HanonRules;

impl HanonRules {
    /// "Ascending" and "Descending": the repeat section split after the measure ending with a
    /// double barline (turning point); without one, at half of its measures (with an odd count,
    /// descending has one measure more).
    pub fn directions(&self, score: &Score) -> [Range; 2] {
        let section = self.performance_range(score);
        let measure = |index: u32| score.onsets[index as usize].notes[0].id.measure;
        let first = measure(section.start);
        let last = measure(section.end);
        let descending = (first..last)
            .find(|&m| score.measures.get(m as usize).is_some_and(|m| m.double_bar))
            .map_or(first + (last + 1 - first) / 2, |turn| turn + 1);
        let split = (section.start..=section.end)
            .find(|&i| measure(i) >= descending)
            .unwrap_or(section.end + 1);
        [
            Range {
                start: section.start,
                end: split - 1,
            },
            Range {
                start: split,
                end: section.end,
            },
        ]
    }

    /// First notated pitch of `hand`.
    fn first_pitch(score: &Score, hand: Hand) -> Option<u8> {
        score
            .onsets
            .iter()
            .flat_map(|o| &o.notes)
            .find(|n| n.hand == hand)
            .map(|n| n.pitch)
    }
}

impl WorkRules for HanonRules {
    fn work_id(&self) -> &'static str {
        "hanon"
    }

    fn check(&self, score: &Score) -> Result<(), String> {
        if score.sections.is_empty() {
            return Err("Hanon, Teil 1: Wiederholungsabschnitt fehlt".into());
        }
        Ok(())
    }

    fn candidates(&self, piece: &str) -> Vec<Candidate> {
        VARIATIONS
            .iter()
            .flat_map(|&variation| {
                (0..12).map(move |transposition| Candidate {
                    piece: piece.to_owned(),
                    variation,
                    transposition,
                })
            })
            .collect()
    }

    /// The repeat section, without "Ende".
    fn performance_range(&self, score: &Score) -> Range {
        let section = &score.sections[0];
        Range {
            start: section.start,
            end: section.end,
        }
    }

    fn display_variants(&self) -> Vec<DisplayVariant> {
        vec![
            DisplayVariant::Complete,
            DisplayVariant::Abbreviated,
            DisplayVariant::FirstMeasures,
            DisplayVariant::FirstMeasure,
        ]
    }

    fn shown_measures(&self, score: &Score, variant: DisplayVariant) -> Option<Vec<Shown>> {
        let measure = |index: u32| score.onsets[index as usize].notes[0].id.measure;
        let [up, down] = self
            .directions(score)
            .map(|r| (measure(r.start), measure(r.end)));
        let shown = |measure, simile, new_system| Shown {
            measure,
            simile,
            new_system,
        };
        match variant {
            DisplayVariant::Complete => None,
            // First and last two measures of each direction, everything else as is.
            DisplayVariant::Abbreviated => {
                let keep = |m: u32| {
                    [up, down]
                        .iter()
                        .all(|&(first, last)| m < first + 2 || m + 2 > last)
                };
                Some(
                    (0..score.measures.len() as u32)
                        .filter(|&m| keep(m))
                        .map(|m| shown(m, m > 0 && !keep(m - 1), false))
                        .collect(),
                )
            }
            DisplayVariant::FirstMeasures => {
                Some(vec![shown(up.0, false, false), shown(down.0, false, true)])
            }
            DisplayVariant::FirstMeasure => Some(vec![shown(up.0, false, false)]),
        }
    }

    /// Both hands are transposed by the same 0–11 half tones and notated an octave apart,
    /// so the possible first tones of the hands do not overlap.
    fn hand_from_position(&self, score: &Score, first_pitch: u8) -> Option<Hand> {
        [Hand::Left, Hand::Right].into_iter().find(|&hand| {
            Self::first_pitch(score, hand)
                .is_some_and(|start| (start..start.saturating_add(12)).contains(&first_pitch))
        })
    }
}
