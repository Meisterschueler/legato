//! Reads the MusicXML subset of contracts/notendatei.md.

use std::collections::BTreeMap;
use std::io::{Cursor, Read};

use super::model::{
    Fraction, Hand, Measure, Metadata, Note, NoteId, Onset, Score, Section, SectionKind,
};
use super::xml::{self, Element};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InvalidReason {
    /// Neither valid XML nor a readable `.mxl` archive.
    Unreadable(String),
    /// Only `score-partwise` is supported.
    NotPartwise,
    NoPitchedNotes,
    UnpairedRepeats,
    /// A work rule recognises the file as incompatible with the work specification.
    IncompatibleWithWork(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InvalidScoreFile {
    pub reason: InvalidReason,
}

impl From<InvalidReason> for InvalidScoreFile {
    fn from(reason: InvalidReason) -> Self {
        InvalidScoreFile { reason }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ReadScore {
    pub metadata: Metadata,
    pub score: Score,
}

pub fn read(bytes: &[u8]) -> Result<ReadScore, InvalidScoreFile> {
    let root = parse_document(bytes)?;
    if root.name != "score-partwise" {
        return Err(InvalidReason::NotPartwise.into());
    }
    let metadata = Metadata {
        composer: root
            .child("identification")
            .and_then(|i| {
                i.children("creator")
                    .find(|c| c.attribute("type") == Some("composer"))
            })
            .map(|c| c.text.trim().to_owned()),
        work_title: root
            .child("work")
            .and_then(|w| w.child_text("work-title"))
            .map(str::to_owned),
        movement_number: root.child_text("movement-number").map(str::to_owned),
    };
    Ok(ReadScore {
        metadata,
        score: read_score(&root)?,
    })
}

/// XML text, or `.mxl` (ZIP with `META-INF/container.xml`).
pub(crate) fn parse_document(bytes: &[u8]) -> Result<Element, InvalidScoreFile> {
    let unreadable = |message: String| InvalidScoreFile::from(InvalidReason::Unreadable(message));
    if !bytes.starts_with(b"PK") {
        let text = std::str::from_utf8(bytes).map_err(|e| unreadable(e.to_string()))?;
        return xml::parse(text).map_err(unreadable);
    }
    let mut archive =
        zip::ZipArchive::new(Cursor::new(bytes)).map_err(|e| unreadable(e.to_string()))?;
    let mut read_entry = |name: &str| -> Result<String, InvalidScoreFile> {
        let mut text = String::new();
        archive
            .by_name(name)
            .map_err(|e| unreadable(format!("{name}: {e}")))?
            .read_to_string(&mut text)
            .map_err(|e| unreadable(e.to_string()))?;
        Ok(text)
    };
    let container = xml::parse(&read_entry("META-INF/container.xml")?).map_err(unreadable)?;
    let path = container
        .child("rootfiles")
        .and_then(|r| r.child("rootfile"))
        .and_then(|r| r.attribute("full-path"))
        .ok_or_else(|| unreadable("container.xml without rootfile".into()))?
        .to_owned();
    xml::parse(&read_entry(&path)?).map_err(unreadable)
}

struct RepeatMark {
    time: Fraction,
    forward: bool,
}

fn read_score(root: &Element) -> Result<Score, InvalidScoreFile> {
    let parts: Vec<&Element> = root.children("part").take(2).collect();
    let two_parts = parts.len() == 2;
    let mut measures = Vec::new();
    let mut notes: Vec<(Fraction, Note)> = Vec::new();
    let mut rest_ends = Vec::new();
    let mut repeats = Vec::new();
    // Note index per measure across all parts (NoteId::index).
    let mut indices: Vec<u32> = Vec::new();

    for (part_index, part) in parts.iter().enumerate() {
        let mut divisions = 1i64;
        let mut measure_start = Fraction::ZERO;
        for (measure_index, measure) in part.children("measure").enumerate() {
            if part_index == 0 {
                measures.push(Measure {
                    number: measure.attribute("number").unwrap_or_default().to_owned(),
                });
            }
            if indices.len() <= measure_index {
                indices.push(0);
            }
            let mut position = 0i64;
            let mut end = 0i64;
            let mut last_start = 0i64;
            for child in &measure.children {
                let duration = || {
                    child
                        .child_text("duration")
                        .and_then(|d| d.parse::<i64>().ok())
                        .unwrap_or(0)
                };
                let at = |pos: i64| measure_start + Fraction::new(pos, divisions);
                match child.name.as_str() {
                    "attributes" => {
                        if let Some(d) = child.child_text("divisions").and_then(|d| d.parse().ok())
                        {
                            divisions = d;
                        }
                    }
                    "backup" => position -= duration(),
                    "forward" => position += duration(),
                    "barline" if part_index == 0 => {
                        if let Some(repeat) = child.child("repeat") {
                            let forward = repeat.attribute("direction") == Some("forward");
                            let pos = if forward { position } else { position.max(end) };
                            repeats.push(RepeatMark {
                                time: at(pos),
                                forward,
                            });
                        }
                    }
                    "note" if !child.has("grace") => {
                        let start = if child.has("chord") {
                            last_start
                        } else {
                            position
                        };
                        if !child.has("chord") {
                            last_start = position;
                            position += duration();
                        }
                        if child.has("rest") {
                            rest_ends.push(at(start + duration()));
                        } else if child.has("cue") {
                            // Cue notes take time but are not played.
                        } else if let Some(pitch) = child.child("pitch").and_then(midi_pitch) {
                            let tie_stop = child
                                .children("tie")
                                .any(|t| t.attribute("type") == Some("stop"));
                            let index = indices[measure_index];
                            indices[measure_index] += 1;
                            if !tie_stop {
                                notes.push((
                                    at(start),
                                    note(
                                        child,
                                        part_index,
                                        two_parts,
                                        measure_index,
                                        index,
                                        pitch,
                                        Fraction::new(start, divisions * 4),
                                    ),
                                ));
                            }
                        }
                    }
                    _ => {}
                }
                end = end.max(position);
            }
            measure_start = measure_start + Fraction::new(end, divisions);
        }
    }
    if notes.is_empty() {
        return Err(InvalidReason::NoPitchedNotes.into());
    }

    let mut by_time: BTreeMap<Fraction, Vec<Note>> = BTreeMap::new();
    for (time, note) in notes {
        by_time.entry(time).or_default().push(note);
    }
    let count = by_time.len() as u32;
    let mut onsets: Vec<Onset> = by_time
        .into_iter()
        .zip(0..)
        .map(|((time, notes), index)| Onset {
            index,
            time,
            notes,
            successors: if index + 1 < count {
                vec![index + 1]
            } else {
                vec![]
            },
        })
        .collect();

    let first_at = |time: Fraction| onsets.iter().position(|o| o.time >= time);
    let rests_before = rest_ends
        .into_iter()
        .filter_map(|t| onsets.iter().find(|o| o.time == t))
        .map(|o| o.index)
        .collect();

    let mut sections = Vec::new();
    let mut open: Option<Fraction> = None;
    let mut previous_end: Option<u32> = None;
    for mark in &repeats {
        if mark.forward {
            if open.is_some() {
                return Err(InvalidReason::UnpairedRepeats.into());
            }
            open = Some(mark.time);
            continue;
        }
        // A backward repeat without forward repeat goes back to the start or the previous section.
        let start = match open.take() {
            Some(time) => first_at(time),
            None => Some(previous_end.map_or(0, |e| e as usize + 1)),
        };
        let end = onsets.iter().rposition(|o| o.time < mark.time);
        if let (Some(start), Some(end)) = (start, end)
            && start <= end
        {
            sections.push(Section {
                kind: SectionKind::Repeat,
                start: start as u32,
                end: end as u32,
            });
            previous_end = Some(end as u32);
        }
    }
    if open.is_some() {
        return Err(InvalidReason::UnpairedRepeats.into());
    }
    for section in &sections {
        onsets[section.end as usize].successors.push(section.start);
    }

    Ok(Score {
        onsets,
        sections,
        rests_before,
        measures,
    })
}

fn note(
    element: &Element,
    part_index: usize,
    two_parts: bool,
    measure: usize,
    index: u32,
    pitch: u8,
    onset_in_measure: Fraction,
) -> Note {
    let staff = if two_parts {
        part_index as u8
    } else {
        element
            .child_text("staff")
            .and_then(|s| s.parse::<u8>().ok())
            .unwrap_or(1)
            .saturating_sub(1)
    };
    let finger = element
        .child("notations")
        .and_then(|n| n.child("technical"))
        .and_then(|t| t.child_text("fingering"))
        .and_then(|f| f.parse::<u8>().ok())
        .filter(|f| (1..=5).contains(f));
    Note {
        id: NoteId {
            measure: measure as u32,
            voice: element
                .child_text("voice")
                .and_then(|v| v.parse().ok())
                .unwrap_or(1),
            index,
        },
        pitch,
        hand: if staff == 0 { Hand::Right } else { Hand::Left },
        finger,
        staff,
        onset_in_measure,
    }
}

const STEP_SEMITONES: [(&str, i32); 7] = [
    ("C", 0),
    ("D", 2),
    ("E", 4),
    ("F", 5),
    ("G", 7),
    ("A", 9),
    ("B", 11),
];

pub(crate) fn midi_pitch(pitch: &Element) -> Option<u8> {
    let step = pitch.child_text("step")?;
    let semitone = STEP_SEMITONES.iter().find(|(s, _)| *s == step)?.1;
    let alter: i32 = pitch
        .child_text("alter")
        .map_or(Some(0), |a| a.parse::<f32>().ok().map(|a| a.round() as i32))?;
    let octave: i32 = pitch.child_text("octave")?.parse().ok()?;
    u8::try_from((octave + 1) * 12 + semitone + alter)
        .ok()
        .filter(|p| *p <= 127)
}
