//! The methods of the recognition (FR-085, research.md R15): SC-002 with Hanon Nr. 1 and
//! SC-014 with Hanon Nr. 1–20.

#[path = "support/synth.rs"]
mod synth;

use std::sync::Arc;

use legato_core::performance::Setup;
use legato_core::preprocessing::{Grouping, Preprocessor};
use legato_core::recognition::edit_distance::{distance, substitution};
use legato_core::recognition::fingerprint::{Hash, triples};
use legato_core::recognition::{
    Decision, Onset, Recognition, RecognitionMethod, Repertoire, reference,
};
use legato_core::score::model::{Candidate, Score, Variation};
use legato_core::score::musicxml_read::read;
use legato_core::works::hanon::HanonRules;
use legato_core::works::{Range, WorkRules};
use synth::Synth;

const METHODS: [RecognitionMethod; 3] = [
    RecognitionMethod::Exhaustive,
    RecognitionMethod::EditDistance,
    RecognitionMethod::Fingerprint,
];
const VARIATIONS: [Variation; 3] = [
    Variation::BothHands,
    Variation::LeftHandOnly,
    Variation::RightHandOnly,
];
/// Quarter notes per second: 0.5- to 3-fold of 2.
const TEMPOS: [f64; 4] = [1.0, 2.0, 4.0, 6.0];

struct Piece {
    id: String,
    score: Score,
    range: Range,
}

fn hanon(number: u32) -> Piece {
    let file = std::fs::read(format!(
        "{}/../testdata/legato/hanon/hanon-{number:02}.musicxml",
        env!("CARGO_MANIFEST_DIR")
    ))
    .unwrap();
    let score = read(&file).unwrap().score;
    let range = HanonRules.performance_range(&score);
    Piece {
        id: format!("h{number}"),
        score,
        range,
    }
}

fn setups(pieces: &[Piece], permitted: impl Fn(&Candidate) -> bool) -> Vec<Setup> {
    pieces
        .iter()
        .flat_map(|p| {
            HanonRules
                .candidates(&p.id)
                .into_iter()
                .filter(&permitted)
                .map(|c| Setup::new(&p.score, c, p.range))
        })
        .collect()
}

fn repertoire(setups: &[Setup], method: RecognitionMethod) -> Arc<Repertoire> {
    Arc::new(Repertoire::for_setups(
        setups,
        HanonRules.recognition_length(),
        method,
    ))
}

/// Errors after the recognition, agogics, spread chords and pedal.
fn synth(beats_per_second: f64, seed: u64) -> Synth {
    Synth {
        agogics: 0.1,
        spread: 0.005,
        omissions: 0.05,
        substitutions: 0.05,
        additions: 0.05,
        pedal: true,
        seed,
        ..Synth::new(beats_per_second)
    }
}

/// Feeds note-ons until a decision; returns it, the tones fed and the followed candidates after
/// every observation.
fn recognize(
    repertoire: &Arc<Repertoire>,
    notes: &[(f64, u8)],
) -> (Decision, usize, Vec<Vec<usize>>) {
    let mut recognition = Recognition::start(Arc::clone(repertoire));
    let mut preprocessor = Preprocessor::new(Grouping::Window { period: 0.01 });
    let (mut tones, mut followed) = (0, Vec::new());
    let mut observations: Vec<_> = notes
        .iter()
        .filter_map(|&(time, pitch)| preprocessor.note_on(time, pitch))
        .collect();
    observations.extend(preprocessor.finish());
    for observation in observations {
        tones += observation.pitches.len();
        let decision = recognition.observe(&observation);
        followed.push(recognition.followed());
        if decision != Decision::Pending {
            return (decision, tones, followed);
        }
    }
    (Decision::Pending, tones, followed)
}

fn hands(variation: Variation) -> usize {
    if variation == Variation::BothHands {
        2
    } else {
        1
    }
}

