#[path = "support/synth.rs"]
mod synth;

use legato_core::following::deviations::DeviationKind;
use legato_core::performance::{Event, Limits, Performances, Setup};
use legato_core::preprocessing::{Grouping, Preprocessor};
use legato_core::recognition::reference;
use legato_core::score::model::{Candidate, Score, Variation};
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

const PLAYED: Candidate = Candidate {
    piece: String::new(),
    variation: Variation::RightHandOnly,
    transposition: 4,
};

/// Deviations `(kind, expected pitch, played pitch)` of a right-hand performance in E major.
fn deviations(
    change: impl Fn(&mut Vec<(f64, u8)>),
) -> Vec<(DeviationKind, Option<u8>, Option<u8>)> {
    let (score, range) = hanon();
    let setups = HanonRules
        .candidates("")
        .into_iter()
        .map(|c| Setup::new(&score, c, range))
        .collect();
    let limits = Limits {
        pause: 4.0,
        rest: 10.0,
    };
    let mut performances = Performances::new(setups, HanonRules.recognition_length(), limits);
    let mut notes = synth::clean(&score, &PLAYED, range, 2.0);
    change(&mut notes);
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
    assert_eq!(events.last(), Some(&Event::Completed));
    events
        .into_iter()
        .filter_map(|e| match e {
            Event::Deviation(d) => Some((d.kind, d.note.map(|n| n.pitch + 4), d.played)),
            _ => None,
        })
        .collect()
}

fn pitch(index: usize) -> u8 {
    let (score, range) = hanon();
    reference(&score, &PLAYED, range)[index].pitch
}

#[test]
fn clean_performance_has_no_deviations() {
    assert_eq!(deviations(|_| {}), []);
}

#[test]
fn left_out_note_is_an_omission() {
    let left_out = pitch(60);
    assert_eq!(
        deviations(|notes| {
            notes.remove(60);
        }),
        [(DeviationKind::Omission, Some(left_out), None)]
    );
}

#[test]
fn neighbouring_tone_is_a_wrong_note() {
    let expected = pitch(60);
    assert_eq!(
        deviations(|notes| notes[60].1 += 1),
        [(
            DeviationKind::Substitution,
            Some(expected),
            Some(expected + 1)
        )]
    );
}

#[test]
fn distant_extra_tone_is_an_addition() {
    let extra = pitch(60) + 19;
    assert_eq!(
        deviations(|notes| {
            let time = (notes[60].0 + notes[61].0) / 2.0;
            notes.insert(61, (time, extra));
        }),
        [(DeviationKind::Addition, None, Some(extra))]
    );
}
