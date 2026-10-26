use std::path::PathBuf;

use legato_core::recognition::RecognitionMethod;
use legato_engine::storage::{
    AbortSignal, ConnectSignal, ErrorMarking, OkSignal, ScoreFollowerKind, Settings,
};

fn temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("legato-storage-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    dir
}

#[test]
fn defaults_as_in_the_data_model() {
    let settings = Settings::default();
    assert_eq!(settings.pause_limit_ms, 4000);
    assert_eq!(settings.rest_limit_ms, 10000);
    assert_eq!(settings.session_idle_timeout_ms, 600_000);
    assert_eq!(settings.ok_signal, OkSignal::WhenStopped);
    assert_eq!(settings.abort_signal, AbortSignal::Immediately);
    assert_eq!(settings.connect_signal, ConnectSignal::On);
    assert_eq!(settings.error_marking_during, ErrorMarking::default());
    assert_eq!(settings.error_marking_after, ErrorMarking::default());
    assert!(!settings.error_marking_during.wrong_note);
    assert_eq!(settings.score_follower, ScoreFollowerKind::Pthmm);
    assert_eq!(settings.recognition_method, RecognitionMethod::EditDistance);
    assert_eq!(settings.midi_device, None);
    assert_eq!(settings.signal_tones.notes, vec![96, 100]);
    assert_eq!(settings.signal_tones.velocity, 30);
}

#[test]
fn missing_file_gives_defaults() {
    assert_eq!(
        Settings::load(&temp_dir("missing")).unwrap(),
        Settings::default()
    );
}

#[test]
fn save_and_load() {
    let dir = temp_dir("roundtrip");
    let settings = Settings {
        pause_limit_ms: 5000,
        ok_signal: OkSignal::EveryPerformance,
        recognition_method: RecognitionMethod::Fingerprint,
        ..Settings::default()
    };
    settings.save(&dir).unwrap();
    assert_eq!(Settings::load(&dir).unwrap(), settings);
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn missing_fields_take_defaults() {
    let dir = temp_dir("partial");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("settings.json"), r#"{ "rest_limit_ms": 8000 }"#).unwrap();
    let settings = Settings::load(&dir).unwrap();
    assert_eq!(settings.rest_limit_ms, 8000);
    assert_eq!(settings.pause_limit_ms, 4000);
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn unreadable_file_is_an_error() {
    let dir = temp_dir("broken");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("settings.json"), "{").unwrap();
    assert!(Settings::load(&dir).is_err());
    std::fs::remove_dir_all(dir).unwrap();
}
