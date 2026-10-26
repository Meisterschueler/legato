// Ported from Matchmaker (https://github.com/pymatchmaker/matchmaker, commit 6c09490,
// matchmaker/prob/hmm.py: PitchHMM, and python-hiddenmarkov 0.1.4: forward_algorithm_step).
// Copyright the Matchmaker authors, Apache-2.0; see NOTICE.

//! `pthmm`: pitch HMM with inserted states between the onsets (method `pthmm` for MIDI).

use super::repeats::Repeats;
use super::{ReferenceNote, ScoreFollower, unique_onsets};

const PIANO_LOW: usize = 21;
const PIANO_KEYS: usize = 88;
const PROFILE: [f64; 5] = [0.02, 0.02, 1.0, 0.02, 0.02];
const EPS: f64 = 0.01;
/// States ahead whose transition probability follows a repeat; the Gumbel density is
/// negligible beyond.
const REDIRECTED: usize = 8;

pub struct PitchHmm {
    score_positions: Vec<f64>,
    /// Row `i`: probabilities of going from state `i` to every state.
    transition: Vec<Vec<f64>>,
    initial: Vec<f64>,
    /// Bernoulli probability of every piano key per state.
    pitch_profiles: Vec<[f64; PIANO_KEYS]>,
    forward: Option<Vec<f64>>,
    current_index: usize,
    log_evidence: f64,
}

impl PitchHmm {
    /// `has_insertions`: states halfway between consecutive onsets (Matchmaker default for `pthmm`).
    pub fn new(notes: &[ReferenceNote], has_insertions: bool) -> Self {
        Self::with_repeats(notes, has_insertions, &Repeats::default())
    }

    /// Transitions past the end of a repeat section also lead to its start (FR-014).
    pub fn with_repeats(notes: &[ReferenceNote], has_insertions: bool, repeats: &Repeats) -> Self {
        let (onsets, chords) = unique_onsets(notes);
        let mut score_positions = Vec::with_capacity(2 * onsets.len());
        for (i, &onset) in onsets.iter().enumerate() {
            if has_insertions && i > 0 {
                score_positions.push(onsets[i - 1] + 0.5 * (onset - onsets[i - 1]));
            }
            score_positions.push(onset);
        }
        let n = score_positions.len();
        PitchHmm {
            transition: transition_matrix(n, has_insertions, repeats),
            initial: initial_distribution(n),
            pitch_profiles: pitch_profiles(&chords, has_insertions),
            score_positions,
            forward: None,
            current_index: 0,
            log_evidence: 0.0,
        }
    }

    /// Log-likelihood of all observations so far (not in Matchmaker; for the recognition).
    pub fn log_evidence(&self) -> f64 {
        self.log_evidence
    }

    pub fn transition_matrix(&self) -> &[Vec<f64>] {
        &self.transition
    }

    pub fn initial_probabilities(&self) -> &[f64] {
        &self.initial
    }

    pub fn pitch_profiles(&self) -> &[[f64; PIANO_KEYS]] {
        &self.pitch_profiles
    }
}

impl ScoreFollower for PitchHmm {
    fn step(&mut self, pitches: &[u8], _perf_time: f64) -> Option<f64> {
        let mut observed = [false; PIANO_KEYS];
        for &pitch in pitches {
            if let Some(key) = (pitch as usize)
                .checked_sub(PIANO_LOW)
                .filter(|k| *k < PIANO_KEYS)
            {
                observed[key] = true;
            }
        }
        let prior: Vec<f64> = match &self.forward {
            None => self.initial.clone(),
            Some(forward) => (0..self.score_positions.len())
                .map(|j| {
                    forward
                        .iter()
                        .zip(&self.transition)
                        .map(|(f, row)| row[j] * f)
                        .sum()
                })
                .collect(),
        };
        let total: f64 = prior.iter().sum();
        let mut forward: Vec<f64> = prior
            .iter()
            .zip(&self.pitch_profiles)
            .map(|(p, profile)| {
                let likelihood: f64 = profile
                    .iter()
                    .zip(observed)
                    .map(|(&q, o)| if o { q } else { 1.0 - q })
                    .product();
                likelihood * p
            })
            .collect();
        let raw: f64 = forward.iter().sum();
        self.log_evidence += raw.max(f64::MIN_POSITIVE).ln() - total.ln();
        let sum = raw.max(1e-6);
        forward.iter_mut().for_each(|f| *f /= sum);
        self.current_index = argmax(&forward);
        self.forward = Some(forward);
        Some(self.score_positions[self.current_index])
    }

    fn current_index(&self) -> usize {
        self.current_index
    }

    fn score_positions(&self) -> &[f64] {
        &self.score_positions
    }

    /// The forward variable.
    fn state(&self) -> Option<&[f64]> {
        self.forward.as_deref()
    }
}

