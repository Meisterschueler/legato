//! Score following (research.md R8): methods ported from Matchmaker.

pub mod arzt;
pub mod deviations;
pub mod outer_hmm;
pub mod pthmm;

/// One note of the reference, as in partitura's `note_array`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ReferenceNote {
    pub onset_beat: f64,
    pub pitch: u8,
}

pub trait ScoreFollower {
    /// Processes the pitches played at `perf_time` (seconds); returns the score position in beats,
    /// or `None` if the input does not move the follower.
    fn step(&mut self, pitches: &[u8], perf_time: f64) -> Option<f64>;

    fn current_index(&self) -> usize;

    /// Score position (beats) of every state.
    fn score_positions(&self) -> &[f64];

    /// Probabilities of the states after the last step, where the method has them.
    fn state(&self) -> Option<&[f64]> {
        None
    }
}

/// Sorted unique onsets and the pitches of each, like `np.unique(note_array["onset_beat"])`.
pub(crate) fn unique_onsets(notes: &[ReferenceNote]) -> (Vec<f64>, Vec<Vec<u8>>) {
    let mut onsets: Vec<f64> = notes.iter().map(|n| n.onset_beat).collect();
    onsets.sort_by(f64::total_cmp);
    onsets.dedup();
    let chords = onsets
        .iter()
        .map(|&onset| {
            notes
                .iter()
                .filter(|n| n.onset_beat == onset)
                .map(|n| n.pitch)
                .collect()
        })
        .collect();
    (onsets, chords)
}
