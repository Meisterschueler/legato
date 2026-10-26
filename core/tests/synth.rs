//! Checks of the generator for synthetic performances (T072).

#[path = "support/synth.rs"]
mod synth;

use legato_core::following::ReferenceNote;
use legato_core::recognition::reference;
use legato_core::score::model::{Candidate, Variation};
use legato_core::score::musicxml_read::read;
use legato_core::works::WorkRules;
use legato_core::works::hanon::HanonRules;
use synth::{Performance, SUBSTITUTION_RANGE, Synth, Truth};

fn hanon(variation: Variation) -> Vec<ReferenceNote> {
    let file = std::fs::read(format!(
        "{}/../app/assets/pieces/hanon-01.musicxml",
        env!("CARGO_MANIFEST_DIR")
    ))
    .unwrap();
    let score = read(&file).unwrap().score;
    let range = HanonRules.performance_range(&score);
    let candidate = Candidate {
        piece: "h1".into(),
        variation,
        transposition: 2,
    };
    reference(&score, &candidate, range)
}

/// Every note of every run is played, replaced or left out exactly once; messages are sorted.
fn check(reference: &[ReferenceNote], synth: &Synth, performance: &Performance) {
    let mut seen = vec![vec![0; reference.len()]; synth.runs];
    for m in &performance.messages {
        let pitch = m.data[1];
        match m.truth {
            Truth::Note(i) => {
                assert_eq!(pitch, reference[i].pitch);
                seen[m.run][i] += 1;
            }
            Truth::Substitution(i) => {
                assert!((1..=SUBSTITUTION_RANGE).contains(&pitch.abs_diff(reference[i].pitch)));
                seen[m.run][i] += 1;
            }
            Truth::Addition | Truth::Pedal => {}
        }
    }
    for &(run, i) in &performance.omissions {
        seen[run][i] += 1;
    }
    assert!(seen.iter().flatten().all(|&n| n == 1), "{seen:?}");
    assert!(performance.messages.is_sorted_by(|a, b| a.time <= b.time));
}

#[test]
fn clean_performance_plays_every_note_in_tempo() {
    let reference = hanon(Variation::BothHands);
    let synth = Synth::new(2.0);
    let performance = synth.play(&reference);
    check(&reference, &synth, &performance);

    let expected: Vec<(usize, Truth)> = (0..reference.len()).map(|i| (0, Truth::Note(i))).collect();
    assert_eq!(performance.alignment(), expected);
    let note_ons = performance.note_ons();
    for (&(time, pitch), note) in note_ons.iter().zip(&reference) {
        assert_eq!(pitch, note.pitch);
        assert_eq!(time, (note.onset_beat - reference[0].onset_beat) / 2.0);
    }
}

#[test]
fn scattered_errors_pedal_and_repetitions_are_known() {
    let reference = hanon(Variation::RightHandOnly);
    let synth = Synth {
        agogics: 0.3,
        spread: 0.005,
        omissions: 0.03,
        substitutions: 0.03,
        additions: 0.03,
        pauses: vec![(100, 2.0)],
        pedal: true,
        runs: 3,
        seed: 7,
        ..Synth::new(3.0)
    };
    let performance = synth.play(&reference);
    check(&reference, &synth, &performance);
    assert_eq!(synth.play(&reference), performance);

    let alignment = performance.alignment();
    assert_eq!(alignment.len(), performance.note_ons().len());
    let count = |wanted: fn(&Truth) -> bool| alignment.iter().filter(|(_, t)| wanted(t)).count();
    assert!(count(|t| matches!(t, Truth::Substitution(_))) > 0);
    assert!(count(|t| *t == Truth::Addition) > 0);
    assert!(!performance.omissions.is_empty());
    assert!(performance.messages.iter().any(|m| m.truth == Truth::Pedal));
    for run in 0..synth.runs {
        let first: Vec<Truth> = alignment
            .iter()
            .filter(|(r, _)| *r == run)
            .map(|(_, t)| *t)
            .take(synth.clean_start)
            .collect();
        let clean: Vec<Truth> = (0..synth.clean_start).map(Truth::Note).collect();
        assert_eq!(first, clean, "run {run}");
    }
}

#[test]
fn key_change_replaces_near_notes_and_adds_far_ones() {
    let reference = hanon(Variation::BothHands);
    for (shift, replaced) in [(1, true), (-2, true), (5, false)] {
        let synth = Synth {
            omissions: 0.05,
            substitutions: 0.05,
            additions: 0.05,
            key_change: Some((40..56, shift)),
            ..Synth::new(2.0)
        };
        let performance = synth.play(&reference);
        check(&reference, &synth, &performance);
        let alignment = performance.alignment();
        for i in 36..60 {
            let substituted = alignment.contains(&(0, Truth::Substitution(i)));
            let omitted = performance.omissions.contains(&(0, i));
            let changed = (40..56).contains(&i);
            assert_eq!(substituted, changed && replaced, "{shift}: {i}");
            assert_eq!(omitted, changed && !replaced, "{shift}: {i}");
        }
    }
}
