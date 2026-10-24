use crate::ignore::{IgnoreBudget, Rules};
use crate::{Options, Report, io_error, limit};
use ccm_core::Error;
use ccm_scan::Language;
use std::collections::BTreeSet;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// A file chosen for counting. Workers read its size and time from the open file, so
/// selection needs no `stat` call per entry.
pub(super) struct SelectedFile {
    pub name: String,
    pub path: PathBuf,
    pub language: Language,
}

struct Entry {
    path: PathBuf,
    relative: PathBuf,
    /// The type the directory listing reports, without following symbolic links.
    kind: std::fs::FileType,
    depth: usize,
    rules: Arc<Rules>,
}

struct Walker<'a> {
    options: &'a Options,
    report: Report,
    selected: usize,
    deduplicate: bool,
    seen: BTreeSet<PathBuf>,
    visited: usize,
    budget: IgnoreBudget,
    excludes: Vec<PathBuf>,
}

pub(super) fn select(
    paths: &[PathBuf],
    options: &Options,
    mut consume: impl FnMut(SelectedFile) -> Result<(), Error>,
) -> Result<Report, Error> {
    let mut walker = Walker {
        options,
        report: Report {
            complete: true,
            ..Report::default()
        },
        selected: 0,
        deduplicate: paths.len() > 1,
        seen: BTreeSet::new(),
        visited: 0,
        budget: IgnoreBudget::for_files(options.max_files),
        excludes: normalize_excludes(&options.exclude),
    };
    for root in paths {
        let metadata = std::fs::symlink_metadata(root).map_err(|e| io_error(root, e))?;
        if metadata.file_type().is_symlink() {
            walker.report.skipped_symlinks += 1;
            continue;
        }
        let path = root.canonicalize().map_err(|e| io_error(root, e))?;
        let rules = Rules::ancestors(&path, options, &mut walker.budget)?;
        let mut stack = vec![Entry {
            path,
            relative: PathBuf::new(),
            kind: metadata.file_type(),
            depth: 0,
            rules,
        }];
        while let Some(entry) = stack.pop() {
            walker.visited += 1;
            if entry.depth > options.max_depth
                || walker.visited > options.max_files.saturating_mul(10)
            {
                return Err(limit("Traversal depth or entry budget exceeded"));
            }
            if entry.kind.is_symlink() {
                walker.report.skipped_symlinks += 1;
            } else if entry.kind.is_dir() {
                walker.directory(entry, &mut stack)?;
            } else if entry.kind.is_file() {
                if let Some(file) = walker.file(entry)? {
                    consume(file)?;
                }
            }
        }
    }
    Ok(walker.report)
}

