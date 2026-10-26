//! Legato engine: framework-neutral runtime around `legato-core`
//! (contracts/engine-schnittstelle.md).

pub mod api;
pub mod ffi;
pub mod input;
pub mod repertoire;
mod runtime;
mod session;
mod signals;
pub mod storage;
mod warning;

pub use repertoire::{PieceInfo, Repertoire};

pub use legato_core::VERSION as CORE_VERSION;
