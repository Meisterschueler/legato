//! Comparison with the Matchmaker reference (contracts/matchmaker-referenz.md).

use legato_core::following::{ReferenceNote, ScoreFollower};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Expected {
    pub note_array: Vec<ExpectedNote>,
    pub score_positions: Vec<f64>,
    #[serde(default)]
    pub intermediates: Option<serde_json::Map<String, serde_json::Value>>,
    pub steps: Vec<Step>,
}

#[derive(Deserialize)]
pub struct ExpectedNote {
    pub onset_beat: f64,
    pub pitch: u8,
}

#[derive(Deserialize)]
pub struct Step {
    pub perf_time: f64,
    pub input: Vec<u8>,
    pub position: Option<f64>,
    pub onset_index: usize,
    #[serde(default)]
    pub state: Option<Vec<f64>>,
}

impl Expected {
    pub fn load(piece: &str, method: &str) -> Self {
        let path = format!(
            "{}/../testdata/matchmaker/expected/{piece}.{method}.json",
            env!("CARGO_MANIFEST_DIR")
        );
        serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap()
    }

    pub fn notes(&self) -> Vec<ReferenceNote> {
        self.note_array
            .iter()
            .map(|n| ReferenceNote {
                onset_beat: n.onset_beat,
                pitch: n.pitch,
            })
            .collect()
    }

    pub fn matrix(&self, name: &str) -> Vec<Vec<f64>> {
        serde_json::from_value(self.intermediates.as_ref().unwrap()[name].clone()).unwrap()
    }

    pub fn vector(&self, name: &str) -> Vec<f64> {
        serde_json::from_value(self.intermediates.as_ref().unwrap()[name].clone()).unwrap()
    }
}

/// Relative 1e-6; values below 1e-12 count as zero (underflow differs between implementations).
pub fn assert_close(actual: &[f64], expected: &[f64], what: &str) {
    assert_eq!(actual.len(), expected.len(), "{what}: length");
    for (i, (a, e)) in actual.iter().zip(expected).enumerate() {
        let tolerance = 1e-6 * a.abs().max(e.abs()) + 1e-12;
        assert!((a - e).abs() <= tolerance, "{what}[{i}]: {a} != {e}");
    }
}

/// Feeds the recorded inputs; checks positions (≥ 99 % identical, no deviation > 1) and states.
pub fn check_steps(follower: &mut dyn ScoreFollower, expected: &Expected) {
    assert_close(
        follower.score_positions(),
        &expected.score_positions,
        "score_positions",
    );
    let mut identical = 0;
    for (n, step) in expected.steps.iter().enumerate() {
        let position = follower.step(&step.input, step.perf_time);
        let index = follower.current_index();
        let deviation = index.abs_diff(step.onset_index);
        assert!(
            deviation <= 1,
            "step {n}: onset_index {index} != {}",
            step.onset_index
        );
        if deviation == 0 {
            identical += 1;
            assert_eq!(
                position.is_some(),
                step.position.is_some(),
                "step {n}: position"
            );
        }
        if let (Some(state), Some(actual)) = (&step.state, follower.state()) {
            assert_close(actual, state, &format!("step {n}: state"));
        }
    }
    let share = identical as f64 / expected.steps.len() as f64;
    assert!(
        share >= 0.99,
        "only {:.1} % identical positions",
        share * 100.0
    );
}
