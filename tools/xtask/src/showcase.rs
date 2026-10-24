//! Writes the showcase reports: one HTML report per large source tree in `examples/`, and
//! the terminal capture that the website shows for one of them.
//!
//! CCM runs from each tree's parent directory with the tree's name as a relative path, so
//! the reports name `linux-6.12/...` rather than a path on the machine that made them.
use std::path::{Path, PathBuf};
use std::process::Command;

/// The terminal width of the capture, which matches the rendered terminal window.
const COLUMNS: &str = "72";

/// Runs `cargo xtask showcase [--terminal NAME] NAME=PATH...`. Each tree is measured with
/// the release build and written to `examples/NAME.html`; the tree named by `--terminal`
/// also gets `docs/shared/_static/NAME-terminal.ansi`, which `cargo xtask docs` renders.
pub(super) fn run(arguments: &[String]) -> Result<(), String> {
    let Request { terminal, trees } = parse(arguments)?;
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .map_err(error)?;
    let status = Command::new("cargo")
        .current_dir(&root)
        .args(["build", "--release", "-p", "ccm"])
        .status()
        .map_err(error)?;
    if !status.success() {
        return Err("cargo build --release -p ccm failed".into());
    }
    let executable = root.join("target/release/ccm");
    for (name, path) in &trees {
        let (parent, tree) = split(path)?;
        let output = root.join("examples").join(format!("{name}.html"));
        let mut report = Command::new(&executable);
        report
            .current_dir(&parent)
            .arg(&tree)
            .args(["--html", "--force", "--output"])
            .arg(&output);
        run_ccm(&mut report)?;
        println!("{name}: {}", output.display());
        if terminal.as_deref() == Some(name.as_str()) {
            let mut capture = Command::new(&executable);
            capture
                .current_dir(&parent)
                .env("COLUMNS", COLUMNS)
                .args(["--color=always"])
                .arg(&tree);
            let text = run_ccm(&mut capture)?;
            let ansi = root.join(format!("docs/shared/_static/{name}-terminal.ansi"));
            std::fs::write(&ansi, text).map_err(error)?;
            println!("{name}: {}", ansi.display());
        }
    }
    Ok(())
}

/// The trees to measure, by name, and the one whose terminal output we capture.
struct Request {
    terminal: Option<String>,
    trees: Vec<(String, PathBuf)>,
}

/// Reads `--terminal NAME` and the `NAME=PATH` pairs. Names become file names, so they
/// hold only letters, digits, `-`, and `_`.
fn parse(arguments: &[String]) -> Result<Request, String> {
    const USAGE: &str = "Usage: cargo xtask showcase [--terminal NAME] NAME=PATH...";
    let mut terminal = None;
    let mut trees = Vec::new();
    let mut arguments = arguments.iter();
    while let Some(argument) = arguments.next() {
        if argument == "--terminal" {
            terminal = Some(arguments.next().ok_or(USAGE)?.clone());
            continue;
        }
        let (name, path) = argument.split_once('=').ok_or(USAGE)?;
        let plain = name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'));
        if name.is_empty() || !plain {
            return Err(format!(
                "Showcase name `{name}` must be letters, digits, - or _."
            ));
        }
        trees.push((name.to_owned(), PathBuf::from(path)));
    }
    if trees.is_empty() {
        return Err(USAGE.into());
    }
    if let Some(name) = &terminal {
        if !trees.iter().any(|(tree, _)| tree == name) {
            return Err(format!(
                "--terminal names `{name}`, which no NAME=PATH defines."
            ));
        }
    }
    Ok(Request { terminal, trees })
}

/// Splits a tree path into its canonical parent directory and its final name.
fn split(path: &Path) -> Result<(PathBuf, PathBuf), String> {
    let path = path
        .canonicalize()
        .map_err(|e| format!("{}: {e}", path.display()))?;
    let parent = path.parent().ok_or("A showcase tree cannot be the root")?;
    let name = path.file_name().ok_or("A showcase tree needs a name")?;
    Ok((parent.to_path_buf(), PathBuf::from(name)))
}

/// Runs CCM and returns its standard output. Exit status 2 means an incomplete report,
/// which a large tree may give for a few files that no rule can count, so we accept it
/// and pass its notices on.
fn run_ccm(command: &mut Command) -> Result<Vec<u8>, String> {
    let output = command
        .output()
        .map_err(|e| format!("Cannot run {command:?}: {e}"))?;
    match output.status.code() {
        Some(0) => Ok(output.stdout),
        Some(2) => {
            eprint!("{}", String::from_utf8_lossy(&output.stderr));
            Ok(output.stdout)
        }
        _ => Err(format!(
            "{command:?}:\n{}",
            String::from_utf8_lossy(&output.stderr)
        )),
    }
}

fn error(value: impl std::fmt::Display) -> String {
    value.to_string()
}

#[cfg(test)]
mod tests {
    use super::parse;

    #[test]
    fn showcase_arguments_name_trees_and_one_terminal() {
        let arguments = ["--terminal", "linux", "linux=a/linux", "bsd=b"].map(String::from);
        let request = parse(&arguments).unwrap();
        assert_eq!(request.terminal.as_deref(), Some("linux"));
        assert_eq!(request.trees.len(), 2);
        assert!(parse(&["../x=a".into()]).is_err());
        assert!(parse(&["--terminal".into(), "z".into(), "a=b".into()]).is_err());
        assert!(parse(&[]).is_err());
    }
}
