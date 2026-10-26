use std::collections::BTreeMap;
use std::fs;
use std::hash::{Hash, Hasher};
use std::io;
use std::path::{Path, PathBuf};

use legato_core::performance::Setup;
use legato_core::score::model::Variation;
use legato_core::score::musicxml_read::{InvalidReason, InvalidScoreFile};
use legato_core::score::musicxml_write;
use legato_core::works::DisplayVariant;
use legato_core::works::detect::read_with_rules;
use serde::{Deserialize, Serialize};

use crate::session::PieceSetups;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PieceInfo {
    pub id: String,
    pub work: String,
    pub composer: Option<String>,
    pub title: String,
    pub number: Option<String>,
    pub has_fingering: bool,
    pub display_variants: Vec<DisplayVariant>,
}

#[derive(Clone, Debug)]
struct StoredPiece {
    info: PieceInfo,
    musicxml: String,
    /// Copy in `data_dir/pieces`.
    file: PathBuf,
}

#[derive(Clone, Debug, Default)]
pub struct Repertoire {
    data_dir: PathBuf,
    pieces: BTreeMap<String, StoredPiece>,
}

impl Repertoire {
    pub fn load(data_dir: &Path, resource_dir: &Path) -> io::Result<Self> {
        let mut repertoire = Self {
            data_dir: data_dir.to_path_buf(),
            pieces: BTreeMap::new(),
        };
        repertoire.ensured_pieces_dir()?;

        for entry in fs::read_dir(repertoire.pieces_dir())? {
            let path = entry?.path();
            if path.is_dir() {
                continue;
            }
            if matches!(
                path.extension().and_then(|s| s.to_str()),
                Some("musicxml" | "mxl")
            ) {
                let _ = repertoire.load_piece(&path);
            }
        }

        // Last, so that it replaces copies of older app versions.
        if let Some(bundled) = bundled_hanon(resource_dir) {
            let _ = repertoire.load_piece(&bundled);
        }

        Ok(repertoire)
    }

    pub fn list(&self) -> Vec<PieceInfo> {
        let mut items: Vec<_> = self
            .pieces
            .values()
            .map(|piece| piece.info.clone())
            .collect();
        items.sort_by(|lhs, rhs| {
            (
                &lhs.composer,
                &lhs.title,
                lhs.number.as_deref().map(number_key),
                &lhs.id,
            )
                .cmp(&(
                    &rhs.composer,
                    &rhs.title,
                    rhs.number.as_deref().map(number_key),
                    &rhs.id,
                ))
        });
        items
    }

    pub fn load_piece(&mut self, path: &Path) -> Result<PieceInfo, InvalidScoreFile> {
        let bytes = fs::read(path)
            .map_err(|e| InvalidScoreFile::from(InvalidReason::Unreadable(e.to_string())))?;
        let (read, rules) = read_with_rules(&bytes)?;
        let id = format!("0x{:016x}", content_hash(&bytes));
        let number = read.metadata.movement_number.clone();
        let title = read
            .metadata
            .work_title
            .clone()
            .or_else(|| path.file_stem().map(|s| s.to_string_lossy().to_string()))
            .unwrap_or_else(|| "Untitled".to_string());
        let has_fingering = read
            .score
            .onsets
            .iter()
            .flat_map(|o| &o.notes)
            .all(|note| note.finger.is_some());
        let info = PieceInfo {
            id: id.clone(),
            work: rules.work_id().to_owned(),
            composer: read.metadata.composer.clone(),
            title,
            number,
            has_fingering,
            display_variants: rules.display_variants(),
        };

        let destination = self.pieces_dir().join(format!("{id}.musicxml"));
        if !self.pieces.contains_key(&id) {
            // Another version of the same piece is replaced, not listed twice.
            let replaced: Vec<String> = self
                .pieces
                .values()
                .filter(|p| {
                    (&p.info.composer, &p.info.title, &p.info.number)
                        == (&info.composer, &info.title, &info.number)
                })
                .map(|p| p.info.id.clone())
                .collect();
            for old in replaced {
                if let Some(old) = self.pieces.remove(&old) {
                    let _ = fs::remove_file(old.file);
                }
            }
            // Stored copies may carry an older id in their name; they stay where they are.
            let file = if path.parent() == Some(self.pieces_dir().as_path()) {
                path.to_path_buf()
            } else {
                fs::create_dir_all(self.pieces_dir()).map_err(|e| {
                    InvalidScoreFile::from(InvalidReason::Unreadable(e.to_string()))
                })?;
                fs::write(&destination, &bytes).map_err(|e| {
                    InvalidScoreFile::from(InvalidReason::Unreadable(e.to_string()))
                })?;
                destination
            };
            self.pieces.insert(
                id.clone(),
                StoredPiece {
                    info: info.clone(),
                    musicxml: String::from_utf8_lossy(&bytes).into_owned(),
                    file,
                },
            );
        }

        Ok(info)
    }

    pub fn render_score(
        &self,
        piece_id: &str,
        transposition: i8,
        _variation: Variation,
        display_variant: DisplayVariant,
    ) -> Option<String> {
        let piece = self.pieces.get(piece_id)?;
        if !piece.info.display_variants.contains(&display_variant) {
            return None;
        }
        let source = piece.musicxml.as_bytes();
        let (read, rules) = read_with_rules(source).ok()?;
        let shown = rules.shown_measures(&read.score, display_variant);
        musicxml_write::write(source, transposition, shown.as_deref()).ok()
    }

    /// Score and candidates of every piece for the recognition.
    pub(crate) fn setups(&self) -> Vec<PieceSetups> {
        self.pieces
            .values()
            .filter_map(|piece| {
                let (read, rules) = read_with_rules(piece.musicxml.as_bytes()).ok()?;
                let range = rules.performance_range(&read.score);
                let setups = rules
                    .candidates(&piece.info.id)
                    .into_iter()
                    .map(|c| Setup::new(&read.score, c, range))
                    .collect();
                Some(PieceSetups {
                    id: piece.info.id.clone(),
                    score: read.score,
                    setups,
                    recognition_length: rules.recognition_length(),
                })
            })
            .collect()
    }

    fn pieces_dir(&self) -> PathBuf {
        self.data_dir.join("pieces")
    }

    fn ensured_pieces_dir(&mut self) -> io::Result<()> {
        fs::create_dir_all(self.pieces_dir())
    }
}

fn bundled_hanon(resource_dir: &Path) -> Option<PathBuf> {
    let path = resource_dir.join("pieces").join("hanon-01.musicxml");
    path.is_file().then_some(path)
}

/// Sorts "2" before "10" and "1a" before "1b".
fn number_key(number: &str) -> (u32, &str) {
    let digits = number.len()
        - number
            .trim_start_matches(|c: char| c.is_ascii_digit())
            .len();
    (
        number[..digits].parse().unwrap_or(u32::MAX),
        &number[digits..],
    )
}

fn content_hash(bytes: &[u8]) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    bytes.hash(&mut hasher);
    hasher.finish()
}
