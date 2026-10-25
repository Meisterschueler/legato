//! Recognises the work from the MusicXML metadata.

use super::WorkRules;
use super::generic::GenericRules;
use super::hanon::HanonRules;
use crate::score::model::Metadata;
use crate::score::musicxml_read::{InvalidReason, InvalidScoreFile, ReadScore, read};

/// Reads a score file and checks it against the rules of its work.
pub fn read_with_rules(bytes: &[u8]) -> Result<(ReadScore, Box<dyn WorkRules>), InvalidScoreFile> {
    let read = read(bytes)?;
    let rules = detect(&read.metadata);
    rules
        .check(&read.score)
        .map_err(InvalidReason::IncompatibleWithWork)?;
    Ok((read, rules))
}

/// Hanon part 1 (exercises 1–20) by composer and movement number; general rules otherwise.
pub fn detect(metadata: &Metadata) -> Box<dyn WorkRules> {
    let hanon = metadata
        .composer
        .as_deref()
        .is_some_and(|c| c.to_lowercase().contains("hanon"));
    let part_one = metadata
        .movement_number
        .as_deref()
        .and_then(|n| n.trim().parse::<u32>().ok())
        .is_some_and(|n| (1..=20).contains(&n));
    if hanon && part_one {
        Box::new(HanonRules)
    } else {
        Box::new(GenericRules)
    }
}
