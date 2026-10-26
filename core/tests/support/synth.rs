//! Synthetic performances with known alignment (T072): tempo and agogics, scattered omissions,
//! wrong and added tones, pauses, pedal, a short change of key and seamless repetitions. Played
//! with one hand by taking the reference of a one-handed candidate.
#![allow(dead_code)]

use std::ops::Range as Indices;

use legato_core::following::ReferenceNote;
use legato_core::preprocessing::played_pitch;
use legato_core::recognition::reference;
use legato_core::score::model::{Candidate, Score};
use legato_core::works::Range;

/// Largest distance in half tones at which a wrong tone replaces a note (as in
/// `following::deviations`); farther tones are additions and leave the note out.
pub const SUBSTITUTION_RANGE: u8 = 2;
/// Onsets after the current one where a played pitch is searched (as in
/// `following::deviations`); wrong and added tones match none of them.
const LOOKAHEAD: usize = 3;
/// Reference notes from one scattered error to the next at least.
const ERROR_GAP: usize = 4;
/// The pedal is changed every half note.
const PEDAL_BEATS: f64 = 2.0;
/// Seconds between the onset and lifting or pressing the pedal.
const PEDAL_DELAY: f64 = 0.02;
const VELOCITY: u8 = 64;

/// What a message is known to be.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Truth {
    /// Sustain pedal (CC 64); no tone.
    Pedal,
    /// The reference note with this index.
    Note(usize),
    /// A wrong tone in place of the reference note with this index.
    Substitution(usize),
    /// A tone without a note.
    Addition,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Message {
    /// Seconds.
    pub time: f64,
    /// Raw MIDI bytes.
    pub data: [u8; 3],
    /// Performance the message belongs to, from 0.
    pub run: usize,
    pub truth: Truth,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Performance {
    /// Sorted by time.
    pub messages: Vec<Message>,
    /// `(run, reference index)` of the notes left out, including those replaced by additions.
    pub omissions: Vec<(usize, usize)>,
}

impl Performance {
    /// Note-ons `(seconds, pitch)` as they reach the core: the pedal is filtered (FR-003).
    pub fn note_ons(&self) -> Vec<(f64, u8)> {
        self.messages
            .iter()
            .filter_map(|m| Some((m.time, played_pitch(&m.data)?)))
            .collect()
    }

    /// `(run, truth)` of every note-on, in the order of [`Performance::note_ons`].
    pub fn alignment(&self) -> Vec<(usize, Truth)> {
        self.messages
            .iter()
            .filter(|m| played_pitch(&m.data).is_some())
            .map(|m| (m.run, m.truth))
            .collect()
    }
}

/// Settings of a synthetic performance; [`Synth::new`] plays without errors.
#[derive(Clone, Debug, PartialEq)]
pub struct Synth {
    /// Quarter notes per second.
    pub beats_per_second: f64,
    /// Largest relative deviation of each step from the tempo; below 0.5.
    pub agogics: f64,
    /// Largest delay (seconds) of the tones of an onset after its time; below half a step.
    pub spread: f64,
    /// Probabilities per reference note of scattered errors.
    pub omissions: f64,
    pub substitutions: f64,
    pub additions: f64,
    /// Reference notes before this index have no scattered errors (recognition).
    pub clean_start: usize,
    /// Extra silence (seconds) before the onset of the reference note with the index.
    pub pauses: Vec<(usize, f64)>,
    /// Sustain pedal changed every half note.
    pub pedal: bool,
    /// Reference notes played transposed by the half tones, without scattered errors nearby.
    pub key_change: Option<(Indices<usize>, i8)>,
    /// Performances played seamlessly one after another.
    pub runs: usize,
    pub seed: u64,
}

impl Synth {
    pub fn new(beats_per_second: f64) -> Self {
        Synth {
            beats_per_second,
            agogics: 0.0,
            spread: 0.0,
            omissions: 0.0,
            substitutions: 0.0,
            additions: 0.0,
            clean_start: 16,
            pauses: Vec::new(),
            pedal: false,
            key_change: None,
            runs: 1,
            seed: 1,
        }
    }

    pub fn play(&self, reference: &[ReferenceNote]) -> Performance {
        let groups = onsets(reference);
        let last = groups.len() - 1;
        let beat = |g: usize| reference[groups[g].start].onset_beat;
        // Nominal seconds from onset `g` to the next.
        let step = |g: usize| (beat(g + 1) - beat(g)) / self.beats_per_second;
        let mut rng = Rng(self.seed);
        let mut messages = Vec::new();
        let mut omissions = Vec::new();
        let mut start = 0.0;
        for run in 0..self.runs {
            // Deviation from the tempo; exactly 0 without agogics and pauses.
            let mut offset = 0.0;
            let mut last_error = None;
            for (g, notes) in groups.iter().enumerate() {
                if g > 0 {
                    offset += step(g - 1) * self.agogics * rng.symmetric();
                }
                offset += self
                    .pauses
                    .iter()
                    .filter(|(i, _)| notes.contains(i))
                    .map(|(_, seconds)| seconds)
                    .sum::<f64>();
                let onset = start + (beat(g) - beat(0)) / self.beats_per_second + offset;
                if self.pedal && (g == 0 || half_note(beat(g)) != half_note(beat(g - 1))) {
                    if g > 0 {
                        messages.push(pedal(onset - PEDAL_DELAY, run, 0));
                    }
                    messages.push(pedal(onset + PEDAL_DELAY, run, 127));
                }
                for i in notes.clone() {
                    let time = onset + self.spread * rng.unit();
                    let pitch = reference[i].pitch;
                    let (pitch, truth) = if let Some((changed, shift)) = &self.key_change
                        && changed.contains(&i)
                    {
                        if shift.unsigned_abs() > SUBSTITUTION_RANGE {
                            omissions.push((run, i));
                            (pitch.saturating_add_signed(*shift), Truth::Addition)
                        } else {
                            (pitch.saturating_add_signed(*shift), Truth::Substitution(i))
                        }
                    } else if self.may_err(i, g, last, last_error) {
                        let u = rng.unit();
                        if u < self.omissions {
                            omissions.push((run, i));
                            last_error = Some(i);
                            continue;
                        } else if u < self.omissions + self.substitutions
                            && let Some(wrong) =
                                wrong_pitch(&mut rng, reference, &groups[g..=g + LOOKAHEAD], pitch)
                        {
                            last_error = Some(i);
                            (wrong, Truth::Substitution(i))
                        } else {
                            (pitch, Truth::Note(i))
                        }
                    } else {
                        (pitch, Truth::Note(i))
                    };
                    messages.push(Message {
                        time,
                        data: [0x90, pitch, VELOCITY],
                        run,
                        truth,
                    });
                }
                let i = notes.end - 1;
                if self.may_err(i, g, last, last_error)
                    && rng.unit() < self.additions
                    && let Some(extra) =
                        far_pitch(&mut rng, reference, &groups[g..=g + LOOKAHEAD + 1])
                {
                    last_error = Some(i);
                    messages.push(Message {
                        time: onset + step(g) / 2.0,
                        data: [0x90, extra, VELOCITY],
                        run,
                        truth: Truth::Addition,
                    });
                }
            }
            start += (beat(last) - beat(0)) / self.beats_per_second + offset + step(last - 1);
        }
        messages.sort_by(|a, b| a.time.total_cmp(&b.time));
        Performance {
            messages,
            omissions,
        }
    }

    /// Whether a scattered error may follow the reference note `i` of onset `g`: not during the
    /// recognition, not near the end or the change of key, not close to the last error.
    fn may_err(&self, i: usize, g: usize, last: usize, last_error: Option<usize>) -> bool {
        i >= self.clean_start
            && g + LOOKAHEAD < last
            && last_error.is_none_or(|e| i >= e + ERROR_GAP)
            && !self.key_change.as_ref().is_some_and(|(changed, _)| {
                i + ERROR_GAP >= changed.start && i < changed.end + ERROR_GAP
            })
    }
}

/// Note-ons `(seconds, pitch)` of a performance of `range` without errors, `beats_per_second`
/// quarter notes per second.
pub fn clean(
    score: &Score,
    candidate: &Candidate,
    range: Range,
    beats_per_second: f64,
) -> Vec<(f64, u8)> {
    Synth::new(beats_per_second)
        .play(&reference(score, candidate, range))
        .note_ons()
}

/// Index ranges of the notes of each onset; the reference is sorted by onset.
fn onsets(reference: &[ReferenceNote]) -> Vec<Indices<usize>> {
    let mut groups: Vec<Indices<usize>> = Vec::new();
    for (i, note) in reference.iter().enumerate() {
        match groups.last_mut() {
            Some(g) if reference[g.start].onset_beat == note.onset_beat => g.end = i + 1,
            _ => groups.push(i..i + 1),
        }
    }
    groups
}

fn half_note(beat: f64) -> i64 {
    (beat / PEDAL_BEATS).floor() as i64
}

fn pedal(time: f64, run: usize, value: u8) -> Message {
    Message {
        time,
        data: [0xB0, 64, value],
        run,
        truth: Truth::Pedal,
    }
}

fn pitches(reference: &[ReferenceNote], near: &[Indices<usize>]) -> Vec<u8> {
    near.iter()
        .flat_map(|g| reference[g.clone()].iter().map(|n| n.pitch))
        .collect()
}

/// A tone next to `pitch` that is no note of `near`; such a note would be played early.
fn wrong_pitch(
    rng: &mut Rng,
    reference: &[ReferenceNote],
    near: &[Indices<usize>],
    pitch: u8,
) -> Option<u8> {
    let expected = pitches(reference, near);
    let mut steps = [1, -1, 2, -2];
    steps.rotate_left(rng.below(4));
    steps
        .into_iter()
        .map(|s| pitch.saturating_add_signed(s))
        .find(|p| !expected.contains(p))
}

/// A tone within an octave around `near` and farther than [`SUBSTITUTION_RANGE`] from it.
fn far_pitch(rng: &mut Rng, reference: &[ReferenceNote], near: &[Indices<usize>]) -> Option<u8> {
    let expected = pitches(reference, near);
    let low = expected.iter().min()?.saturating_sub(12);
    let high = expected.iter().max()?.saturating_add(12).min(127);
    (0..16)
        .map(|_| low + rng.below(usize::from(high - low) + 1) as u8)
        .find(|p| expected.iter().all(|q| q.abs_diff(*p) > SUBSTITUTION_RANGE))
}

/// SplitMix64: reproducible without dependencies.
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// Uniform in `[0, 1)`.
    fn unit(&mut self) -> f64 {
        (self.next() >> 11) as f64 / (1u64 << 53) as f64
    }

    /// Uniform in `[-1, 1)`.
    fn symmetric(&mut self) -> f64 {
        2.0 * self.unit() - 1.0
    }

    fn below(&mut self, n: usize) -> usize {
        (self.next() % n as u64) as usize
    }
}
