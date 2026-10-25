//! Simulated MIDI port for tests: records what the engine sends to the MIDI layer.

use std::sync::{Arc, Mutex};

use crate::runtime::MidiOutput;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Call {
    Connect(u32),
    Disconnect(u32),
    Send(u32, Vec<u8>),
    Vibrate,
    ScanBluetooth(bool),
}

/// Clone it before handing it to the engine to inspect the calls.
#[derive(Clone, Default)]
pub struct SimulatedPort {
    calls: Arc<Mutex<Vec<Call>>>,
}

impl SimulatedPort {
    pub fn calls(&self) -> Vec<Call> {
        self.calls.lock().unwrap().clone()
    }
}

impl MidiOutput for SimulatedPort {
    fn connect(&mut self, device: u32) {
        self.calls.lock().unwrap().push(Call::Connect(device));
    }

    fn disconnect(&mut self, device: u32) {
        self.calls.lock().unwrap().push(Call::Disconnect(device));
    }

    fn send(&mut self, device: u32, data: &[u8]) {
        self.calls
            .lock()
            .unwrap()
            .push(Call::Send(device, data.to_vec()));
    }

    fn vibrate(&mut self) {
        self.calls.lock().unwrap().push(Call::Vibrate);
    }

    fn scan_bluetooth(&mut self, enabled: bool) {
        self.calls
            .lock()
            .unwrap()
            .push(Call::ScanBluetooth(enabled));
    }
}
