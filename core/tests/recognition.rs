#[path = "support/synth.rs"]
mod synth;

use legato_core::following::ScoreFollower;
use legato_core::performance::{Event, Limits, Performances, Setup};
use legato_core::preprocessing::{Grouping, Preprocessor};
use legato_core::recognition::{Decision, Recognition, RecognitionMethod, reference};
use legato_core::score::model::{Candidate, Score, Variation};
use legato_core::score::musicxml_read::read;
use legato_core::works::hanon::HanonRules;
use legato_core::works::{Range, WorkRules};
use synth::Synth;

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

/// Feeds note-ons until a decision; returns it and the tones fed.
fn recognize(
    score: &Score,
    range: Range,
    candidates: &[Candidate],
    notes: &[(f64, u8)],
) -> (Decision, usize) {
    let mut recognition = Recognition::new(
        candidates
            .iter()
            .map(|c| (c.clone(), reference(score, c, range)))
            .collect(),
        HanonRules.recognition_length(),
    );
    let mut preprocessor = Preprocessor::new(Grouping::Window { period: 0.01 });
    let mut tones = 0;
    for &(time, pitch) in notes {
        if let Some(observation) = preprocessor.note_on(time, pitch) {
            tones += observation.pitches.len();
            let decision = recognition.observe(&observation);
            if decision != Decision::Pending {
                return (decision, tones);
            }
        }
    }
    (Decision::Pending, tones)
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

fn find(candidates: &[Candidate], variation: Variation, transposition: i8) -> Candidate {
    candidates
        .iter()
        .find(|c| c.variation == variation && c.transposition == transposition)
        .unwrap()
        .clone()
}

/// FR-018: only the permitted candidates are followed; agogics and pedal change nothing.
#[test]
fn restricted_candidates_are_recognized() {
    let (score, range) = hanon();
    let permitted: Vec<Candidate> = HanonRules
        .candidates("h1")
        .into_iter()
        .filter(|c| [0, 7].contains(&c.transposition))
        .collect();
    assert_eq!(permitted.len(), 6);
    let synth = Synth {
        agogics: 0.3,
        pedal: true,
        ..Synth::new(2.0)
    };
    for (played, candidate) in permitted.iter().enumerate() {
        let notes = synth.play(&reference(&score, candidate, range)).note_ons();
        let (decision, tones) = recognize(&score, range, &permitted, &notes);
        assert_eq!(decision, Decision::Recognized(played), "{candidate:?}");
        let hands = if candidate.variation == Variation::BothHands {
            2
        } else {
            1
        };
        assert!(tones <= 8 * hands, "{candidate:?}: {tones} tones");
    }
}

/// Candidates that cannot be told apart give no performance after the recognition length.
#[test]
fn no_unique_candidate_is_no_performance() {
    let (score, range) = hanon();
    let alike: Vec<Candidate> = ["a", "b", "c"]
        .map(|piece| Candidate {
            piece: piece.into(),
            variation: Variation::BothHands,
            transposition: 0,
        })
        .into();
    let notes = synth::clean(&score, &alike[0], range, 2.0);
    assert_eq!(
        recognize(&score, range, &alike, &notes),
        (Decision::NoPerformance, 2 * HanonRules.recognition_length())
    );
}

/// US 3 scenario 2: another candidate played seamlessly after `Completed` is recognized.
#[test]
fn change_after_completion_is_recognized() {
    let (score, range) = hanon();
    let candidates = HanonRules.candidates("h1");
    let first = find(&candidates, Variation::BothHands, 0);
    let second = find(&candidates, Variation::RightHandOnly, 5);
    let once = synth::clean(&score, &first, range, 2.0);
    // Both hands: the first two tones form the first onset.
    let offset = once.last().unwrap().0 + once[2].0 - once[0].0;
    let notes: Vec<(f64, u8)> = once
        .iter()
        .copied()
        .chain(
            synth::clean(&score, &second, range, 2.0)
                .into_iter()
                .map(|(t, p)| (t + offset, p)),
        )
        .collect();

    let setups = candidates
        .iter()
        .map(|c| Setup::new(&score, c.clone(), range))
        .collect();
    let limits = Limits {
        pause: 4.0,
        rest: 10.0,
    };
    let mut performances = Performances::new(
        setups,
        HanonRules.recognition_length(),
        limits,
        RecognitionMethod::Exhaustive,
    );
    let mut preprocessor = Preprocessor::new(Grouping::Window { period: 0.01 });
    let mut events = Vec::new();
    for (time, pitch) in notes {
        if let Some(observation) = preprocessor.note_on(time, pitch) {
            events.extend(performances.observe(&observation));
        }
    }
    if let Some(observation) = preprocessor.finish() {
        events.extend(performances.observe(&observation));
    }
    let lifecycle: Vec<Event> = events
        .into_iter()
        .filter(|e| {
            matches!(
                e,
                Event::Recognized(_) | Event::Completed | Event::Aborted(_)
            )
        })
        .collect();
    assert_eq!(
        lifecycle,
        [
            Event::Recognized(first),
            Event::Completed,
            Event::Recognized(second),
            Event::Completed
        ]
    );
}

/// The final note ("Ende") is no part of the performance; the next performance may follow it
/// and start on the same pitches.
#[test]
fn next_performance_after_the_final_note_is_recognized() {
    let (score, range) = hanon();
    let candidates = HanonRules.candidates("h1");
    let first = find(&candidates, Variation::BothHands, 0);
    for second in [
        first.clone(),
        find(&candidates, Variation::BothHands, 5),
        find(&candidates, Variation::RightHandOnly, 0),
    ] {
        let once = synth::clean(&score, &first, range, 2.0);
        let step = once[2].0 - once[0].0;
        let end = once.last().unwrap().0 + step;
        let ende: Vec<(f64, u8)> = score.onsets[range.end as usize + 1]
            .notes
            .iter()
            .map(|n| (end, n.pitch))
            .collect();
        // The final note is a half note: eight steps.
        let offset = end + 8.0 * step;
        let notes: Vec<(f64, u8)> = once
            .iter()
            .copied()
            .chain(ende)
            .chain(
                synth::clean(&score, &second, range, 2.0)
                    .into_iter()
                    .map(|(t, p)| (t + offset, p)),
            )
            .collect();

        let setups = candidates
            .iter()
            .map(|c| Setup::new(&score, c.clone(), range))
            .collect();
        let limits = Limits {
            pause: 4.0,
            rest: 10.0,
        };
        let mut performances = Performances::new(
            setups,
            HanonRules.recognition_length(),
            limits,
            RecognitionMethod::Exhaustive,
        );
        let mut preprocessor = Preprocessor::new(Grouping::Window { period: 0.01 });
        let mut events = Vec::new();
        for (time, pitch) in notes {
            events.extend(performances.tick(time));
            if let Some(observation) = preprocessor.note_on(time, pitch) {
                events.extend(performances.observe(&observation));
            }
        }
        if let Some(observation) = preprocessor.finish() {
            events.extend(performances.observe(&observation));
        }
        let deviations = events
            .iter()
            .filter(|e| matches!(e, Event::Deviation(_)))
            .count();
        let lifecycle: Vec<Event> = events
            .into_iter()
            .filter(|e| {
                matches!(
                    e,
                    Event::Recognized(_) | Event::Completed | Event::Aborted(_)
                )
            })
            .collect();
        let case = format!("{:?} {}", second.variation, second.transposition);
        assert_eq!(
            lifecycle,
            [
                Event::Recognized(first.clone()),
                Event::Completed,
                Event::Recognized(second.clone()),
                Event::Completed
            ],
            "{case}"
        );
        assert_eq!(deviations, 0, "{case}");
    }
}
