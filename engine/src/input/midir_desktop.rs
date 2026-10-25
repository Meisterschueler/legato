//! MIDI on Windows and Linux with `midir` inside the engine (research.md R4).
//! A thread polls the input ports every 250 ms; the runtime connects through `MidiOutput`.

use std::collections::BTreeMap;
use std::io;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex, MutexGuard, Weak};
use std::thread;
use std::time::Duration;

use midir::{Ignore, MidiInputConnection, MidiOutputConnection};

use crate::runtime::{Input, MidiInput, MidiOutput};
use crate::storage::Transport;

const POLL_INTERVAL: Duration = Duration::from_millis(250);
const CLIENT_NAME: &str = "Legato";

#[derive(Clone, Debug, PartialEq, Eq)]
struct Port {
    id: String,
    name: String,
}

#[derive(Default)]
struct Ports {
    /// Port id → device number; stable while the engine runs.
    numbers: BTreeMap<String, u32>,
    present: BTreeMap<u32, Port>,
    inputs: BTreeMap<u32, MidiInputConnection<()>>,
    outputs: BTreeMap<u32, MidiOutputConnection>,
}

impl Ports {
    /// Compares with the current port list; connections of removed ports are closed.
    fn update(&mut self, current: Vec<Port>) -> Vec<MidiInput> {
        let gone: Vec<u32> = self
            .present
            .iter()
            .filter(|(_, port)| !current.contains(port))
            .map(|(&device, _)| device)
            .collect();
        let mut changes = Vec::new();
        for device in gone {
            self.present.remove(&device);
            self.close(device);
            changes.push(MidiInput::DeviceRemoved { device });
        }
        for port in current {
            if self.present.values().any(|p| *p == port) {
                continue;
            }
            let next = self.numbers.len() as u32 + 1;
            let device = *self.numbers.entry(port.id.clone()).or_insert(next);
            changes.push(MidiInput::DeviceAdded {
                device,
                name: port.name.clone(),
                transport: transport(&port.id),
            });
            self.present.insert(device, port);
        }
        changes
    }

    /// Explicitly, because the WinRT backend of midir does not close on drop.
    fn close(&mut self, device: u32) {
        if let Some(connection) = self.inputs.remove(&device) {
            connection.close();
        }
        if let Some(connection) = self.outputs.remove(&device) {
            connection.close();
        }
    }
}

/// shortcut: Bluetooth is recognized only by the WinRT port id of BLE devices
/// (e.g. `\\?\SWD#MMDEVAPI#MIDII_3DF03567.BLE10#{...}`); extend for Linux if needed.
fn transport(port_id: &str) -> Transport {
    if port_id.to_ascii_uppercase().contains(".BLE") {
        Transport::Bluetooth
    } else {
        Transport::Usb
    }
}

pub(crate) struct DesktopMidi {
    ports: Arc<Mutex<Ports>>,
    inputs: Sender<Input>,
}

impl DesktopMidi {
    /// Starts polling; the thread ends with the runtime that owns this value.
    pub(crate) fn start(inputs: Sender<Input>) -> io::Result<DesktopMidi> {
        let ports = Arc::new(Mutex::new(Ports::default()));
        let weak = Arc::downgrade(&ports);
        let sender = inputs.clone();
        thread::Builder::new()
            .name("legato-midir".into())
            .spawn(move || poll(weak, sender))?;
        Ok(DesktopMidi { ports, inputs })
    }

    fn ports(&self) -> MutexGuard<'_, Ports> {
        self.ports.lock().unwrap_or_else(|e| e.into_inner())
    }
}

fn poll(ports: Weak<Mutex<Ports>>, inputs: Sender<Input>) {
    let Ok(midi) = midir::MidiInput::new(CLIENT_NAME) else {
        return;
    };
    loop {
        let current = midi
            .ports()
            .iter()
            .filter_map(|port| {
                Some(Port {
                    id: port.id(),
                    name: midi.port_name(port).ok()?,
                })
            })
            .collect();
        let Some(ports) = ports.upgrade() else {
            return;
        };
        let changes = ports
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .update(current);
        for change in changes {
            if inputs.send(Input::Midi(change)).is_err() {
                return;
            }
        }
        drop(ports);
        thread::sleep(POLL_INTERVAL);
    }
}

impl MidiOutput for DesktopMidi {
    fn connect(&mut self, device: u32) {
        let inputs = self.inputs.clone();
        let mut ports = self.ports();
        let Some(port) = ports.present.get(&device).cloned() else {
            return;
        };
        let input = midir::MidiInput::new(CLIENT_NAME)
            .ok()
            .and_then(|mut midi| {
                // Active sensing would count as "receiving".
                midi.ignore(Ignore::All);
                let found = midi.find_port_by_id(&port.id)?;
                midi.connect(
                    &found,
                    CLIENT_NAME,
                    move |time_us, data, _| {
                        let _ = inputs.send(Input::Midi(MidiInput::Message {
                            device,
                            time_us,
                            data: data.to_vec(),
                        }));
                    },
                    (),
                )
                .ok()
            });
        // The output for the signals (US 4) is the port with the same name.
        let output = midir::MidiOutput::new(CLIENT_NAME).ok().and_then(|midi| {
            let found = midi
                .ports()
                .into_iter()
                .find(|p| midi.port_name(p).is_ok_and(|name| name == port.name))?;
            midi.connect(&found, CLIENT_NAME).ok()
        });
        if let Some(connection) = input {
            ports.inputs.insert(device, connection);
        }
        if let Some(connection) = output {
            ports.outputs.insert(device, connection);
        }
    }

    fn disconnect(&mut self, device: u32) {
        self.ports().close(device);
    }

    fn send(&mut self, device: u32, data: &[u8]) {
        if let Some(connection) = self.ports().outputs.get_mut(&device) {
            let _ = connection.send(data);
        }
    }

    fn vibrate(&mut self) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    fn port(id: &str, name: &str) -> Port {
        Port {
            id: id.into(),
            name: name.into(),
        }
    }

    #[test]
    fn reports_changes_and_keeps_device_numbers() {
        let mut ports = Ports::default();
        let usb = port("usb-1", "Piano");
        let ble = port(
            r"\\?\SWD#MMDEVAPI#MIDII_3DF03567.BLE10#{504be32c-ccf6-4d2c-b73f-6f8b3747e22b}",
            "WIDI",
        );
        assert_eq!(
            ports.update(vec![usb.clone(), ble.clone()]),
            vec![
                MidiInput::DeviceAdded {
                    device: 1,
                    name: "Piano".into(),
                    transport: Transport::Usb
                },
                MidiInput::DeviceAdded {
                    device: 2,
                    name: "WIDI".into(),
                    transport: Transport::Bluetooth
                },
            ]
        );
        assert_eq!(ports.update(vec![usb.clone(), ble.clone()]), vec![]);
        assert_eq!(
            ports.update(vec![ble.clone()]),
            vec![MidiInput::DeviceRemoved { device: 1 }]
        );
        assert_eq!(
            ports.update(vec![ble, usb]),
            vec![MidiInput::DeviceAdded {
                device: 1,
                name: "Piano".into(),
                transport: Transport::Usb
            }]
        );
    }
}
