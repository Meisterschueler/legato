//! Legato core: score model, work rules, score following, recognition and performances.
//!
//! Pure logic without I/O, clock or randomness, so that app, backend and web app
//! produce the same results (research.md R2).

/// Version of the core; stored with every evaluation.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