/// Plays `candidate` of `piece` and checks that it is recognized within the recognition length.
fn check(repertoire: &Arc<Repertoire>, piece: &Piece, candidate: &Candidate, synth: &Synth) {
    let notes = synth
        .play(&reference(&piece.score, candidate, piece.range))
        .note_ons();
    let (decision, tones, _) = recognize(repertoire, &notes);
    let expected = repertoire
        .candidates()
        .iter()
        .position(|c| c == candidate)
        .unwrap();
    let case = format!(
        "{:?} {} {:?} {} at {}",
        repertoire.method(),
        candidate.piece,
        candidate.variation,
        candidate.transposition,
        synth.beats_per_second
    );
    assert_eq!(decision, Decision::Recognized(expected), "{case}");
    let length = HanonRules.recognition_length() * hands(candidate.variation);
    assert!(tones <= length, "{case}: {tones} tones");
}

/// SC-002: every key and variation of Hanon Nr. 1 at 0.5- to 3-fold tempo.
#[test]
fn every_method_recognizes_hanon_1_in_every_key_and_variation() {
    let piece = hanon(1);
    let setups = setups(std::slice::from_ref(&piece), |_| true);
    for method in METHODS {
        let repertoire = repertoire(&setups, method);
        for (i, setup) in setups.iter().enumerate() {
            let synth = synth(TEMPOS[i % TEMPOS.len()], i as u64);
            check(&repertoire, &piece, &setup.candidate, &synth);
        }
    }
}

/// FR-018: only permitted candidates are followed and recognized.
#[test]
fn restriction_is_respected() {
    let piece = hanon(1);
    let setups = setups(std::slice::from_ref(&piece), |c| {
        [0, 7].contains(&c.transposition)
    });
    for method in METHODS {
        let repertoire = repertoire(&setups, method);
        for (i, setup) in setups.iter().enumerate() {
            check(&repertoire, &piece, &setup.candidate, &synth(2.0, i as u64));
        }
    }
}

/// Without a fitting candidate no performance: a key that is not permitted. Only with a
/// preselection: `Exhaustive` has no minimum quality (research.md R8) and takes the closest key.
#[test]
fn without_fitting_candidate_no_performance() {
    let piece = hanon(1);
    let setups = setups(std::slice::from_ref(&piece), |c| {
        [0, 7].contains(&c.transposition)
    });
    let played = Candidate {
        piece: piece.id.clone(),
        variation: Variation::BothHands,
        transposition: 2,
    };
    let notes = synth(2.0, 1)
        .play(&reference(&piece.score, &played, piece.range))
        .note_ons();
    for method in METHODS
        .into_iter()
        .filter(|m| *m != RecognitionMethod::Exhaustive)
    {
        let (decision, _, _) = recognize(&repertoire(&setups, method), &notes);
        assert_eq!(decision, Decision::NoPerformance, "{method:?}");
    }
}

/// Constitution VI, VIII: the same input gives the same preselection.
#[test]
fn same_input_same_preselection() {
    let piece = hanon(1);
    let setups = setups(std::slice::from_ref(&piece), |_| true);
    let played = &setups[17].candidate;
    let notes = synth(2.0, 3)
        .play(&reference(&piece.score, played, piece.range))
        .note_ons();
    for method in METHODS {
        let first = recognize(&repertoire(&setups, method), &notes);
        let second = recognize(&repertoire(&setups, method), &notes);
        assert_eq!(first, second, "{method:?}");
    }
}

/// SC-014: Hanon Nr. 1–20 in the repertoire (720 candidates); one candidate per piece.
fn recognizes_hanon_1_to_20(method: RecognitionMethod) {
    let pieces: Vec<Piece> = (1..=20).map(hanon).collect();
    let setups = setups(&pieces, |_| true);
    assert_eq!(setups.len(), 720);
    let repertoire = repertoire(&setups, method);
    for (i, piece) in pieces.iter().enumerate() {
        let candidate = Candidate {
            piece: piece.id.clone(),
            variation: VARIATIONS[i % VARIATIONS.len()],
            transposition: (5 * i % 12) as i8,
        };
        let synth = synth(TEMPOS[i % TEMPOS.len()], i as u64);
        check(&repertoire, piece, &candidate, &synth);
    }
}

#[test]
fn edit_distance_recognizes_hanon_1_to_20() {
    recognizes_hanon_1_to_20(RecognitionMethod::EditDistance);
}

