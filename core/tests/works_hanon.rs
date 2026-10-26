use legato_core::score::model::{Hand, Metadata, Variation};
use legato_core::score::musicxml_read::{InvalidReason, read};
use legato_core::works::detect::{detect, read_with_rules};
use legato_core::works::hanon::HanonRules;
use legato_core::works::{Range, WorkRules, hand_for};

fn hanon() -> Vec<u8> {
    std::fs::read(format!(
        "{}/../app/assets/pieces/hanon-01.musicxml",
        env!("CARGO_MANIFEST_DIR")
    ))
    .unwrap()
}

#[test]
fn hanon_is_detected_from_metadata() {
    let (read, rules) = read_with_rules(&hanon()).unwrap();
    assert_eq!(rules.work_id(), "hanon");
    assert_eq!(read.metadata.movement_number.as_deref(), Some("1"));

    let other = Metadata {
        composer: Some("Hanon".into()),
        movement_number: Some("21".into()),
        ..Metadata::default()
    };
    assert_eq!(detect(&other).work_id(), "generic");
    assert_eq!(detect(&Metadata::default()).work_id(), "generic");
}

#[test]
fn transposition_0_to_11_for_both_hands_equally() {
    let candidates = HanonRules.candidates("h1");
    assert_eq!(candidates.len(), 3 * 12);
    for variation in [
        Variation::BothHands,
        Variation::LeftHandOnly,
        Variation::RightHandOnly,
    ] {
        let transpositions: Vec<i8> = candidates
            .iter()
            .filter(|c| c.variation == variation)
            .map(|c| c.transposition)
            .collect();
        assert_eq!(transpositions, (0..12).collect::<Vec<i8>>());
    }
}

#[test]
fn performance_is_the_repeat_section_without_ende() {
    let score = read(&hanon()).unwrap().score;
    assert_eq!(
        HanonRules.performance_range(&score),
        Range { start: 0, end: 231 }
    );
    // Onset 232 is "Ende".
    assert_eq!(score.onsets.len(), 233);
}

#[test]
fn ascending_and_descending_split_at_the_double_barline() {
    let score = read(&hanon()).unwrap().score;
    let [ascending, descending] = HanonRules.directions(&score);
    // 29 measures: 14 ascending, 15 descending, 8 onsets each.
    assert_eq!(
        ascending,
        Range {
            start: 0,
            end: 14 * 8 - 1
        }
    );
    assert_eq!(
        descending,
        Range {
            start: 14 * 8,
            end: 231
        }
    );

    // No. 17: 14 measures ascending, 13 descending; not the halves.
    let score = read(&hanon_17()).unwrap().score;
    assert!(score.measures[13].double_bar);
    let [ascending, descending] = HanonRules.directions(&score);
    assert_eq!(ascending.end, 14 * 8 - 1);
    assert_eq!(descending.start, 14 * 8);
}

#[test]
fn without_double_barline_ascending_and_descending_are_the_halves() {
    let xml = String::from_utf8(hanon_17()).unwrap().replace(
        "<bar-style>light-light</bar-style>",
        "<bar-style>regular</bar-style>",
    );
    let score = read(xml.as_bytes()).unwrap().score;
    assert!(score.measures.iter().all(|m| !m.double_bar));
    let [ascending, descending] = HanonRules.directions(&score);
    // 27 measures: the descending half has one measure more.
    assert_eq!(ascending.end, 13 * 8 - 1);
    assert_eq!(descending.start, 13 * 8);
}

fn hanon_17() -> Vec<u8> {
    std::fs::read(format!(
        "{}/../testdata/legato/hanon/hanon-17.musicxml",
        env!("CARGO_MANIFEST_DIR")
    ))
    .unwrap()
}

#[test]
fn recognition_length_is_8_tones_per_hand() {
    assert_eq!(HanonRules.recognition_length(), 8);
}

#[test]
fn hand_from_position_and_restriction() {
    let score = read(&hanon()).unwrap().score;
    // Notated: left hand from C2 (36), right hand from C3 (48). F sharp major: +6.
    assert_eq!(HanonRules.hand_from_position(&score, 42), Some(Hand::Left));
    assert_eq!(HanonRules.hand_from_position(&score, 54), Some(Hand::Right));
    assert_eq!(HanonRules.hand_from_position(&score, 35), None);
    assert_eq!(HanonRules.hand_from_position(&score, 60), None);
    // A restriction (FR-018) wins even in the range of the other hand.
    assert_eq!(
        hand_for(&HanonRules, &score, 42, Some(Hand::Right)),
        Some(Hand::Right)
    );
    assert_eq!(hand_for(&HanonRules, &score, 42, None), Some(Hand::Left));
}

#[test]
fn hanon_without_repeat_section_is_rejected() {
    let source = String::from_utf8(hanon())
        .unwrap()
        .replace("<repeat direction=\"forward\"/>", "")
        .replace("<repeat direction=\"backward\"/>", "");
    let error = read_with_rules(source.as_bytes()).err().unwrap();
    assert!(matches!(
        error.reason,
        InvalidReason::IncompatibleWithWork(_)
    ));
}
