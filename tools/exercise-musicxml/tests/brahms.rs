use exercise_musicxml::{Book, file_name, generate, parse};
use legato_core::score::model::{Fraction, Hand, Score};
use legato_core::works::detect::read_with_rules;

fn root(path: &str) -> String {
    format!("{}/../../{path}", env!("CARGO_MANIFEST_DIR"))
}

fn book() -> Book {
    parse(&std::fs::read_to_string(root("docs/werke/brahms-51.toml")).unwrap()).unwrap()
}

fn score(number: &str) -> Score {
    let book = book();
    let exercise = book.exercise.iter().find(|e| e.number == number).unwrap();
    let xml = generate(&book, exercise).unwrap();
    let (read, rules) = read_with_rules(xml.as_bytes()).unwrap();
    assert_eq!(rules.work_id(), "generic");
    assert_eq!(read.metadata.movement_number.as_deref(), Some(number));
    read.score
}

/// (time in quarter notes, MIDI pitch) of one hand in a measure (0-based).
fn hand(score: &Score, hand: Hand, measure: u32) -> Vec<(Fraction, u8)> {
    score
        .onsets
        .iter()
        .flat_map(|o| o.notes.iter().map(move |n| (o.time, n)))
        .filter(|(_, n)| n.hand == hand && n.id.measure == measure)
        .map(|(t, n)| (t, n.pitch))
        .collect()
}

fn pitches(notes: &[(Fraction, u8)]) -> Vec<u8> {
    notes.iter().map(|&(_, p)| p).collect()
}

#[test]
fn testdata_is_up_to_date() {
    let book = book();
    for exercise in &book.exercise {
        let path = root(&format!(
            "testdata/legato/brahms/{}",
            file_name(&book, exercise)
        ));
        let committed = std::fs::read_to_string(&path).unwrap_or_default();
        assert!(
            generate(&book, exercise).unwrap() == committed,
            "{path} is outdated: cargo run -p exercise-musicxml"
        );
    }
}

/// 14 measures up, 14 down, transposed diatonically in D major; the hands swap triplets and
/// sixteenths at the turning point.
#[test]
fn exercise_1a_follows_the_description() {
    let score = score("1a");
    assert_eq!(score.measures.len(), 28);
    let right = hand(&score, Hand::Right, 0);
    assert_eq!(
        pitches(&right),
        [66, 62, 57, 54, 55, 57, 59, 61, 62, 64, 66]
    );
    let times: Vec<Fraction> = right.iter().map(|&(t, _)| t).collect();
    let expected: Vec<Fraction> = [
        (0, 1),
        (1, 3),
        (2, 3),
        (1, 1),
        (5, 4),
        (3, 2),
        (7, 4),
        (2, 1),
        (9, 4),
        (5, 2),
        (11, 4),
    ]
    .iter()
    .map(|&(n, d)| Fraction::new(n, d))
    .collect();
    assert_eq!(times, expected);
    assert_eq!(
        pitches(&hand(&score, Hand::Left, 0)),
        [38, 42, 45, 50, 49, 47, 45, 43, 42]
    );
    assert_eq!(
        pitches(&hand(&score, Hand::Right, 1)),
        [67, 64, 59, 55, 57, 59, 61, 62, 64, 66, 67]
    );
    assert_eq!(
        pitches(&hand(&score, Hand::Right, 14)),
        [90, 86, 81, 78, 79, 81, 83, 85, 86]
    );
    let left = hand(&score, Hand::Left, 14);
    assert_eq!(pitches(&left), [62, 66, 69, 74, 73, 71, 69, 67, 66, 64, 62]);
    assert_eq!(left[0].0, Fraction::new(42, 1));
    assert_eq!(
        pitches(&hand(&score, Hand::Right, 27)),
        [67, 64, 59, 55, 57, 59, 61, 62, 64]
    );
}

/// The upper staff stays in the treble clef; the lower one changes to the treble clef in measure
/// 11 and back to the bass clef in measure 20.
#[test]
fn clefs_change_for_whole_measures() {
    let book = book();
    let xml = generate(&book, &book.exercise[0]).unwrap();
    assert!(!xml.contains("<clef number=\"1\"><sign>F"));
    assert_eq!(xml.matches("<clef number=\"2\">").count(), 3);
    assert!(xml.contains("<measure number=\"11\"><attributes><clef number=\"2\"><sign>G"));
    assert!(xml.contains("<measure number=\"20\"><attributes><clef number=\"2\"><sign>F"));
}

#[test]
fn invalid_figures_are_refused() {
    let exercise = |right: &str| {
        parse(&format!(
            r#"
            work = "51 Exercises"
            composer = "Brahms"
            change = "clef"
            [[exercise]]
            number = "0"
            fifths = 2
            down = []
            [[exercise.up]]
            right = {right}
            left = {{ notes = "D2 E2 F#2", rhythm = [1, 1, 1] }}
            steps = {{ from = 0, to = 0 }}
            "#
        ))
        .unwrap()
    };
    let generates = |right: &str| {
        let book = exercise(right);
        generate(&book, &book.exercise[0]).is_ok()
    };
    assert!(generates(r#"{ notes = "D4 E4 F#4", rhythm = [1, 1, 1] }"#));
    // Not in the key.
    assert!(!generates(r#"{ notes = "D4 E4 F4", rhythm = [1, 1, 1] }"#));
    // Rhythm does not fit the notes or the time signature of the left hand.
    assert!(!generates(r#"{ notes = "D4 E4 F#4", rhythm = [1, 2] }"#));
    // One finger per note.
    assert!(!generates(
        r#"{ notes = "D4 E4 F#4", rhythm = [1, 1, 1], fingers = [1, 2] }"#
    ));
}
