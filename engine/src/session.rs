//! Practice session (T084): tones of the selected input → core; performance and session
//! events (FR-016, FR-020, FR-021, NFR-005).

use std::collections::BTreeMap;

use legato_core::performance::{AbortReason, Event as Performance, Limits, Performances, Setup};
use legato_core::preprocessing::{Grouping, Observation, Preprocessor, played_pitch};
use legato_core::recognition::plays;
use legato_core::score::model::{Candidate, Score};
use serde::Serialize;

use crate::runtime::Event;
use crate::storage::Settings;

/// Observation windows of `pthmm` (research.md R8).
const WINDOW: Grouping = Grouping::Window { period: 0.01 };
/// Wait after the last tone before its window is closed.
const FLUSH_MS: u64 = 20;
/// Clock heartbeat while a session runs (pause limits, idle timeout).
const TICK_MS: u64 = 250;

/// A piece for the recognition: its score and its permissible candidates.
#[derive(Clone)]
pub(crate) struct PieceSetups {
    pub id: String,
    pub score: Score,
    pub setups: Vec<Setup>,
    pub recognition_length: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum Outcome {
    Completed,
    Aborted,
}

pub(crate) struct Session {
    scores: BTreeMap<String, Score>,
    performances: Performances,
    preprocessor: Preprocessor,
    running: Option<Candidate>,
    /// Arrival minus driver time of the last tone (µs); maps the engine clock onto driver time.
    offset_us: i64,
    last_tone_ms: u64,
    ticked_ms: u64,
    pending: bool,
    active: bool,
    idle_timeout_ms: u64,
}

impl Session {
    /// Changes of the settings or pieces take effect with a new session object, i.e. from
    /// the next performance on.
    pub fn new(pieces: &[PieceSetups], settings: &Settings) -> Self {
        let filter = &settings.recognition_filter;
        let setups = pieces
            .iter()
            .filter(|p| filter.pieces.is_empty() || filter.pieces.contains(&p.id))
            .flat_map(|p| &p.setups)
            .filter(|s| {
                (filter.variations.is_empty() || filter.variations.contains(&s.candidate.variation))
                    && (filter.transpositions.is_empty()
                        || filter.transpositions.contains(&s.candidate.transposition))
            })
            .cloned()
            .collect();
        // shortcut: one recognition length for all pieces (every work has 8 so far).
        let recognition_length = pieces
            .iter()
            .map(|p| p.recognition_length)
            .max()
            .unwrap_or(8);
        let limits = Limits {
            pause: f64::from(settings.pause_limit_ms) / 1000.0,
            rest: f64::from(settings.rest_limit_ms) / 1000.0,
        };
        Session {
            scores: pieces
                .iter()
                .map(|p| (p.id.clone(), p.score.clone()))
                .collect(),
            performances: Performances::new(setups, recognition_length, limits),
            preprocessor: Preprocessor::new(WINDOW),
            running: None,
            offset_us: 0,
            last_tone_ms: 0,
            ticked_ms: 0,
            pending: false,
            active: false,
            idle_timeout_ms: u64::from(settings.session_idle_timeout_ms),
        }
    }

    /// A MIDI message of the selected input; only note-ons count (FR-003).
    pub fn message(&mut self, data: &[u8], driver_us: u64, now_ms: u64) -> Vec<Event> {
        let Some(pitch) = played_pitch(data) else {
            return Vec::new();
        };
        let mut events = Vec::new();
        if !self.active {
            self.active = true;
            events.push(Event::PracticeSessionChanged { active: true });
        }
        self.offset_us = now_ms as i64 * 1000 - driver_us as i64;
        self.last_tone_ms = now_ms;
        self.ticked_ms = now_ms;
        self.pending = true;
        if let Some(observation) = self.preprocessor.note_on(driver_us as f64 / 1e6, pitch) {
            events.extend(self.observe(&observation));
        }
        events
    }

    pub fn next_deadline(&self) -> Option<u64> {
        self.active.then_some(if self.pending {
            self.last_tone_ms + FLUSH_MS
        } else {
            self.ticked_ms + TICK_MS
        })
    }

    pub fn tick(&mut self, now_ms: u64) -> Vec<Event> {
        if !self.active {
            return Vec::new();
        }
        self.ticked_ms = now_ms;
        let now = (now_ms as i64 * 1000 - self.offset_us) as f64 / 1e6;
        let mut events = Vec::new();
        if self.pending && now_ms >= self.last_tone_ms + FLUSH_MS {
            self.pending = false;
            if let Some(observation) = self.preprocessor.tick(now) {
                events.extend(self.observe(&observation));
            }
        }
        for performance in self.performances.tick(now) {
            events.extend(self.convert(performance, now));
        }
        if now_ms.saturating_sub(self.last_tone_ms) > self.idle_timeout_ms {
            self.active = false;
            events.push(Event::PracticeSessionChanged { active: false });
        }
        events
    }

    /// Input changed or lost: a running performance is aborted (FR-002, FR-004).
    pub fn interrupt(&mut self) -> Vec<Event> {
        self.preprocessor = Preprocessor::new(WINDOW);
        self.pending = false;
        let ended = self.performances.interrupt();
        ended
            .into_iter()
            .flat_map(|p| self.convert(p, 0.0))
            .collect()
    }

    /// The app went to the background: the performance is discarded and the session ends (NFR-005).
    pub fn background(&mut self) -> Vec<Event> {
        let mut events = self.interrupt();
        if self.active {
            self.active = false;
            events.push(Event::PracticeSessionChanged { active: false });
        }
        events
    }

    fn observe(&mut self, observation: &Observation) -> Vec<Event> {
        let performances = self.performances.observe(observation);
        performances
            .into_iter()
            .flat_map(|p| self.convert(p, observation.time))
            .collect()
    }

    /// `time`: driver time (seconds) of the observation behind the event.
    fn convert(&mut self, performance: Performance, time: f64) -> Option<Event> {
        Some(match performance {
            Performance::Recognized(candidate) => {
                let event = Event::PerformanceRecognized {
                    piece: candidate.piece.clone(),
                    variation: candidate.variation,
                    transposition: candidate.transposition,
                };
                self.running = Some(candidate);
                event
            }
            Performance::Position(index) => {
                let candidate = self.running.as_ref()?;
                let onset = &self.scores.get(&candidate.piece)?.onsets[index as usize];
                Event::Position {
                    note_ids: onset
                        .notes
                        .iter()
                        .filter(|n| plays(candidate.variation, n.hand))
                        .map(|n| n.display_id(candidate.transposition))
                        .collect(),
                    driver_time_us: (time * 1e6) as u64,
                }
            }
            Performance::Completed => self.ended(Outcome::Completed, None),
            Performance::Aborted(reason) => self.ended(Outcome::Aborted, Some(reason)),
        })
    }

    fn ended(&mut self, outcome: Outcome, reason: Option<AbortReason>) -> Event {
        self.running = None;
        Event::PerformanceEnded { outcome, reason }
    }
}