impl Walker<'_> {
    fn directory(&mut self, entry: Entry, stack: &mut Vec<Entry>) -> Result<(), Error> {
        let mut pending = Vec::new();
        let mut present = 0;
        for child in std::fs::read_dir(&entry.path).map_err(|e| io_error(&entry.path, e))? {
            let child = child.map_err(|e| io_error(&entry.path, e))?;
            let name = child.file_name();
            present |= crate::ignore::presence(&name);
            let relative = entry.relative.join(&name);
            let kind = child.file_type().map_err(|e| io_error(&child.path(), e))?;
            if self.excluded(&name.to_string_lossy(), &relative, kind.is_dir()) {
                self.report.ignored_entries += 1;
                continue;
            }
            if pending.len() >= self.options.max_files.saturating_mul(10) {
                return Err(limit("Directory entry budget exceeded"));
            }
            pending
                .try_reserve(1)
                .map_err(|_| limit("Cannot reserve directory entries"))?;
            pending.push((child, relative, kind));
        }
        let rules = Rules::load(
            &entry.path,
            entry.rules,
            self.options,
            &mut self.budget,
            Some(present),
        )?;
        let mut children = Vec::new();
        for (child, relative, kind) in pending {
            let path = child.path();
            if rules.excluded(&path, kind.is_dir(), &mut self.budget)? {
                self.report.ignored_entries += 1;
                continue;
            }
            children.push(Entry {
                path,
                relative,
                kind,
                depth: entry.depth + 1,
                rules: rules.clone(),
            });
            if children.len() > self.options.max_files.saturating_mul(10) {
                return Err(limit("Directory entry budget exceeded"));
            }
        }
        children.sort_by(|a, b| a.path.cmp(&b.path));
        stack.extend(children.into_iter().rev());
        Ok(())
    }

    fn file(&mut self, entry: Entry) -> Result<Option<SelectedFile>, Error> {
        if self.deduplicate && !self.seen.insert(entry.path.clone()) {
            return Ok(None);
        }
        let extension = entry
            .path
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("");
        if self
            .options
            .exclude_extensions
            .iter()
            .any(|e| e.eq_ignore_ascii_case(extension))
            || (!self.options.include_extensions.is_empty()
                && !self
                    .options
                    .include_extensions
                    .iter()
                    .any(|e| e.eq_ignore_ascii_case(extension)))
        {
            self.report.ignored_entries += 1;
            return Ok(None);
        }
        let language = self.language(&entry.path, extension)?;
        if language.is_none() && binary_extension(extension) {
            self.report.skipped_binary += 1;
            return Ok(None);
        }
        let Some(language) = language else {
            self.report.unsupported_files += 1;
            *self
                .report
                .unknown_extensions
                .entry(extension.to_owned())
                .or_default() += 1;
            return Ok(None);
        };
        if self.selected >= self.options.max_files {
            return Err(limit("Selected file budget exceeded"));
        }
        let name = entry_name(&entry)?;
        self.selected += 1;
        Ok(Some(SelectedFile {
            name,
            path: entry.path,
            language,
        }))
    }

    /// Chooses a lexical policy: an explicit `--count-as` mapping first, then the registry.
    /// Ambiguous extensions (such as `.ccm`) and extensionless files are decided by their
    /// first 512 bytes; any other file is not opened here.
    fn language(&self, path: &Path, extension: &str) -> Result<Option<Language>, Error> {
        if let Some((_, language)) = self
            .options
            .count_as
            .iter()
            .find(|(ext, _)| ext.eq_ignore_ascii_case(extension))
        {
            return Ok(Some(*language));
        }
        if Language::needs_header(path) {
            return Ok(Language::from_path_and_header(path, &read_header(path)?));
        }
        let language = Language::from_path(path);
        if language.is_none() && path.extension().is_none() {
            return Ok(Language::from_shebang(&read_header(path)?));
        }
        Ok(language)
    }

    /// Applies default and explicit `--exclude` selection to one directory entry.
    fn excluded(&self, name: &str, relative: &Path, directory: bool) -> bool {
        if self.options.default_exclusions && default_excluded(name, directory) {
            return true;
        }
        self.excludes
            .iter()
            .any(|exclude| name == exclude.as_os_str() || relative.starts_with(exclude))
    }
}

/// Reads at most the first 512 bytes of a file for header-based language detection.
fn read_header(path: &Path) -> Result<Vec<u8>, Error> {
    let mut header = Vec::with_capacity(512);
    std::fs::File::open(path)
        .and_then(|file| file.take(512).read_to_end(&mut header))
        .map_err(|e| io_error(path, e))?;
    Ok(header)
}

fn entry_name(entry: &Entry) -> Result<String, Error> {
    let label = if entry.relative.as_os_str().is_empty() {
        Path::new(entry.path.file_name().unwrap_or(entry.path.as_os_str()))
    } else {
        &entry.relative
    };
    let name = label.to_str().ok_or_else(|| {
        Error::new(
            "path-encoding",
            "A source path is not UTF-8",
            "Rename the source path to UTF-8 before measuring it.",
        )
    })?;
    let name = if cfg!(windows) {
        name.replace('\\', "/")
    } else {
        name.into()
    };
    Ok(name)
}

/// Normalizes `--exclude` values once: `./src/` and `src` denote the same prefix.
/// Values that normalize to nothing, such as `.`, would exclude everything and are dropped.
fn normalize_excludes(excludes: &[String]) -> Vec<PathBuf> {
    excludes
        .iter()
        .map(|exclude| {
            Path::new(exclude)
                .components()
                .filter(|part| !matches!(part, std::path::Component::CurDir))
                .collect::<PathBuf>()
        })
        .filter(|exclude| !exclude.as_os_str().is_empty())
        .collect()
}

// Version-control metadata is excluded whatever its type; a worktree's `.git` is a file.
// Build and vendor names are excluded only as directories, so a script named `build` counts.
fn default_excluded(name: &str, directory: bool) -> bool {
    const METADATA: &[&str] = &[".git", ".jj"];
    const DIRECTORIES: &[&str] = &[
        "target",
        "node_modules",
        ".venv",
        "vendor",
        "dist",
        "build",
        "_build",
        "__pycache__",
    ];
    METADATA.contains(&name) || (directory && DIRECTORIES.contains(&name))
}

fn binary_extension(extension: &str) -> bool {
    [
        "png", "jpg", "jpeg", "gif", "webp", "ico", "pdf", "woff", "woff2", "ttf", "otf", "eot",
        "zip", "gz", "xz", "bz2", "7z", "exe", "dll", "so", "a", "rlib", "rmeta", "o", "obj",
        "class", "wasm", "pyc",
    ]
    .iter()
    .any(|ext| ext.eq_ignore_ascii_case(extension))
}
