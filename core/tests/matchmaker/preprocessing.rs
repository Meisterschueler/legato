//! The preprocessor reproduces Matchmaker's observations from the raw note-ons (T036).

use legato_core::preprocessing::{Grouping, Observation, Preprocessor};

use crate::compare::Expected;

const PIECES: [&str; 3] = ["simple_mozart", "mozart_k265_var1", "bach_fugue"];

fn assert_same(observations: &[Observation], expected: &Expected, what: &str) {
    assert_eq!(observations.len(), expected.steps.len(), "{what}: count");
    for (n, (observation, step)) in observations.iter().zip(&expected.steps).enumerate() {
        let mut pitches = observation.pitches.clone();
        pitches.sort_unstable();
        assert_eq!(pitches, step.input, "{what}: step {n}");
        assert!(
            (observation.time - step.perf_time).abs() < 1e-12,
            "{what}: step {n} time"
        );
    }
}

/// Without clock: every note goes in, the last observation comes from `finish`.
fn stream(grouping: Grouping, expected: &Expected) -> Vec<Observation> {
    let mut preprocessor = Preprocessor::new(grouping);
    let mut observations: Vec<Observation> = expected
        .midi_note_ons
        .iter()
        .filter_map(|&(time, pitch)| preprocessor.note_on(time, pitch))
        .collect();
    observations.extend(preprocessor.finish());
    observations
}

#[test]
fn pthmm_windows_of_10_ms() {
    for piece in PIECES {
        let expected = Expected::load(piece, "pthmm");
        let observations = stream(Grouping::Window { period: 0.01 }, &expected);
        assert_same(&observations, &expected, piece);
    }
}

#[test]
fn outerhmm_windows_of_1_ms() {
    for piece in PIECES {
        let expected = Expected::load(piece, "outerhmm");
        let observations = stream(Grouping::Window { period: 0.001 }, &expected);
        assert_same(&observations, &expected, piece);
    }
}

/// `arzt`: chords within 35 ms, with Matchmaker's 1 ms polling as clock.
#[test]
fn arzt_chords_with_clock() {
    const PERIOD: f64 = 0.001;
    for piece in PIECES {
        let expected = Expected::load(piece, "arzt");
        let mut preprocessor = Preprocessor::new(Grouping::Chord { threshold: 0.035 });
        let mut observations = Vec::new();
        let mut notes = expected.midi_note_ons.iter().peekable();
        let frames = (expected.midi_end_time / PERIOD).ceil() as usize;
        for frame in 0..frames {
            let end = (frame + 1) as f64 * PERIOD;
            let mut emitted = false;
            while let Some(&&(time, pitch)) = notes.peek() {
                if time > end {
                    break;
                }
                notes.next();
                if let Some(observation) = preprocessor.note_on(time, pitch) {
                    observations.push(observation);
                    emitted = true;
                }
            }
            // Matchmaker emits at most one chord per frame.
            if !emitted {
                observations.extend(preprocessor.tick(frame as f64 * PERIOD + 0.5 * PERIOD));
            }
        }
        observations.extend(preprocessor.finish());
        assert_same(&observations, &expected, piece);
    }
}
