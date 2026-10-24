//! Builds an equal-input benchmark corpus: the files that CCM, scc, and cloc all select.
//!
//! The three tools choose files by different default policies, so timing them on the same
//! directory would compare unequal work. We copy the intersection of their selections
//! into a fresh directory with an empty `.git` repository boundary, and we write a
//! manifest of each file's path, size, and SHA-256 so the corpus can be checked later.
//! Corpora and manifests stay under `target/`; they hold other projects' files, which we
//! do not commit.
use crate::bench::csv_fields;
use crate::sha256::hex_digest;
use std::collections::BTreeSet;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Runs `cargo xtask corpus [--ccm PATH] [--no-cloc] [--cloc-processes N] SOURCE
/// DESTINATION`. The manifest is written next to the destination as `DESTINATION.csv`.
/// An existing destination is replaced only when it holds a `.git` boundary that this
/// command created. `--ccm` selects the CCM executable, by default `target/release/ccm`;
/// `--no-cloc` leaves cloc out of the selection.
pub(super) fn run(arguments: &[String]) -> Result<(), String> {
    let (programs, arguments) = crate::bench::programs(arguments)?;
    let [source, destination] = arguments.as_slice() else {
        return Err(concat!(
            "Usage: cargo xtask corpus [--ccm PATH] [--no-cloc] [--cloc-processes N] ",
            "SOURCE DESTINATION"
        )
        .into());
    };
    let source = Path::new(source).canonicalize().map_err(error)?;
    let destination = PathBuf::from(destination);
    let ours = selection(
        &programs.ccm,
        &["--no-cost", "--by-file", "--format", "csv"],
        &source,
        0,
    )?;
    let theirs = selection(
        Path::new("scc"),
        &[
            "--by-file",
            "--format",
            "csv",
            "--count-ignore",
            "--no-config",
        ],
        &source,
        1,
    )?;
    let cloc = if programs.cloc {
        Some(cloc_selection(&source, programs.cloc_processes)?)
    } else {
        None
    };
    let common: Vec<_> = ours
        .intersection(&theirs)
        .filter(|path| cloc.as_ref().is_none_or(|cloc| cloc.contains(*path)))
        .collect();
    prepare(&destination)?;
    let manifest_path = destination.with_extension("csv");
    let mut manifest = std::fs::File::create(&manifest_path).map_err(error)?;
    writeln!(manifest, "path,bytes,sha256").map_err(error)?;
    let mut bytes = 0u64;
    for relative in &common {
        let data = std::fs::read(source.join(relative)).map_err(error)?;
        let target = destination.join(relative);
        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent).map_err(error)?;
        }
        std::fs::write(&target, &data).map_err(error)?;
        let digest = hex_digest(&data);
        writeln!(manifest, "{},{},{digest}", csv_field(relative), data.len()).map_err(error)?;
        bytes += data.len() as u64;
    }
    println!(
        "CCM selects {}, scc {}, cloc {}; copied {} common files, {bytes} bytes.",
        ours.len(),
        theirs.len(),
        cloc.map_or("not run".into(), |cloc| cloc.len().to_string()),
        common.len()
    );
    println!("Manifest: {}", manifest_path.display());
    Ok(())
}

/// Returns the root-relative paths a tool selects, read from its per-file CSV report.
/// `column` is the CSV column that holds the path.
fn selection(
    program: &Path,
    options: &[&str],
    source: &Path,
    column: usize,
) -> Result<BTreeSet<String>, String> {
    let output = Command::new(program)
        .args(options)
        .arg(source)
        .env("NO_COLOR", "1")
        .output()
        .map_err(|e| format!("Cannot run {}: {e}", program.display()))?;
    // CCM exits with 2 when some files could not be counted; its CSV still lists the rest.
    if !output.status.success() && output.status.code() != Some(2) {
        return Err(String::from_utf8_lossy(&output.stderr).into_owned());
    }
    let text = String::from_utf8(output.stdout).map_err(error)?;
    let mut paths = BTreeSet::new();
    for row in text.lines().skip(1) {
        let fields = csv_fields(row)?;
        let path = fields.get(column).ok_or("Missing CSV path column")?;
        let path = Path::new(path);
        let relative = path.strip_prefix(source).unwrap_or(path);
        paths.insert(relative.to_string_lossy().into_owned());
    }
    Ok(paths)
}

/// Returns the root-relative paths that cloc counts. We turn off its check for duplicate
/// files, which the other tools do not make, and let it start `processes` processes.
/// A tab separates the fields, because cloc does not quote a path that holds a comma.
fn cloc_selection(source: &Path, processes: usize) -> Result<BTreeSet<String>, String> {
    let output = Command::new("cloc")
        .args([
            "--by-file",
            "--csv",
            "--csv-delimiter=\t",
            "--quiet",
            "--skip-uniqueness",
        ])
        .arg(format!("--processes={processes}"))
        .arg(source)
        .output()
        .map_err(|e| format!("Cannot run cloc: {e}"))?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).into_owned());
    }
    let text = String::from_utf8_lossy(&output.stdout);
    let mut paths = BTreeSet::new();
    for row in text.lines().filter(|row| !row.trim().is_empty()).skip(1) {
        let mut fields = row.split('\t');
        let (Some(language), Some(path)) = (fields.next(), fields.next()) else {
            return Err(format!("Unexpected cloc row: {row}"));
        };
        if language == "SUM" {
            continue;
        }
        let path = Path::new(path);
        let relative = path.strip_prefix(source).unwrap_or(path);
        paths.insert(relative.to_string_lossy().into_owned());
    }
    Ok(paths)
}

/// Creates an empty destination with a `.git` boundary, so the corpus does not inherit
/// ignore rules from an enclosing repository.
fn prepare(destination: &Path) -> Result<(), String> {
    if destination.exists() {
        let boundary = destination.join(".git");
        let empty = std::fs::read_dir(&boundary).is_ok_and(|mut d| d.next().is_none());
        if !empty {
            return Err(format!(
                "Refusing to replace {}, which is not a corpus made by this command",
                destination.display()
            ));
        }
        std::fs::remove_dir_all(destination).map_err(error)?;
    }
    std::fs::create_dir_all(destination.join(".git")).map_err(error)
}

/// Quotes a CSV field only when it holds a comma, a quote, or a line break.
fn csv_field(value: &str) -> String {
    if value.contains([',', '"', '\n', '\r']) {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_owned()
    }
}

fn error(error: impl std::fmt::Display) -> String {
    error.to_string()
}
