use legato_engine::input::state::{Action, InputMachine, InputState};
use legato_engine::storage::{MidiDevice, Transport};

fn piano() -> MidiDevice {
    MidiDevice {
        name: "Piano".into(),
        transport: Transport::Usb,
        bluetooth_id: None,
    }
}

fn connected() -> InputMachine {
    let mut input = InputMachine::new(None);
    input.device_added(1, "Piano".into(), Transport::Usb);
    assert_eq!(input.select(Some(1)), vec![Action::Connect(1)]);
    input
}

#[test]
fn without_selection_nothing_is_assigned_or_warned() {
    let mut input = InputMachine::new(None);
    assert_eq!(
        input.device_added(1, "Piano".into(), Transport::Usb),
        vec![]
    );
    assert!(!input.message(1));
    assert_eq!(input.device_removed(1, 0), vec![]);
    assert_eq!(input.status().state, InputState::Disconnected);
    assert_eq!(input.next_deadline(), None);
}

#[test]
fn messages_count_only_from_the_selected_device() {
    let mut input = connected();
    input.device_added(2, "Other".into(), Transport::Usb);
    assert!(!input.message(2));
    assert!(!input.status().receiving);
    assert!(input.message(1));
    assert!(input.status().receiving);
}

#[test]
fn disconnect_warns_immediately_then_after_10_and_30_seconds() {
    let mut input = connected();
    assert_eq!(input.device_removed(1, 5_000), vec![Action::Warn]);
    let status = input.status();
    assert_eq!(status.state, InputState::ConnectionLost);
    assert!(status.warning_active);
    assert_eq!(input.tick(14_999), vec![]);
    assert_eq!(input.tick(15_000), vec![Action::Warn]);
    assert_eq!(input.next_deadline(), Some(35_000));
    assert_eq!(input.tick(35_000), vec![Action::Warn]);
    // Afterwards only the symbol shows the loss.
    assert_eq!(input.tick(1_000_000), vec![]);
    assert_eq!(input.next_deadline(), None);
    assert!(!input.status().warning_active);
    assert_eq!(input.status().state, InputState::ConnectionLost);
}

#[test]
fn acknowledged_warning_is_not_repeated() {
    let mut input = connected();
    input.device_removed(1, 0);
    input.tick(10_000);
    input.acknowledge_warning();
    assert_eq!(input.tick(30_000), vec![]);
    assert_eq!(input.status().state, InputState::ConnectionLost);
}

#[test]
fn returning_device_is_reconnected_and_ends_the_warning() {
    let mut input = connected();
    input.device_removed(1, 0);
    // The platform may assign a new id.
    assert_eq!(
        input.device_added(7, "Piano".into(), Transport::Usb),
        vec![Action::Connect(7)]
    );
    assert_eq!(input.status().state, InputState::MidiConnected);
    assert_eq!(input.tick(30_000), vec![]);
}

#[test]
fn restored_input_connects_when_present_without_warning_before() {
    let mut input = InputMachine::new(Some(piano()));
    let status = input.status();
    assert_eq!(status.state, InputState::Disconnected);
    assert_eq!(status.device.as_deref(), Some("Piano"));
    assert!(!status.warning_active);
    assert_eq!(
        input.device_added(3, "Piano".into(), Transport::Usb),
        vec![Action::Connect(3)]
    );
    // Same name over another transport is another device.
    let mut input = InputMachine::new(Some(piano()));
    assert_eq!(
        input.device_added(4, "Piano".into(), Transport::Bluetooth),
        vec![]
    );
}

#[test]
fn switching_input_disconnects_the_old_device() {
    let mut input = connected();
    input.device_added(2, "Other".into(), Transport::Bluetooth);
    assert_eq!(
        input.select(Some(2)),
        vec![Action::Disconnect(1), Action::Connect(2)]
    );
    assert_eq!(input.selected().map(|d| d.name.as_str()), Some("Other"));
    assert_eq!(input.select(None), vec![Action::Disconnect(2)]);
    assert_eq!(input.status().state, InputState::Disconnected);
    assert_eq!(input.select(Some(99)), vec![]);
}

#[test]
fn reconnect_reopens_the_selected_device_if_present() {
    let mut input = connected();
    assert_eq!(
        input.reconnect(),
        vec![Action::Disconnect(1), Action::Connect(1)]
    );
    input.device_removed(1, 0);
    assert_eq!(input.reconnect(), vec![]);
    assert_eq!(input.status().state, InputState::ConnectionLost);
    assert_eq!(InputMachine::new(None).reconnect(), vec![]);
}

#[test]
fn selecting_during_connection_loss_ends_the_warning() {
    let mut input = connected();
    input.device_added(2, "Other".into(), Transport::Usb);
    input.device_removed(1, 0);
    assert_eq!(input.select(Some(2)), vec![Action::Connect(2)]);
    assert!(!input.status().warning_active);
    assert_eq!(input.tick(30_000), vec![]);
}
