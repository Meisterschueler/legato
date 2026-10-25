//! Comparison of `arzt` (event level, MIDI) with `*.arzt.json`. Matchmaker's `tests/test_oltw_arzt.py`
//! only tests the frame-level variant for audio; there is no MIDI test to transfer.

use legato_core::following::ScoreFollower;
use legato_core::following::arzt::ArztEvent;

use crate::compare::{Expected, check_steps};

#[test]
fn every_position_is_a_score_onset() {
    let expected = Expected::load("mozart_k265_var1", "arzt");
    let mut arzt = ArztEvent::new(&expected.notes());
    for step in &expected.steps {
        let beat = arzt.step(&step.input, step.perf_time).unwrap();
        assert!(expected.score_positions.contains(&beat), "{beat}");
    }
}

#[test]
fn steps_match_matchmaker() {
    for piece in ["simple_mozart", "mozart_k265_var1", "bach_fugue"] {
        let expected = Expected::load(piece, "arzt");
        check_steps(&mut ArztEvent::new(&expected.notes()), &expected);
    }
}
