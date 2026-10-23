use crate::walk::SelectedFile;
use crate::{FileReport, Notice, Options, Report, io_error, limit};
use ccm_core::Error;
use ccm_scan::{BINARY_INPUT, is_binary, measure};
use std::io::Read;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, mpsc};

/// Files handed to a worker at a time. One handoff per file costs a thread wake-up per
/// file, which dominates when the walker and a worker share a core. Small batches keep
/// the work balanced across workers on small projects.
const BATCH: usize = 16;

pub(super) fn scan(paths: &[PathBuf], options: &Options) -> Result<Report, Error> {
    let jobs = options.jobs.clamp(1, 64);
    let (sender, receiver) = mpsc::sync_channel::<Vec<SelectedFile>>(jobs * 4);
    let shared = Arc::new(Mutex::new(receiver));
    let read_bytes = AtomicU64::new(0);
    let (selection, results) = std::thread::scope(|scope| {
        let mut handles = Vec::new();
        for _ in 0..jobs {
            let receiver = shared.clone();
            let read_bytes = &read_bytes;
            handles.push(scope.spawn(move || worker(options, receiver, read_bytes)));
        }
        // If every worker fails, dropping the last receiver disconnects a blocked sender.
        drop(shared);
        let mut batch = Vec::with_capacity(BATCH);
        let selection = crate::walk::select(paths, options, |file| {
            batch.push(file);
            if batch.len() == BATCH {
                let full = std::mem::replace(&mut batch, Vec::with_capacity(BATCH));
                sender.send(full).map_err(|_| worker_error())?;
            }
            Ok(())
        });
        // A failed send means every worker has stopped; their results report why.
        if !batch.is_empty() {
            let _ = sender.send(batch);
        }
        drop(sender);
        let results = handles
            .into_iter()
            .map(|handle| handle.join().map_err(|_| worker_error())?)
            .collect::<Result<Vec<_>, Error>>();
        (selection, results)
    });
    let results = results?;
    let mut report = selection?;
    for result in results {
        report.complete &= result.complete;
        report.files.extend(result.files);
        report.notices.extend(result.notices);
        report.skipped_binary += result.skipped_binary;
        report.skipped_generated += result.skipped_generated;
        report.skipped_oversized += result.skipped_oversized;
    }
    report
        .files
        .sort_by(|a, b| (&a.name, &a.path).cmp(&(&b.name, &b.path)));
    report
        .notices
        .sort_by(|a, b| (&a.path, &a.message).cmp(&(&b.path, &b.message)));
    for file in &report.files {
        report.totals.add(file.counts);
    }
    Ok(report)
}

struct WorkerReport {
    files: Vec<FileReport>,
    notices: Vec<Notice>,
    complete: bool,
    skipped_binary: usize,
    skipped_generated: usize,
    skipped_oversized: usize,
}

/// The size and modification time of a file as read.
type Stamp = (u64, Option<std::time::SystemTime>);

/// What reading one selected file produced.
enum Contents {
    /// The whole file is in the buffer.
    Read(Stamp),
    /// The file has this many bytes, more than `Options::max_file_bytes`.
    Oversized(u64),
}

fn worker(
    options: &Options,
    receiver: Arc<Mutex<mpsc::Receiver<Vec<SelectedFile>>>>,
    read_bytes: &AtomicU64,
) -> Result<WorkerReport, Error> {
    let mut report = WorkerReport {
        files: Vec::new(),
        notices: Vec::new(),
        complete: true,
        skipped_binary: 0,
        skipped_generated: 0,
        skipped_oversized: 0,
    };
    let mut buffer = Vec::new();
    loop {
        let received = receiver.lock().map_err(|_| worker_error())?.recv();
        let Ok(batch) = received else {
            break;
        };
        for file in batch {
            scan_file(file, &mut report, &mut buffer, options, read_bytes)?;
        }
    }
    Ok(report)
}

