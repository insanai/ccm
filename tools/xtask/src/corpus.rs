//! Builds an equal-input benchmark corpus: the files that both CCM and scc select.
//!
//! The two tools choose files by different default policies, so timing them on the same
//! directory would compare unequal work. We copy the intersection of their selections
//! into a fresh directory with an empty `.git` repository boundary, and we write a
//! manifest of each file's path, size, and SHA-256 so the corpus can be checked later.
use crate::bench::csv_fields;
use crate::sha256::hex_digest;
use std::collections::BTreeSet;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Runs `cargo xtask corpus SOURCE DESTINATION`. The manifest is written next to the
/// destination as `DESTINATION.csv`. An existing destination is replaced only when it
/// holds a `.git` boundary that this command created.
pub(super) fn run(arguments: &[String]) -> Result<(), String> {
    let [source, destination] = arguments else {
        return Err("Usage: cargo xtask corpus SOURCE DESTINATION".into());
    };
    let source = Path::new(source).canonicalize().map_err(error)?;
    let destination = PathBuf::from(destination);
    let ccm = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/release/ccm");
    let ours = selection(
        &ccm,
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
    let common: Vec<_> = ours.intersection(&theirs).collect();
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
        "CCM selects {}, scc selects {}; copied {} common files, {bytes} bytes.",
        ours.len(),
        theirs.len(),
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
