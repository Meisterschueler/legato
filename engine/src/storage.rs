//! Settings (data-model.md#settings-einstellungen-persistiert), stored as JSON in `data_dir`.

use std::collections::BTreeMap;
use std::io;
use std::path::Path;

use legato_core::score::model::Variation;
use legato_core::works::DisplayVariant;
use serde::{Deserialize, Serialize};

const FILE_NAME: &str = "settings.json";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Transport {
    Usb,
    Bluetooth,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MidiDevice {
    pub name: String,
    pub transport: Transport,
    pub bluetooth_id: Option<String>,
}

/// Restricts the recognition (FR-018); empty lists allow everything.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecognitionFilter {
    pub pieces: Vec<String>,
    pub variations: Vec<Variation>,
    pub transpositions: Vec<i8>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScoreFollowerKind {
    Pthmm,
    OuterHmm,
    Arzt,
}

/// Error marking per deviation kind.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ErrorMarking {
    pub omission: bool,
    pub wrong_note: bool,
    pub addition: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConnectSignal {
    On,
    Off,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum OkSignal {
    WhenStopped,
    EveryPerformance,
    Off,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AbortSignal {
    Immediately,
    Off,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignalTones {
    /// At most 2 notes (MIDI numbers).
    pub notes: Vec<u8>,
    pub velocity: u8,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub midi_device: Option<MidiDevice>,
    /// `None`: the MIDI device (US 1, scenario 14).
    pub signal_device: Option<MidiDevice>,
    pub recognition_filter: RecognitionFilter,
    pub pause_limit_ms: u32,
    pub rest_limit_ms: u32,
    pub session_idle_timeout_ms: u32,
    pub score_follower: ScoreFollowerKind,
    /// Per work id; missing works show `Complete`.
    pub display_variant: BTreeMap<String, DisplayVariant>,
    pub error_marking_during: ErrorMarking,
    pub error_marking_after: ErrorMarking,
    pub connect_signal: ConnectSignal,
    pub ok_signal: OkSignal,
    pub abort_signal: AbortSignal,
    pub signal_tones: SignalTones,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            midi_device: None,
            signal_device: None,
            recognition_filter: RecognitionFilter::default(),
            pause_limit_ms: 4000,
            rest_limit_ms: 10000,
            session_idle_timeout_ms: 600_000,
            // Matchmaker's default for MIDI; final choice after spike S3 (FR-032).
            score_follower: ScoreFollowerKind::Pthmm,
            display_variant: BTreeMap::new(),
            error_marking_during: ErrorMarking::default(),
            error_marking_after: ErrorMarking::default(),
            connect_signal: ConnectSignal::On,
            ok_signal: OkSignal::WhenStopped,
            abort_signal: AbortSignal::Immediately,
            // C7 and E7, quiet.
            signal_tones: SignalTones {
                notes: vec![96, 100],
                velocity: 30,
            },
        }
    }
}

impl Settings {
    /// Defaults if there is no file yet; an unreadable file is an error, so it is not overwritten.
    pub fn load(data_dir: &Path) -> io::Result<Settings> {
        match std::fs::read(data_dir.join(FILE_NAME)) {
            Ok(bytes) => serde_json::from_slice(&bytes).map_err(io::Error::other),
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(Settings::default()),
            Err(e) => Err(e),
        }
    }

    /// Writes to a temporary file first, so a crash cannot leave a half-written file.
    pub fn save(&self, data_dir: &Path) -> io::Result<()> {
        std::fs::create_dir_all(data_dir)?;
        let temporary = data_dir.join(format!("{FILE_NAME}.tmp"));
        std::fs::write(
            &temporary,
            serde_json::to_vec_pretty(self).map_err(io::Error::other)?,
        )?;
        std::fs::rename(temporary, data_dir.join(FILE_NAME))
    }
}
