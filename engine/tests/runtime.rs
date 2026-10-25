use std::path::PathBuf;
use std::time::Duration;

use legato_engine::api::{Engine, Event, InputState, MidiInput, StartConfig};
use legato_engine::input::sim::{Call, SimulatedPort};
use legato_engine::storage::{Settings, Transport};

const TIMEOUT: Duration = Duration::from_secs(5);

fn start(name: &str) -> (Engine, SimulatedPort, PathBuf) {
    let data_dir =
        std::env::temp_dir().join(format!("legato-runtime-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&data_dir);
    let port = SimulatedPort::default();
    let engine = Engine::start(
        StartConfig {
            data_dir: data_dir.clone(),
            resource_dir: PathBuf::new(),
            locale: "de-DE".into(),
        },
        Box::new(port.clone()),
    )
    .unwrap();
    (engine, port, data_dir)
}

#[test]
fn starts_disconnected_and_reports_background() {
    let (engine, _, _) = start("events");
    let events = engine.take_events().unwrap();
    assert!(engine.take_events().is_none());
    assert_eq!(
        events.recv_timeout(TIMEOUT).unwrap(),
        Event::InputStateChanged {
            state: InputState::Disconnected,
            device: None,
            receiving: false,
            warning_active: false
        }
    );
    engine.set_foreground(false);
    assert_eq!(
        events.recv_timeout(TIMEOUT).unwrap(),
        Event::BackgroundNotice {
            performance_discarded: false
        }
    );
}

#[test]
fn settings_are_saved_and_survive_a_restart() {
    let (engine, _, data_dir) = start("settings");
    assert_eq!(engine.get_settings(), Settings::default());
    let changed = Settings {
        rest_limit_ms: 12000,
        ..Settings::default()
    };
    engine.set_settings(changed.clone()).unwrap();
    assert_eq!(engine.get_settings(), changed);
    drop(engine);

    let restarted = Engine::start(
        StartConfig {
            data_dir: data_dir.clone(),
            resource_dir: PathBuf::new(),
            locale: "de-DE".into(),
        },
        Box::new(SimulatedPort::default()),
    )
    .unwrap();
    assert_eq!(restarted.get_settings(), changed);
    std::fs::remove_dir_all(data_dir).unwrap();
}

#[test]
fn simulated_device_is_connected_in_order() {
    let (engine, port, _) = start("midi");
    engine.midi(MidiInput::DeviceAdded {
        device: 3,
        name: "Piano".into(),
        transport: Transport::Bluetooth,
    });
    engine.midi(MidiInput::Message {
        device: 3,
        time_us: 1_000,
        data: vec![0x90, 60, 80],
    });
    // A command after the MIDI input is answered only after the input was processed.
    engine.get_settings();
    assert_eq!(port.calls(), vec![Call::Connect(3)]);
}
