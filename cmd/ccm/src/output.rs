use crate::{Args, Error};
use std::io::Write;

/// Writes a rendered report to `--output` or standard output within the report budget.
/// A closed pipe is not an error.
pub(crate) fn write(args: &Args, output: &str) -> Result<(), Error> {
    if output.len() > args.max_report_bytes {
        return Err(crate::report::limit_error());
    }
    let result = if let Some(path) = &args.output {
        if args.force {
            replace(path, output.as_bytes())
        } else {
            create(path, output.as_bytes())
        }
    } else {
        std::io::stdout().lock().write_all(output.as_bytes())
    };
    match result {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::BrokenPipe => Ok(()),
        Err(e) => Err(Error::new(
            "report-io",
            e.to_string(),
            "Choose a new filename, or use --force to replace an existing report.",
        )),
    }
}

/// Writes a new report file and never replaces an existing one. A failed write removes the
/// partial file, so a report on disk is always complete.
fn create(path: &std::path::Path, output: &[u8]) -> std::io::Result<()> {
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)?;
    let result = file.write_all(output).and_then(|()| file.sync_all());
    if result.is_err() {
        drop(file);
        let _ = std::fs::remove_file(path);
    }
    result
}

/// Replaces a report atomically: a synced temporary file in the same directory is renamed
/// over the target, so readers see either the old or the new report.
fn replace(path: &std::path::Path, output: &[u8]) -> std::io::Result<()> {
    use std::sync::atomic::{AtomicU64, Ordering};
    static NEXT: AtomicU64 = AtomicU64::new(0);
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(std::path::Path::new("."));
    let temporary = parent.join(format!(
        ".ccm-{}-{}.tmp",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary)?;
    let result = (|| {
        file.write_all(output)?;
        file.sync_all()?;
        drop(file);
        std::fs::rename(&temporary, path)
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(&temporary);
    }
    result
}
