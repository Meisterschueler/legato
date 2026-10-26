//! Writes the MusicXML for the display: the source document, transposed and, for an
//! abbreviated display variant, reduced to some measures (FR-011).

use super::musicxml_read::{InvalidScoreFile, midi_pitch, parse_document};
use super::xml::Element;

const STEPS: [&str; 7] = ["C", "D", "E", "F", "G", "A", "B"];
const STEP_SEMITONES: [i32; 7] = [0, 2, 4, 5, 7, 9, 11];

/// A measure of an abbreviated display, by index in document order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Shown {
    pub measure: u32,
    /// Marks the left-out measures before it with *simile*.
    pub simile: bool,
    pub new_system: bool,
}

/// Transposes `source` (MusicXML or `.mxl`) up by `transposition` half tones (0–11) and keeps
/// only the `shown` measures (all if `None`).
/// Display ids of the complete result: `Note::display_id(transposition)`.
pub fn write(
    source: &[u8],
    transposition: i8,
    shown: Option<&[Shown]>,
) -> Result<String, InvalidScoreFile> {
    let mut root = parse_document(source)?;
    let semitones = i32::from(transposition).rem_euclid(12);
    transpose(&mut root, semitones, &mut None);
    if let Some(shown) = shown {
        abbreviate(&mut root, shown);
    }
    let mut out = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    root.write(&mut out);
    Ok(out)
}

fn abbreviate(root: &mut Element, shown: &[Shown]) {
    let find = |index: usize| shown.iter().find(|s| s.measure as usize == index);
    let lone_repeats = root
        .child("part")
        .map(|part| lone_repeats(part, |m| find(m).is_some()))
        .unwrap_or_default();
    let text = |name: &str, text: &str| Element {
        name: name.into(),
        text: text.into(),
        ..Element::default()
    };
    let parts = root.children.iter_mut().filter(|c| c.name == "part");
    for (part_index, part) in parts.enumerate() {
        // Clef, key and time changes of left-out measures still apply.
        let mut carried = Vec::new();
        for (index, mut measure) in std::mem::take(&mut part.children).into_iter().enumerate() {
            let Some(s) = find(index) else {
                carried.extend(
                    measure
                        .children
                        .into_iter()
                        .filter(|c| c.name == "attributes"),
                );
                continue;
            };
            if lone_repeats.contains(&index) {
                measure
                    .children
                    .retain(|c| !(c.name == "barline" && c.has("repeat")));
            }
            let mut head = Vec::new();
            if s.new_system {
                head.push(Element {
                    name: "print".into(),
                    attributes: vec![("new-system".into(), "yes".into())],
                    ..Element::default()
                });
            }
            head.append(&mut carried);
            if s.simile && part_index == 0 {
                head.push(Element {
                    name: "direction".into(),
                    attributes: vec![("placement".into(), "above".into())],
                    children: vec![Element {
                        name: "direction-type".into(),
                        children: vec![text("words", "simile")],
                        ..Element::default()
                    }],
                    ..Element::default()
                });
            }
            measure.children.splice(0..0, head);
            part.children.push(measure);
        }
    }
}

/// Measures with a repeat barline whose partner is not shown.
fn lone_repeats(part: &Element, is_shown: impl Fn(usize) -> bool) -> Vec<usize> {
    let mut lone = Vec::new();
    let mut open = None;
    let mut section_start = 0;
    for (index, measure) in part.children("measure").enumerate() {
        for repeat in measure
            .children("barline")
            .filter_map(|b| b.child("repeat"))
        {
            if repeat.attribute("direction") == Some("forward") {
                open = Some(index);
                continue;
            }
            // A backward repeat without forward repeat goes back to the previous section.
            let start = open.take().unwrap_or(section_start);
            if !(is_shown(start) && is_shown(index)) {
                lone.extend([start, index]);
            }
            section_start = index + 1;
        }
    }
    lone.extend(open);
    lone
}

/// Walks the document; `fifths_shift` is fixed by the first key signature.
fn transpose(element: &mut Element, semitones: i32, fifths_shift: &mut Option<i32>) {
    match element.name.as_str() {
        "key" => {
            if let Some(fifths) = element.child_mut("fifths") {
                let old: i32 = fifths.text.trim().parse().unwrap_or(0);
                let shift = *fifths_shift.get_or_insert_with(|| key_shift(old, semitones));
                fifths.text = (old + shift).to_string();
            }
            return;
        }
        "note" => {
            // Stale after transposition; OSMD derives accidentals from pitch and key.
            element.children.retain(|c| c.name != "accidental");
        }
        "pitch" => {
            let shift = *fifths_shift.get_or_insert_with(|| key_shift(0, semitones));
            transpose_pitch(element, semitones, shift);
            return;
        }
        _ => {}
    }
    for child in &mut element.children {
        transpose(child, semitones, fifths_shift);
    }
}

/// Shift in the circle of fifths for `semitones` up, keeping the new key within ±6 accidentals
/// (flats preferred at 6).
fn key_shift(fifths: i32, semitones: i32) -> i32 {
    let up = (7 * semitones).rem_euclid(12);
    let down = up - 12;
    if (fifths + up).abs() < (fifths + down).abs() {
        up
    } else {
        down
    }
}

