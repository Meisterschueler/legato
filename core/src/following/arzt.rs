// Ported from Matchmaker (https://github.com/pymatchmaker/matchmaker, commit 6c09490,
// matchmaker/dp/oltw_arzt.py: OnlineTimeWarpingArztEvent; matchmaker/features/midi.py:
// onset_pianoroll). Copyright the Matchmaker authors, Apache-2.0; see NOTICE.

//! `arzt`: event-level online time warping after Arzt & Widmer, one step per chord onset.

use super::repeats::{Repeats, way};
use super::{ReferenceNote, ScoreFollower, unique_onsets};

const PIANO_LOW: usize = 21;
const PIANO_KEYS: usize = 88;

pub struct ArztEvent {
    score_positions: Vec<f64>,
    /// Binary onset pianoroll of the reference, one row per onset.
    reference: Vec<[f32; PIANO_KEYS]>,
    window_size: usize,
    start_window_size: usize,
    step_size: usize,
    /// Accumulated costs of the previous input step, row `k + 1` = reference onset `k`.
    /// Stored as `f32` like Matchmaker's cost matrix.
    previous: Vec<f32>,
    /// Reference steps on the best way to every row of `previous`; the onset index without
    /// repeats, as Matchmaker normalizes with.
    previous_steps: Vec<usize>,
    repeats: Repeats,
    input_index: usize,
    current_index: usize,
    /// Reference steps up to `current_index`; the index itself without repeats.
    current_steps: usize,
}

impl ArztEvent {
    /// Matchmaker's defaults for `arzt` with MIDI: window 30, start window 5, step size 5.
    pub fn new(notes: &[ReferenceNote]) -> Self {
        Self::with_parameters(notes, 30, 5, 5)
    }

    /// The window and the steps also lead from the end of a repeat section to its start (FR-014).
    pub fn with_repeats(notes: &[ReferenceNote], repeats: Repeats) -> Self {
        ArztEvent {
            repeats,
            ..Self::new(notes)
        }
    }

    pub fn with_parameters(
        notes: &[ReferenceNote],
        window_size: usize,
        start_window_size: usize,
        step_size: usize,
    ) -> Self {
        let (score_positions, chords) = unique_onsets(notes);
        let reference = chords.iter().map(|chord| vector(chord)).collect();
        ArztEvent {
            previous: vec![f32::INFINITY; score_positions.len() + 1],
            previous_steps: vec![0; score_positions.len() + 1],
            repeats: Repeats::default(),
            score_positions,
            reference,
            window_size,
            start_window_size,
            step_size,
            input_index: 0,
            current_index: 0,
            current_steps: 0,
        }
    }

    /// Matchmaker's `min_index.clamp(current, current + step_size)` along the graph: an onset
    /// not ahead on its best way stays at the current one, one too far ahead is approached by
    /// `step_size` steps. Returns the onset and its reference steps.
    fn advance(&self, target: usize, target_steps: usize) -> (usize, usize) {
        let current = (self.current_index, self.current_steps);
        if target_steps <= self.current_steps {
            return current;
        }
        let n = self.reference.len();
        let reached = self
            .repeats
            .reachable(self.current_index, self.window_size, n);
        let Some(i) = reached.iter().position(|r| r.onset == target) else {
            return current;
        };
        match way(&reached, i).get(self.step_size) {
            Some(&onset) if onset != target => (onset, self.current_steps + self.step_size),
            _ => (target, target_steps),
        }
    }
}

impl ScoreFollower for ArztEvent {
    fn step(&mut self, pitches: &[u8], _perf_time: f64) -> Option<f64> {
        let input = vector(pitches);
        let n = self.reference.len();
        let window = if self.input_index < self.start_window_size {
            self.start_window_size
        } else {
            self.window_size
        };
        let start = self.current_index.saturating_sub(self.step_size);
        // Without repeats `start..start + window`.
        let cells: Vec<usize> = self
            .repeats
            .reachable(start, window.saturating_sub(1), n)
            .into_iter()
            .take(window)
            .map(|r| r.onset)
            .collect();

        let mut current = vec![f32::INFINITY; n + 1];
        let mut steps = vec![0; n + 1];
        let mut min_cost = f64::INFINITY;
        let mut min_index = start;
        for score_index in cells {
            let row = score_index + 1;
            let distance = cosine_distance(&self.reference[score_index], &input);
            if score_index == 0 && self.input_index == 0 {
                current[1] = distance as f32;
                min_cost = distance;
                min_index = 0;
                continue;
            }
            // Matchmaker: from `current[row - 1]`, `previous[row]` and `previous[row - 1]`;
            // the end of a repeat section ending before this onset is a previous row as well.
            let jumps = self
                .repeats
                .jumps()
                .iter()
                .filter(|(_, s)| *s == score_index)
                .map(|(e, _)| e + 1);
            let mut best = (f64::from(current[row - 1]) + distance, steps[row - 1] + 1);
            let mut consider = |cost: f64, length: usize| {
                // Like `f64::min`.
                if cost < best.0 || (best.0.is_nan() && !cost.is_nan()) {
                    best = (cost, length);
                }
            };
            consider(
                f64::from(self.previous[row]) + distance,
                self.previous_steps[row],
            );
            consider(
                f64::from(self.previous[row - 1]) + distance,
                self.previous_steps[row - 1] + 1,
            );
            for from in jumps {
                consider(f64::from(current[from]) + distance, steps[from] + 1);
                consider(
                    f64::from(self.previous[from]) + distance,
                    self.previous_steps[from] + 1,
                );
            }
            let (best, length) = best;
            current[row] = best as f32;
            steps[row] = length;
            let normalized = best / (self.input_index + length + 1) as f64;
            if normalized < min_cost {
                min_cost = normalized;
                min_index = score_index;
            }
        }
        self.previous = current;

        (self.current_index, self.current_steps) = if self.input_index > 0 {
            self.advance(min_index, steps[min_index + 1])
        } else {
            (min_index, steps[min_index + 1])
        };
        self.previous_steps = steps;
        self.input_index += 1;
        Some(self.score_positions[self.current_index])
    }

    fn current_index(&self) -> usize {
        self.current_index
    }

    fn score_positions(&self) -> &[f64] {
        &self.score_positions
    }
}

fn vector(pitches: &[u8]) -> [f32; PIANO_KEYS] {
    let mut vector = [0.0; PIANO_KEYS];
    for &pitch in pitches {
        if let Some(key) = (pitch as usize)
            .checked_sub(PIANO_LOW)
            .filter(|k| *k < PIANO_KEYS)
        {
            vector[key] = 1.0;
        }
    }
    vector
}

/// `scipy.spatial.distance.cosine`.
fn cosine_distance(u: &[f32; PIANO_KEYS], v: &[f32; PIANO_KEYS]) -> f64 {
    let dot: f64 = u
        .iter()
        .zip(v)
        .map(|(a, b)| f64::from(*a) * f64::from(*b))
        .sum();
    let norm = |x: &[f32; PIANO_KEYS]| x.iter().map(|a| f64::from(*a).powi(2)).sum::<f64>().sqrt();
    1.0 - dot / (norm(u) * norm(v))
}
