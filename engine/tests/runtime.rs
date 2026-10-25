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
fn messages_without_selection_connect_nothing() {
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
    assert_eq!(port.calls(), vec![]);
}

fn next_state(events: &std::sync::mpsc::Receiver<Event>) -> (InputState, bool, bool) {
    loop {
        if let Event::InputStateChanged {
            state,
            receiving,
            warning_active,
            ..
        } = events.recv_timeout(TIMEOUT).unwrap()
        {
            return (state, receiving, warning_active);
        }
    }
}

#[test]
fn selected_input_warns_on_loss_reconnects_and_is_restored() {
    let (engine, port, data_dir) = start("select");
    let events = engine.take_events().unwrap();
    next_state(&events);
    engine.midi(MidiInput::DeviceAdded {
        device: 3,
        name: "Piano".into(),
        transport: Transport::Usb,
    });
    let list = engine.list_inputs();
    assert_eq!(list.midi.len(), 1);
    assert_eq!(list.selected, None);
    assert_eq!(
        engine.select_input(Some(3)).unwrap(),
        InputState::MidiConnected
    );
    assert_eq!(
        next_state(&events),
        (InputState::MidiConnected, false, false)
    );
    engine.midi(MidiInput::Message {
        device: 3,
        time_us: 1_000,
        data: vec![0x90, 60, 80],
    });
    assert_eq!(
        next_state(&events),
        (InputState::MidiConnected, true, false)
    );

    engine.midi(MidiInput::DeviceRemoved { device: 3 });
    // SC-003: the warning goes out right with the state change.
    assert_eq!(
        next_state(&events),
        (InputState::ConnectionLost, false, true)
    );
    assert_eq!(port.calls(), vec![Call::Connect(3), Call::Vibrate]);
    engine.acknowledge_warning();
    assert_eq!(
        next_state(&events),
        (InputState::ConnectionLost, false, false)
    );
    engine.midi(MidiInput::DeviceAdded {
        device: 4,
        name: "Piano".into(),
        transport: Transport::Usb,
    });
    assert_eq!(
        next_state(&events),
        (InputState::MidiConnected, false, false)
    );
    assert_eq!(port.calls().last(), Some(&Call::Connect(4)));
    drop(engine);

    let port = SimulatedPort::default();
    let restarted = Engine::start(
        StartConfig {
            data_dir: data_dir.clone(),
            resource_dir: PathBuf::new(),
            locale: "de-DE".into(),
        },
        Box::new(port.clone()),
    )
    .unwrap();
    restarted.midi(MidiInput::DeviceAdded {
        device: 9,
        name: "Piano".into(),
        transport: Transport::Usb,
    });
    assert_eq!(
        restarted.list_inputs().selected.map(|d| d.name),
        Some("Piano".to_string())
    );
    assert_eq!(port.calls(), vec![Call::Connect(9)]);
    std::fs::remove_dir_all(data_dir).unwrap();
}
