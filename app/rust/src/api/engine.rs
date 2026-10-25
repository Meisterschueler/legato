//! Engine interface for Flutter. Settings and events cross the bridge as JSON
//! (contracts/engine-schnittstelle.md); the Dart side holds no domain logic.

use std::path::PathBuf;

use flutter_rust_bridge::frb;
use legato_engine::api::{Engine, MidiOutput, StartConfig};
use legato_engine::storage::Settings;

use crate::frb_generated::StreamSink;

/// MIDI output until the native layer is connected (US 1).
struct NoMidi;

impl MidiOutput for NoMidi {
    fn connect(&mut self, _device: u32) {}
    fn disconnect(&mut self, _device: u32) {}
    fn send(&mut self, _device: u32, _data: &[u8]) {}
    fn warn(&mut self) {}
}

#[frb(opaque)]
pub struct LegatoEngine {
    engine: Engine,
}

impl LegatoEngine {
    pub fn start(
        data_dir: String,
        resource_dir: String,
        locale: String,
    ) -> Result<LegatoEngine, String> {
        let config = StartConfig {
            data_dir: PathBuf::from(data_dir),
            resource_dir: PathBuf::from(resource_dir),
            locale,
        };
        let engine = Engine::start(config, Box::new(NoMidi)).map_err(|e| e.to_string())?;
        Ok(LegatoEngine { engine })
    }

    /// Events as JSON objects with a `type` field; can be subscribed once.
    pub fn events(&self, sink: StreamSink<String>) -> Result<(), String> {
        let events = self
            .engine
            .take_events()
            .ok_or("events are already subscribed")?;
        std::thread::spawn(move || {
            for event in events {
                let json = serde_json::to_string(&event).expect("events are serializable");
                if sink.add(json).is_err() {
                    break;
                }
            }
        });
        Ok(())
    }

    #[frb(sync)]
    pub fn set_foreground(&self, foreground: bool) {
        self.engine.set_foreground(foreground);
    }

    pub fn get_settings(&self) -> String {
        serde_json::to_string(&self.engine.get_settings()).expect("settings are serializable")
    }

    /// Returns the settings as saved.
    pub fn set_settings(&self, json: String) -> Result<String, String> {
        let settings: Settings = serde_json::from_str(&json).map_err(|e| e.to_string())?;
        let saved = self
            .engine
            .set_settings(settings)
            .map_err(|e| e.to_string())?;
        Ok(serde_json::to_string(&saved).expect("settings are serializable"))
    }
}
