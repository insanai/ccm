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

/// Chromium-based browsers that take headless screenshots, by program name on PATH.
const BROWSER_NAMES: &[&str] = &[
    "chromium",
    "chromium-browser",
    "google-chrome",
    "google-chrome-stable",
    "chrome",
    "microsoft-edge",
    "msedge",
];

/// Where macOS and Windows install those browsers outside PATH. On macOS we also look
/// under the user's own `~/Applications`.
const BROWSER_PATHS: &[&str] = &[
    "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
    "/Applications/Chromium.app/Contents/MacOS/Chromium",
    "/Applications/Microsoft Edge.app/Contents/MacOS/Microsoft Edge",
    r"C:\Program Files\Google\Chrome\Application\chrome.exe",
    r"C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe",
];

/// Finds a Chromium-based browser for screenshots. `CCM_BROWSER` names one explicitly,
/// as a program on PATH or a file path; otherwise we try the common names on PATH and
/// then the standard install locations. Returns `Ok(None)` when no browser is installed,
/// and an error when `CCM_BROWSER` names one that does not exist.
pub(super) fn browser() -> Result<Option<PathBuf>, String> {
    let search = std::env::var_os("PATH").unwrap_or_default();
    if let Some(chosen) = std::env::var("CCM_BROWSER").ok().filter(|s| !s.is_empty()) {
        return resolve(&chosen, &search).map(Some).ok_or_else(|| {
            missing(
                &chosen,
                "CCM_BROWSER names the browser that captures the screenshots.",
                "Set CCM_BROWSER to an installed Chrome, Chromium, or Edge executable.",
            )
        });
    }
    let home = std::env::var_os("HOME").map(PathBuf::from);
    let installed = BROWSER_PATHS.iter().flat_map(|path| {
        let user = path
            .strip_prefix('/')
            .filter(|p| p.starts_with("Applications/"))
            .and_then(|p| Some(home.as_ref()?.join(p)));
        [Some(PathBuf::from(path)), user]
    });
    Ok(BROWSER_NAMES
        .iter()
        .find_map(|name| resolve(name, &search))
        .or_else(|| installed.flatten().find(|path| path.is_file())))
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
