//! Signal plan (data-model.md#signalplanner-signalplan, US 4): neutral MIDI tone sequences for
//! connection, completed and aborted performances (FR-027 to FR-029, FR-031).

use crate::storage::{AbortSignal, ConnectSignal, OkSignal, Settings};

/// Sounding time of a tone.
const TONE_MS: u64 = 150;
/// Distance between the onsets of two tones.
const STEP_MS: u64 = 200;
/// shortcut: desktop MIDI opens the output in the background, so the connection signal waits a
/// fixed time; replace by a confirmation from the MIDI layer if it still gets lost.
const CONNECT_DELAY_MS: u64 = 500;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Trigger {
    Connected,
    Completed,
    Aborted,
    Stopped,
}

#[derive(Default)]
pub(crate) struct SignalPlanner {
    /// A performance was completed since the last OK signal (FR-028).
    completed: bool,
    /// Due time and MIDI message, by due time.
    queue: Vec<(u64, Vec<u8>)>,
}

impl SignalPlanner {
    /// Plans the signal for `trigger` with the current settings (US 4 scenario 7).
    pub fn trigger(&mut self, trigger: Trigger, settings: &Settings, now_ms: u64) {
        let mut notes = settings.signal_tones.notes.clone();
        notes.sort_unstable();
        let (notes, start) = match trigger {
            Trigger::Connected if settings.connect_signal == ConnectSignal::On => {
                let steady = notes.first().map(|&n| vec![n; notes.len()]);
                (steady.unwrap_or_default(), now_ms + CONNECT_DELAY_MS)
            }
            Trigger::Completed => match settings.ok_signal {
                OkSignal::EveryPerformance => (notes, now_ms),
                OkSignal::WhenStopped => {
                    self.completed = true;
                    return;
                }
                OkSignal::Off => return,
            },
            Trigger::Stopped
                if settings.ok_signal == OkSignal::WhenStopped
                    && std::mem::take(&mut self.completed) =>
            {
                (notes, now_ms)
            }
            Trigger::Aborted if settings.abort_signal == AbortSignal::Immediately => {
                notes.reverse();
                (notes, now_ms)
            }
            _ => return,
        };
        // A signal never overlaps the one before.
        let mut at = self
            .queue
            .last()
            .map_or(start, |(t, _)| start.max(t + STEP_MS));
        let velocity = settings.signal_tones.velocity;
        for pitch in notes {
            self.queue.push((at, vec![0x90, pitch, velocity]));
            self.queue.push((at + TONE_MS, vec![0x80, pitch, 0]));
            at += STEP_MS;
        }
        self.queue.sort_by_key(|(t, _)| *t);
    }

    /// The connection ended: planned tones are dropped.
    pub fn cancel(&mut self) {
        self.queue.clear();
    }

    pub fn next_deadline(&self) -> Option<u64> {
        self.queue.first().map(|(t, _)| *t)
    }

    /// Messages due at `now_ms`, in order.
    pub fn due(&mut self, now_ms: u64) -> Vec<Vec<u8>> {
        let due = self.queue.iter().take_while(|(t, _)| *t <= now_ms).count();
        self.queue.drain(..due).map(|(_, data)| data).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn note_ons(planner: &mut SignalPlanner) -> Vec<u8> {
        planner
            .due(u64::MAX)
            .into_iter()
            .filter(|m| m[0] == 0x90)
            .map(|m| m[1])
            .collect()
    }

    #[test]
    fn ascending_descending_and_steady() {
        let settings = Settings {
            ok_signal: OkSignal::EveryPerformance,
            ..Settings::default()
        };
        let mut planner = SignalPlanner::default();
        planner.trigger(Trigger::Completed, &settings, 0);
        assert_eq!(note_ons(&mut planner), [96, 100]);
        planner.trigger(Trigger::Aborted, &settings, 0);
        assert_eq!(note_ons(&mut planner), [100, 96]);
        planner.trigger(Trigger::Connected, &settings, 0);
        assert_eq!(planner.next_deadline(), Some(CONNECT_DELAY_MS));
        assert_eq!(note_ons(&mut planner), [96, 96]);
    }

    #[test]
    fn tones_are_quiet_and_end() {
        let settings = Settings {
            ok_signal: OkSignal::EveryPerformance,
            ..Settings::default()
        };
        let mut planner = SignalPlanner::default();
        planner.trigger(Trigger::Completed, &settings, 1000);
        assert_eq!(planner.due(1000), [vec![0x90, 96, 30]]);
        assert_eq!(planner.due(1000 + TONE_MS), [vec![0x80, 96, 0]]);
        assert_eq!(
            planner.due(u64::MAX),
            [vec![0x90, 100, 30], vec![0x80, 100, 0]]
        );
    }

    #[test]
    fn signals_do_not_overlap() {
        let settings = Settings::default();
        let mut planner = SignalPlanner::default();
        planner.trigger(Trigger::Completed, &settings, 0);
        planner.trigger(Trigger::Aborted, &settings, 0);
        planner.trigger(Trigger::Stopped, &settings, 0);
        let times: Vec<u64> = planner.queue.iter().map(|(t, _)| *t).collect();
        assert!(times.is_sorted());
        assert_eq!(note_ons(&mut planner), [100, 96, 96, 100]);
    }
}
