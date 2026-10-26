//! US 4: signals through the simulated MIDI port (T090).

use std::path::PathBuf;
use std::sync::mpsc::Receiver;
use std::thread;
use std::time::{Duration, Instant};

use legato_core::performance::{AbortReason, Setup};
use legato_core::score::model::{Candidate, Variation};
use legato_core::score::musicxml_read::read;
use legato_core::works::WorkRules;
use legato_core::works::hanon::HanonRules;
use legato_engine::api::{Engine, Event, MidiInput, Outcome, StartConfig};
use legato_engine::input::sim::{Call, SimulatedPort};
use legato_engine::storage::{
    AbortSignal, ConnectSignal, OkSignal, Settings, SignalTones, Transport,
};

/// Debug builds need a while for the recognition.
const TIMEOUT: Duration = Duration::from_secs(60);
/// Longer than any planned signal.
const QUIET: Duration = Duration::from_millis(1500);
const OK: [u8; 2] = [96, 100];
const ABORT: [u8; 2] = [100, 96];

fn assets() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../app/assets")
}

fn data_dir(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!("legato-signals-{name}-{}", std::process::id()))
}

/// Short pause limit, so stopping is detected quickly; no connection signal.
fn settings() -> Settings {
    Settings {
        pause_limit_ms: 1000,
        connect_signal: ConnectSignal::Off,
        ..Settings::default()
    }
}

/// Engine with the given settings and the piano (device 1) selected.
fn start(name: &str, settings: Settings) -> (Engine, Receiver<Event>, SimulatedPort) {
    let data_dir = data_dir(name);
    let _ = std::fs::remove_dir_all(&data_dir);
    settings.save(&data_dir).unwrap();
    let (engine, events, port) = restart(name);
    engine.select_input(Some(1)).unwrap();
    (engine, events, port)
}

/// Engine on the existing data of `name`; the piano is present.
fn restart(name: &str) -> (Engine, Receiver<Event>, SimulatedPort) {
    let port = SimulatedPort::default();
    let engine = Engine::start(
        StartConfig {
            data_dir: data_dir(name),
            resource_dir: assets(),
            locale: "de-DE".into(),
        },
        Box::new(port.clone()),
    )
    .unwrap();
    let events = engine.take_events().unwrap();
    engine.midi(MidiInput::DeviceAdded {
        device: 1,
        name: "Piano".into(),
        transport: Transport::Usb,
    });
    (engine, events, port)
}

/// Note-ons of a clean right-hand performance, 2 quarter notes per second, from `start_us`.
fn performance(start_us: u64) -> Vec<(u64, u8)> {
    let score = read(&std::fs::read(assets().join("pieces/hanon-01.musicxml")).unwrap())
        .unwrap()
        .score;
    let candidate = Candidate {
        piece: String::new(),
        variation: Variation::RightHandOnly,
        transposition: 0,
    };
    Setup::new(&score, candidate, HanonRules.performance_range(&score))
        .reference()
        .iter()
        .map(|n| (start_us + (n.onset_beat * 500_000.0) as u64, n.pitch))
        .collect()
}

