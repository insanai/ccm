//! Filesystem selection and bounded snapshots. Never executes project code or external tools.
mod glob;
mod ignore;
mod scan;
mod walk;

use ccm_core::{Counts, Error};
use ccm_scan::Language;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

/// Selection and resource policy shared by current and comparison snapshots.
#[derive(Clone, Debug)]
pub struct Options {
    /// Excluded entry names or root-relative path prefixes; a leading `./` is ignored.
    /// No glob interpretation.
    pub exclude: Vec<String>,
    /// Whether nested .gitignore, .ignore, and .ccmignore rules apply.
    pub respect_ignore: bool,
    /// Whether build and vendor directory defaults apply.
    pub default_exclusions: bool,
    /// Number of scanner workers, from 1 to 64.
    pub jobs: usize,
    /// Limit to these extensions, without dots; empty selects all registered policies.
    pub include_extensions: Vec<String>,
    /// Explicitly excluded extensions.
    pub exclude_extensions: Vec<String>,
    /// Extension-to-policy overrides.
    pub count_as: Vec<(String, Language)>,
    /// Exclude files marked generated in their first 4096 bytes.
    pub skip_generated: bool,
    /// Collect optional source-work-v1 tokens for technical inspection.
    pub collect_work: bool,
    /// Maximum selected files per snapshot.
    pub max_files: usize,
    /// Maximum bytes in a single file; larger supported files are skipped with a notice.
    pub max_file_bytes: u64,
    /// Maximum bytes read per snapshot.
    pub max_total_bytes: u64,
    /// Maximum traversal depth, without following symlinks.
    pub max_depth: usize,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            exclude: Vec::new(),
            respect_ignore: true,
            default_exclusions: true,
            jobs: std::thread::available_parallelism().map_or(1, |n| n.get().min(16)),
            include_extensions: Vec::new(),
            exclude_extensions: Vec::new(),
            count_as: Vec::new(),
            skip_generated: false,
            collect_work: false,
            // Sized for a large monorepo: a Linux kernel checkout has about 90,000
            // files and 1.5 GB. Each file record takes about 330 bytes, so a million
            // files need about 330 MB. Files are read one at a time per worker, so the
            // byte total bounds work, not memory.
            max_files: 1_000_000,
            max_file_bytes: 64 * 1024 * 1024,
            max_total_bytes: 64 * 1024 * 1024 * 1024,
            max_depth: 128,
        }
    }
}

/// Counts for one selected file.
#[derive(Clone, Debug)]
pub struct FileReport {
    /// Root-relative name, or filename when the root is a file.
    pub name: String,
    /// Canonical filesystem path used for later comparison.
    pub path: PathBuf,
    /// Applied lexical policy.
    pub language: Language,
    /// Physical measurements.
    pub counts: Counts,
    /// Whether the file declares itself generated in its header.
    pub generated: bool,
    /// Lexical source-work-v1 tokens, when work collection is enabled for program source.
    pub source_tokens: Option<u64>,
    stamp: (u64, Option<std::time::SystemTime>),
}

/// A skipped supported file or a symlink, with an explicit explanation.
#[derive(Clone, Debug)]
pub struct Notice {
    /// Display path; renderers must escape untrusted text.
    pub path: String,
    /// Why no counts are available.
    pub message: String,
}

/// An owned, deterministically ordered snapshot report.
#[derive(Clone, Debug, Default)]
pub struct Report {
    /// Successfully counted files, sorted by name and path.
    pub files: Vec<FileReport>,
    /// Counts over successfully measured files only.
    pub totals: Counts,
    /// Supported files that could not be counted and skipped symlinks.
    pub notices: Vec<Notice>,
    /// Files with no registered lexical policy.
    pub unsupported_files: usize,
    /// Symlinks intentionally not followed; this is not a scan failure.
    pub skipped_symlinks: usize,
    /// Selected files containing binary NUL bytes.
    pub skipped_binary: usize,
    /// Generated files excluded by explicit policy.
    pub skipped_generated: usize,
    /// Supported files larger than `Options::max_file_bytes`; they make the report incomplete.
    pub skipped_oversized: usize,
    /// Entries pruned by ignore rules or selection exclusions, including whole directories.
    pub ignored_entries: usize,
    /// Unrecognized extension counts; an empty key denotes an extensionless filename.
    pub unknown_extensions: BTreeMap<String, usize>,
    /// Whether all selected supported regular text files were counted.
    pub complete: bool,
}

impl FileReport {
    /// Returns whether this file is hand-written program source: a source-kind language
    /// policy and no generated-content declaration. Only such files count as program lines.
    pub fn is_program(&self) -> bool {
        self.language.kind() == ccm_scan::FileKind::Source && !self.generated
    }
}

impl Report {
    /// Returns nonblank content lines for the requested file purpose, generated files included.
    pub fn content_lines(&self, kind: ccm_scan::FileKind) -> u64 {
        self.files
            .iter()
            .filter(|f| f.language.kind() == kind)
            .map(|f| f.counts.code)
            .sum()
    }

    /// Returns program lines: code lines of files for which `FileReport::is_program` holds.
    /// Every report uses this one definition, and it is the default AI rebuild scope.
    pub fn program_lines(&self) -> u64 {
        self.files
            .iter()
            .filter(|f| f.is_program())
            .map(|f| f.counts.code)
            .sum()
    }

