//! Framework-neutral interface of the engine (contracts/engine-schnittstelle.md).

use std::io;
use std::path::PathBuf;
use std::sync::Mutex;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::Instant;

use crate::input::state::InputMachine;
pub use crate::input::state::{InputInfo, InputList, InputState};
pub use crate::runtime::{Event, MidiInput, MidiOutput};
use crate::runtime::{Input, Runtime};
use crate::storage::Settings;
use crate::warning::Speaker;

pub struct StartConfig {
    pub data_dir: PathBuf,
    pub resource_dir: PathBuf,
    pub locale: String,
}

pub struct Engine {
    inputs: Sender<Input>,
    events: Mutex<Option<Receiver<Event>>>,
}

impl Engine {
    /// Loads the settings and starts the runtime thread.
    pub fn start(config: StartConfig, output: Box<dyn MidiOutput>) -> io::Result<Engine> {
        let settings = Settings::load(&config.data_dir)?;
        let (inputs, receiver) = mpsc::channel();
        let (events_sender, events) = mpsc::channel();
        let runtime = Runtime {
            data_dir: config.data_dir,
            input: InputMachine::new(settings.midi_device.clone()),
            settings,
            output,
            speaker: Speaker::start()?,
            events: events_sender,
            started: Instant::now(),
        };
        thread::Builder::new()
            .name("legato-engine".into())
            .spawn(move || runtime.run(receiver))?;
        Ok(Engine {
            inputs,
            events: Mutex::new(Some(events)),
        })
    }

    /// The event stream, in the order the events occur; there is one stream per engine.
    pub fn take_events(&self) -> Option<Receiver<Event>> {
        self.events.lock().unwrap_or_else(|e| e.into_inner()).take()
    }

    pub fn set_foreground(&self, foreground: bool) {
        self.send(Input::SetForeground(foreground));
    }

    pub fn get_settings(&self) -> Settings {
        let (reply, answer) = mpsc::channel();
        self.send(Input::GetSettings(reply));
        answer.recv().expect("engine runtime stopped")
    }

    /// Saves and applies the settings from the next performance on.
    pub fn set_settings(&self, settings: Settings) -> io::Result<Settings> {
        let (reply, answer) = mpsc::channel();
        self.send(Input::SetSettings(Box::new(settings), reply));
        answer.recv().expect("engine runtime stopped")
    }

    pub fn list_inputs(&self) -> InputList {
        let (reply, answer) = mpsc::channel();
        self.send(Input::ListInputs(reply));
        answer.recv().expect("engine runtime stopped")
    }

    /// Selects a present MIDI device or none and saves the choice (FR-002).
    pub fn select_input(&self, device: Option<u32>) -> io::Result<InputState> {
        let (reply, answer) = mpsc::channel();
        self.send(Input::Select(device, reply));
        answer.recv().expect("engine runtime stopped")
    }

    pub fn acknowledge_warning(&self) {
        self.send(Input::AcknowledgeWarning);
    }

    /// Entry point for the MIDI layer; returns immediately.
    pub fn midi(&self, input: MidiInput) {
        self.send(Input::Midi(input));
    }

    fn send(&self, input: Input) {
        // The runtime only ends with the engine; a send error cannot occur.
        let _ = self.inputs.send(input);
    }
}
