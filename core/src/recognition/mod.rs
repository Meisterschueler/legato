//! Recognition of piece, variation and key (FR-016, FR-085, research.md R8 and R15) in two
//! stages: a preselection picks a few candidates after every onset, each of them is followed by
//! a `pthmm`, and the posterior over them decides.

use std::sync::Arc;

use crate::following::pthmm::{PitchHmm, Transition};
use crate::following::repeats::Repeats;
use crate::following::{ReferenceNote, ScoreFollower};
use crate::performance::Setup;
use crate::preprocessing::Observation;
use crate::score::model::{Candidate, Hand, Score, Variation};
use crate::works::Range;

/// Tones per played hand before any decision.
const MIN_TONES_PER_HAND: usize = 4;
const MIN_POSTERIOR: f64 = 0.95;
const MIN_RATIO: f64 = 20.0;
/// Posterior needed once the recognition length is reached.
const FINAL_POSTERIOR: f64 = 0.5;
/// Observations within this time (seconds) after the first tone form one onset (as `arzt`).
const ONSET_WINDOW: f64 = 0.035;
/// Candidates in a preselection, except `Exhaustive`.
pub const PRESELECTED: usize = 6;

/// How the candidates followed during the recognition are chosen (FR-085, research.md R15).
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum RecognitionMethod {
    /// All candidates (research.md R8).
    Exhaustive,
}

/// Pitches played or notated together; `time` in seconds when played, in beats in a reference.
#[derive(Clone, Debug, PartialEq)]
pub struct Onset {
    pub time: f64,
    /// Sorted, without duplicates.
    pub pitches: Vec<u8>,
}

/// First stage of the recognition: the candidates worth a `pthmm`.
pub trait Preselector: Send {
    /// The onsets played so far in this recognition; the last one is new or has grown since the
    /// previous call.
    fn observe(&mut self, onsets: &[Onset]);

    /// Indices of at most [`PRESELECTED`] candidates of at least the minimum quality, best first,
    /// equal ones in the order of the candidates.
    fn preselection(&self) -> Vec<usize>;
}

/// No preselection: every candidate.
struct Exhaustive {
    candidates: usize,
}

impl Preselector for Exhaustive {
    fn observe(&mut self, _onsets: &[Onset]) {}

    fn preselection(&self) -> Vec<usize> {
        (0..self.candidates).collect()
    }
}

/// The candidates with what every recognition needs, built once per repertoire and settings.
pub struct Repertoire {
    candidates: Vec<Candidate>,
    references: Vec<Vec<ReferenceNote>>,
    /// Shared by the candidates of a piece and variation.
    transitions: Vec<Transition>,
    /// Tones per hand (`WorkRules::recognition_length`).
    recognition_length: usize,
    method: RecognitionMethod,
}

impl Repertoire {
    /// `candidates`: each with its reference and repeats (FR-014).
    pub fn new(
        candidates: Vec<(Candidate, Vec<ReferenceNote>, Repeats)>,
        recognition_length: usize,
        method: RecognitionMethod,
    ) -> Self {
        let mut shared: Vec<(&Candidate, Transition)> = Vec::new();
        let transitions = candidates
            .iter()
            .map(|(candidate, notes, repeats)| {
                let same = |c: &&Candidate| {
                    c.piece == candidate.piece && c.variation == candidate.variation
                };
                if let Some((_, t)) = shared.iter().find(|(c, _)| same(c)) {
                    return Arc::clone(t);
                }
                let t = PitchHmm::transition_for(notes, true, repeats);
                shared.push((candidate, Arc::clone(&t)));
                t
            })
            .collect();
        let (candidates, references) = candidates.into_iter().map(|(c, n, _)| (c, n)).unzip();
        Repertoire {
            candidates,
            references,
            transitions,
            recognition_length,
            method,
        }
    }

    pub fn for_setups(
        setups: &[Setup],
        recognition_length: usize,
        method: RecognitionMethod,
    ) -> Self {
        let candidates = setups
            .iter()
            .map(|s| (s.candidate.clone(), s.reference(), s.repeats.clone()))
            .collect();
        Self::new(candidates, recognition_length, method)
    }

    pub fn candidates(&self) -> &[Candidate] {
        &self.candidates
    }

    fn preselector(&self) -> Box<dyn Preselector> {
        match self.method {
            RecognitionMethod::Exhaustive => Box::new(Exhaustive {
                candidates: self.candidates.len(),
            }),
        }
    }

    fn follower(&self, candidate: usize) -> PitchHmm {
        PitchHmm::with_transition(
            &self.references[candidate],
            true,
            Arc::clone(&self.transitions[candidate]),
        )
    }
}

/// Notes of `range` as the candidate expects them: its hands, transposed.
pub fn reference(score: &Score, candidate: &Candidate, range: Range) -> Vec<ReferenceNote> {
    Setup::new(score, candidate.clone(), range).reference()
}

/// Whether the variation plays notes of `hand`.
pub fn plays(variation: Variation, hand: Hand) -> bool {
    match variation {
        Variation::BothHands => true,
        Variation::LeftHandOnly => hand == Hand::Left,
        Variation::RightHandOnly => hand == Hand::Right,
    }
}

