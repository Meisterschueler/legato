use std::fs;
use std::path::PathBuf;

use legato_engine::api::{Engine, MidiOutput, StartConfig};

struct NoMidi;

impl MidiOutput for NoMidi {
    fn connect(&mut self, _device: u32) {}
    fn disconnect(&mut self, _device: u32) {}
    fn send(&mut self, _device: u32, _data: &[u8]) {}
    fn vibrate(&mut self) {}
}

fn resource_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../app/assets")
}

fn temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("legato-repertoire-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    dir
}

#[test]
fn fresh_install_contains_bundled_hanon_only() {
    let data_dir = temp_dir("fresh");
    let engine = Engine::start(
        StartConfig {
            data_dir: data_dir.clone(),
            resource_dir: resource_dir(),
            locale: "de-DE".into(),
        },
        Box::new(NoMidi),
    )
    .unwrap();

    let repertoire = engine.repertoire();
    assert_eq!(repertoire.len(), 1);
    assert!(repertoire[0].has_fingering);
    assert_eq!(repertoire[0].number.as_deref(), Some("1"));
    assert!(repertoire[0].id.starts_with("0x") || !repertoire[0].id.is_empty());

    std::fs::remove_dir_all(data_dir).unwrap();
}

#[test]
fn valid_and_invalid_files_are_loaded_and_rejected() {
    let data_dir = temp_dir("load");
    let engine = Engine::start(
        StartConfig {
            data_dir: data_dir.clone(),
            resource_dir: resource_dir(),
            locale: "de-DE".into(),
        },
        Box::new(NoMidi),
    )
    .unwrap();

    let valid = engine
        .load_piece(resource_dir().join("pieces/hanon-01.musicxml"))
        .unwrap();
    assert!(valid.has_fingering);
    assert_eq!(valid.number.as_deref(), Some("1"));

    let invalid = engine.load_piece(data_dir.join("not-a-score.xml"));
    assert!(invalid.is_err());

    let no_fingering = data_dir.join("no-fingering.musicxml");
    let xml = fs::read(resource_dir().join("pieces/hanon-01.musicxml")).unwrap();
    let xml = String::from_utf8(xml).unwrap().replace("<fingering>", "<dummy>")
        .replace("</fingering>", "</dummy>");
    fs::write(&no_fingering, xml).unwrap();

    let piece = engine.load_piece(no_fingering).unwrap();
    assert!(!piece.has_fingering);

    std::fs::remove_dir_all(data_dir).unwrap();
}

#[test]
fn pieces_persist_across_restarts() {
    let data_dir = temp_dir("persist");
    let resource_dir = resource_dir();

    let engine = Engine::start(
        StartConfig {
            data_dir: data_dir.clone(),
            resource_dir: resource_dir.clone(),
            locale: "de-DE".into(),
        },
        Box::new(NoMidi),
    )
    .unwrap();

    let file = data_dir.join("custom.musicxml");
    fs::write(&file, fs::read(resource_dir.join("pieces/hanon-01.musicxml")).unwrap()).unwrap();
    let loaded = engine.load_piece(file).unwrap();

    drop(engine);

    let restarted = Engine::start(
        StartConfig {
            data_dir: data_dir.clone(),
            resource_dir,
            locale: "de-DE".into(),
        },
        Box::new(NoMidi),
    )
    .unwrap();

    let piece = restarted
        .repertoire()
        .into_iter()
        .find(|p| p.id == loaded.id)
        .unwrap();
    assert!(piece.has_fingering);

    std::fs::remove_dir_all(data_dir).unwrap();
}