#[test]
fn fingerprint_recognizes_hanon_1_to_20() {
    recognizes_hanon_1_to_20(RecognitionMethod::Fingerprint);
}

/// Informative only (research.md R15): 720 dense `pthmm`.
#[test]
#[ignore = "slow: 720 dense pthmm"]
fn exhaustive_recognizes_hanon_1_to_20() {
    recognizes_hanon_1_to_20(RecognitionMethod::Exhaustive);
}

fn onset(time: f64, pitches: &[u8]) -> Onset {
    Onset {
        time,
        pitches: pitches.to_vec(),
    }
}

/// A wrong tone costs 1, a missing tone of a chord also.
#[test]
fn edit_distance_costs() {
    assert_eq!(substitution(&[62], &[60]), 1);
    assert_eq!(substitution(&[60], &[48, 60]), 1);
    assert_eq!(substitution(&[48, 61], &[48, 60]), 1);
    assert_eq!(substitution(&[48, 60], &[48, 60]), 0);

    let reference: Vec<Onset> = [60, 62, 64, 65, 67]
        .iter()
        .enumerate()
        .map(|(i, &p)| onset(i as f64, &[p]))
        .collect();
    let played = |pitches: &[u8]| -> Vec<Onset> {
        pitches
            .iter()
            .enumerate()
            .map(|(i, &p)| onset(i as f64, &[p]))
            .collect()
    };
    // Free end: a beginning costs nothing.
    assert_eq!(distance(&played(&[60, 62, 64]), &reference), 0);
    assert_eq!(distance(&played(&[60, 63, 64]), &reference), 1);
    // Left out and added.
    assert_eq!(distance(&played(&[60, 64, 65]), &reference), 1);
    assert_eq!(distance(&played(&[60, 62, 80, 64]), &reference), 1);
    // Fixed start.
    assert_eq!(distance(&played(&[62, 64, 65]), &reference), 1);
    assert_eq!(distance(&played(&[64, 65, 67]), &reference), 2);
}

/// Hashes depend neither on the key nor on the tempo; the ratio of the onset distances counts.
#[test]
fn fingerprint_hashes_ignore_key_and_tempo() {
    let onsets: Vec<Onset> = [
        (0.0, &[48, 60][..]),
        (0.5, &[52, 64]),
        (1.0, &[55]),
        (2.0, &[53, 65]),
    ]
    .iter()
    .map(|&(t, p)| onset(t, p))
    .collect();
    let moved: Vec<Onset> = onsets
        .iter()
        .map(|o| {
            onset(
                0.3 + 1.7 * o.time,
                &o.pitches.iter().map(|p| p + 5).collect::<Vec<_>>(),
            )
        })
        .collect();
    let hashes = |onsets: &[Onset]| -> Vec<(usize, Hash)> {
        triples(onsets, 0..onsets.len())
            .into_iter()
            .map(|t| (t.onset, t.hash))
            .collect()
    };
    assert_eq!(hashes(&onsets), hashes(&moved));
    let pitches = |onsets: &[Onset]| -> Vec<u8> {
        triples(onsets, 0..onsets.len())
            .into_iter()
            .map(|t| t.pitch)
            .collect()
    };
    assert!(
        pitches(&onsets)
            .iter()
            .zip(pitches(&moved))
            .all(|(a, b)| a + 5 == b)
    );

    // Lowest and highest tone of each onset: 2 · 2 · 1 tone triples for onsets 0, 1, 2.
    let first = triples(&onsets, 2..3);
    assert_eq!(first.len(), 4);
    assert!(
        first.contains(&legato_core::recognition::fingerprint::Triple {
            onset: 0,
            pitch: 48,
            hash: Hash {
                first: 4,
                second: 3,
                ratio: 0
            },
        })
    );
    // Onsets 1, 2, 3: twice the distance.
    let last = triples(&onsets, 3..4);
    assert!(last.iter().any(|t| t.onset == 1 && t.hash.ratio == 1));
    // Onsets 0, 1, 3 (one onset left out): three times the distance.
    assert!(last.iter().any(|t| t.onset == 0 && t.hash.ratio == 2));
}