/// Reads and counts one selected file into `report`.
fn scan_file(
    file: SelectedFile,
    report: &mut WorkerReport,
    buffer: &mut Vec<u8>,
    options: &Options,
    read_bytes: &AtomicU64,
) -> Result<(), Error> {
    let stamp = match read_selected(&file, buffer, options, read_bytes)? {
        Contents::Read(stamp) => stamp,
        // One oversized file is a coverage gap, not a failure of the whole snapshot.
        Contents::Oversized(bytes) => {
            report.skipped_oversized += 1;
            report.complete = false;
            report.notices.push(Notice {
                path: file.name,
                message: format!(
                    "{bytes} bytes exceed the per-file limit of {} (--max-file-bytes)",
                    options.max_file_bytes
                ),
            });
            return Ok(());
        }
    };
    let generated = is_generated(buffer);
    if generated && options.skip_generated {
        if is_binary(buffer) {
            report.skipped_binary += 1;
        } else {
            report.skipped_generated += 1;
        }
        return Ok(());
    }
    // `measure` detects binary input in the same pass that reads it as text.
    match measure(buffer, file.language, options.collect_work) {
        Err(error) if error.code == BINARY_INPUT => report.skipped_binary += 1,
        Ok(measurement) => report.files.push(FileReport {
            name: file.name,
            path: file.path,
            language: file.language,
            counts: measurement.counts,
            source_tokens: measurement.source_tokens,
            stamp,
            generated,
        }),
        Err(error) => {
            report.complete = false;
            report.notices.push(Notice {
                path: file.name,
                message: error.message,
            });
        }
    }
    Ok(())
}

/// Looks for a generated-file marker, ignoring ASCII case, in the first 4 KiB.
fn is_generated(source: &[u8]) -> bool {
    const MARKERS: &[&[u8]] = &[
        b"do not edit",
        b"@generated",
        b"<auto-generated",
        b"automatically generated",
    ];
    // The bytes that can begin a marker, in either case.
    const FIRST: [bool; 256] = {
        let mut table = [false; 256];
        let mut index = 0;
        while index < MARKERS.len() {
            let byte = MARKERS[index][0];
            table[byte as usize] = true;
            table[byte.to_ascii_uppercase() as usize] = true;
            index += 1;
        }
        table
    };
    let head = &source[..source.len().min(4096)];
    // One pass: only a byte that can begin a marker leads to a comparison.
    head.iter()
        .enumerate()
        .filter(|&(_, &byte)| FIRST[byte as usize])
        .any(|(start, _)| {
            MARKERS.iter().any(|marker| {
                head.get(start..start + marker.len())
                    .is_some_and(|window| window.eq_ignore_ascii_case(marker))
            })
        })
}

/// Reads a selected file into `buffer`. We take the size and time from the open file
/// before and after reading, so a file that changes while we read it is an error, and
/// the per-file and snapshot byte budgets apply before we allocate.
fn read_selected(
    selected: &SelectedFile,
    buffer: &mut Vec<u8>,
    options: &Options,
    read_bytes: &AtomicU64,
) -> Result<Contents, Error> {
    let path = &selected.path;
    let mut file = std::fs::File::open(path).map_err(|e| io_error(path, e))?;
    let before = file.metadata().map_err(|e| io_error(path, e))?;
    let bytes = before.len();
    if before.is_file() && bytes > options.max_file_bytes {
        return Ok(Contents::Oversized(bytes));
    }
    let total = read_bytes
        .fetch_add(bytes, Ordering::Relaxed)
        .saturating_add(bytes);
    if total > options.max_total_bytes {
        return Err(limit("Snapshot byte budget exceeded"));
    }
    buffer.clear();
    let capacity = usize::try_from(bytes)
        .ok()
        .and_then(|n| n.checked_add(1))
        .ok_or_else(|| limit("File cannot fit in the address space"))?;
    buffer
        .try_reserve(capacity)
        .map_err(|_| limit("Cannot reserve scanner buffer"))?;
    file.by_ref()
        .take(bytes.saturating_add(1))
        .read_to_end(buffer)
        .map_err(|e| io_error(path, e))?;
    let after = file.metadata().map_err(|e| io_error(path, e))?;
    if !before.is_file()
        || buffer.len() as u64 != bytes
        || after.len() != bytes
        || after.modified().ok() != before.modified().ok()
    {
        return Err(Error::new(
            "source-changed",
            format!("{} changed during scanning", selected.name),
            "Scan an immutable checkout or pause writers and retry.",
        ));
    }
    Ok(Contents::Read((after.len(), after.modified().ok())))
}

fn worker_error() -> Error {
    Error::new(
        "worker-failed",
        "A scanner worker stopped unexpectedly",
        "Retry with --jobs 1 and report the failing input.",
    )
}