pub(crate) fn hands(variation: Variation) -> usize {
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
    repertoire: Arc<Repertoire>,
    preselector: Box<dyn Preselector>,
    /// All observations of this recognition, replayed to newly preselected candidates.
    observations: Vec<Observation>,
    onsets: Vec<Onset>,
    /// The preselected candidates with their `pthmm`, in the order of the candidates.
    followers: Vec<(usize, PitchHmm)>,
    tones: usize,
}

impl Recognition {
    /// All `candidates`, without repeats.
    pub fn new(
        candidates: Vec<(Candidate, Vec<ReferenceNote>)>,
        recognition_length: usize,
    ) -> Self {
        let candidates = candidates
            .into_iter()
            .map(|(c, notes)| (c, notes, Repeats::default()))
            .collect();
        Self::start(Arc::new(Repertoire::new(
            candidates,
            recognition_length,
            RecognitionMethod::Exhaustive,
        )))
    }

    pub fn start(repertoire: Arc<Repertoire>) -> Self {
        Recognition {
            preselector: repertoire.preselector(),
            repertoire,
            observations: Vec::new(),
            onsets: Vec::new(),
            followers: Vec::new(),
            tones: 0,
        }
    }

    pub fn candidates(&self) -> &[Candidate] {
        &self.repertoire.candidates
    }

    pub fn observe(&mut self, observation: &Observation) -> Decision {
        for (_, follower) in &mut self.followers {
            follower.step(&observation.pitches, observation.time);
        }
        self.observations.push(observation.clone());
        self.tones += observation.pitches.len();
        if self
            .onsets
            .last()
            .is_none_or(|last| observation.time - last.time > ONSET_WINDOW)
        {
            self.onsets.push(Onset {
                time: observation.time,
                pitches: Vec::new(),
            });
        }
        let onset = self.onsets.last_mut().expect("onset pushed above");
        onset.pitches.extend(&observation.pitches);
        onset.pitches.sort_unstable();
        onset.pitches.dedup();
        self.preselector.observe(&self.onsets);
        self.follow(self.preselector.preselection());
        self.decide()
    }

    /// Followers for the preselected candidates only; new ones replay the observations so far.
    fn follow(&mut self, preselection: Vec<usize>) {
        let mut selected = vec![false; self.repertoire.candidates.len()];
        for &c in &preselection {
            selected[c] = true;
        }
        self.followers.retain(|(c, _)| selected[*c]);
        for (c, _) in &self.followers {
            selected[*c] = false;
        }
        for c in (0..selected.len()).filter(|&c| selected[c]) {
            let mut follower = self.repertoire.follower(c);
            for o in &self.observations {
                follower.step(&o.pitches, o.time);
            }
            let at = self.followers.partition_point(|(f, _)| *f < c);
            self.followers.insert(at, (c, follower));
        }
    }

    /// Posterior of every candidate under a uniform prior over the preselected ones; zero for
    /// the others.
    pub fn posteriors(&self) -> Vec<f64> {
        let mut posteriors = vec![0.0; self.repertoire.candidates.len()];
        let logs: Vec<f64> = self
            .followers
            .iter()
            .map(|(_, f)| f.log_evidence())
            .collect();
        let max = logs.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        let weights: Vec<f64> = logs.iter().map(|l| (l - max).exp()).collect();
        let sum: f64 = weights.iter().sum();
        for ((c, _), w) in self.followers.iter().zip(weights) {
            posteriors[*c] = w / sum;
        }
        posteriors
    }

    /// The follower of a recognized candidate, positioned after all observations so far.
    pub fn into_follower(self, index: usize) -> (Candidate, PitchHmm) {
        let follower = self
            .followers
            .into_iter()
            .find_map(|(c, f)| (c == index).then_some(f))
            .expect("preselected candidate");
        (self.repertoire.candidates[index].clone(), follower)
    }

    fn decide(&self) -> Decision {
        let candidates = &self.repertoire.candidates;
        let length = self.repertoire.recognition_length;
        if self.followers.is_empty() {
            let Some(hands) = candidates.iter().map(|c| hands(c.variation)).max() else {
                return Decision::NoPerformance;
            };
            return if self.tones < length * hands {
                Decision::Pending
            } else {
                Decision::NoPerformance
            };
        }
        let posteriors = self.posteriors();
        let mut order: Vec<usize> = self.followers.iter().map(|(c, _)| *c).collect();
        order.sort_by(|&a, &b| posteriors[b].total_cmp(&posteriors[a]));
        let best = order[0];
        let second = order.get(1).map_or(0.0, |&i| posteriors[i]);
        let hands = hands(candidates[best].variation);
        if self.tones < MIN_TONES_PER_HAND * hands {
            return Decision::Pending;
        }
        let p = posteriors[best];
        if p >= MIN_POSTERIOR && p >= MIN_RATIO * second {
            Decision::Recognized(best)
        } else if self.tones < length * hands {
            Decision::Pending
        } else if p >= FINAL_POSTERIOR {
            Decision::Recognized(best)
        } else {
            Decision::NoPerformance
        }
    }
}
