#[path = "support/synth.rs"]
mod synth;

use legato_core::performance::{AbortReason, Event, Limits, Performances, Setup};
use legato_core::preprocessing::{Grouping, Preprocessor};
use legato_core::recognition::reference;
use legato_core::score::model::{Candidate, Hand, Score, Variation};
use legato_core::score::musicxml_read::read;
use legato_core::works::hanon::HanonRules;
use legato_core::works::{Range, WorkRules};
use synth::Synth;

const LIMITS: Limits = Limits {
    pause: 4.0,
    rest: 10.0,
};

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

fn performances(score: &Score, range: Range) -> Performances {
    let setups = HanonRules
        .candidates("h1")
        .into_iter()
        .map(|c| Setup::new(score, c, range))
        .collect();
    Performances::new(setups, HanonRules.recognition_length(), LIMITS)
}

fn candidate(variation: Variation, transposition: i8) -> Candidate {
    Candidate {
        piece: "h1".into(),
        variation,
        transposition,
    }
}

/// Feeds note-ons `(seconds, pitch)` with a clock tick before each.
fn play(performances: &mut Performances, notes: &[(f64, u8)]) -> Vec<Event> {
    let mut preprocessor = Preprocessor::new(Grouping::Window { period: 0.01 });
    let mut events = Vec::new();
    for &(time, pitch) in notes {
        events.extend(performances.tick(time));
        if let Some(observation) = preprocessor.note_on(time, pitch) {
            events.extend(performances.observe(&observation));
        }
    }
    if let Some(observation) = preprocessor.finish() {
        events.extend(performances.observe(&observation));
    }
    events
}

fn count(events: &[Event], wanted: &Event) -> usize {
    events.iter().filter(|e| *e == wanted).count()
}

/// Events of a synthetic performance of `played`, fed into fresh performances.
fn events_of(score: &Score, range: Range, played: &Candidate, synth: &Synth) -> Vec<Event> {
    let notes = synth.play(&reference(score, played, range)).note_ons();
    play(&mut performances(score, range), &notes)
}

fn has_deviations(events: &[Event]) -> bool {
    events.iter().any(|e| matches!(e, Event::Deviation(_)))
}

