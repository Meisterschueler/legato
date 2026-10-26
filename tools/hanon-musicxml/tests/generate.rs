use hanon_musicxml::{Book, file_name, generate, midi, natural, parse, staves};
use legato_core::score::model::Hand;
use legato_core::score::musicxml_read::read;
use legato_core::works::detect::read_with_rules;

fn root(path: &str) -> String {
    format!("{}/../../{path}", env!("CARGO_MANIFEST_DIR"))
}

fn book() -> Book {
    parse(&std::fs::read_to_string(root("docs/werke/hanon-teil1.toml")).unwrap()).unwrap()
}

#[test]
fn exercise_1_matches_the_shipped_score() {
    let book = book();
    let generated = generate(&book.exercise[0]).unwrap();
    let shipped = std::fs::read_to_string(root("app/assets/pieces/hanon-01.musicxml")).unwrap();
    assert!(
        generated == shipped,
        "regenerate app/assets/pieces/hanon-01.musicxml or fix the tool"
    );
}

#[test]
fn testdata_is_up_to_date() {
    for exercise in &book().exercise {
        let path = root(&format!(
            "testdata/legato/hanon/{}",
            file_name(exercise.number)
        ));
        let committed = std::fs::read_to_string(&path).unwrap_or_default();
        assert!(
            generate(exercise).unwrap() == committed,
            "{path} is outdated: cargo run -p hanon-musicxml"
        );
    }
}

#[test]
fn every_exercise_is_read_as_hanon() {
    for exercise in &book().exercise {
        let xml = generate(exercise).unwrap();
        let (read, rules) = read_with_rules(xml.as_bytes()).unwrap();
        assert_eq!(rules.work_id(), "hanon", "Nr. {}", exercise.number);
        let number = exercise.number.to_string();
        assert_eq!(
            read.metadata.movement_number.as_deref(),
            Some(number.as_str())
        );
    }
}

/// Part 1 has no thumb crossings: a higher note takes a smaller finger on the left and a larger
/// one on the right.
#[test]
fn fingering_follows_the_notes() {
    for exercise in &book().exercise {
        for section in exercise.up.iter().chain(&exercise.down) {
            let notes: Vec<i32> = section
                .figure
                .split_whitespace()
                .map(|n| midi(natural(n).unwrap()))
                .collect();
            for i in 1..notes.len() {
                let up = (notes[i] - notes[i - 1]).signum();
                let left = (section.left[i - 1] as i32 - section.left[i] as i32).signum();
                let right = (section.right[i] as i32 - section.right[i - 1] as i32).signum();
                assert!(
                    up == 0 || (left == up && right == up),
                    "Nr. {}, {:?}, note {}",
                    exercise.number,
                    section.figure,
                    i + 1
                );
            }
        }
    }
}

#[test]
fn unchecked_exercises_are_refused() {
    let book = parse(
        r#"
        [[exercise]]
        number = 1
        end = "C2"
        check = "offen"
        up = []
        down = []
        "#,
    )
    .unwrap();
    assert!(generate(&book.exercise[0]).is_err());
}

/// Both hands start on the lower staff. Right hand: measure 6 (A3–F4) needs two ledger lines in
/// both clefs, measure 7 (B3–G4) one in the treble clef instead of three; going down, measure 22 is
/// the last one with that advantage. Left hand: the same for measures 14 and 15 only. The hand
/// still follows from the voice.
#[test]
fn hands_change_staff_for_whole_measures() {
    let xml = generate(&book().exercise[0]).unwrap();
    assert!(!xml.contains("<clef number=\"1\"><sign>F"));
    assert!(!xml.contains("<clef number=\"2\"><sign>G"));
    let score = read(xml.as_bytes()).unwrap().score;
    let upper = |hand: Hand| -> Vec<u32> {
        let mut measures: Vec<u32> = score
            .onsets
            .iter()
            .flat_map(|o| &o.notes)
            .filter(|n| n.hand == hand && n.staff == 0)
            .map(|n| n.id.measure + 1)
            .collect();
        measures.dedup();
        measures
    };
    assert_eq!(upper(Hand::Right), (7..=22).collect::<Vec<_>>());
    assert_eq!(upper(Hand::Left), [14, 15]);
    for onset in &score.onsets {
        let [right, left] = &onset.notes[..] else {
            panic!("two notes per onset expected")
        };
        assert_eq!((right.hand, left.hand), (Hand::Right, Hand::Left));
        assert_eq!(right.pitch, left.pitch + 12);
    }
}

/// A figure that leaves the lower staff for a single measure only stays there.
#[test]
fn no_change_for_a_single_measure() {
    let notes = |n: &str| vec![natural(n).unwrap()];
    let measures = [
        notes("C2"),
        notes("G4"),
        notes("C2"),
        notes("G4"),
        notes("G4"),
    ];
    assert_eq!(staves(&measures), [2, 2, 2, 1, 1]);
}

#[test]
fn naturals_and_midi() {
    assert_eq!(midi(natural("C4").unwrap()), 60);
    assert_eq!(midi(natural("B3").unwrap()), 59);
    // The last figure up of exercise 1 starts 13 steps above C2 (measure 14).
    assert_eq!(natural("C2").unwrap() + 13, natural("B3").unwrap());
}