/// First index of the maximum, like `numpy.argmax`.
fn argmax(values: &[f64]) -> usize {
    let mut best = 0;
    for (i, v) in values.iter().enumerate() {
        if *v > values[best] {
            best = i;
        }
    }
    best
}

/// Density of the left-skewed Gumbel distribution (`scipy.stats.gumbel_l`).
fn gumbel_l_pdf(x: f64, loc: f64, scale: f64) -> f64 {
    let z = (x - loc) / scale;
    (z - z.exp()).exp() / scale
}

/// `stable_transition_matrix` with `mp_trans_state = 1`, `scale = 1.0`; the probability of
/// going `d` states ahead is shared among the states `d` steps ahead in the graph.
fn transition_matrix(n: usize, inserted_states: bool, repeats: &Repeats) -> Vec<Vec<f64>> {
    (0..n)
        .map(|i| {
            let loc = if inserted_states && i % 2 == 0 {
                i + 2
            } else {
                i + 1
            } as f64;
            let mut row: Vec<f64> = (0..n).map(|k| gumbel_l_pdf(k as f64, loc, 1.0)).collect();
            if !repeats.is_empty() {
                let mut moved = Vec::new();
                for d in 1..=REDIRECTED {
                    let targets = states_ahead(i, d, n, inserted_states, repeats);
                    let linear = if i + d < n {
                        vec![(i + d, 1.0)]
                    } else {
                        vec![]
                    };
                    if targets != linear {
                        if i + d < n {
                            row[i + d] = 0.0;
                        }
                        let p = gumbel_l_pdf((i + d) as f64, loc, 1.0);
                        moved.extend(targets.into_iter().map(|(t, share)| (t, share * p)));
                    }
                }
                for (t, p) in moved {
                    row[t] += p;
                }
            }
            let sum: f64 = row.iter().sum();
            row.into_iter().map(|p| p / sum).collect()
        })
        .collect()
}

/// States `d` steps after state `i` among `n` with the share of the ways leading there. Onset `k`
/// is state `2k` with inserted states, `k` without; a jump back has no inserted state.
fn states_ahead(
    i: usize,
    d: usize,
    n: usize,
    inserted_states: bool,
    repeats: &Repeats,
) -> Vec<(usize, f64)> {
    if !inserted_states {
        return repeats.ahead(i, d, n);
    }
    let onsets = n.div_ceil(2);
    if i % 2 == 1 {
        // An inserted state leads to the next onset only.
        return match d {
            1 => vec![(i + 1, 1.0)],
            _ => states_ahead(i + 1, d - 1, n, inserted_states, repeats),
        };
    }
    let reached = repeats.ahead(i / 2, d / 2, onsets);
    if d.is_multiple_of(2) {
        return reached
            .into_iter()
            .map(|(k, share)| (2 * k, share))
            .collect();
    }
    reached
        .into_iter()
        .flat_map(|(k, share)| {
            let successors: Vec<usize> = repeats.successors(k, onsets).collect();
            let part = share / successors.len() as f64;
            successors
                .into_iter()
                .filter(move |&s| s == k + 1)
                .map(move |_| (2 * k + 1, part))
        })
        .collect()
}

/// `init_dist` with `loc = 0`, `scale = 10` (unnormalised, as in Matchmaker).
fn initial_distribution(n: usize) -> Vec<f64> {
    let scale = if 10.0 < n as f64 {
        10.0
    } else {
        n as f64 / 10.0
    };
    (0..n).map(|k| gumbel_l_pdf(k as f64, 0.0, scale)).collect()
}

/// `compute_discrete_pitch_profiles` with piano range.
fn pitch_profiles(chords: &[Vec<u8>], inserted_states: bool) -> Vec<[f64; PIANO_KEYS]> {
    let mut profiles = Vec::with_capacity(2 * chords.len());
    for (i, chord) in chords.iter().enumerate() {
        if inserted_states && i > 0 {
            profiles.push(normalized([EPS; PIANO_KEYS]));
        }
        let mut row = [0.0; 128];
        for &pitch in chord {
            row[pitch as usize] = 1.0;
        }
        let mut profile = [0.0; PIANO_KEYS];
        for (key, value) in profile.iter_mut().enumerate() {
            let center = key + PIANO_LOW;
            let convolved: f64 = (0..PROFILE.len())
                .filter_map(|k| {
                    (center + k)
                        .checked_sub(2)
                        .filter(|p| *p < 128)
                        .map(|p| row[p] * PROFILE[k])
                })
                .sum();
            *value = convolved + EPS;
        }
        profiles.push(normalized(profile));
    }
    profiles
}

fn normalized(mut values: [f64; PIANO_KEYS]) -> [f64; PIANO_KEYS] {
    let sum: f64 = values.iter().sum();
    values.iter_mut().for_each(|v| *v /= sum);
    values
}
