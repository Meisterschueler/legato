//! Legato engine: framework-neutral runtime around `legato-core`
//! (contracts/engine-schnittstelle.md).

pub mod api;
pub mod ffi;
pub mod input;
mod runtime;
pub mod storage;

pub use legato_core::VERSION as CORE_VERSION;
