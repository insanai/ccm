use crate::glob::Glob;
use crate::{Options, io_error, limit, read_source};
use ccm_core::Error;
use std::path::{Path, PathBuf};
use std::sync::Arc;

#[derive(Debug)]
struct Rule {
    glob: Glob,
    basename: bool,
    directory: bool,
    include: bool,
}

#[derive(Debug, Default)]
pub(super) struct Rules {
    base: PathBuf,
    parent: Option<Arc<Rules>>,
    local: Vec<Rule>,
}

pub(super) struct IgnoreBudget {
    pub bytes: usize,
    pub records: usize,
    pub work: u64,
}

impl Default for IgnoreBudget {
    fn default() -> Self {
        Self {
            bytes: 0,
            records: 0,
            work: 200_000_000,
        }
    }
}

impl Rules {
    pub fn ancestors(
        root: &Path,
        options: &Options,
        budget: &mut IgnoreBudget,
    ) -> Result<Arc<Self>, Error> {
        if root.join(".git").exists() {
            return Ok(Arc::new(Self::default()));
        }
        let mut chain = Vec::new();
        let mut directory = root.parent();
        while let Some(path) = directory {
            chain.push(path.to_path_buf());
            if path.join(".git").exists() || chain.len() >= 128 {
                break;
            }
            directory = path.parent();
        }
        let mut rules = Arc::new(Self::default());
        for path in chain.into_iter().rev() {
            rules = Self::load(&path, rules, options, budget, None)?;
        }
        Ok(rules)
    }

    pub fn load(
        directory: &Path,
        parent: Arc<Self>,
        options: &Options,
        budget: &mut IgnoreBudget,
        present: Option<u8>,
    ) -> Result<Arc<Self>, Error> {
        if !options.respect_ignore {
            return Ok(parent);
        }
        let mut local = Vec::new();
        for (index, filename) in [".git/info/exclude", ".gitignore", ".ignore", ".ccmignore"]
            .into_iter()
            .enumerate()
        {
            if present.is_some_and(|mask| mask & (1 << index) == 0) {
                continue;
            }
            if filename.starts_with(".git/") && !directory.join(".git").is_dir() {
                continue;
            }
            let path = directory.join(filename);
            let metadata = match std::fs::symlink_metadata(&path) {
                Ok(metadata) => metadata,
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => continue,
                Err(e) => return Err(io_error(&path, e)),
            };
            if !metadata.is_file() {
                continue;
            }
            let mut bytes = Vec::new();
            read_source(&path, 256 * 1024, &mut bytes)?;
            budget.bytes += bytes.len();
            if budget.bytes > 4 * 1024 * 1024 {
                return Err(limit("Ignore files exceed 4 MiB"));
            }
            let text = std::str::from_utf8(&bytes).map_err(|_| {
                Error::new(
                    "ignore-encoding",
                    format!("{} is not UTF-8", path.display()),
                    "Save ignore files as UTF-8.",
                )
            })?;
            for line in text.lines() {
                if let Some(rule) = parse_rule(line)? {
                    local.push(rule);
                    budget.records += 1;
                    if budget.records > 10_000 {
                        return Err(limit("Ignore rules exceed 10,000"));
                    }
                }
            }
        }
        if local.is_empty() {
            Ok(parent)
        } else {
            Ok(Arc::new(Self {
                base: directory.to_path_buf(),
                parent: Some(parent),
                local,
            }))
        }
    }

    pub fn excluded(
        &self,
        path: &Path,
        directory: bool,
        budget: &mut IgnoreBudget,
    ) -> Result<bool, Error> {
        let mut current = Some(self);
        while let Some(rules) = current {
            let relative = path.strip_prefix(&rules.base).ok();
            let basename = relative
                .and_then(|p| p.file_name())
                .and_then(|s| s.to_str());
            let full = relative.and_then(|p| p.to_str());
            for rule in rules.local.iter().rev() {
                if rule.directory && !directory {
                    continue;
                }
                let text = if rule.basename { basename } else { full };
                if let Some(text) = text {
                    let normalized = if cfg!(windows) {
                        std::borrow::Cow::Owned(text.replace('\\', "/"))
                    } else {
                        std::borrow::Cow::Borrowed(text)
                    };
                    if rule.glob.matches(&normalized, &mut budget.work)? {
                        return Ok(!rule.include);
                    }
                }
            }
            current = rules.parent.as_deref();
        }
        Ok(false)
    }
}

fn parse_rule(line: &str) -> Result<Option<Rule>, Error> {
    let mut end = line.len();
    while end > 0 && line.as_bytes()[end - 1] == b' ' {
        let preceding = line[..end - 1]
            .bytes()
            .rev()
            .take_while(|&b| b == b'\\')
            .count();
        if preceding % 2 == 1 {
            break;
        }
        end -= 1;
    }
    let mut pattern = &line[..end];
    if pattern.is_empty() || pattern.starts_with('#') {
        return Ok(None);
    }
    let include = pattern.starts_with('!');
    if include {
        pattern = &pattern[1..];
    }
    let directory = pattern.ends_with('/');
    if directory {
        pattern = &pattern[..pattern.len() - 1];
    }
    let basename = !pattern.contains('/');
    pattern = pattern.strip_prefix('/').unwrap_or(pattern);
    if pattern.is_empty() {
        return Ok(None);
    }
    Ok(Some(Rule {
        glob: Glob::compile(pattern)?,
        basename,
        directory,
        include,
    }))
}

pub(super) fn presence(name: &std::ffi::OsStr) -> u8 {
    match name.to_str() {
        Some(".git") => 1,
        Some(".gitignore") => 2,
        Some(".ignore") => 4,
        Some(".ccmignore") => 8,
        _ => 0,
    }
}
