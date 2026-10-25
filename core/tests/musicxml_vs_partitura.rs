//! The MusicXML reader against partitura's `note_array` (T035).

use legato_core::score::musicxml_read::read;
use serde::Deserialize;

#[derive(Deserialize)]
struct NoteArray {
    note_array: Vec<Note>,
}

#[derive(Deserialize)]
struct Note {
    onset_beat: f64,
    pitch: u8,
}

fn file(path: &str) -> Vec<u8> {
    std::fs::read(format!("{}/../{path}", env!("CARGO_MANIFEST_DIR"))).unwrap()
}

/// Sorted (onset in quarters, pitch) of all attacks.
fn ours(path: &str) -> Vec<(f64, u8)> {
    let score = read(&file(path)).unwrap().score;
    let mut notes: Vec<(f64, u8)> = score
        .onsets
        .iter()
        .flat_map(|o| {
            let time = o.time.numerator as f64 / o.time.denominator as f64;
            o.notes.iter().map(move |n| (time, n.pitch))
        })
        .collect();
    notes.sort_by(|a, b| a.partial_cmp(b).unwrap());
    notes
}

fn partitura(path: &str) -> Vec<(f64, u8)> {
    let array: NoteArray = serde_json::from_slice(&file(path)).unwrap();
    let mut notes: Vec<(f64, u8)> = array
        .note_array
        .iter()
        .map(|n| (n.onset_beat, n.pitch))
        .collect();
    notes.sort_by(|a, b| a.partial_cmp(b).unwrap());
    notes
}

#[test]
fn hanon_no_1_matches_partitura() {
    // Hanon No. 1 is in 2/4, so beats are quarters.
    assert_eq!(
        ours("app/assets/pieces/hanon-01.musicxml"),
        partitura("testdata/matchmaker/expected/hanon-01.note_array.json")
    );
}

#[test]
fn matchmaker_pieces_match_partitura() {
    for (stem, piece) in [
        ("simple_mozart_k265_var1", "simple_mozart"),
        ("mozart_k265_var1", "mozart_k265_var1"),
        ("Bach-fugue_bwv_858", "bach_fugue"),
    ] {
        let ours = ours(&format!("testdata/matchmaker/pieces/{stem}.musicxml"));
        let theirs = partitura(&format!("testdata/matchmaker/expected/{piece}.pthmm.json"));
        assert_eq!(ours.len(), theirs.len(), "{piece}: number of notes");
        assert_eq!(ours, theirs, "{piece}");
    }
}
