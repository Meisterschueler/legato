//! Tests from Matchmaker `tests/test_prob_hmm.py` (MIDI part), plus the comparison with `*.pthmm.json`.
//! Not transferred: `TestBaseHMM` (generic categorical HMM), `TestPitchIOIHMM` (method `hmm`, not ported)
//! and `TestBernoulliGaussianPitchIOIHMM` (audio).

use legato_core::following::ScoreFollower;
use legato_core::following::pthmm::PitchHmm;

use crate::compare::{Expected, assert_close, check_steps};

/// `TestPitchHMM.test_symbolic`: every returned position is a score onset (without inserted states).
#[test]
fn test_pitch_hmm_symbolic() {
    let expected = Expected::load("mozart_k265_var1", "pthmm");
    let notes = expected.notes();
    let mut onsets: Vec<f64> = notes.iter().map(|n| n.onset_beat).collect();
    onsets.sort_by(f64::total_cmp);
    onsets.dedup();

    let mut hmm = PitchHmm::new(&notes, false);
    for step in &expected.steps {
        let position = hmm.step(&step.input, step.perf_time).unwrap();
        assert!(onsets.contains(&position), "{position}");
    }
}

#[test]
fn intermediates_match_simple_mozart() {
    let expected = Expected::load("simple_mozart", "pthmm");
    let hmm = PitchHmm::new(&expected.notes(), true);
    let transition = expected.matrix("transition_matrix");
    assert_eq!(hmm.transition_matrix().len(), transition.len());
    for (i, (row, expected_row)) in hmm.transition_matrix().iter().zip(&transition).enumerate() {
        assert_close(row, expected_row, &format!("transition_matrix[{i}]"));
    }
    assert_close(
        hmm.initial_probabilities(),
        &expected.vector("initial_probabilities"),
        "initial_probabilities",
    );
    for (i, (profile, expected_profile)) in hmm
        .pitch_profiles()
        .iter()
        .zip(expected.matrix("pitch_profiles"))
        .enumerate()
    {
        assert_close(profile, &expected_profile, &format!("pitch_profiles[{i}]"));
    }
}

#[test]
fn steps_match_matchmaker() {
    for piece in ["simple_mozart", "mozart_k265_var1", "bach_fugue"] {
        let expected = Expected::load(piece, "pthmm");
        check_steps(&mut PitchHmm::new(&expected.notes(), true), &expected);
    }
}
