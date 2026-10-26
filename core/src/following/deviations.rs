//! Classification of deviations during a performance (FR-012, FR-013): omission, wrong note
//! and addition, each with note, hand and finger (constitution VI).

use crate::performance::Expected;
use crate::score::model::{Hand, Note};

/// Largest distance in half tones at which a wrong tone replaces a note instead of being added.
const SUBSTITUTION_RANGE: u8 = 2;

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
pub enum DeviationKind {
    Omission,
    Substitution,
    Addition,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Deviation {
    pub kind: DeviationKind,
    /// Not for additions.
    pub note: Option<Note>,
    /// Played pitch; not for omissions.
    pub played: Option<u8>,
    /// From the note or, for additions, from the nearest note of the onset.
    pub hand: Hand,
    pub finger: Option<u8>,
}

impl Deviation {
    fn new(kind: DeviationKind, note: &Note, played: Option<u8>) -> Self {
        Deviation {
            kind,
            note: (kind != DeviationKind::Addition).then(|| note.clone()),
            played,
            hand: note.hand,
            finger: note.finger,
        }
    }
}

/// Which expected notes are played or judged, per onset of a setup.
///
/// Tones are matched greedily from the current onset on: the follower's position jumps ahead
/// for a moment after a gap and would report false omissions. The follower only gives the
/// start and resynchronizes when the matching falls behind by more than [`LOOKAHEAD`].
///
/// A tone equal to a note that a wrong tone has taken shows that the matching ran ahead (e.g.
/// during a short change of key); it resynchronizes there instead of shifting every following
/// tone onto the next onset.
pub struct Deviations {
    handled: Vec<Vec<bool>>,
    /// Notes taken by a wrong tone.
    replaced: Vec<Vec<bool>>,
    /// Onset being played; `None` until the first position after the recognition.
    current: Option<usize>,
}

/// Onsets after the current one in which a played pitch is searched.
const LOOKAHEAD: usize = 3;

impl Deviations {
    pub fn new(onsets: &[Expected]) -> Self {
        let open: Vec<Vec<bool>> = onsets
            .iter()
            .map(|o| vec![false; o.pitches.len()])
            .collect();
        Deviations {
            handled: open.clone(),
            replaced: open,
            current: None,
        }
    }

    /// Judges `pitches`; `found` is the follower's onset (`None`: between onsets). Returns the
    /// deviations and whether each tone was assigned to a note.
    pub fn step(
        &mut self,
        onsets: &[Expected],
        found: Option<usize>,
        pitches: &[u8],
    ) -> (Vec<Deviation>, Vec<bool>) {
        let mut deviations = Vec::new();
        match (self.current, found) {
            // `pitches` decided the recognition: they and everything before are not judged. The
            // follower may lag or lead by an onset, so the onset holding them is searched nearby.
            (None, Some(i)) => {
                let last = (i + LOOKAHEAD).min(onsets.len() - 1);
                let start = (i.saturating_sub(LOOKAHEAD)..=last)
                    .max_by_key(|&k| {
                        let hits = pitches
                            .iter()
                            .filter(|p| onsets[k].pitches.contains(p))
                            .count();
                        (hits, std::cmp::Reverse(k.abs_diff(i)))
                    })
                    .unwrap_or(i);
                self.handled[..=start]
                    .iter_mut()
                    .for_each(|h| h.iter_mut().for_each(|h| *h = true));
                self.current = Some(start);
                return (deviations, vec![true; pitches.len()]);
            }
            (None, None) => return (deviations, vec![false; pitches.len()]),
            (Some(current), Some(i)) if i > current + LOOKAHEAD => {
                deviations.extend(self.skip(onsets, current, i));
            }
            _ => {}
        }
        let mut assigned = Vec::with_capacity(pitches.len());
        for &pitch in pitches {
            let current = self.current.expect("set above");
            let last = (current + LOOKAHEAD).min(onsets.len() - 1);
            let exact = (current..=last).find_map(|k| {
                let n = (0..onsets[k].pitches.len())
                    .find(|&n| !self.handled[k][n] && onsets[k].pitches[n] == pitch)?;
                Some((k, n))
            });
            if let Some((k, n)) = exact {
                deviations.extend(self.skip(onsets, current, k));
                self.handled[k][n] = true;
                assigned.push(true);
                continue;
            }
            let taken = (current.saturating_sub(1)..=current).find_map(|k| {
                let n = (0..onsets[k].pitches.len())
                    .find(|&n| self.replaced[k][n] && onsets[k].pitches[n] == pitch)?;
                Some((k, n))
            });
            if let Some((k, n)) = taken {
                self.replaced[k][n] = false;
                self.current = Some(k);
                assigned.push(true);
                continue;
            }
            assigned.push(false);
            // A wrong tone replaces a near note of this onset or, once it is complete, the next.
            let onset_done = self.handled[current].iter().all(|h| *h);
            let target = if onset_done && current + 1 < onsets.len() {
                current + 1
            } else {
                current
            };
            let near = (0..onsets[target].pitches.len()).find(|&n| {
                !self.handled[target][n]
                    && onsets[target].pitches[n].abs_diff(pitch) <= SUBSTITUTION_RANGE
            });
            if let Some(n) = near {
                self.current = Some(target);
                self.handled[target][n] = true;
                self.replaced[target][n] = true;
                deviations.push(Deviation::new(
                    DeviationKind::Substitution,
                    &onsets[target].notes[n],
                    Some(pitch),
                ));
            } else {
                let onset = &onsets[current];
                let nearest = (0..onset.pitches.len())
                    .min_by_key(|&n| onset.pitches[n].abs_diff(pitch))
                    .expect("onsets have notes");
                deviations.push(Deviation::new(
                    DeviationKind::Addition,
                    &onset.notes[nearest],
                    Some(pitch),
                ));
            }
        }
        (deviations, assigned)
    }

    /// Moves from onset `from` to `to`; notes left unplayed before `to` are omissions.
    fn skip(&mut self, onsets: &[Expected], from: usize, to: usize) -> Vec<Deviation> {
        let mut omissions = Vec::new();
        for (k, onset) in onsets.iter().enumerate().take(to).skip(from) {
            for (n, handled) in self.handled[k].iter_mut().enumerate() {
                if !*handled {
                    *handled = true;
                    omissions.push(Deviation::new(
                        DeviationKind::Omission,
                        &onset.notes[n],
                        None,
                    ));
                }
            }
        }
        self.current = Some(to.max(from));
        omissions
    }
}
