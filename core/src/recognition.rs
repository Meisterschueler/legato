//! Recognition of piece, variation and key (FR-016, research.md R8): every permissible
//! candidate is followed in parallel by a `pthmm`; the posterior over the candidates decides.

use crate::following::pthmm::PitchHmm;
use crate::following::{ReferenceNote, ScoreFollower};
use crate::preprocessing::Observation;
use crate::score::model::{Candidate, Hand, Score, Variation};
use crate::works::Range;

/// Tones per played hand before any decision.
const MIN_TONES_PER_HAND: usize = 4;
const MIN_POSTERIOR: f64 = 0.95;
const MIN_RATIO: f64 = 20.0;
/// Posterior needed once the recognition length is reached.
const FINAL_POSTERIOR: f64 = 0.5;

/// Notes of `range` as the candidate expects them: its hands, transposed.
pub fn reference(score: &Score, candidate: &Candidate, range: Range) -> Vec<ReferenceNote> {
    score.onsets[range.start as usize..=range.end as usize]
        .iter()
        .flat_map(|onset| {
            let beat = onset.time.numerator as f64 / onset.time.denominator as f64;
            onset
                .notes
                .iter()
                .filter(|n| plays(candidate.variation, n.hand))
                .map(move |n| ReferenceNote {
                    onset_beat: beat,
                    pitch: n.pitch.saturating_add_signed(candidate.transposition),
                })
        })
        .collect()
}

fn plays(variation: Variation, hand: Hand) -> bool {
    match variation {
        Variation::BothHands => true,
        Variation::LeftHandOnly => hand == Hand::Left,
        Variation::RightHandOnly => hand == Hand::Right,
    }
}

fn hands(variation: Variation) -> usize {
    if variation == Variation::BothHands {
        2
    } else {
        1
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Decision {
    Pending,
    /// Index of the recognized candidate.
    Recognized(usize),
    /// No unique candidate within the recognition length: no performance.
    NoPerformance,
}

pub struct Recognition {
    candidates: Vec<Candidate>,
    // shortcut: one dense pthmm per candidate (36 for Hanon); band the matrices if T102 is missed.
    followers: Vec<PitchHmm>,
    /// Tones per hand (`WorkRules::recognition_length`).
    recognition_length: usize,
    tones: usize,
}

impl Recognition {
    pub fn new(
        candidates: Vec<(Candidate, Vec<ReferenceNote>)>,
        recognition_length: usize,
    ) -> Self {
        let (candidates, followers) = candidates
            .into_iter()
            .map(|(c, notes)| (c, PitchHmm::new(&notes, true)))
            .unzip();
        Recognition {
            candidates,
            followers,
            recognition_length,
            tones: 0,
        }
    }

    pub fn candidates(&self) -> &[Candidate] {
        &self.candidates
    }

    pub fn observe(&mut self, observation: &Observation) -> Decision {
        for follower in &mut self.followers {
            follower.step(&observation.pitches, observation.time);
        }
        self.tones += observation.pitches.len();
        self.decide()
    }

    /// Posterior of every candidate under a uniform prior.
    pub fn posteriors(&self) -> Vec<f64> {
        let logs: Vec<f64> = self.followers.iter().map(|f| f.log_evidence()).collect();
        let max = logs.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        let weights: Vec<f64> = logs.iter().map(|l| (l - max).exp()).collect();
        let sum: f64 = weights.iter().sum();
        weights.iter().map(|w| w / sum).collect()
    }

    /// The follower of a recognized candidate, positioned after all observations so far.
    pub fn into_follower(self, index: usize) -> (Candidate, PitchHmm) {
        let follower = self
            .followers
            .into_iter()
            .nth(index)
            .expect("candidate index");
        (self.candidates[index].clone(), follower)
    }

    fn decide(&self) -> Decision {
        let posteriors = self.posteriors();
        let mut order: Vec<usize> = (0..posteriors.len()).collect();
        order.sort_by(|&a, &b| posteriors[b].total_cmp(&posteriors[a]));
        let Some(&best) = order.first() else {
            return Decision::NoPerformance;
        };
        let second = order.get(1).map_or(0.0, |&i| posteriors[i]);
        let hands = hands(self.candidates[best].variation);
        if self.tones < MIN_TONES_PER_HAND * hands {
            return Decision::Pending;
        }
        let p = posteriors[best];
        if p >= MIN_POSTERIOR && p >= MIN_RATIO * second {
            Decision::Recognized(best)
        } else if self.tones < self.recognition_length * hands {
            Decision::Pending
        } else if p >= FINAL_POSTERIOR {
            Decision::Recognized(best)
        } else {
            Decision::NoPerformance
        }
    }
}
