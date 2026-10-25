//! Tests from Matchmaker `tests/test_prob_outer_product_hmm.py`, plus the comparison with `*.outerhmm.json`.
//! partitura's `EXAMPLE_MUSICXML` is the note list of `example_notes()`.

use legato_core::following::outer_hmm::OuterProductHmm;
use legato_core::following::{ReferenceNote, ScoreFollower};

use crate::compare::{Expected, assert_close, check_steps};

/// partitura `EXAMPLE_MUSICXML`: A4, then the chord C5 + E5.
fn example_notes() -> Vec<ReferenceNote> {
    [(0.0, 69), (2.0, 72), (2.0, 76)]
        .map(|(onset_beat, pitch)| ReferenceNote { onset_beat, pitch })
        .to_vec()
}

#[test]
fn test_compute_outer_product_hmm_pitch_probabilities() {
    let hmm = OuterProductHmm::new(&example_notes());
    let b_table = hmm.b_table();
    let argmax = (0..128)
        .max_by(|&a, &b| b_table[0][a].total_cmp(&b_table[0][b]))
        .unwrap();
    assert_eq!(argmax, 69);
    assert_eq!(b_table[1][72], b_table[1][76]);
}

#[test]
fn test_compute_transition_matrix() {
    let hmm = OuterProductHmm::new(&example_notes());
    let alpha = hmm.transition_matrix();
    assert_eq!((alpha.len(), alpha[0].len()), (2, 2));
    for row in alpha {
        assert!((row.iter().sum::<f64>() - 1.0).abs() < 1e-12);
    }
}

/// `TestOuterProductHMM.test_outer_product_hmm`: every returned beat is a score onset.
#[test]
fn test_outer_product_hmm() {
    let expected = Expected::load("mozart_k265_var1", "outerhmm");
    let mut hmm = OuterProductHmm::new(&expected.notes());
    for step in &expected.steps {
        let beat = hmm.step(&step.input, step.perf_time).unwrap();
        assert!(expected.score_positions.contains(&beat), "{beat}");
    }
}

#[test]
fn intermediates_match_simple_mozart() {
    let expected = Expected::load("simple_mozart", "outerhmm");
    let hmm = OuterProductHmm::new(&expected.notes());
    for (i, (row, expected_row)) in hmm
        .transition_matrix()
        .iter()
        .zip(expected.matrix("alpha"))
        .enumerate()
    {
        assert_close(row, &expected_row, &format!("alpha[{i}]"));
    }
    for (i, (row, expected_row)) in hmm
        .b_table()
        .iter()
        .zip(expected.matrix("b_table"))
        .enumerate()
    {
        assert_close(row, &expected_row, &format!("b_table[{i}]"));
    }
}

#[test]
fn steps_match_matchmaker() {
    for piece in ["simple_mozart", "mozart_k265_var1", "bach_fugue"] {
        let expected = Expected::load(piece, "outerhmm");
        check_steps(&mut OuterProductHmm::new(&expected.notes()), &expected);
    }
}
