//! Writes one MusicXML file per exercise:
//! `cargo run -p hanon-musicxml [-- <description.toml> <output directory>]`,
//! by default docs/werke/hanon-teil1.toml to testdata/legato/hanon/.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

fn main() -> ExitCode {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut args = std::env::args().skip(1);
    let input = args
        .next()
        .map(PathBuf::from)
        .unwrap_or_else(|| root.join("docs/werke/hanon-teil1.toml"));
    let output = args
        .next()
        .map(PathBuf::from)
        .unwrap_or_else(|| root.join("testdata/legato/hanon"));
    match run(&input, &output) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{e}");
            ExitCode::FAILURE
        }
    }
}

fn run(input: &Path, output: &Path) -> Result<(), String> {
    let text = std::fs::read_to_string(input).map_err(|e| format!("{}: {e}", input.display()))?;
    let book = hanon_musicxml::parse(&text)?;
    std::fs::create_dir_all(output).map_err(|e| format!("{}: {e}", output.display()))?;
    for exercise in &book.exercise {
        let xml = hanon_musicxml::generate(exercise)?;
        let path = output.join(hanon_musicxml::file_name(exercise.number));
        std::fs::write(&path, xml).map_err(|e| format!("{}: {e}", path.display()))?;
        println!("{}", path.display());
    }
    Ok(())
}
