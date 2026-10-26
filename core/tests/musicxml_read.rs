use std::io::Write;

use legato_core::score::model::{Fraction, Hand, NoteId, SectionKind};
use legato_core::score::musicxml_read::{InvalidReason, read};

fn file(name: &str) -> Vec<u8> {
    std::fs::read(format!(
        "{}/../testdata/legato/musicxml/{name}",
        env!("CARGO_MANIFEST_DIR")
    ))
    .unwrap()
}

fn pitches(score: &legato_core::score::model::Score) -> Vec<Vec<u8>> {
    score
        .onsets
        .iter()
        .map(|o| o.notes.iter().map(|n| n.pitch).collect())
        .collect()
}

#[test]
fn score_partwise_with_two_staves() {
    let read = read(&file("two-staves.musicxml")).unwrap();
    assert_eq!(read.metadata.composer.as_deref(), Some("Tester"));
    assert_eq!(read.metadata.work_title.as_deref(), Some("Test"));
    assert_eq!(read.metadata.movement_number.as_deref(), Some("7"));

    let score = read.score;
    // Chord E4+G4 is one onset; the tied D4 and the grace note give no onset; rests give none.
    assert_eq!(
        pitches(&score),
        vec![
            vec![60, 48],
            vec![64, 67],
            vec![48],
            vec![62],
            vec![43],
            vec![66],
            vec![67]
        ]
    );
    assert_eq!(score.onsets[3].time, Fraction::new(3, 1));
    assert_eq!(
        score.rests_before.iter().copied().collect::<Vec<_>>(),
        vec![3]
    );
    assert_eq!(score.measures.len(), 2);

    let c4 = &score.onsets[0].notes[0];
    assert_eq!(c4.hand, Hand::Right);
    assert_eq!(c4.finger, Some(1));
    let c3 = &score.onsets[0].notes[1];
    assert_eq!(c3.hand, Hand::Left);
    assert_eq!(
        c3.id,
        NoteId {
            measure: 0,
            voice: 5,
            index: 4
        }
    );
    assert_eq!(score.onsets[1].notes[0].finger, None);

    // The tied D4 in measure 2 keeps its index, so F#4 has index 1.
    assert_eq!(
        score.onsets[5].notes[0].id,
        NoteId {
            measure: 1,
            voice: 1,
            index: 1
        }
    );

    assert_eq!(score.sections.len(), 1);
    assert_eq!(score.sections[0].kind, SectionKind::Repeat);
    assert_eq!((score.sections[0].start, score.sections[0].end), (0, 6));
    assert_eq!(score.onsets[2].successors, vec![3]);
    assert_eq!(score.onsets[6].successors, vec![0]);
}

#[test]
fn display_ids_follow_the_contract() {
    let score = read(&file("two-staves.musicxml")).unwrap().score;
    assert_eq!(score.onsets[0].notes[0].display_id(0), "0/0/0:1/60");
    assert_eq!(score.onsets[0].notes[1].display_id(0), "0/1/0:1/48");
    assert_eq!(score.onsets[3].notes[0].display_id(0), "0/0/3:4/62");
    assert_eq!(score.onsets[5].notes[0].display_id(2), "1/0/1:4/68");
}

#[test]
fn two_parts_are_two_hands() {
    let score = read(&file("two-parts.musicxml")).unwrap().score;
    assert_eq!(pitches(&score), vec![vec![72, 48]]);
    let hands: Vec<_> = score.onsets[0]
        .notes
        .iter()
        .map(|n| (n.hand, n.staff))
        .collect();
    assert_eq!(hands, vec![(Hand::Right, 0), (Hand::Left, 1)]);
}

#[test]
fn compressed_mxl() {
    let mut buffer = std::io::Cursor::new(Vec::new());
    let mut zip = zip::ZipWriter::new(&mut buffer);
    let options =
        zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);
    zip.start_file("META-INF/container.xml", options).unwrap();
    zip.write_all(br#"<container><rootfiles><rootfile full-path="score/piece.musicxml"/></rootfiles></container>"#)
        .unwrap();
    zip.start_file("score/piece.musicxml", options).unwrap();
    zip.write_all(&file("two-staves.musicxml")).unwrap();
    zip.finish().unwrap();

    let score = read(buffer.get_ref()).unwrap().score;
    assert_eq!(score.onsets.len(), 7);
}

