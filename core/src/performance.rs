//! Performance lifecycle (data-model.md#performance-durchlauf): recognition, following,
//! completion and aborts (FR-016, FR-020, FR-021).

use std::collections::VecDeque;

use crate::following::deviations::{Deviation, Deviations};
use crate::following::pthmm::PitchHmm;
use crate::following::repeats::Repeats;
use crate::following::{ReferenceNote, ScoreFollower};
use crate::preprocessing::Observation;
use crate::recognition::{Decision, Recognition, hands, plays};
use crate::score::model::{Candidate, Note, Score};
use crate::works::Range;

/// Share of the last tones that must be assigned (FR-021).
const MIN_ASSIGNED: f64 = 0.3;

/// One onset as a candidate expects it.
#[derive(Clone, Debug, PartialEq)]
pub struct Expected {
    /// Onset index in the score.
    pub index: u32,
    /// Quarter notes since the start of the piece.
    pub beat: f64,
    /// Transposed pitches of the candidate's hands.
    pub pitches: Vec<u8>,
    /// The notes behind `pitches`, as notated.
    pub notes: Vec<Note>,
    /// A notated rest precedes the onset (FR-020).
    pub rest_before: bool,
}

/// A candidate with its expected onsets.
#[derive(Clone, Debug, PartialEq)]
pub struct Setup {
    pub candidate: Candidate,
    pub onsets: Vec<Expected>,
    /// Repeat sections as positions in `onsets` (FR-014).
    pub repeats: Repeats,
}

impl Setup {
    /// Onsets of `range` without notes of the candidate's hands are left out. A repeat from the
    /// last onset of `range` is a new performance, not an edge (Hanon).
    pub fn new(score: &Score, candidate: Candidate, range: Range) -> Self {
        let onsets: Vec<Expected> = score.onsets[range.start as usize..=range.end as usize]
            .iter()
            .filter_map(|onset| {
                let notes: Vec<Note> = onset
                    .notes
                    .iter()
                    .filter(|n| plays(candidate.variation, n.hand))
                    .cloned()
                    .collect();
                (!notes.is_empty()).then(|| Expected {
                    index: onset.index,
                    beat: onset.time.numerator as f64 / onset.time.denominator as f64,
                    pitches: notes
                        .iter()
                        .map(|n| n.pitch.saturating_add_signed(candidate.transposition))
                        .collect(),
                    notes,
                    rest_before: score.rests_before.contains(&onset.index),
                })
            })
            .collect();
        let jumps = score.onsets[range.start as usize..range.end as usize]
            .iter()
            .flat_map(|o| {
                o.successors
                    .iter()
                    .filter(move |&&s| s <= o.index && s >= range.start)
                    .map(move |&s| (o.index, s))
            })
            .filter_map(|(end, start)| {
                let end = onsets.iter().rposition(|o| o.index <= end)?;
                let start = onsets.iter().position(|o| o.index >= start)?;
                (start <= end).then_some((end, start))
            })
            .collect();
        Setup {
            candidate,
            onsets,
            repeats: Repeats::new(jumps),
        }
    }

    pub fn reference(&self) -> Vec<ReferenceNote> {
        self.onsets
            .iter()
            .flat_map(|o| {
                o.pitches.iter().map(|&pitch| ReferenceNote {
                    onset_beat: o.beat,
                    pitch,
                })
            })
            .collect()
    }
}

/// Seconds without a tone before a performance is aborted (FR-020).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Limits {
    /// Outside notated rests.
    pub pause: f64,
    /// Before an onset preceded by a notated rest.
    pub rest: f64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
pub enum AbortReason {
    PauseLimit,
    AlignmentLoss,
    /// Input changed or disconnected, app in the background.
    Interrupted,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Event {
    Recognized(Candidate),
    /// Score onset index of the current position.
    Position(u32),
    Deviation(Deviation),
    Completed,
    Aborted(AbortReason),
}

enum State {
    /// No performance; the next tone starts a recognition.
    Idle,
    Recognizing(Recognition),
    Running {
        setup: usize,
        follower: PitchHmm,
        /// Position in `Setup::onsets`.
        current: usize,
        /// Whether each of the last tones was assigned.
        recent: VecDeque<bool>,
        deviations: Deviations,
    },
}

/// `PerformanceState`; `Idle` stands for "no performance" before the first tone and after
/// `Completed` and `Aborted`.
pub struct Performances {
    setups: Vec<Setup>,
    /// Tones per hand.
    recognition_length: usize,
    limits: Limits,
    state: State,
    last_tone: f64,
}

impl Performances {
    pub fn new(setups: Vec<Setup>, recognition_length: usize, limits: Limits) -> Self {
        Performances {
            setups,
            recognition_length,
            limits,
            state: State::Idle,
            last_tone: 0.0,
        }
    }

