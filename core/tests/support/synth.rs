//! Synthetic performances with known alignment (T072).

use legato_core::recognition::reference;
use legato_core::score::model::{Candidate, Score};
use legato_core::works::Range;

/// Note-ons `(seconds, pitch)` of a performance of `range` without errors, `beats_per_second`
/// quarter notes per second.
pub fn clean(
    score: &Score,
    candidate: &Candidate,
    range: Range,
    beats_per_second: f64,
) -> Vec<(f64, u8)> {
    let notes = reference(score, candidate, range);
    let start = notes.first().map_or(0.0, |n| n.onset_beat);
    notes
        .iter()
        .map(|n| ((n.onset_beat - start) / beats_per_second, n.pitch))
        .collect()
}
