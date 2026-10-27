//! Framework-neutral interface of the engine (contracts/engine-schnittstelle.md).

use std::io;
use std::path::PathBuf;
use std::sync::Mutex;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::Instant;

use std::path::Path;

use legato_core::score::model::Variation;
use legato_core::works::DisplayVariant;

use crate::input::state::InputMachine;
pub use crate::input::state::{InputInfo, InputList, InputState};
pub use crate::repertoire::PieceInfo;
use crate::repertoire::Repertoire;
pub use crate::runtime::{Event, MidiInput, MidiOutput};
use crate::runtime::{Input, Runtime};
use crate::session::Session;
pub use crate::session::{DeviationInfo, FollowedCandidate, Outcome};
use crate::signals::SignalPlanner;
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
    repertoire: Mutex<Repertoire>,
}

impl Engine {
    /// Loads the settings and starts the runtime thread.
    pub fn start(config: StartConfig, output: Box<dyn MidiOutput>) -> io::Result<Engine> {
        Self::start_with(config, |_| Ok(output))
    }

    /// Windows and Linux: MIDI through `midir` inside the engine (research.md R4).
    #[cfg(any(target_os = "windows", target_os = "linux"))]
    pub fn start_desktop(config: StartConfig) -> io::Result<Engine> {
        Self::start_with(config, |inputs| {
            Ok(Box::new(crate::input::midir_desktop::DesktopMidi::start(
                inputs,
            )?))
        })
    }

    fn start_with(
        config: StartConfig,
        output: impl FnOnce(Sender<Input>) -> io::Result<Box<dyn MidiOutput>>,
    ) -> io::Result<Engine> {
        let settings = Settings::load(&config.data_dir)?;
        let repertoire = Repertoire::load(&config.data_dir, &config.resource_dir)?;
        let (inputs, receiver) = mpsc::channel();
        let (events_sender, events) = mpsc::channel();
        let pieces = repertoire.setups();
        let runtime = Runtime {
            data_dir: config.data_dir,
            input: InputMachine::new(settings.midi_device.clone()),
            session: Session::new(&pieces, &settings),
            signals: SignalPlanner::default(),
            pieces,
            settings,
            output: output(inputs.clone())?,
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
            repertoire: Mutex::new(repertoire),
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

    /// Reopens the selected device if it is present (FR-006).
    pub fn reconnect(&self) -> InputState {
        let (reply, answer) = mpsc::channel();
        self.send(Input::Reconnect(reply));
        answer.recv().expect("engine runtime stopped")
    }

    pub fn scan_bluetooth(&self, enabled: bool) {
        self.send(Input::ScanBluetooth(enabled));
    }

    pub fn repertoire(&self) -> Vec<PieceInfo> {
        self.repertoire
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .list()
    }

    pub fn load_piece(
        &self,
        path: impl AsRef<Path>,
    ) -> Result<PieceInfo, legato_core::score::musicxml_read::InvalidScoreFile> {
        let mut repertoire = self.repertoire.lock().unwrap_or_else(|e| e.into_inner());
        let piece = repertoire.load_piece(path.as_ref())?;
        self.send(Input::Pieces(repertoire.setups()));
        Ok(piece)
    }

    pub fn render_score(
        &self,
        piece_id: &str,
        transposition: i8,
        variation: Variation,
        display_variant: DisplayVariant,
    ) -> Option<String> {
        self.repertoire
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .render_score(piece_id, transposition, variation, display_variant)
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
