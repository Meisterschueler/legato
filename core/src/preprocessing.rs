// Grouping after Matchmaker (https://github.com/pymatchmaker/matchmaker, commit 6c09490:
// MidiStream, PitchProcessor, ChordOnsetProcessor). Copyright the Matchmaker authors,
// Apache-2.0; see NOTICE.

//! Turns the note-on stream into observations for the score followers (research.md R8).

/// Pitches played together and the time of the first of them (seconds).
#[derive(Clone, Debug, PartialEq)]
pub struct Observation {
    pub pitches: Vec<u8>,
    pub time: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Grouping {
    /// All notes in fixed windows `(k·p, (k+1)·p]` (`pthmm`: 10 ms, `outerhmm`: 1 ms).
    Window { period: f64 },
    /// Notes within `threshold` of the first note of a chord (`arzt`, 35 ms); `tick` emits a chord
    /// once the threshold has passed.
    Chord { threshold: f64 },
}

pub struct Preprocessor {
    grouping: Grouping,
    pending: Vec<u8>,
    pending_time: Option<f64>,
    pending_window: i64,
}

impl Preprocessor {
    pub fn new(grouping: Grouping) -> Self {
        Preprocessor {
            grouping,
            pending: Vec::new(),
            pending_time: None,
            pending_window: 0,
        }
    }

    /// A note-on with velocity > 0 at `time` (seconds); returns an observation that is complete.
    pub fn note_on(&mut self, time: f64, pitch: u8) -> Option<Observation> {
        let mut done = None;
        match self.grouping {
            Grouping::Window { period } => {
                let window = window_of(time, period);
                if self.pending_time.is_some() && window != self.pending_window {
                    done = self.flush();
                }
                self.pending_window = window;
            }
            Grouping::Chord { threshold } => {
                if self.pending_time.is_some_and(|t| time - t > threshold) {
                    done = self.flush();
                }
            }
        }
        self.pending.push(pitch);
        self.pending_time = Some(self.pending_time.map_or(time, |t| t.min(time)));
        done
    }

    /// Clock heartbeat at `time`; returns an observation whose window or chord has ended.
    pub fn tick(&mut self, time: f64) -> Option<Observation> {
        let ended = match self.grouping {
            Grouping::Window { period } => window_of(time, period) > self.pending_window,
            Grouping::Chord { threshold } => {
                self.pending_time.is_some_and(|t| time - t > threshold)
            }
        };
        if ended { self.flush() } else { None }
    }

    /// End of the stream.
    pub fn finish(&mut self) -> Option<Observation> {
        self.flush()
    }

    fn flush(&mut self) -> Option<Observation> {
        let time = self.pending_time.take()?;
        Some(Observation {
            pitches: std::mem::take(&mut self.pending),
            time,
        })
    }
}

/// Pitch of a note-on in raw MIDI bytes; `None` for everything else, so the sustain pedal
/// (CC 64, FR-003) and note-ons with velocity 0 (note-offs) never reach the followers.
pub fn played_pitch(data: &[u8]) -> Option<u8> {
    match data {
        [status, pitch, velocity, ..] if status & 0xF0 == 0x90 && *velocity > 0 => Some(*pitch),
        _ => None,
    }
}

/// Window `k` holds `(k·p, (k+1)·p]`; window 0 also holds time 0.
fn window_of(time: f64, period: f64) -> i64 {
    let mut k = (time / period).ceil() as i64 - 1;
    while k > 0 && time <= k as f64 * period {
        k -= 1;
    }
    while time > (k + 1) as f64 * period {
        k += 1;
    }
    k.max(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_note_ons_are_played() {
        assert_eq!(played_pitch(&[0x93, 60, 40]), Some(60));
        assert_eq!(played_pitch(&[0x90, 60, 0]), None);
        assert_eq!(played_pitch(&[0x80, 60, 40]), None);
        assert_eq!(played_pitch(&[0xB0, 64, 127]), None);
        assert_eq!(played_pitch(&[0x90, 60]), None);
    }

    #[test]
    fn windows_include_their_end() {
        assert_eq!(window_of(0.0, 0.01), 0);
        assert_eq!(window_of(0.01, 0.01), 0);
        assert_eq!(window_of(0.0101, 0.01), 1);
        assert_eq!(window_of(0.02, 0.01), 1);
    }

    #[test]
    fn chord_is_emitted_by_the_next_note_or_the_clock() {
        let mut p = Preprocessor::new(Grouping::Chord { threshold: 0.035 });
        assert_eq!(p.note_on(1.0, 60), None);
        assert_eq!(p.note_on(1.02, 64), None);
        assert_eq!(p.tick(1.03), None);
        let chord = p.tick(1.04).unwrap();
        assert_eq!((chord.pitches, chord.time), (vec![60, 64], 1.0));
        assert_eq!(p.note_on(2.0, 62), None);
        assert_eq!(p.note_on(2.5, 65).unwrap().pitches, vec![62]);
        assert_eq!(p.finish().unwrap().pitches, vec![65]);
    }
}
