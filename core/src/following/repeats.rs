//! Repeats as edges in the state graph instead of unrolling them (FR-014, research.md R8): the
//! last onset of a repeat section is followed by the next onset and by the first of the section.

/// Jumps from the last onset of a repeat section back to its first, as indices of the unique
/// onsets of a reference. Without jumps every method behaves as in Matchmaker.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Repeats {
    /// `(end, start)` with `start <= end`.
    jumps: Vec<(usize, usize)>,
}

/// An onset reached from another one, see [`Repeats::reachable`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Reached {
    pub onset: usize,
    /// Steps from the start.
    pub distance: usize,
    /// Position of the previous onset on the way in the returned list.
    pub via: Option<usize>,
    /// The way jumps back at the end of a repeat section.
    pub jumped: bool,
}

impl Repeats {
    /// `jumps`: `(end, start)` of every repeat section.
    pub fn new(jumps: Vec<(usize, usize)>) -> Self {
        assert!(jumps.iter().all(|(end, start)| start <= end));
        Repeats { jumps }
    }

    pub fn is_empty(&self) -> bool {
        self.jumps.is_empty()
    }

    pub fn jumps(&self) -> &[(usize, usize)] {
        &self.jumps
    }

    /// Onsets after `k` among `n`: `k + 1` and the start of every section ending at `k`.
    pub fn successors(&self, k: usize, n: usize) -> impl Iterator<Item = usize> + '_ {
        (k + 1 < n).then_some(k + 1).into_iter().chain(
            self.jumps
                .iter()
                .filter(move |(end, _)| *end == k)
                .map(|(_, start)| *start),
        )
    }

    /// Onsets `steps` after `k` with the share of the ways leading there; the ways split evenly
    /// at the end of a repeat section.
    pub fn ahead(&self, k: usize, steps: usize, n: usize) -> Vec<(usize, f64)> {
        let mut frontier = vec![(k, 1.0)];
        for _ in 0..steps {
            let mut next = Vec::new();
            for (onset, share) in frontier {
                let successors: Vec<usize> = self.successors(onset, n).collect();
                let part = share / successors.len() as f64;
                next.extend(successors.into_iter().map(|s| (s, part)));
            }
            frontier = next;
        }
        frontier
    }

    /// Onsets at most `steps` after `k`, breadth-first, each once.
    pub fn reachable(&self, k: usize, steps: usize, n: usize) -> Vec<Reached> {
        let mut seen = vec![false; n];
        seen[k] = true;
        let mut reached = vec![Reached {
            onset: k,
            distance: 0,
            via: None,
            jumped: false,
        }];
        let mut i = 0;
        while i < reached.len() {
            let from = reached[i];
            if from.distance < steps {
                for onset in self.successors(from.onset, n) {
                    if !seen[onset] {
                        seen[onset] = true;
                        reached.push(Reached {
                            onset,
                            distance: from.distance + 1,
                            via: Some(i),
                            jumped: from.jumped || onset <= from.onset,
                        });
                    }
                }
            }
            i += 1;
        }
        reached
    }
}

/// Onsets on the way to `reached[i]`, from the start of [`Repeats::reachable`].
pub fn way(reached: &[Reached], i: usize) -> Vec<usize> {
    let mut way = vec![reached[i].onset];
    let mut via = reached[i].via;
    while let Some(v) = via {
        way.push(reached[v].onset);
        via = reached[v].via;
    }
    way.reverse();
    way
}
