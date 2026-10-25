// Ported from Matchmaker (https://github.com/pymatchmaker/matchmaker, commit 6c09490,
// matchmaker/prob/outer_product_hmm.py: OuterProductHMM). Copyright the Matchmaker authors,
// Apache-2.0; see NOTICE.

//! `outerhmm`: outer-product HMM after Nakamura et al. (2014), one Viterbi step per input note;
//! notes closer than 35 ms form one chord observation.

use super::{ReferenceNote, ScoreFollower, unique_onsets};

const PIANO_LOW: usize = 21;
const PIANO_KEYS: usize = 88;
const IOI_THRESHOLD: f64 = 0.035;
const OTHER_PROB: f64 = 1e-6;
const CORRECT: f64 = 0.9497;
const SEMITONE: f64 = 0.0145 / 2.0;
const WHOLE_TONE: f64 = 0.0224 / 2.0;
const OCTAVE: f64 = 0.0047 / 2.0;
const WITHIN_OCTAVE: f64 = 0.0086 / 9.0 / 2.0;
const TRANSITIONS: [(isize, f64); 7] = [
    (-3, 0.00509),
    (-2, 0.00516),
    (-1, 0.00886),
    (0, 0.01342),
    (1, 0.94531),
    (2, 0.00610),
    (3, 0.00073),
];
const D1: usize = 3;
const D2: usize = 3;

pub struct OuterProductHmm {
    score_positions: Vec<f64>,
    alpha: Vec<Vec<f64>>,
    /// Skip-from and skip-to probabilities (uniform).
    skip: f64,
    /// Emission probability of every MIDI pitch per state.
    b_table: Vec<[f64; 128]>,
    probabilities: Vec<f64>,
    pre_chord: Vec<f64>,
    chord: [bool; PIANO_KEYS],
    previous_time: Option<f64>,
    current_index: usize,
}

impl OuterProductHmm {
    pub fn new(notes: &[ReferenceNote]) -> Self {
        let (score_positions, chords) = unique_onsets(notes);
        let n = score_positions.len();
        let mut probabilities = vec![0.0; n];
        probabilities[0] = 1.0;
        OuterProductHmm {
            alpha: transition_matrix(n),
            skip: 1.0 / n as f64,
            b_table: chords.iter().map(|c| pitch_probabilities(c)).collect(),
            pre_chord: probabilities.clone(),
            probabilities,
            score_positions,
            chord: [false; PIANO_KEYS],
            previous_time: None,
            current_index: 0,
        }
    }

    pub fn transition_matrix(&self) -> &[Vec<f64>] {
        &self.alpha
    }

    pub fn b_table(&self) -> &[[f64; 128]] {
        &self.b_table
    }

    fn viterbi_step(&self, previous: &[f64], observation: &[bool; PIANO_KEYS]) -> Vec<f64> {
        let n = self.score_positions.len();
        let log_emission: Vec<f64> = self
            .b_table
            .iter()
            .map(|b| {
                (0..PIANO_KEYS)
                    .filter(|&k| observation[k])
                    .map(|k| b[k + PIANO_LOW].max(1e-300).ln())
                    .sum()
            })
            .collect();
        let max = log_emission
            .iter()
            .copied()
            .fold(f64::NEG_INFINITY, f64::max);
        let global_skip = previous.iter().map(|p| p * self.skip).fold(0.0, f64::max);
        let mut next: Vec<f64> = (0..n)
            .map(|i| {
                let local = (i.saturating_sub(D2)..n.min(i + D1 + 1))
                    .map(|j| previous[j] * self.alpha[j][i])
                    .fold(0.0, |m: f64, v| if v > m { v } else { m });
                let skip = self.skip * global_skip;
                (log_emission[i] - max).exp() * if skip >= local { skip } else { local }
            })
            .collect();
        let sum: f64 = next.iter().sum();
        if sum > 0.0 {
            next.iter_mut().for_each(|p| *p /= sum);
        } else {
            next = vec![1.0 / n as f64; n];
        }
        next
    }
}

impl ScoreFollower for OuterProductHmm {
    fn step(&mut self, pitches: &[u8], perf_time: f64) -> Option<f64> {
        let mut observation = [false; PIANO_KEYS];
        for &pitch in pitches {
            if let Some(key) = (pitch as usize)
                .checked_sub(PIANO_LOW)
                .filter(|k| *k < PIANO_KEYS)
            {
                observation[key] = true;
            }
        }
        let continuation = self
            .previous_time
            .is_some_and(|t| perf_time - t < IOI_THRESHOLD);
        self.previous_time = Some(perf_time);
        if continuation {
            // Redo the step from the state before the chord with all chord notes so far.
            for (c, o) in self.chord.iter_mut().zip(observation) {
                *c |= o;
            }
            self.probabilities = self.viterbi_step(&self.pre_chord, &self.chord);
        } else {
            self.pre_chord = self.probabilities.clone();
            self.chord = observation;
            self.probabilities = self.viterbi_step(&self.probabilities, &observation);
        }
        self.current_index = argmax(&self.probabilities);
        Some(self.score_positions[self.current_index])
    }

    fn current_index(&self) -> usize {
        self.current_index
    }

    fn score_positions(&self) -> &[f64] {
        &self.score_positions
    }

    fn state(&self) -> Option<&[f64]> {
        Some(&self.probabilities)
    }
}

fn argmax(values: &[f64]) -> usize {
    let mut best = 0;
    for (i, v) in values.iter().enumerate() {
        if *v > values[best] {
            best = i;
        }
    }
    best
}

/// `compute_transition_matrix` with the default transitions.
fn transition_matrix(n: usize) -> Vec<Vec<f64>> {
    (0..n)
        .map(|i| {
            let mut row = vec![1e-6; n];
            for (delta, p) in TRANSITIONS {
                if let Some(j) = i.checked_add_signed(delta).filter(|j| *j < n) {
                    row[j] = p;
                }
            }
            row.iter_mut().for_each(|v| *v += f64::EPSILON);
            let sum: f64 = row.iter().sum();
            row.into_iter().map(|v| v / sum).collect()
        })
        .collect()
}

/// `compute_OuterProductHMM_pitch_probabilities` with the default pitch error probabilities.
fn pitch_probabilities(chord: &[u8]) -> [f64; 128] {
    let mut table = [OTHER_PROB; 128];
    let mut class = [0u8; 128]; // 0 other, 1 correct, 2 semitone, 3 whole tone, 4 octave, 5 within octave
    let mut correct: Vec<usize> = chord.iter().map(|&p| p as usize).collect();
    correct.sort_unstable();
    correct.dedup();
    for &p in &correct {
        class[p] = 1;
    }
    for (offsets, kind) in [(&[1, -1][..], 2), (&[2, -2][..], 3), (&[12, -12][..], 4)] {
        for &p in &correct {
            for &offset in offsets {
                if let Some(q) = p.checked_add_signed(offset).filter(|q| *q < 128)
                    && class[q] == 0
                {
                    class[q] = kind;
                }
            }
        }
    }
    for &p in &correct {
        for kind in &mut class[p.saturating_sub(11)..(p + 12).min(128)] {
            if *kind == 0 {
                *kind = 5;
            }
        }
    }
    let count = |kind: u8| class.iter().filter(|&&c| c == kind).count().max(1) as f64;
    let probabilities = [0.0, CORRECT, SEMITONE, WHOLE_TONE, OCTAVE, WITHIN_OCTAVE];
    for (p, &kind) in class.iter().enumerate() {
        if kind != 0 {
            table[p] = probabilities[kind as usize] / count(kind);
        }
    }
    table
}
