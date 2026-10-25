//! MIDI on Windows and Linux with `midir` inside the engine (research.md R4).
//! A thread polls the input ports every 250 ms; the runtime connects through `MidiOutput`.

use std::collections::{BTreeMap, BTreeSet};
use std::io;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex, MutexGuard, Weak};
use std::thread;
use std::time::{Duration, Instant};

use midir::{Ignore, MidiInputConnection, MidiOutputConnection};

use crate::runtime::{Input, MidiInput, MidiOutput};
use crate::storage::Transport;

const POLL_INTERVAL: Duration = Duration::from_millis(250);
const CLIENT_NAME: &str = "Legato";
/// Larger differences between driver and arrival intervals are driver errors, not delivery delay.
const MAX_DRIFT_US: i64 = 1_000_000;

/// Maps the driver timestamps of one connection onto a common clock. Each connection starts
/// at its own zero, and Windows 10 lets Bluetooth timestamps jump by about 8.1 s
/// (research.md, Spike-Ergebnis S1 unter Windows); then the arrival time is taken anew.
#[derive(Default)]
struct Clock {
    offset: i64,
    last: Option<(u64, u64)>,
    last_out: u64,
}

impl Clock {
    fn map(&mut self, driver_us: u64, arrival_us: u64) -> u64 {
        let jumped = self.last.is_none_or(|(driver, arrival)| {
            let driver_step = driver_us as i64 - driver as i64;
            let arrival_step = arrival_us as i64 - arrival as i64;
            (driver_step - arrival_step).abs() > MAX_DRIFT_US
        });
        if jumped {
            self.offset = arrival_us as i64 - driver_us as i64;
        }
        self.last = Some((driver_us, arrival_us));
        self.last_out = ((driver_us as i64 + self.offset).max(0) as u64).max(self.last_out);
        self.last_out
    }
}

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
    /// Devices the runtime connected; an opening that finishes later is kept only for these.
    wanted: BTreeSet<u32>,
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
    epoch: Instant,
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
        Ok(DesktopMidi {
            ports,
            inputs,
            epoch: Instant::now(),
        })
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
    /// Opens in the background: WinRT blocks for Bluetooth devices that are paired but not
    /// reachable, and the runtime must not wait for that.
    /// shortcut: such an opening never returns and its thread stays blocked; replace when
    /// Bluetooth on Windows gets its own connection handling (T105).
    fn connect(&mut self, device: u32) {
        let Some(port) = self.ports().present.get(&device).cloned() else {
            return;
        };
        self.ports().wanted.insert(device);
        let ports = Arc::downgrade(&self.ports);
        let inputs = self.inputs.clone();
        let epoch = self.epoch;
        let _ = thread::Builder::new()
            .name("legato-midir-open".into())
            .spawn(move || {
                let (input, output) = open(device, &port, inputs, epoch);
                let Some(ports) = ports.upgrade() else {
                    return;
                };
                let mut ports = ports.lock().unwrap_or_else(|e| e.into_inner());
                let keep = ports.wanted.contains(&device)
                    && ports.present.contains_key(&device)
                    && !ports.inputs.contains_key(&device);
                if let Some(connection) = input {
                    if keep {
                        ports.inputs.insert(device, connection);
                    } else {
                        connection.close();
                    }
                }
                if let Some(connection) = output {
                    if keep {
                        ports.outputs.insert(device, connection);
                    } else {
                        connection.close();
                    }
                }
            });
    }

    fn disconnect(&mut self, device: u32) {
        let mut ports = self.ports();
        ports.wanted.remove(&device);
        ports.close(device);
    }

    fn send(&mut self, device: u32, data: &[u8]) {
        if let Some(connection) = self.ports().outputs.get_mut(&device) {
            let _ = connection.send(data);
        }
    }

    fn vibrate(&mut self) {}
}

fn open(
    device: u32,
    port: &Port,
    inputs: Sender<Input>,
    epoch: Instant,
) -> (
    Option<MidiInputConnection<()>>,
    Option<MidiOutputConnection>,
) {
    let mut clock = Clock::default();
    let input = midir::MidiInput::new(CLIENT_NAME)
        .ok()
        .and_then(|mut midi| {
            // Active sensing would count as "receiving".
            midi.ignore(Ignore::All);
            let found = midi.find_port_by_id(&port.id)?;
            midi.connect(
                &found,
                CLIENT_NAME,
                move |driver_us, data, _| {
                    let time_us = clock.map(driver_us, epoch.elapsed().as_micros() as u64);
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
    (input, output)
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
    fn clock_keeps_driver_steps_and_reanchors_after_jumps() {
        // (arrival ms, driver us) measured with midi_probe over Bluetooth.
        let mut clock = Clock::default();
        let mut map = |arrival_ms: u64, driver_us: u64| clock.map(driver_us, arrival_ms * 1000);
        assert_eq!(map(2393, 2_309_000), 2_393_000);
        // Delivery is bunched; the driver steps stay.
        assert_eq!(map(6713, 6_609_000), 6_693_000);
        assert_eq!(map(6713, 6_610_000), 6_694_000);
        // Jump by about 8.1 s within a chord: take the arrival time anew.
        assert_eq!(map(6714, 14_685_000), 6_714_000);
        assert_eq!(map(7556, 15_632_000), 7_661_000);
        // Never backwards.
        assert_eq!(map(7557, 5_000_000), 7_661_000);
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
