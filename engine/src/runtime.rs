//! Engine runtime: one queue for commands and input events, processed in order on one thread.

use std::io;
use std::path::PathBuf;
use std::sync::mpsc::{Receiver, RecvTimeoutError, Sender};
use std::time::{Duration, Instant};

use serde::Serialize;

use crate::input::state::{Action, InputList, InputMachine, InputState, Status};
use crate::storage::{Settings, Transport};

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
    fn disconnect(&mut self, device: u32);
    fn send(&mut self, device: u32, data: &[u8]);
    /// Warning sound over the device speaker plus vibration where available (FR-005).
    fn warn(&mut self);
}

pub(crate) enum Input {
    SetForeground(bool),
    GetSettings(Sender<Settings>),
    SetSettings(Box<Settings>, Sender<io::Result<Settings>>),
    ListInputs(Sender<InputList>),
    Select(Option<u32>, Sender<io::Result<InputState>>),
    AcknowledgeWarning,
    Midi(MidiInput),
}

pub(crate) struct Runtime {
    pub data_dir: PathBuf,
    pub settings: Settings,
    pub output: Box<dyn MidiOutput>,
    pub events: Sender<Event>,
    pub input: InputMachine,
    pub started: Instant,
}

impl Runtime {
    pub fn run(mut self, inputs: Receiver<Input>) {
        let mut reported = self.input.status();
        self.emit_status(&reported);
        loop {
            let received = match self.input.next_deadline() {
                Some(due) => {
                    inputs.recv_timeout(Duration::from_millis(due.saturating_sub(self.now_ms())))
                }
                None => inputs.recv().map_err(|_| RecvTimeoutError::Disconnected),
            };
            match received {
                Ok(input) => self.handle(input),
                Err(RecvTimeoutError::Timeout) => {}
                Err(RecvTimeoutError::Disconnected) => break,
            }
            let actions = self.input.tick(self.now_ms());
            self.apply(actions);
            let status = self.input.status();
            if status != reported {
                self.emit_status(&status);
                reported = status;
            }
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
            Input::SetSettings(mut settings, reply) => {
                // The input is changed only through `select_input`.
                settings.midi_device = self.input.selected().cloned();
                let result = settings.save(&self.data_dir).map(|()| {
                    self.settings = *settings;
                    self.settings.clone()
                });
                let _ = reply.send(result);
            }
            Input::ListInputs(reply) => {
                let _ = reply.send(self.input.list());
            }
            Input::Select(device, reply) => {
                let actions = self.input.select(device);
                self.apply(actions);
                let mut settings = self.settings.clone();
                settings.midi_device = self.input.selected().cloned();
                let result = settings.save(&self.data_dir).map(|()| {
                    self.settings = settings;
                    self.input.status().state
                });
                let _ = reply.send(result);
            }
            Input::AcknowledgeWarning => self.input.acknowledge_warning(),
            Input::Midi(MidiInput::DeviceAdded {
                device,
                name,
                transport,
            }) => {
                let actions = self.input.device_added(device, name, transport);
                self.apply(actions);
            }
            Input::Midi(MidiInput::DeviceRemoved { device }) => {
                let actions = self.input.device_removed(device, self.now_ms());
                self.apply(actions);
            }
            // Assignment to the score follows with US 3.
            Input::Midi(MidiInput::Message { device, .. }) => {
                self.input.message(device);
            }
        }
    }

    fn apply(&mut self, actions: Vec<Action>) {
        for action in actions {
            match action {
                Action::Connect(device) => self.output.connect(device),
                Action::Disconnect(device) => self.output.disconnect(device),
                Action::Warn => self.output.warn(),
            }
        }
    }

    fn now_ms(&self) -> u64 {
        self.started.elapsed().as_millis() as u64
    }

    fn emit_status(&self, status: &Status) {
        self.emit(Event::InputStateChanged {
            state: status.state,
            device: status.device.clone(),
            receiving: status.receiving,
            warning_active: status.warning_active,
        });
    }

    fn emit(&self, event: Event) {
        // Nobody listening is not an error.
        let _ = self.events.send(event);
    }
}
