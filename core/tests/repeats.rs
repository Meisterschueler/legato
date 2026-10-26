#[path = "support/synth.rs"]
mod synth;

use legato_core::following::arzt::ArztEvent;
use legato_core::following::outer_hmm::OuterProductHmm;
use legato_core::following::pthmm::PitchHmm;
use legato_core::following::repeats::Repeats;
use legato_core::following::{ReferenceNote, ScoreFollower};
use legato_core::performance::{Event, Limits, Performances, Setup};
use legato_core::preprocessing::{Grouping, Preprocessor};
use legato_core::recognition::reference;
use legato_core::score::model::{Candidate, Score, Variation};
use legato_core::score::musicxml_read::read;
use legato_core::works::generic::GenericRules;
use legato_core::works::hanon::HanonRules;
use legato_core::works::{Range, WorkRules};
use synth::Synth;

const PITCHES: [u8; 8] = [60, 62, 64, 65, 67, 69, 71, 72];
/// Onsets 0–5 form a repeat section, 6 and 7 follow.
const SECTION_END: usize = 5;

fn notes() -> Vec<ReferenceNote> {
    PITCHES
        .iter()
        .zip(0..)
        .map(|(&pitch, beat)| ReferenceNote {
            onset_beat: f64::from(beat),
            pitch,
        })
        .collect()
}

/// The section three times, then the rest.
fn played() -> Vec<usize> {
    let mut played: Vec<usize> = (0..3).flat_map(|_| 0..=SECTION_END).collect();
    played.extend(SECTION_END + 1..PITCHES.len());
    played
}

/// Positions (beats) after every played onset.
fn positions(follower: &mut dyn ScoreFollower) -> Vec<f64> {
    played()
        .into_iter()
        .zip(0..)
        .map(|(onset, i)| {
            follower.step(&[PITCHES[onset]], 0.5 * f64::from(i));
            follower.score_positions()[follower.current_index()]
        })
        .collect()
}

#[test]
fn ahead_splits_at_the_end_of_a_section() {
    let repeats = Repeats::new(vec![(3, 1)]);
    assert_eq!(repeats.ahead(2, 2, 6), vec![(4, 0.5), (1, 0.5)]);
    assert_eq!(repeats.ahead(0, 2, 6), vec![(2, 1.0)]);
    // At the end of the reference the jump is the only way.
    assert_eq!(repeats.ahead(3, 1, 4), vec![(1, 1.0)]);
    assert_eq!(repeats.successors(3, 6).collect::<Vec<_>>(), vec![4, 1]);
}

/// A method with repeats and, where it cannot follow them, the same without.
type Followers = (
    &'static str,
    Box<dyn ScoreFollower>,
    Option<Box<dyn ScoreFollower>>,
);

/// FR-014: every method follows any number of repetitions, without unrolling them.
#[test]
fn every_method_follows_the_repetitions() {
    let expected: Vec<f64> = played().into_iter().map(|o| o as f64).collect();
    let repeats = Repeats::new(vec![(SECTION_END, 0)]);
    let notes = notes();
    // `outerhmm` reaches the start without repeats as well, by its global skip.
    let followers: Vec<Followers> = vec![
        (
            "pthmm",
            Box::new(PitchHmm::with_repeats(&notes, true, &repeats)),
            Some(Box::new(PitchHmm::new(&notes, true))),
        ),
        (
            "outerhmm",
            Box::new(OuterProductHmm::with_repeats(&notes, &repeats)),
            None,
        ),
        (
            "arzt",
            Box::new(ArztEvent::with_repeats(&notes, repeats.clone())),
            Some(Box::new(ArztEvent::new(&notes))),
        ),
    ];
    for (name, mut with, without) in followers {
        assert_eq!(positions(with.as_mut()), expected, "{name}");
        if let Some(mut without) = without {
            assert_ne!(
                positions(without.as_mut()),
                expected,
                "{name} without repeats"
            );
        }
    }
}