fn transpose_pitch(pitch: &mut Element, semitones: i32, fifths_shift: i32) {
    let Some(midi) = midi_pitch(pitch) else {
        return;
    };
    let step_index = STEPS
        .iter()
        .position(|s| Some(*s) == pitch.child_text("step"))
        .unwrap_or(0) as i32;
    let octave: i32 = pitch
        .child_text("octave")
        .and_then(|o| o.parse().ok())
        .unwrap_or(4);
    // A fifth is four diatonic steps.
    let steps = step_index + (4 * fifths_shift).rem_euclid(7);
    let new_step = steps % 7;
    let new_octave = octave + steps / 7;
    let natural = (new_octave + 1) * 12 + STEP_SEMITONES[new_step as usize];
    let alter = i32::from(midi) + semitones - natural;

    let text = |name: &str, text: String| Element {
        name: name.into(),
        text,
        ..Element::default()
    };
    pitch.children = vec![text("step", STEPS[new_step as usize].into())];
    if alter != 0 {
        pitch.children.push(text("alter", alter.to_string()));
    }
    pitch.children.push(text("octave", new_octave.to_string()));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn note(step: &str, alter: i32, octave: i32) -> String {
        let alter = if alter == 0 {
            String::new()
        } else {
            format!("<alter>{alter}</alter>")
        };
        format!(
            "<score-partwise><part id=\"P1\"><measure number=\"1\"><attributes><key><fifths>0</fifths></key></attributes>\
             <note><pitch><step>{step}</step>{alter}<octave>{octave}</octave></pitch><duration>1</duration><accidental>sharp</accidental></note>\
             </measure></part></score-partwise>"
        )
    }

    fn transposed(source: &str, semitones: i8) -> String {
        write(source.as_bytes(), semitones, None).unwrap()
    }

    #[test]
    fn abbreviation_keeps_shown_measures_and_their_attributes() {
        let measure =
            |n: usize, content: &str| format!("<measure number=\"{n}\">{content}</measure>");
        let forward = "<barline location=\"left\"><repeat direction=\"forward\"/></barline>";
        let backward = "<barline location=\"right\"><repeat direction=\"backward\"/></barline>";
        let source = format!(
            "<score-partwise><part id=\"P1\">{}{}{}{}</part></score-partwise>",
            measure(1, forward),
            measure(2, "<attributes><clef><sign>F</sign></clef></attributes>"),
            measure(3, backward),
            measure(4, ""),
        );
        let shown = |measure, simile, new_system| Shown {
            measure,
            simile,
            new_system,
        };

        let out = write(
            source.as_bytes(),
            0,
            Some(&[shown(0, false, false), shown(2, true, false)]),
        )
        .unwrap();
        assert!(
            !out.contains("number=\"2\"") && !out.contains("number=\"4\""),
            "{out}"
        );
        assert!(
            out.contains(
                "<measure number=\"3\"><attributes><clef><sign>F</sign></clef></attributes>\
                 <direction placement=\"above\"><direction-type><words>simile</words>"
            ),
            "{out}"
        );
        assert!(out.contains("forward") && out.contains("backward"), "{out}");

        // Without its partner, a repeat barline goes.
        let out = write(
            source.as_bytes(),
            0,
            Some(&[shown(0, false, false), shown(3, false, true)]),
        )
        .unwrap();
        assert!(!out.contains("repeat"), "{out}");
        assert!(
            out.contains("<measure number=\"4\"><print new-system=\"yes\"/>"),
            "{out}"
        );
        assert!(!out.contains("simile"), "{out}");
    }

    #[test]
    fn c_major_up_one_is_d_flat_major() {
        let out = transposed(&note("C", 0, 4), 1);
        assert!(out.contains("<fifths>-5</fifths>"), "{out}");
        assert!(
            out.contains("<step>D</step><alter>-1</alter><octave>4</octave>"),
            "{out}"
        );
        assert!(!out.contains("accidental"));
    }

    #[test]
    fn spelling_follows_the_new_key() {
        // C major up 2 = D major: F# stays a sharp, B becomes C#5.
        let out = transposed(&note("B", 0, 4), 2);
        assert!(out.contains("<fifths>2</fifths>"), "{out}");
        assert!(
            out.contains("<step>C</step><alter>1</alter><octave>5</octave>"),
            "{out}"
        );
        // Up 6: flats preferred (G flat major).
        let out = transposed(&note("C", 0, 4), 6);
        assert!(out.contains("<fifths>-6</fifths>"), "{out}");
        assert!(
            out.contains("<step>G</step><alter>-1</alter><octave>4</octave>"),
            "{out}"
        );
    }

    #[test]
    fn every_transposition_keeps_the_pitch_distance() {
        for semitones in 0..12 {
            let out = transposed(&note("E", 0, 3), semitones);
            let tree = super::super::xml::parse(&out).unwrap();
            let pitch = tree
                .child("part")
                .unwrap()
                .child("measure")
                .unwrap()
                .child("note")
                .unwrap()
                .child("pitch")
                .unwrap();
            assert_eq!(midi_pitch(pitch), Some(52 + semitones as u8), "{semitones}");
        }
    }
}