/// Events in short form for failure messages.
fn summary(events: &[Event]) -> String {
    events
        .iter()
        .map(|e| match e {
            Event::Recognized(c) => format!("R({:?} {})", c.variation, c.transposition),
            Event::Position(i) => format!("{i}"),
            Event::Deviation(d) => format!("{:?}", d.kind),
            other => format!("{other:?}"),
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// FR-017: the same result at 0.5- to 3-fold tempo.
#[test]
fn clean_performance_is_followed_to_the_end_at_any_tempo() {
    let (score, range) = hanon();
    let played = candidate(Variation::BothHands, 3);
    for beats_per_second in [1.0, 2.0, 6.0] {
        let mut performances = performances(&score, range);
        let notes = synth::clean(&score, &played, range, beats_per_second);
        let events = play(&mut performances, &notes);

        assert_eq!(events[0], Event::Recognized(played.clone()));
        let positions: Vec<u32> = events
            .iter()
            .filter_map(|e| match e {
                Event::Position(i) => Some(*i),
                _ => None,
            })
            .collect();
        assert!(positions.is_sorted(), "{beats_per_second}: {positions:?}");
        assert_eq!(positions.last(), Some(&range.end));
        assert_eq!(events.last(), Some(&Event::Completed));
        assert_eq!(count(&events, &Event::Completed), 1);
    }
}

/// US 3 scenario 2: a seamless repetition is a new performance.
#[test]
fn seamless_repetition_is_recognized_again() {
    let (score, range) = hanon();
    let played = candidate(Variation::RightHandOnly, 0);
    let once = synth::clean(&score, &played, range, 2.0);
    let step = once[1].0 - once[0].0;
    let offset = once.last().unwrap().0 + step;
    let twice: Vec<(f64, u8)> = once
        .iter()
        .copied()
        .chain(once.iter().map(|&(t, p)| (t + offset, p)))
        .collect();

    let events = play(&mut performances(&score, range), &twice);
    assert_eq!(count(&events, &Event::Recognized(played)), 2);
    assert_eq!(count(&events, &Event::Completed), 2);
}

#[test]
fn silence_longer_than_the_pause_limit_aborts() {
    let (score, range) = hanon();
    let played = candidate(Variation::LeftHandOnly, 5);
    let notes = synth::clean(&score, &played, range, 2.0);
    let mut performances = performances(&score, range);
    let events = play(&mut performances, &notes[..40]);
    assert_eq!(events[0], Event::Recognized(played));

    let last = notes[39].0;
    assert!(performances.tick(last + LIMITS.pause - 0.5).is_empty());
    assert_eq!(
        performances.tick(last + LIMITS.pause + 0.5),
        [Event::Aborted(AbortReason::PauseLimit)]
    );
}

/// FR-021: playing something else seamlessly loses the alignment.
#[test]
fn playing_something_else_aborts_with_alignment_loss() {
    let (score, range) = hanon();
    let played = candidate(Variation::BothHands, 0);
    let notes = synth::clean(&score, &played, range, 2.0);
    let elsewhere: Vec<(f64, u8)> = notes[..40]
        .iter()
        .copied()
        .chain(notes[40..120].iter().map(|&(t, p)| (t, p + 6)))
        .collect();

    let events = play(&mut performances(&score, range), &elsewhere);
    assert_eq!(events[0], Event::Recognized(played));
    assert_eq!(
        count(&events, &Event::Aborted(AbortReason::AlignmentLoss)),
        1,
        "{events:?}"
    );
}

#[test]
fn interruption_aborts_a_running_performance() {
    let (score, range) = hanon();
    let notes = synth::clean(&score, &candidate(Variation::BothHands, 9), range, 2.0);
    let mut performances = performances(&score, range);
    play(&mut performances, &notes[..40]);
    assert_eq!(
        performances.interrupt(),
        [Event::Aborted(AbortReason::Interrupted)]
    );
    assert!(performances.interrupt().is_empty());
}

/// FR-020: before an onset after a notated rest the performance waits up to the rest limit.
#[test]
fn notated_rest_waits_until_the_rest_limit() {
    let (mut score, range) = hanon();
    let played = candidate(Variation::LeftHandOnly, 0);
    let onsets = Setup::new(&score, played.clone(), range).onsets;
    score.rests_before.insert(onsets[40].index);
    let notes = synth::clean(&score, &played, range, 2.0);
    let mut performances = performances(&score, range);
    let events = play(&mut performances, &notes[..40]);
    let position = events.iter().rev().find_map(|e| match e {
        Event::Position(i) => Some(*i),
        _ => None,
    });
    assert_eq!(position, Some(onsets[39].index));

    let last = notes[39].0;
    assert!(performances.tick(last + LIMITS.pause + 0.5).is_empty());
    assert!(performances.tick(last + LIMITS.rest - 0.5).is_empty());
    assert_eq!(
        performances.tick(last + LIMITS.rest + 0.5),
        [Event::Aborted(AbortReason::PauseLimit)]
    );
}

/// FR-017, FR-003: tempo (0.5- to 3-fold), agogics and pedal leave the result unchanged.
#[test]
fn tempo_agogics_and_pedal_change_nothing() {
    let (score, range) = hanon();
    let played = candidate(Variation::RightHandOnly, 2);
    let errors = Synth {
        omissions: 0.03,
        substitutions: 0.03,
        additions: 0.03,
        seed: 11,
        ..Synth::new(2.0)
    };
    let expected = events_of(&score, range, &played, &errors);
    assert_eq!(expected[0], Event::Recognized(played.clone()));
    assert_eq!(expected.last(), Some(&Event::Completed), "{expected:?}");
    assert!(has_deviations(&expected));
    for synth in [
        Synth {
            beats_per_second: 1.0,
            ..errors.clone()
        },
        Synth {
            beats_per_second: 6.0,
            ..errors.clone()
        },
        Synth {
            agogics: 0.3,
            ..errors.clone()
        },
        Synth {
            pedal: true,
            ..errors.clone()
        },
    ] {
        assert_eq!(
            events_of(&score, range, &played, &synth),
            expected,
            "{synth:?}"
        );
    }
}

/// A short change of key gives deviations but keeps the candidate.
#[test]
fn short_key_change_keeps_the_candidate() {
    let (score, range) = hanon();
    let played = candidate(Variation::BothHands, 0);
    let synth = Synth {
        key_change: Some((64..72, 1)),
        ..Synth::new(2.0)
    };
    let events = events_of(&score, range, &played, &synth);
    let recognized = events
        .iter()
        .filter(|e| matches!(e, Event::Recognized(_)))
        .count();
    assert_eq!(recognized, 1, "{}", summary(&events));
    assert_eq!(events[0], Event::Recognized(played));
    assert!(has_deviations(&events));
    assert!(
        !events.iter().any(|e| matches!(e, Event::Aborted(_))),
        "{}",
        summary(&events)
    );
    assert_eq!(events.last(), Some(&Event::Completed));
}

/// FR-019: playing with one hand leaves out no notes of the other hand.
#[test]
fn one_hand_is_judged_by_its_own_notes() {
    let (score, range) = hanon();
    for (variation, hand) in [
        (Variation::LeftHandOnly, Hand::Left),
        (Variation::RightHandOnly, Hand::Right),
    ] {
        let played = candidate(variation, 9);
        let clean = events_of(&score, range, &played, &Synth::new(2.0));
        assert!(!has_deviations(&clean), "{variation:?}: {clean:?}");
        assert_eq!(clean.last(), Some(&Event::Completed), "{variation:?}");

        let errors = Synth {
            omissions: 0.05,
            substitutions: 0.05,
            additions: 0.05,
            ..Synth::new(2.0)
        };
        let hands: Vec<Hand> = events_of(&score, range, &played, &errors)
            .iter()
            .filter_map(|e| match e {
                Event::Deviation(d) => Some(d.hand),
                _ => None,
            })
            .collect();
        assert!(!hands.is_empty(), "{variation:?}");
        assert!(hands.iter().all(|h| *h == hand), "{variation:?}: {hands:?}");
    }
}
