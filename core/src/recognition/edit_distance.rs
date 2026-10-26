//! Preselection `EditDistance` (research.md R15): edit distance of the played onsets to the
//! beginning of every candidate with fixed start and free end (Sellers, 1980; docs/literatur.md).

use std::sync::Arc;

use super::{Onset, PRESELECTED, Preselector};

pub(super) struct EditDistance {
    /// Reference onsets of every candidate.
    references: Arc<Vec<Vec<Onset>>>,
    /// Per candidate the row before the last played onset and the row after it.
    before: Vec<Vec<u32>>,
    after: Vec<Vec<u32>>,
    /// Played onsets in `before`.
    rows: usize,
    tones: usize,
}

impl EditDistance {
    pub(super) fn new(references: Arc<Vec<Vec<Onset>>>) -> Self {
        EditDistance {
            references,
            before: Vec::new(),
            after: Vec::new(),
            rows: 0,
            tones: 0,
        }
    }
}

impl Preselector for EditDistance {
    fn observe(&mut self, onsets: &[Onset]) {
        let Some(last) = onsets.last() else {
            return;
        };
        if onsets.len() == 1 {
            self.before = self.references.iter().map(|r| first_row(r)).collect();
        } else if onsets.len() > self.rows + 1 {
            self.before = std::mem::take(&mut self.after);
        }
        self.rows = onsets.len() - 1;
        self.after = self
            .references
            .iter()
            .zip(&self.before)
            .map(|(reference, before)| next_row(before, reference, &last.pitches))
            .collect();
        self.tones = onsets.iter().map(|o| o.pitches.len()).sum();
    }

    /// Distance at most half the played tones.
    fn preselection(&self) -> Vec<usize> {
        let mut ranked: Vec<(u32, usize)> = self
            .after
            .iter()
            .enumerate()
            .map(|(c, row)| (row.iter().copied().min().unwrap_or(0), c))
            .filter(|&(d, _)| 2 * d as usize <= self.tones)
            .collect();
        ranked.sort_unstable();
        ranked
            .into_iter()
            .take(PRESELECTED)
            .map(|(_, c)| c)
            .collect()
    }
}

/// Distance of `played` to the beginning of `reference`: fixed start, free end.
pub fn distance(played: &[Onset], reference: &[Onset]) -> u32 {
    let mut row = first_row(reference);
    for onset in played {
        row = next_row(&row, reference, &onset.pitches);
    }
    row.into_iter().min().unwrap_or(0)
}

/// Replacing onset `b` by `a` (sorted pitches): a wrong tone costs 1, a missing chord tone also.
pub fn substitution(a: &[u8], b: &[u8]) -> u32 {
    let (mut i, mut j, mut common) = (0, 0, 0);
    while i < a.len() && j < b.len() {
        match a[i].cmp(&b[j]) {
            std::cmp::Ordering::Less => i += 1,
            std::cmp::Ordering::Greater => j += 1,
            std::cmp::Ordering::Equal => {
                common += 1;
                i += 1;
                j += 1;
            }
        }
    }
    (a.len().max(b.len()) - common) as u32
}

/// No onset played: omitting the reference onsets.
fn first_row(reference: &[Onset]) -> Vec<u32> {
    let mut row = Vec::with_capacity(reference.len() + 1);
    row.push(0);
    for onset in reference {
        row.push(row[row.len() - 1] + onset.pitches.len() as u32);
    }
    row
}

/// The row after playing `pitches`: addition `|A|`, omission `|B|`, substitution.
fn next_row(before: &[u32], reference: &[Onset], pitches: &[u8]) -> Vec<u32> {
    let added = pitches.len() as u32;
    let mut row = Vec::with_capacity(before.len());
    row.push(before[0] + added);
    for (j, onset) in reference.iter().enumerate() {
        let cost = (before[j + 1] + added)
            .min(row[j] + onset.pitches.len() as u32)
            .min(before[j] + substitution(pitches, &onset.pitches));
        row.push(cost);
    }
    row
}