    pub fn observe(&mut self, observation: &Observation) -> Vec<Event> {
        self.last_tone = observation.time;
        if matches!(self.state, State::Idle) {
            self.state = State::Recognizing(Recognition::for_setups(
                &self.setups,
                self.recognition_length,
            ));
        }
        match &mut self.state {
            State::Idle => Vec::new(),
            State::Recognizing(recognition) => match recognition.observe(observation) {
                Decision::Pending => Vec::new(),
                // shortcut: the tones so far are dropped; the next tone starts a new recognition.
                Decision::NoPerformance => {
                    self.state = State::Idle;
                    Vec::new()
                }
                Decision::Recognized(setup) => {
                    let State::Recognizing(recognition) =
                        std::mem::replace(&mut self.state, State::Idle)
                    else {
                        unreachable!()
                    };
                    let (candidate, follower) = recognition.into_follower(setup);
                    self.state = State::Running {
                        setup,
                        follower,
                        current: 0,
                        recent: VecDeque::new(),
                        deviations: Deviations::new(&self.setups[setup].onsets),
                    };
                    let mut events = vec![Event::Recognized(candidate)];
                    // The deciding tones are judged at the first position.
                    events.extend(self.follow(&observation.pitches));
                    events
                }
            },
            State::Running { follower, .. } => {
                follower.step(&observation.pitches, observation.time);
                self.follow(&observation.pitches)
            }
        }
    }

    /// Clock heartbeat (seconds, same clock as the observations).
    pub fn tick(&mut self, now: f64) -> Vec<Event> {
        let silence = now - self.last_tone;
        match &self.state {
            State::Recognizing(_) if silence > self.limits.pause => {
                self.state = State::Idle;
                Vec::new()
            }
            State::Running { setup, current, .. } => {
                let setup = &self.setups[*setup];
                let rest_next = setup
                    .repeats
                    .successors(*current, setup.onsets.len())
                    .any(|next| setup.onsets[next].rest_before);
                let limit = if rest_next {
                    self.limits.rest
                } else {
                    self.limits.pause
                };
                if silence > limit {
                    self.end(Event::Aborted(AbortReason::PauseLimit))
                } else {
                    Vec::new()
                }
            }
            _ => Vec::new(),
        }
    }

    /// Input changed or disconnected, app in the background (FR-002, FR-004, NFR-005).
    pub fn interrupt(&mut self) -> Vec<Event> {
        match self.state {
            State::Running { .. } => self.end(Event::Aborted(AbortReason::Interrupted)),
            _ => {
                self.state = State::Idle;
                Vec::new()
            }
        }
    }

    /// Position and assignment after the follower has seen `pitches`.
    fn follow(&mut self, pitches: &[u8]) -> Vec<Event> {
        let State::Running {
            setup,
            follower,
            current,
            recent,
            deviations,
        } = &mut self.state
        else {
            return Vec::new();
        };
        let onsets = &self.setups[*setup].onsets;
        let beat = follower.score_positions()[follower.current_index()];
        // Inserted states between onsets match no onset.
        let found = onsets.iter().position(|o| o.beat == beat);
        let (found_deviations, assigned) = deviations.step(&self.setups[*setup], found, pitches);
        let mut events: Vec<Event> = found_deviations.into_iter().map(Event::Deviation).collect();
        let n = self.recognition_length * hands(self.setups[*setup].candidate.variation);
        for a in assigned {
            recent.push_back(a);
            if recent.len() > n {
                recent.pop_front();
            }
        }
        let assigned = recent.iter().filter(|a| **a).count();
        if recent.len() == n && (assigned as f64) < MIN_ASSIGNED * n as f64 {
            events.extend(self.end(Event::Aborted(AbortReason::AlignmentLoss)));
            return events;
        }
        let Some(i) = found else {
            return events;
        };
        *current = i;
        events.push(Event::Position(onsets[i].index));
        // shortcut: notes of the last onset still missing at completion are not judged, since the
        // other hand may follow a few milliseconds later.
        if i == onsets.len() - 1 {
            events.extend(self.end(Event::Completed));
        }
        events
    }

    fn end(&mut self, event: Event) -> Vec<Event> {
        self.state = State::Idle;
        vec![event]
    }
}
