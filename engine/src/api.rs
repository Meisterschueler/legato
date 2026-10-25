//! Framework-neutral interface of the engine (contracts/engine-schnittstelle.md).

use std::io;
use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;

pub use crate::runtime::{Event, InputState, MidiInput, MidiOutput};
use crate::runtime::{Input, Runtime};
use crate::storage::Settings;

pub struct StartConfig {
    pub data_dir: PathBuf,
    pub resource_dir: PathBuf,
    pub locale: String,
}

pub struct Engine {
    inputs: Sender<Input>,
    events: Receiver<Event>,
}

impl Engine {
    /// Loads the settings and starts the runtime thread.
    pub fn start(config: StartConfig, output: Box<dyn MidiOutput>) -> io::Result<Engine> {
        let settings = Settings::load(&config.data_dir)?;
        let (inputs, receiver) = mpsc::channel();
        let (events_sender, events) = mpsc::channel();
        let runtime = Runtime {
            data_dir: config.data_dir,
            settings,
            output,
            events: events_sender,
        };
        thread::Builder::new()
            .name("legato-engine".into())
            .spawn(move || runtime.run(receiver))?;
        Ok(Engine { inputs, events })
    }

    /// Events in the order they occur.
    pub fn events(&self) -> &Receiver<Event> {
        &self.events
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

    /// Entry point for the MIDI layer; returns immediately.
    pub fn midi(&self, input: MidiInput) {
        self.send(Input::Midi(input));
    }

    fn send(&self, input: Input) {
        // The runtime only ends with the engine; a send error cannot occur.
        let _ = self.inputs.send(input);
    }
}
