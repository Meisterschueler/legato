//! Writes one MusicXML file per exercise:
//! `cargo run -p exercise-musicxml [-- <description.toml> [<output directory>]]`,
//! by default every docs/werke/*.toml to testdata/legato/<composer>/.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

fn main() -> ExitCode {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut args = std::env::args().skip(1);
    let result = match args.next() {
        Some(input) => run(&root, Path::new(&input), args.next().map(PathBuf::from)),
        None => descriptions(&root.join("docs/werke"))
            .and_then(|inputs| inputs.iter().try_for_each(|i| run(&root, i, None))),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{e}");
            ExitCode::FAILURE
        }
    }
}

fn descriptions(dir: &Path) -> Result<Vec<PathBuf>, String> {
    let entries = std::fs::read_dir(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    let mut paths: Vec<PathBuf> = entries
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|x| x == "toml"))
        .collect();
    paths.sort();
    Ok(paths)
}

fn run(root: &Path, input: &Path, output: Option<PathBuf>) -> Result<(), String> {
    let text = std::fs::read_to_string(input).map_err(|e| format!("{}: {e}", input.display()))?;
    let book = exercise_musicxml::parse(&text).map_err(|e| format!("{}: {e}", input.display()))?;
    let output = output.unwrap_or_else(|| {
        root.join("testdata/legato")
            .join(exercise_musicxml::prefix(&book))
    });
    std::fs::create_dir_all(&output).map_err(|e| format!("{}: {e}", output.display()))?;
    for exercise in &book.exercise {
        let xml = exercise_musicxml::generate(&book, exercise)?;
        let path = output.join(exercise_musicxml::file_name(&book, exercise));
        std::fs::write(&path, xml).map_err(|e| format!("{}: {e}", path.display()))?;
        println!("{}", path.display());
    }
    Ok(())
}
