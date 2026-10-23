use crate::{Args, Error};
use std::io::Write;

pub(crate) fn write(args: &Args, output: &str) -> Result<(), Error> {
    if output.len() > args.max_report_bytes {
        return Err(crate::report::limit_error());
    }
    let result = if let Some(path) = &args.output {
        if args.force {
            replace(path, output.as_bytes())
        } else {
            std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(path)
                .and_then(|mut file| file.write_all(output.as_bytes()))
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
