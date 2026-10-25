#[path = "support/synth.rs"]
mod synth;

use legato_core::following::ScoreFollower;
use legato_core::preprocessing::{Grouping, Preprocessor};
use legato_core::recognition::{Decision, Recognition, reference};
use legato_core::score::model::{Score, Variation};
use legato_core::score::musicxml_read::read;
use legato_core::works::hanon::HanonRules;
use legato_core::works::{Range, WorkRules};

fn hanon() -> (Score, Range) {
    let file = std::fs::read(format!(
        "{}/../app/assets/pieces/hanon-01.musicxml",
        env!("CARGO_MANIFEST_DIR")
    ))
    .unwrap();
    let score = read(&file).unwrap().score;
    let range = HanonRules.performance_range(&score);
    (score, range)
}

#[test]
fn reference_takes_the_hands_and_the_key_of_the_candidate() {
    let (score, range) = hanon();
    let candidates = HanonRules.candidates("h1");
    let first = |variation, transposition| {
        let c = candidates
            .iter()
            .find(|c| c.variation == variation && c.transposition == transposition)
            .unwrap();
        let notes = reference(&score, c, range);
        let mut pitches: Vec<u8> = notes
            .iter()
            .take_while(|n| n.onset_beat == notes[0].onset_beat)
            .map(|n| n.pitch)
            .collect();
        pitches.sort();
        pitches
    };
    assert_eq!(first(Variation::BothHands, 0), [36, 48]);
    assert_eq!(first(Variation::LeftHandOnly, 2), [38]);
    assert_eq!(first(Variation::RightHandOnly, 11), [59]);
}

/// SC-002; tones before the decision belong to the performance.
#[test]
fn candidates_are_recognized_within_the_recognition_length() {
    let (score, range) = hanon();
    let candidates = HanonRules.candidates("h1");
    for (variation, transposition) in [
        (Variation::BothHands, 0),
        (Variation::BothHands, 7),
        (Variation::RightHandOnly, 0),
        (Variation::LeftHandOnly, 0),
        (Variation::RightHandOnly, 11),
        (Variation::LeftHandOnly, 6),
    ] {
        let played = candidates
            .iter()
            .position(|c| c.variation == variation && c.transposition == transposition)
            .unwrap();
        let mut recognition = Recognition::new(
            candidates
                .iter()
                .map(|c| (c.clone(), reference(&score, c, range)))
                .collect(),
            HanonRules.recognition_length(),
        );
        let mut preprocessor = Preprocessor::new(Grouping::Window { period: 0.01 });
        let (mut decision, mut tones, mut last_time) = (Decision::Pending, 0, 0.0);
        for (time, pitch) in synth::clean(&score, &candidates[played], range, 2.0) {
            if let Some(observation) = preprocessor.note_on(time, pitch) {
                tones += observation.pitches.len();
                last_time = observation.time;
                decision = recognition.observe(&observation);
                if decision != Decision::Pending {
                    break;
                }
            }
        }
        let case = format!("{variation:?} {transposition}");
        assert_eq!(decision, Decision::Recognized(played), "{case}");
        let hands = if variation == Variation::BothHands {
            2
        } else {
            1
        };
        assert!(tones <= 8 * hands, "{case}: {tones} tones");

        let (candidate, follower) = recognition.into_follower(played);
        assert_eq!(candidate, candidates[played]);
        let position = follower.score_positions()[follower.current_index()];
        assert_eq!(position, last_time * 2.0, "{case}");
    }
}
