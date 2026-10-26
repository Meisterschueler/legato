use exercise_musicxml::{Book, Clef, clefs, file_name, generate, natural, parse};
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
    let generated = generate(&book, &book.exercise[0]).unwrap();
    let shipped = std::fs::read_to_string(root("app/assets/pieces/hanon-01.musicxml")).unwrap();
    assert!(
        generated == shipped,
        "regenerate app/assets/pieces/hanon-01.musicxml or fix the tool"
    );
}

#[test]
fn testdata_is_up_to_date() {
    let book = book();
    for exercise in &book.exercise {
        let path = root(&format!(
            "testdata/legato/hanon/{}",
            file_name(&book, exercise)
        ));
        let committed = std::fs::read_to_string(&path).unwrap_or_default();
        assert!(
            generate(&book, exercise).unwrap() == committed,
            "{path} is outdated: cargo run -p exercise-musicxml"
        );
    }
}

#[test]
fn every_exercise_is_read_as_hanon() {
    let book = book();
    for exercise in &book.exercise {
        let xml = generate(&book, exercise).unwrap();
        let (read, rules) = read_with_rules(xml.as_bytes()).unwrap();
        assert_eq!(rules.work_id(), "hanon", "Nr. {}", exercise.number);
        assert_eq!(
            read.metadata.movement_number.as_deref(),
            Some(exercise.number.as_str())
        );
    }
}

/// The right hand plays the notes of the left hand an octave higher, in sixteenths.
#[test]
fn hands_play_in_octaves() {
    for exercise in &book().exercise {
        for section in exercise.up.iter().chain(&exercise.down) {
            let notes = |notes: &str| -> Vec<i32> {
                notes
                    .split_whitespace()
                    .map(|n| natural(n, 0).unwrap())
                    .collect()
            };
            let left = notes(&section.left.notes);
            let right: Vec<i32> = notes(&section.right.notes).iter().map(|n| n - 7).collect();
            assert_eq!(right, left, "Nr. {}", exercise.number);
            assert_eq!(section.left.rhythm, [4, 4]);
            assert_eq!(section.right.rhythm, [4, 4]);
        }
    }
}

/// Part 1 has no thumb crossings: a higher note takes a smaller finger on the left and a larger
/// one on the right.
#[test]
fn fingering_follows_the_notes() {
    for exercise in &book().exercise {
        for section in exercise.up.iter().chain(&exercise.down) {
            let notes: Vec<i32> = section
                .left
                .notes
                .split_whitespace()
                .map(|n| natural(n, 0).unwrap())
                .collect();
            let left = section.left.fingers.as_ref().unwrap();
            let right = section.right.fingers.as_ref().unwrap();
            for i in 1..notes.len() {
                let up = (notes[i] - notes[i - 1]).signum();
                let left = (left[i - 1] as i32 - left[i] as i32).signum();
                let right = (right[i] as i32 - right[i - 1] as i32).signum();
                assert!(
                    up == 0 || (left == up && right == up),
                    "Nr. {}, {:?}, note {}",
                    exercise.number,
                    section.left.notes,
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
        work = "The Virtuoso Pianist"
        composer = "Hanon"
        change = "hand"
        [[exercise]]
        number = 1
        check = "offen"
        up = []
        down = []
        "#,
    )
    .unwrap();
    assert!(generate(&book, &book.exercise[0]).is_err());
}

/// Both hands start on the lower staff. Right hand: measure 6 (A3–F4) needs two ledger lines in
/// both clefs, measure 7 (B3–G4) one in the treble clef instead of three; going down, measure 22 is
/// the last one with that advantage. Left hand: the same for measures 14 and 15 only. The hand
/// still follows from the voice.
#[test]
fn hands_change_staff_for_whole_measures() {
    let book = book();
    let xml = generate(&book, &book.exercise[0]).unwrap();
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

/// A figure that leaves the bass clef for a single measure only stays there.
#[test]
fn no_change_for_a_single_measure() {
    let notes = |n: &str| vec![natural(n, 0).unwrap()];
    let measures = [
        notes("C2"),
        notes("G4"),
        notes("C2"),
        notes("G4"),
        notes("G4"),
    ];
    use Clef::{Bass, Treble};
    assert_eq!(clefs(&measures, Bass), [Bass, Bass, Bass, Treble, Treble]);
}

#[test]
fn naturals() {
    // The last figure up of exercise 1 starts 13 steps above C2 (measure 14).
    assert_eq!(natural("C2", 0).unwrap() + 13, natural("B3", 0).unwrap());
    assert_eq!(natural("F#4", 2).unwrap(), natural("F4", 0).unwrap());
    assert_eq!(natural("Bb3", -1).unwrap(), natural("B3", 0).unwrap());
    assert!(natural("F4", 2).is_err());
    assert!(natural("B3", -1).is_err());
}