#[test]
fn without_repeats_the_transitions_are_matchmakers() {
    let notes = notes();
    let none = Repeats::default();
    assert_eq!(
        PitchHmm::with_repeats(&notes, true, &none).transition_matrix(),
        PitchHmm::new(&notes, true).transition_matrix()
    );
    assert_eq!(
        OuterProductHmm::with_repeats(&notes, &none).transition_matrix(),
        OuterProductHmm::new(&notes).transition_matrix()
    );
}

fn hanon() -> Score {
    let file = std::fs::read(format!(
        "{}/../app/assets/pieces/hanon-01.musicxml",
        env!("CARGO_MANIFEST_DIR")
    ))
    .unwrap();
    read(&file).unwrap().score
}

fn beat(score: &Score, onset: u32) -> f64 {
    let time = score.onsets[onset as usize].time;
    time.numerator as f64 / time.denominator as f64
}

/// Hanon's repeat section is one performance (work rules); a repeat at the end of a performance
/// is no edge.
#[test]
fn a_repeat_at_the_end_of_the_performance_is_no_edge() {
    let score = hanon();
    let range = HanonRules.performance_range(&score);
    let candidate = &HanonRules.candidates("h1")[0];
    assert!(
        Setup::new(&score, candidate.clone(), range)
            .repeats
            .is_empty()
    );
}

/// Under the general rules the whole piece is one performance: the section is played twice,
/// then the final chord, without deviations.
#[test]
fn a_performance_goes_through_the_repeat_section() {
    let mut score = hanon();
    // Hanon ends on the first chord of the section: after the section either could follow, and
    // reaching the final chord completes the performance. A third higher it is unambiguous.
    let last = score.onsets.last_mut().unwrap();
    last.notes.iter_mut().for_each(|n| n.pitch += 4);
    let section = HanonRules.performance_range(&score);
    let range = GenericRules.performance_range(&score);
    assert!(section.end < range.end, "a final note follows the section");
    let candidate = Candidate {
        piece: "h1".into(),
        variation: Variation::BothHands,
        transposition: 0,
    };
    let setups: Vec<Setup> = GenericRules
        .candidates("h1")
        .into_iter()
        .map(|c| Setup::new(&score, c, range))
        .collect();
    assert_eq!(setups[0].repeats.jumps(), [(section.end as usize, 0)]);

    let length = beat(&score, section.end + 1) - beat(&score, section.start);
    let pass = reference(&score, &candidate, section);
    let rest = reference(
        &score,
        &candidate,
        Range {
            start: section.end + 1,
            end: range.end,
        },
    );
    let shifted = |notes: &[ReferenceNote], by: f64| -> Vec<ReferenceNote> {
        notes
            .iter()
            .map(|n| ReferenceNote {
                onset_beat: n.onset_beat + by,
                ..*n
            })
            .collect()
    };
    let notes: Vec<ReferenceNote> = [
        shifted(&pass, 0.0),
        shifted(&pass, length),
        shifted(&rest, length),
    ]
    .concat();

    let mut performances = Performances::new(
        setups,
        GenericRules.recognition_length(),
        Limits {
            pause: 4.0,
            rest: 10.0,
        },
    );
    let mut preprocessor = Preprocessor::new(Grouping::Window { period: 0.01 });
    let mut events = Vec::new();
    for (time, pitch) in Synth::new(2.0).play(&notes).note_ons() {
        events.extend(performances.tick(time));
        if let Some(observation) = preprocessor.note_on(time, pitch) {
            events.extend(performances.observe(&observation));
        }
    }
    if let Some(observation) = preprocessor.finish() {
        events.extend(performances.observe(&observation));
    }

    assert_eq!(events.first(), Some(&Event::Recognized(candidate)));
    let deviations = events
        .iter()
        .filter(|e| matches!(e, Event::Deviation(_)))
        .count();
    assert_eq!(deviations, 0);
    let ends: Vec<&Event> = events
        .iter()
        .filter(|e| matches!(e, Event::Completed | Event::Aborted(_)))
        .collect();
    assert_eq!(ends, [&Event::Completed]);
    assert_eq!(events.last(), Some(&Event::Completed));
    let at_end = Event::Position(section.end);
    assert_eq!(events.iter().filter(|e| **e == at_end).count(), 2);
}