    /// Returns code lines of source-kind files that declare themselves generated.
    /// With `program_lines`, this partitions `content_lines(FileKind::Source)`.
    pub fn generated_source_lines(&self) -> u64 {
        self.files
            .iter()
            .filter(|f| f.language.kind() == ccm_scan::FileKind::Source && f.generated)
            .map(|f| f.counts.code)
            .sum()
    }
}

/// A file-level byte comparison; no semantic equivalence is inferred.
#[derive(Clone, Debug)]
pub struct FileChange {
    /// Root-relative artifact name, also usable as an explicit model seed.
    pub name: String,
    /// `added`, `modified`, or `removed`.
    pub kind: &'static str,
    /// Counts before the change, or none for an added file.
    pub before: Option<Counts>,
    /// Counts after the change, or none for a removed file.
    pub after: Option<Counts>,
}

/// Counts selected paths without following symlinks. Snapshot limits and traversal failures
/// are errors. Lexical failures and oversized files appear as notices and make the returned
/// report incomplete.
pub fn measure_paths(paths: &[PathBuf], options: &Options) -> Result<Report, Error> {
    scan::scan(paths, options)
}

/// Compares two complete, single-root reports by exact bytes. Returns no misleading partial diff.
/// Reads each matched file again within the same per-file byte budget.
pub fn compare_reports(
    before: &Report,
    after: &Report,
    options: &Options,
) -> Result<Vec<FileChange>, Error> {
    if !before.complete || !after.complete {
        return Err(Error::new(
            "incomplete-comparison",
            "A snapshot has uncounted source files",
            "Resolve coverage notices or explicitly exclude those files before comparing.",
        ));
    }
    let old: BTreeMap<_, _> = before.files.iter().map(|f| (&f.name, f)).collect();
    let new: BTreeMap<_, _> = after.files.iter().map(|f| (&f.name, f)).collect();
    let mut changes = Vec::new();
    let mut left = Vec::new();
    let mut right = Vec::new();
    for (name, file) in &old {
        if let Some(current) = new.get(name) {
            let old_stamp = read_file(&file.path, options.max_file_bytes, &mut left)?;
            let new_stamp = read_file(&current.path, options.max_file_bytes, &mut right)?;
            if old_stamp != file.stamp || new_stamp != current.stamp {
                return Err(Error::new(
                    "source-changed",
                    "A snapshot changed after counting",
                    "Compare immutable checkouts or pause writers and retry.",
                ));
            }
            if left != right {
                changes.push(change(name, "modified", Some(file), Some(current)));
            }
        } else {
            changes.push(change(name, "removed", Some(file), None));
        }
    }
    for (name, file) in &new {
        if !old.contains_key(name) {
            changes.push(change(name, "added", None, Some(file)));
        }
    }
    changes.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(changes)
}

fn change(
    name: &str,
    kind: &'static str,
    before: Option<&FileReport>,
    after: Option<&FileReport>,
) -> FileChange {
    FileChange {
        name: name.into(),
        kind,
        before: before.map(|f| f.counts),
        after: after.map(|f| f.counts),
    }
}

/// Reads a bounded regular file into reusable storage and checks length and modification time.
/// Symlinks are refused; this is best-effort mutation detection, not an atomic snapshot.
pub fn read_source(path: &Path, limit_bytes: u64, buffer: &mut Vec<u8>) -> Result<(), Error> {
    read_file(path, limit_bytes, buffer).map(|_| ())
}

fn read_file(
    path: &Path,
    limit_bytes: u64,
    buffer: &mut Vec<u8>,
) -> Result<(u64, Option<std::time::SystemTime>), Error> {
    let metadata = std::fs::symlink_metadata(path).map_err(|e| io_error(path, e))?;
    if !metadata.is_file() || metadata.len() > limit_bytes {
        return Err(limit(format!(
            "{} is not a regular file within the byte limit",
            path.display()
        )));
    }
    buffer.clear();
    let capacity = usize::try_from(metadata.len())
        .ok()
        .and_then(|n| n.checked_add(1))
        .ok_or_else(|| limit("File cannot fit in this platform's address space"))?;
    buffer
        .try_reserve(capacity)
        .map_err(|_| limit("Cannot reserve input buffer"))?;
    let file = File::open(path).map_err(|e| io_error(path, e))?;
    let opened = file.metadata().map_err(|e| io_error(path, e))?;
    file.take(limit_bytes.saturating_add(1))
        .read_to_end(buffer)
        .map_err(|e| io_error(path, e))?;
    let after = std::fs::metadata(path).map_err(|e| io_error(path, e))?;
    if buffer.len() as u64 > limit_bytes {
        return Err(limit("File grew beyond the byte budget"));
    }
    if metadata.len() != after.len()
        || opened.len() != after.len()
        || metadata.modified().ok() != after.modified().ok()
    {
        return Err(Error::new(
            "source-changed",
            format!("{} changed during reading", path.display()),
            "Measure an immutable checkout or pause writers and retry.",
        ));
    }
    Ok((after.len(), after.modified().ok()))
}

fn limit(message: impl Into<String>) -> Error {
    Error::new(
        "resource-limit",
        message,
        "Narrow the selected paths or adjust the explicit limit.",
    )
}

fn io_error(path: &Path, error: std::io::Error) -> Error {
    Error::new(
        "source-io",
        format!("{}: {error}", path.display()),
        "Check the path and read permissions.",
    )
}
