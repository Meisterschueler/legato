//! Warning sound over the device speaker (FR-005).

use std::io::{self, Cursor};
use std::sync::mpsc::{self, Sender};
use std::thread;

const SOUND: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../app/assets/warning.wav"
));

/// Plays on its own thread, because the audio stream may not leave the thread that opened it.
pub(crate) struct Speaker(Sender<()>);

impl Speaker {
    pub fn start() -> io::Result<Speaker> {
        let (play, requests) = mpsc::channel::<()>();
        thread::Builder::new()
            .name("legato-warning".into())
            .spawn(move || {
                let mut sink = None;
                for () in requests {
                    // Opened on first use and kept; without an audio device only the vibration remains.
                    if sink.is_none() {
                        sink = rodio::DeviceSinkBuilder::open_default_sink()
                            .ok()
                            .map(|mut s| {
                                s.log_on_drop(false);
                                s
                            });
                    }
                    if let (Some(sink), Ok(sound)) =
                        (&sink, rodio::Decoder::try_from(Cursor::new(SOUND)))
                    {
                        sink.mixer().add(sound);
                    }
                }
            })?;
        Ok(Speaker(play))
    }

    pub fn play(&self) {
        let _ = self.0.send(());
    }
}
