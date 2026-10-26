//! Preselection `Fingerprint` (research.md R15): transposition- and tempo-invariant hashes of
//! tone triples vote for piece, variation and transposition (after Arzt et al., 2012;
//! docs/literatur.md), restricted to the beginning of the candidates.

use std::collections::BTreeMap;
use std::f64::consts::SQRT_2;
use std::sync::Arc;

use super::{Onset, PRESELECTED, Preselector};
use crate::score::model::Candidate;

/// Onsets between the onsets of a triple at most.
const SPAN: usize = 2;
/// Played and reference onset of a hit differ by at most this many onsets (fixed start).
const MAX_OFFSET: usize = 4;
/// Votes of a candidate in the preselection at least.
const MIN_VOTES: u32 = 2;

/// Intervals `b - a`, `c - b` and the quantized ratio of the onset distances.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Hash {
    pub first: i16,
    pub second: i16,
    /// `log2((t2 - t1) / (t1 - t0))` rounded to `-2..=2`.
    pub ratio: i8,
}

/// Tones `a, b, c` from onsets `g0 < g1 < g2`; only the lowest and the highest tone of an onset.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Triple {
    /// Onset `g0`.
    pub onset: usize,
    /// Pitch `a`.
    pub pitch: u8,
    pub hash: Hash,
}

/// Triples whose last onset is in `last`.
pub fn triples(onsets: &[Onset], last: std::ops::Range<usize>) -> Vec<Triple> {
    let mut triples = Vec::new();
    for g2 in last {
        for g1 in g2.saturating_sub(SPAN)..g2 {
            for g0 in g1.saturating_sub(SPAN)..g1 {
                let ratio = ratio(
                    onsets[g1].time - onsets[g0].time,
                    onsets[g2].time - onsets[g1].time,
                );
                for a in extremes(&onsets[g0]) {
                    for b in extremes(&onsets[g1]) {
                        for c in extremes(&onsets[g2]) {
                            triples.push(Triple {
                                onset: g0,
                                pitch: a,
                                hash: Hash {
                                    first: i16::from(b) - i16::from(a),
                                    second: i16::from(c) - i16::from(b),
                                    ratio,
                                },
                            });
                        }
                    }
                }
            }
        }
    }
    triples
}

/// Lowest and highest tone.
fn extremes(onset: &Onset) -> impl Iterator<Item = u8> + '_ {
    let pitches = &onset.pitches;
    let highest = (pitches.len() > 1).then(|| pitches[pitches.len() - 1]);
    pitches.first().copied().into_iter().chain(highest)
}

/// `log2(later / earlier)` rounded to `-2..=2`; the bounds are odd powers of `√2`.
fn ratio(earlier: f64, later: f64) -> i8 {
    let r = later / earlier;
    if r < SQRT_2 / 4.0 {
        -2
    } else if r < SQRT_2 / 2.0 {
        -1
    } else if r <= SQRT_2 {
        0
    } else if r <= 2.0 * SQRT_2 {
        1
    } else {
        2
    }
}

/// Reference triples of every piece and variation.
pub(super) struct Index {
    /// Hash → group, reference onset `g0`, pitch `a`.
    entries: BTreeMap<Hash, Vec<(usize, usize, u8)>>,
    groups: Vec<Group>,
    candidates: usize,
}

/// The candidates of a piece and variation.
struct Group {
    /// Transposition of the indexed reference.
    indexed: i8,
    /// Transposition and index of every candidate.
    candidates: Vec<(i8, usize)>,
}

impl Index {
    /// `candidates` with the reference onsets of each.
    pub(super) fn new(candidates: &[Candidate], references: &[Vec<Onset>]) -> Self {
        let mut index = Index {
            entries: BTreeMap::new(),
            groups: Vec::new(),
            candidates: candidates.len(),
        };
        let mut keys: Vec<&Candidate> = Vec::new();
        for (c, (candidate, reference)) in candidates.iter().zip(references).enumerate() {
            let same =
                |k: &&Candidate| k.piece == candidate.piece && k.variation == candidate.variation;
            let group = match keys.iter().position(same) {
                Some(g) => g,
                None => {
                    let g = index.groups.len();
                    for t in triples(reference, 0..reference.len()) {
                        index
                            .entries
                            .entry(t.hash)
                            .or_default()
                            .push((g, t.onset, t.pitch));
                    }
                    index.groups.push(Group {
                        indexed: candidate.transposition,
                        candidates: Vec::new(),
                    });
                    keys.push(candidate);
                    g
                }
            };
            index.groups[group]
                .candidates
                .push((candidate.transposition, c));
        }
        index
    }

    /// Votes of every candidate: each played triple at most once per candidate.
    fn votes(&self, onsets: &[Onset]) -> Vec<u32> {
        let mut votes = vec![0; self.candidates];
        for played in triples(onsets, 0..onsets.len()) {
            let Some(hits) = self.entries.get(&played.hash) else {
                continue;
            };
            let mut voted: Vec<usize> = Vec::new();
            for &(g, onset, pitch) in hits {
                if played.onset.abs_diff(onset) > MAX_OFFSET {
                    continue;
                }
                let group = &self.groups[g];
                let transposition =
                    i16::from(played.pitch) - i16::from(pitch) + i16::from(group.indexed);
                let Some(&(_, c)) = group
                    .candidates
                    .iter()
                    .find(|(t, _)| i16::from(*t) == transposition)
                else {
                    continue;
                };
                if !voted.contains(&c) {
                    voted.push(c);
                    votes[c] += 1;
                }
            }
        }
        votes
    }
}

pub(super) struct Fingerprint {
    index: Arc<Index>,
    votes: Vec<u32>,
}

impl Fingerprint {
    pub(super) fn new(index: Arc<Index>) -> Self {
        Fingerprint {
            index,
            votes: Vec::new(),
        }
    }
}

impl Preselector for Fingerprint {
    fn observe(&mut self, onsets: &[Onset]) {
        self.votes = self.index.votes(onsets);
    }

    fn preselection(&self) -> Vec<usize> {
        let mut ranked: Vec<(std::cmp::Reverse<u32>, usize)> = self
            .votes
            .iter()
            .enumerate()
            .filter(|&(_, &v)| v >= MIN_VOTES)
            .map(|(c, &v)| (std::cmp::Reverse(v), c))
            .collect();
        ranked.sort_unstable();
        ranked
            .into_iter()
            .take(PRESELECTED)
            .map(|(_, c)| c)
            .collect()
    }
}
