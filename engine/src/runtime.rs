//! Engine runtime: one queue for commands and input events, processed in order on one thread.

use std::io;
use std::path::PathBuf;
use std::sync::mpsc::{Receiver, Sender};

use serde::Serialize;

use crate::storage::{Settings, Transport};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum InputState {
    Disconnected,
    MidiConnected,
    ConnectionLost,
}

/// Events of the engine (contracts/engine-schnittstelle.md#ereignisse-event).
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(tag = "type")]
pub enum Event {
    InputStateChanged {
        state: InputState,
        device: Option<String>,
        receiving: bool,
        warning_active: bool,
    },
    BackgroundNotice {
        performance_discarded: bool,
    },
}

/// Input from the MIDI layer (native layer, desktop MIDI or simulated port).
#[derive(Clone, Debug, PartialEq)]
pub enum MidiInput {
    DeviceAdded {
        device: u32,
        name: String,
        transport: Transport,
    },
    DeviceRemoved {
        device: u32,
    },
    Message {
        device: u32,
        time_us: u64,
        data: Vec<u8>,
    },
}

/// Commands to the MIDI layer.
pub trait MidiOutput: Send {
    fn connect(&mut self, device: u32);
    fn send(&mut self, device: u32, data: &[u8]);
}

pub(crate) enum Input {
    SetForeground(bool),
    GetSettings(Sender<Settings>),
    SetSettings(Box<Settings>, Sender<io::Result<Settings>>),
    Midi(MidiInput),
}

pub(crate) struct Runtime {
    pub data_dir: PathBuf,
    pub settings: Settings,
    pub output: Box<dyn MidiOutput>,
    pub events: Sender<Event>,
}

impl Runtime {
    pub fn run(mut self, inputs: Receiver<Input>) {
        self.emit(Event::InputStateChanged {
            state: InputState::Disconnected,
            device: None,
            receiving: false,
            warning_active: false,
        });
        for input in inputs {
            self.handle(input);
        }
    }

    fn handle(&mut self, input: Input) {
        match input {
            Input::SetForeground(true) => {}
            // NFR-005: a running performance is discarded (performances follow in US 3).
            Input::SetForeground(false) => self.emit(Event::BackgroundNotice {
                performance_discarded: false,
            }),
            Input::GetSettings(reply) => {
                let _ = reply.send(self.settings.clone());
            }
            Input::SetSettings(settings, reply) => {
                let result = settings.save(&self.data_dir).map(|()| {
                    self.settings = *settings;
                    self.settings.clone()
                });
                let _ = reply.send(result);
            }
            // The input state machine follows with US 1; until then every device is connected.
            Input::Midi(MidiInput::DeviceAdded { device, .. }) => self.output.connect(device),
            Input::Midi(_) => {}
        }
    }

    fn emit(&self, event: Event) {
        // Nobody listening is not an error.
        let _ = self.events.send(event);
    }
}
