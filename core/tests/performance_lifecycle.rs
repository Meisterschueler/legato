#[path = "support/synth.rs"]
mod synth;

use legato_core::performance::{AbortReason, Event, Limits, Performances, Setup};
use legato_core::preprocessing::{Grouping, Preprocessor};
use legato_core::score::model::{Candidate, Score, Variation};
use legato_core::score::musicxml_read::read;
use legato_core::works::hanon::HanonRules;
use legato_core::works::{Range, WorkRules};

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