/// `count` performances one after the other without a gap.
fn seamless(count: u64) -> Vec<(u64, u8)> {
    let once = performance(0);
    let length = once.last().unwrap().0 + (once[1].0 - once[0].0);
    (0..count).flat_map(|i| performance(i * length)).collect()
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

/// Events up to and including the `n`-th one that matches.
fn until(events: &Receiver<Event>, n: usize, done: impl Fn(&Event) -> bool) -> Vec<Event> {
    let mut seen = Vec::new();
    let mut matched = 0;
    while matched < n {
        let event = events.recv_timeout(TIMEOUT).unwrap();
        matched += usize::from(done(&event));
        seen.push(event);
    }
    seen
}

fn ended(e: &Event) -> bool {
    matches!(e, Event::PerformanceEnded { .. })
}

/// The note-on messages sent so far.
fn sent(port: &SimulatedPort) -> Vec<Vec<u8>> {
    port.calls()
        .into_iter()
        .filter_map(|c| match c {
            Call::Send(_, data) if data[0] == 0x90 => Some(data),
            _ => None,
        })
        .collect()
}

fn pitches(port: &SimulatedPort) -> Vec<u8> {
    sent(port).iter().map(|m| m[1]).collect()
}

/// Waits until at least `n` note-ons were sent.
fn wait_for(port: &SimulatedPort, n: usize) -> Vec<u8> {
    let start = Instant::now();
    while pitches(port).len() < n && start.elapsed() < TIMEOUT {
        thread::sleep(Duration::from_millis(10));
    }
    pitches(port)
}

/// Scenarios 1 and 10: right after each performance, the same signal whatever the quality.
#[test]
fn ok_after_every_performance_whatever_the_quality() {
    let (engine, events, port) = start(
        "every",
        Settings {
            ok_signal: OkSignal::EveryPerformance,
            ..settings()
        },
    );
    play(&engine, &performance(0));
    until(&events, 1, ended);
    assert_eq!(wait_for(&port, 2), OK);

    let mut wrong = performance(100_000_000);
    wrong[60].1 += 1;
    play(&engine, &wrong);
    let seen = until(&events, 1, ended);
    assert!(matches!(
        seen.last(),
        Some(Event::PerformanceEnded { outcome: Outcome::Completed, deviations, .. })
            if !deviations.is_empty()
    ));
    assert_eq!(wait_for(&port, 4), [OK, OK].concat());
    let messages = sent(&port);
    assert_eq!(messages[..2], messages[2..]);
}

/// Scenarios 2 and 5: one signal after five seamless performances, once the pianist stopped;
/// tones after the last performance are no abort.
#[test]
fn ok_once_after_stopping() {
    let (engine, events, port) = start("stopped", settings());
    let mut notes = seamless(5);
    let end = notes.last().unwrap().0;
    notes.extend(performance(end + 500_000)[..3].iter().copied());
    play(&engine, &notes);
    let seen = until(&events, 5, ended);
    assert!(seen.iter().filter(|e| ended(e)).all(|e| matches!(
        e,
        Event::PerformanceEnded {
            outcome: Outcome::Completed,
            ..
        }
    )));
    assert!(pitches(&port).is_empty());

    assert_eq!(wait_for(&port, 2), OK);
    thread::sleep(QUIET);
    assert_eq!(pitches(&port), OK);
}

/// Scenarios 3 and 4: the abort signal at once, no OK signal without a completed performance.
#[test]
fn abort_at_once_and_no_ok_without_completion() {
    let (engine, events, port) = start("abort", settings());
    play(&engine, &performance(0)[..30]);
    let seen = until(&events, 1, ended);
    assert!(matches!(
        seen.last(),
        Some(Event::PerformanceEnded {
            outcome: Outcome::Aborted,
            reason: Some(AbortReason::PauseLimit),
            ..
        })
    ));
    assert_eq!(wait_for(&port, 2), ABORT);
    thread::sleep(QUIET);
    assert_eq!(pitches(&port), ABORT);
}

/// Scenario 8: nothing is sent without connection.
#[test]
fn nothing_sent_without_connection() {
    let (engine, events, port) = start("lost", settings());
    play(&engine, &performance(0)[..30]);
    until(&events, 1, |e| {
        matches!(e, Event::PerformanceRecognized { .. })
    });
    engine.midi(MidiInput::DeviceRemoved { device: 1 });
    let seen = until(&events, 1, ended);
    assert!(matches!(
        seen.last(),
        Some(Event::PerformanceEnded {
            reason: Some(AbortReason::Interrupted),
            ..
        })
    ));
    thread::sleep(QUIET);
    assert!(pitches(&port).is_empty());
}

/// Scenario 9: the connection signal on selecting, reconnecting and restoring at start.
#[test]
fn connection_signal_on_select_reconnect_and_restore() {
    let name = "connect";
    let (engine, _events, port) = start(
        name,
        Settings {
            connect_signal: ConnectSignal::On,
            ..settings()
        },
    );
    assert_eq!(wait_for(&port, 2), [96, 96]);

    engine.midi(MidiInput::DeviceRemoved { device: 1 });
    engine.midi(MidiInput::DeviceAdded {
        device: 2,
        name: "Piano".into(),
        transport: Transport::Usb,
    });
    assert_eq!(wait_for(&port, 4), [96; 4]);
    assert!(port.calls().contains(&Call::Send(2, vec![0x90, 96, 30])));
    engine.reconnect();
    assert_eq!(wait_for(&port, 6), [96; 6]);
    drop(engine);

    let (_restarted, _events, port) = restart(name);
    assert_eq!(wait_for(&port, 2), [96, 96]);
}

/// Scenario 7: changed signals apply from the next performance on.
#[test]
fn changed_tones_apply_to_the_next_performance() {
    let (engine, events, port) = start(
        "changed",
        Settings {
            ok_signal: OkSignal::EveryPerformance,
            abort_signal: AbortSignal::Off,
            ..settings()
        },
    );
    play(&engine, &performance(0));
    until(&events, 1, ended);
    assert_eq!(wait_for(&port, 2), OK);

    let mut changed = engine.get_settings();
    changed.signal_tones = SignalTones {
        notes: vec![84],
        velocity: 50,
    };
    engine.set_settings(changed).unwrap();
    play(&engine, &performance(100_000_000));
    until(&events, 1, ended);
    wait_for(&port, 3);
    assert_eq!(sent(&port)[2], [0x90, 84, 50]);
}

/// Scenario 6 (FR-030): the echo of a signal tone is not playing.
#[test]
fn echo_of_a_signal_is_not_played() {
    let (engine, events, port) = start(
        "echo",
        Settings {
            connect_signal: ConnectSignal::On,
            ..settings()
        },
    );
    wait_for(&port, 1);
    engine.midi(MidiInput::Message {
        device: 1,
        time_us: 1_000,
        data: vec![0x90, 96, 30],
    });
    engine.get_settings();
    assert!(
        !events
            .try_iter()
            .any(|e| e == Event::PracticeSessionChanged { active: true })
    );

    engine.midi(MidiInput::Message {
        device: 1,
        time_us: 2_000,
        data: vec![0x90, 60, 64],
    });
    until(&events, 1, |e| {
        *e == Event::PracticeSessionChanged { active: true }
    });
}
