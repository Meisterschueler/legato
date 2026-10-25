//! MIDI input of the engine.

#[cfg(any(target_os = "windows", target_os = "linux"))]
pub(crate) mod midir_desktop;
pub mod sim;
pub mod state;
