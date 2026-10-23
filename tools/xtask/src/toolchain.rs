use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::Command;

pub(super) fn require(program: &str, purpose: &str, hint: &str) -> Result<PathBuf, String> {
    let search = std::env::var_os("PATH").unwrap_or_default();
    let path = resolve(program, &search).ok_or_else(|| missing(program, purpose, hint))?;
    let version = Command::new(&path)
        .arg("--version")
        .output()
        .map_err(|_| missing(program, purpose, hint))?;
    if !version.status.success() {
        return Err(missing(program, purpose, hint));
    }
    Ok(path)
}

fn resolve(program: &str, search: &OsStr) -> Option<PathBuf> {
    let path = Path::new(program);
    if path.components().count() > 1 {
        return path.is_file().then(|| path.to_path_buf());
    }
    let executable = format!("{program}{}", std::env::consts::EXE_SUFFIX);
    std::env::split_paths(search)
        .filter(|path| !path.as_os_str().is_empty())
        .map(|path| path.join(&executable))
        .find(|path| path.is_file())
}

fn missing(program: &str, purpose: &str, hint: &str) -> String {
    format!(
        "-- MISSING PUBLICATION TOOL [{program}.missing] --------------------\n\n\
        We could not run {program}.\n\n{purpose}\n\nHint: {hint}\n"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn absent_typst_is_diagnosed_without_changing_the_process_environment() {
        assert!(resolve("typst", OsStr::new("")).is_none());
        let report = missing(
            "typst",
            "Typst CLI renders SVG and PNG assets.",
            "Install https://typst.app/open-source/ and run typst --version.",
        );
        assert!(report.contains("[typst.missing]"));
        assert!(report.contains("Hint:") && report.contains("typst --version"));
    }
}
