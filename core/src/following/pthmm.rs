// Ported from Matchmaker (https://github.com/pymatchmaker/matchmaker, commit 6c09490,
// matchmaker/prob/hmm.py: PitchHMM, and python-hiddenmarkov 0.1.4: forward_algorithm_step).
// Copyright the Matchmaker authors, Apache-2.0; see NOTICE.

//! `pthmm`: pitch HMM with inserted states between the onsets (method `pthmm` for MIDI).

use super::{ReferenceNote, ScoreFollower, unique_onsets};

const PIANO_LOW: usize = 21;
const PIANO_KEYS: usize = 88;
const PROFILE: [f64; 5] = [0.02, 0.02, 1.0, 0.02, 0.02];
const EPS: f64 = 0.01;

pub struct PitchHmm {
    score_positions: Vec<f64>,
    /// Row `i`: probabilities of going from state `i` to every state.
    transition: Vec<Vec<f64>>,
    initial: Vec<f64>,
    /// Bernoulli probability of every piano key per state.
    pitch_profiles: Vec<[f64; PIANO_KEYS]>,
    forward: Option<Vec<f64>>,
    current_index: usize,
}

impl PitchHmm {
    /// `has_insertions`: states halfway between consecutive onsets (Matchmaker default for `pthmm`).
    pub fn new(notes: &[ReferenceNote], has_insertions: bool) -> Self {
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
            transition: transition_matrix(n, has_insertions),
            initial: initial_distribution(n),
            pitch_profiles: pitch_profiles(&chords, has_insertions),
            score_positions,
            forward: None,
            current_index: 0,
        }
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
        let sum = forward.iter().sum::<f64>().max(1e-6);
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

/// `stable_transition_matrix` with `mp_trans_state = 1`, `scale = 1.0`.
fn transition_matrix(n: usize, inserted_states: bool) -> Vec<Vec<f64>> {
    (0..n)
        .map(|i| {
            let loc = if inserted_states && i % 2 == 0 {
                i + 2
            } else {
                i + 1
            } as f64;
            let row: Vec<f64> = (0..n).map(|k| gumbel_l_pdf(k as f64, loc, 1.0)).collect();
            let sum: f64 = row.iter().sum();
            row.into_iter().map(|p| p / sum).collect()
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