#[test]
fn rejections() {
    let reason = |bytes: &[u8]| read(bytes).unwrap_err().reason;
    assert!(matches!(reason(b"no xml"), InvalidReason::Unreadable(_)));
    assert!(matches!(
        reason(b"PK\x03\x04broken"),
        InvalidReason::Unreadable(_)
    ));
    assert_eq!(
        reason(&file("timewise.musicxml")),
        InvalidReason::NotPartwise
    );
    assert_eq!(
        reason(&file("rests-only.musicxml")),
        InvalidReason::NoPitchedNotes
    );
    assert_eq!(
        reason(&file("unpaired-repeat.musicxml")),
        InvalidReason::UnpairedRepeats
    );
}

#[test]
fn hanon_no_1() {
    let source = std::fs::read(format!(
        "{}/../app/assets/pieces/hanon-01.musicxml",
        env!("CARGO_MANIFEST_DIR")
    ))
    .unwrap();
    let read = read(&source).unwrap();
    assert_eq!(read.metadata.composer.as_deref(), Some("Hanon"));
    assert_eq!(read.metadata.movement_number.as_deref(), Some("1"));

    let score = read.score;
    // 29 measures with 8 sixteenths in both hands, plus the final note.
    assert_eq!(score.onsets.len(), 29 * 8 + 1);
    assert!(score.onsets.iter().all(|o| o.notes.len() == 2));
    assert!(
        score
            .onsets
            .iter()
            .flat_map(|o| &o.notes)
            .all(|n| n.finger.is_some())
    );
    assert_eq!(score.sections.len(), 1);
    assert_eq!((score.sections[0].start, score.sections[0].end), (0, 231));
    assert_eq!(score.onsets[231].successors, vec![232, 0]);

    // Display ids by the rule score-view-web applies to OSMD (checked in the browser, spike S2):
    // the staff is the note's `<staff>`. Both hands start on the lower staff; the hand follows from
    // the voice.
    let ids: Vec<String> = score.onsets[..3]
        .iter()
        .flat_map(|o| &o.notes)
        .map(|n| n.display_id(0))
        .collect();
    assert_eq!(
        ids,
        [
            "0/1/0:1/48",
            "0/1/0:1/36",
            "0/1/1:16/52",
            "0/1/1:16/40",
            "0/1/1:8/53",
            "0/1/1:8/41"
        ]
    );
    let hands: Vec<Hand> = score.onsets[0].notes.iter().map(|n| n.hand).collect();
    assert_eq!(hands, [Hand::Right, Hand::Left]);
    let last: Vec<String> = score.onsets[232]
        .notes
        .iter()
        .map(|n| n.display_id(0))
        .collect();
    assert_eq!(last, ["29/1/0:1/48", "29/1/0:1/36"]);
}

fn piano(notes: &str) -> Vec<u8> {
    format!(
        "<score-partwise><part-list><score-part id=\"P1\"/></part-list><part id=\"P1\"><measure number=\"1\">\
         <attributes><divisions>1</divisions><staves>2</staves></attributes>{notes}</measure></part></score-partwise>"
    )
    .into_bytes()
}

fn piano_note(step: &str, voice: u8, staff: u8, backup: bool) -> String {
    let backup = if backup {
        "<backup><duration>1</duration></backup>"
    } else {
        ""
    };
    format!(
        "{backup}<note><pitch><step>{step}</step><octave>4</octave></pitch><duration>1</duration><voice>{voice}</voice><staff>{staff}</staff></note>"
    )
}

#[test]
fn hand_follows_the_voice_across_staves() {
    // Voice 1 (right hand) on the lower staff, voice 5 (left hand) on the upper one.
    let xml = piano(&(piano_note("G", 1, 2, false) + &piano_note("C", 5, 1, true)));
    let notes = &read(&xml).unwrap().score.onsets[0].notes;
    let hands: Vec<_> = notes.iter().map(|n| (n.hand, n.staff)).collect();
    assert_eq!(hands, [(Hand::Right, 1), (Hand::Left, 0)]);
}

#[test]
fn without_voices_5_to_8_the_staff_gives_the_hand() {
    let xml = piano(&(piano_note("G", 1, 1, false) + &piano_note("C", 1, 2, true)));
    let notes = &read(&xml).unwrap().score.onsets[0].notes;
    let hands: Vec<_> = notes.iter().map(|n| n.hand).collect();
    assert_eq!(hands, [Hand::Right, Hand::Left]);
}
