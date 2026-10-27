use std::path::PathBuf;
use std::sync::mpsc::Receiver;
use std::time::Duration;

use legato_core::following::deviations::DeviationKind;
use legato_core::performance::{AbortReason, Setup};
use legato_core::score::model::{Candidate, Variation};
use legato_core::score::musicxml_read::read;
use legato_core::works::WorkRules;
use legato_core::works::hanon::HanonRules;
use legato_engine::api::{DeviationInfo, Engine, Event, MidiInput, Outcome, StartConfig};
use legato_engine::input::sim::SimulatedPort;
use legato_engine::storage::Transport;

/// Debug builds need a while for 36 candidates.
const TIMEOUT: Duration = Duration::from_secs(60);

fn assets() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../app/assets")
}

/// Engine with Hanon No. 1 as the only piece to recognize and a connected piano.
fn start(name: &str) -> (Engine, Receiver<Event>, String) {
    let data_dir =
        std::env::temp_dir().join(format!("legato-session-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&data_dir);
    let engine = Engine::start(
        StartConfig {
            data_dir,
            resource_dir: assets(),
            locale: "de-DE".into(),
        },
        Box::new(SimulatedPort::default()),
    )
    .unwrap();
    let events = engine.take_events().unwrap();
    engine.midi(MidiInput::DeviceAdded {
        device: 1,
        name: "Piano".into(),
        transport: Transport::Usb,
    });
    engine.select_input(Some(1)).unwrap();
    let piece = engine.repertoire()[0].id.clone();
    // Debug builds lag behind the played tones with all 720 candidates of Hanon 1–20.
    let mut settings = engine.get_settings();
    settings.recognition_filter.pieces = vec![piece.clone()];
    engine.set_settings(settings).unwrap();
    (engine, events, piece)
}

/// Note-ons of a clean performance, 2 quarter notes per second.
fn notes(variation: Variation, transposition: i8) -> Vec<(u64, u8)> {
    let score = read(&std::fs::read(assets().join("pieces/hanon-01.musicxml")).unwrap())
        .unwrap()
        .score;
    let candidate = Candidate {
        piece: String::new(),
        variation,
        transposition,
    };
    Setup::new(&score, candidate, HanonRules.performance_range(&score))
        .reference()
        .iter()
        .map(|n| ((n.onset_beat * 500_000.0) as u64, n.pitch))
        .collect()
}

fn play(engine: &Engine, notes: &[(u64, u8)]) {
    for &(time_us, pitch) in notes {
        engine.midi(MidiInput::Message {
            device: 1,
            time_us,
            data: vec![0x90, pitch, 64],
        });
    }
}

/// Events up to and including the first one that matches.
fn until(events: &Receiver<Event>, done: impl Fn(&Event) -> bool) -> Vec<Event> {
    let mut seen = Vec::new();
    loop {
        let event = events.recv_timeout(TIMEOUT).unwrap();
        let last = done(&event);
        seen.push(event);
        if last {
            return seen;
        }
    }
}

#[test]
fn performance_is_recognized_followed_and_completed() {
    let (engine, events, piece) = start("complete");
    play(&engine, &notes(Variation::BothHands, 2));
    let seen = until(&events, |e| matches!(e, Event::PerformanceEnded { .. }));

    assert!(seen.contains(&Event::PracticeSessionChanged { active: true }));
    assert!(seen.contains(&Event::Tone {
        pitch: notes(Variation::BothHands, 2)[0].1,
        velocity: 64,
        driver_time_us: 0,
    }));
    assert!(seen.contains(&Event::PerformanceRecognized {
        piece: piece.clone(),
        variation: Variation::BothHands,
        transposition: 2,
    }));
    // The status of the deciding observation; 3 variations × 12 keys.
    let status = seen
        .iter()
        .take_while(|e| !matches!(e, Event::PerformanceRecognized { .. }))
        .filter_map(|e| match e {
            Event::RecognitionStatus {
                candidates,
                followed,
                ..
            } => Some((candidates, followed)),
            _ => None,
        })
        .last();
    let (candidates, followed) = status.expect("status during the recognition");
    assert_eq!(*candidates, 36);
    assert!(followed.len() <= 6);
    assert_eq!(
        (
            &followed[0].piece,
            followed[0].variation,
            followed[0].transposition
        ),
        (&piece, Variation::BothHands, 2)
    );
    let last_position = seen.iter().rev().find_map(|e| match e {
        Event::Position { note_ids, .. } => Some(note_ids.clone()),
        _ => None,
    });
    // Last onset of the repeat section, D major: both hands.
    assert_eq!(last_position.map(|ids| ids.len()), Some(2));
    assert_eq!(
        seen.last(),
        Some(&Event::PerformanceEnded {
            outcome: Outcome::Completed,
            reason: None,
            deviations: vec![],
        })
    );
}

/// FR-025: with marking during the performance on, a wrong note is reported at once.
#[test]
fn wrong_note_is_marked_and_listed_at_the_end() {
    let (engine, events, _) = start("wrong");
    let mut settings = engine.get_settings();
    settings.error_marking_during.wrong_note = true;
    engine.set_settings(settings).unwrap();
    let mut notes = notes(Variation::RightHandOnly, 0);
    notes[60].1 += 1;
    play(&engine, &notes);
    let seen = until(&events, |e| matches!(e, Event::PerformanceEnded { .. }));

    let wrong = DeviationInfo {
        kind: DeviationKind::Substitution,
        note_id: seen.iter().find_map(|e| match e {
            Event::DeviationDetected(d) => d.note_id.clone(),
            _ => None,
        }),
        pitch: Some(notes[60].1),
    };
    assert!(wrong.note_id.is_some());
    assert!(seen.contains(&Event::DeviationDetected(wrong.clone())));
    assert_eq!(
        seen.last(),
        Some(&Event::PerformanceEnded {
            outcome: Outcome::Completed,
            reason: None,
            deviations: vec![wrong],
        })
    );
}

/// NFR-005: the background discards the performance and ends the session.
#[test]
fn background_discards_the_running_performance() {
    let (engine, events, _) = start("background");
    play(&engine, &notes(Variation::RightHandOnly, 0)[..30]);
    until(&events, |e| {
        matches!(e, Event::PerformanceRecognized { .. })
    });
    engine.set_foreground(false);
    let seen = until(&events, |e| matches!(e, Event::BackgroundNotice { .. }));
    assert_eq!(
        seen[seen.len() - 3..],
        [
            Event::PerformanceEnded {
                outcome: Outcome::Aborted,
                reason: Some(AbortReason::Interrupted),
                deviations: vec![],
            },
            Event::PracticeSessionChanged { active: false },
            Event::BackgroundNotice {
                performance_discarded: true
            },
        ]
    );
}
