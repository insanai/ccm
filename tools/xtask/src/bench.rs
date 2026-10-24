//! Times CCM against scc and cloc on one input.
//!
//! Without a path the input is a deterministic Rust fixture whose counts we declare by
//! hand, and only CCM and scc run. With a path, made by `cargo xtask corpus`, the driver
//! first checks that all three tools see the same files and physical lines, then times
//! alternating CCM/scc pairs and a few cloc runs. Every sample runs under GNU time, so one
//! run gives wall time, CPU time, and peak memory. Samples go to `target/bench-samples.csv`.
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Instant;

const USAGE: &str = concat!(
    "Usage: cargo xtask bench [--ccm PATH] [--no-cloc] [--cloc-processes N] ",
    "[--cloc-runs N] [--one-core] [PAIRS] [PATH]"
);

/// The counters we time. Each runs with its own defaults and its cost or report extras
/// off, so none is charged for work that the others do not do.
#[derive(Clone, Copy, PartialEq)]
enum Tool {
    Ccm,
    Scc,
    Cloc,
}

impl Tool {
    fn name(self) -> &'static str {
        match self {
            Tool::Ccm => "ccm",
            Tool::Scc => "scc",
            Tool::Cloc => "cloc",
        }
    }

    /// Returns the command that counts `path`. With `one_core` each tool also uses a
    /// single worker.
    fn command(self, programs: &Programs, path: &Path, one_core: bool) -> Command {
        let mut command = match self {
            Tool::Ccm => Command::new(&programs.ccm),
            Tool::Scc => Command::new("scc"),
            Tool::Cloc => Command::new("cloc"),
        };
        match self {
            Tool::Ccm => {
                command.args(["--no-cost", "--sort", "name"]);
                if one_core {
                    command.args(["--jobs", "1"]);
                }
            }
            Tool::Scc => {
                command.args(["--count-ignore", "--no-config", "--no-cocomo"]);
                command.args(["--no-complexity", "--no-size", "--sort", "name"]);
            }
            Tool::Cloc => {
                command.args(["--quiet", "--skip-uniqueness"]);
                let processes = if one_core { 0 } else { programs.cloc_processes };
                command.arg(format!("--processes={processes}"));
            }
        }
        command.arg(path).env("NO_COLOR", "1");
        command
    }
}

/// One timed run: wall seconds from our clock, CPU seconds and peak RSS from GNU time.
#[derive(Clone, Copy)]
struct Sample {
    wall: f64,
    user: f64,
    system: f64,
    rss_kib: u64,
}

struct Options {
    programs: Programs,
    pairs: usize,
    corpus: Option<PathBuf>,
    cloc_runs: usize,
    one_core: bool,
}

pub(super) fn run(arguments: &[String]) -> Result<(), String> {
    let options = parse(arguments)?;
    let root = repository_root()?;
    let corpus = match &options.corpus {
        Some(path) => path.canonicalize().map_err(error)?,
        None => root.join("target/bench-fixture-v1"),
    };
    if options.corpus.is_none() {
        create_corpus(&corpus)?;
        verify(&options.programs, &corpus)?;
    } else {
        verify_project(&options.programs, &corpus)?;
    }
    let path = root.join("target/bench-samples.csv");
    let mut csv = std::fs::File::create(&path).map_err(error)?;
    writeln!(
        csv,
        "tool,setting,run,wall_seconds,user_seconds,system_seconds,peak_rss_kib"
    )
    .map_err(error)?;
    println!("Corpus: {}", corpus.display());
    pairs(&options, &corpus, &mut csv)?;
    if options.corpus.is_some() && options.programs.cloc && options.cloc_runs > 0 {
        let samples = repeat(Tool::Cloc, &options, &corpus, options.cloc_runs, false)?;
        record(&mut csv, Tool::Cloc, "all-cores", &samples)?;
        summarize("cloc", &samples);
    }
    if options.one_core {
        one_core(&options, &corpus, &mut csv)?;
    }
    println!("Raw samples: {}", path.display());
    println!("This workload does not establish an all-project speed or memory claim.");
    Ok(())
}

fn parse(arguments: &[String]) -> Result<Options, String> {
    let (programs, arguments) = programs(arguments)?;
    let mut options = Options {
        programs,
        pairs: 30,
        corpus: None,
        cloc_runs: 3,
        one_core: false,
    };
    let mut positional = Vec::new();
    let mut arguments = arguments.into_iter();
    while let Some(argument) = arguments.next() {
        match argument.as_str() {
            "--cloc-runs" => {
                let value = arguments.next().ok_or(USAGE)?;
                options.cloc_runs = value.parse().map_err(error)?;
            }
            "--one-core" => options.one_core = true,
            _ => positional.push(argument),
        }
    }
    match positional.as_slice() {
        [] => {}
        [pairs] => options.pairs = pairs.parse().map_err(error)?,
        [pairs, path] => {
            options.pairs = pairs.parse().map_err(error)?;
            options.corpus = Some(PathBuf::from(path));
        }
        _ => return Err(USAGE.into()),
    }
    if !(10..=500).contains(&options.pairs) {
        return Err("Use 10 to 500 benchmark pairs".into());
    }
    Ok(options)
}

/// The programs we run and how: the CCM executable, whether cloc runs at all, and the
/// number of processes cloc may start, where 0 is cloc's own default of one process.
pub(super) struct Programs {
    pub ccm: PathBuf,
    pub cloc: bool,
    pub cloc_processes: usize,
}

impl Programs {
    /// Returns the tools that take part: CCM and scc always, and cloc unless left out.
    fn tools(&self) -> Vec<Tool> {
        let mut tools = vec![Tool::Ccm, Tool::Scc];
        if self.cloc {
            tools.push(Tool::Cloc);
        }
        tools
    }
}

/// Removes `--ccm PATH`, `--no-cloc`, and `--cloc-processes N` from the arguments.
/// `--no-cloc` leaves cloc out, so CCM and scc can be measured on a tree where cloc
/// would take too long or too much memory, and cloc can follow later. CCM is by default
/// the release build of this repository, and cloc by default starts one process per
/// hardware thread, as CCM and scc use one worker per thread. Each cloc process holds
/// its share of the input, so a large tree on a small machine needs fewer.
pub(super) fn programs(arguments: &[String]) -> Result<(Programs, Vec<String>), String> {
    let mut programs = Programs {
        ccm: repository_root()?.join("target/release/ccm"),
        cloc: true,
        cloc_processes: hardware_threads(),
    };
    let mut rest = Vec::new();
    let mut arguments = arguments.iter();
    while let Some(argument) = arguments.next() {
        match argument.as_str() {
            "--ccm" => {
                let path = arguments.next().ok_or("--ccm needs a path")?;
                programs.ccm = Path::new(path).canonicalize().map_err(error)?;
            }
            "--no-cloc" => programs.cloc = false,
            "--cloc-processes" => {
                let count = arguments.next().ok_or("--cloc-processes needs a number")?;
                programs.cloc_processes = count.parse().map_err(error)?;
            }
            _ => rest.push(argument.clone()),
        }
    }
    Ok((programs, rest))
}

/// Returns the number of hardware threads, the worker count CCM and scc use by default.
fn hardware_threads() -> usize {
    std::thread::available_parallelism().map_or(1, usize::from)
}

fn repository_root() -> Result<PathBuf, String> {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .map_err(error)
}

/// Times alternating CCM/scc pairs after two warm-up runs of each, and reports the median
/// paired speedup with its bootstrap interval.
fn pairs(options: &Options, corpus: &Path, csv: &mut std::fs::File) -> Result<(), String> {
    for _ in 0..2 {
        measure(Tool::Ccm, options, corpus, false)?;
        measure(Tool::Scc, options, corpus, false)?;
    }
    let mut ours = Vec::new();
    let mut theirs = Vec::new();
    for pair in 0..options.pairs {
        if pair % 2 == 0 {
            ours.push(measure(Tool::Ccm, options, corpus, false)?);
            theirs.push(measure(Tool::Scc, options, corpus, false)?);
        } else {
            theirs.push(measure(Tool::Scc, options, corpus, false)?);
            ours.push(measure(Tool::Ccm, options, corpus, false)?);
        }
    }
    record(csv, Tool::Ccm, "all-cores", &ours)?;
    record(csv, Tool::Scc, "all-cores", &theirs)?;
    let mut ratios: Vec<f64> = ours
        .iter()
        .zip(&theirs)
        .map(|(ours, theirs)| theirs.wall / ours.wall)
        .collect();
    println!(
        "{} alternating warm pairs; terminal reports to a null sink.",
        options.pairs
    );
    summarize("ccm", &ours);
    summarize("scc", &theirs);
    let (low, high) = bootstrap(&ratios);
    println!(
        "Paired speedup of CCM over scc: {:.3}x, bootstrap 95% interval [{low:.3}, {high:.3}].",
        median(&mut ratios)
    );
    Ok(())
}

/// Runs every tool pinned to CPU 0 with one worker, so the comparison shows the work each
/// tool does rather than how well it spreads that work over cores.
fn one_core(options: &Options, corpus: &Path, csv: &mut std::fs::File) -> Result<(), String> {
    println!("One core (taskset -c 0, one worker):");
    for tool in options.programs.tools() {
        let runs = if tool == Tool::Cloc {
            options.cloc_runs.max(1)
        } else {
            5
        };
        let samples = repeat(tool, options, corpus, runs, true)?;
        record(csv, tool, "one-core", &samples)?;
        summarize(tool.name(), &samples);
    }
    Ok(())
}

fn repeat(
    tool: Tool,
    options: &Options,
    corpus: &Path,
    runs: usize,
    one_core: bool,
) -> Result<Vec<Sample>, String> {
    (0..runs)
        .map(|_| measure(tool, options, corpus, one_core))
        .collect()
}

/// Runs one tool once under GNU time, with its report going to a null sink.
fn measure(
    tool: Tool,
    options: &Options,
    corpus: &Path,
    one_core: bool,
) -> Result<Sample, String> {
    let counted = tool.command(&options.programs, corpus, one_core);
    let mut command = Command::new("/usr/bin/time");
    command.args(["-f", "%U %S %M", "--"]);
    if one_core {
        command.args(["taskset", "-c", "0"]);
    }
    command
        .arg(counted.get_program())
        .args(counted.get_args())
        .env("NO_COLOR", "1")
        .stdout(Stdio::null());
    let start = Instant::now();
    let output = command
        .output()
        .map_err(|e| format!("Cannot run GNU time: {e}"))?;
    let wall = start.elapsed().as_secs_f64();
    let stderr = String::from_utf8_lossy(&output.stderr);
    if !output.status.success() {
        return Err(format!("{} failed:\n{stderr}", tool.name()));
    }
    let last = stderr.lines().last().unwrap_or("");
    let fields: Vec<_> = last.split_whitespace().collect();
    let [user, system, rss] = fields.as_slice() else {
        return Err(format!("Unexpected GNU time output: {last}"));
    };
    Ok(Sample {
        wall,
        user: user.parse().map_err(error)?,
        system: system.parse().map_err(error)?,
        rss_kib: rss.parse().map_err(error)?,
    })
}

fn record(
    csv: &mut std::fs::File,
    tool: Tool,
    setting: &str,
    samples: &[Sample],
) -> Result<(), String> {
    for (run, sample) in samples.iter().enumerate() {
        writeln!(
            csv,
            "{},{setting},{run},{:.6},{:.2},{:.2},{}",
            tool.name(),
            sample.wall,
            sample.user,
            sample.system,
            sample.rss_kib
        )
        .map_err(error)?;
    }
    Ok(())
}

/// Prints the medians of a tool's samples.
fn summarize(name: &str, samples: &[Sample]) {
    let pick =
        |value: fn(&Sample) -> f64| median(&mut samples.iter().map(value).collect::<Vec<_>>());
    println!(
        "  {name:<4} median of {}: wall {:.3} s, user {:.2} s, system {:.2} s, peak RSS {:.0} KiB",
        samples.len(),
        pick(|s| s.wall),
        pick(|s| s.user),
        pick(|s| s.system),
        pick(|s| s.rss_kib as f64),
    );
}

fn create_corpus(path: &Path) -> Result<(), String> {
    std::fs::create_dir_all(path).map_err(error)?;
    for index in 0..2000 {
        write_fixture(
            &path.join(format!("tiny-{index:04}.rs")),
            format!("// module {index}\npub fn value_{index}() -> usize {{\n    {index}\n}}\n\n")
                .as_bytes(),
        )?;
    }
    let mut large = String::from("// large module\nfn large() {\n");
    for _ in 0..50_000 {
        large.push_str("    let value = 1; // explanation\n");
    }
    large.push_str("}\n\n");
    for index in 0..4 {
        write_fixture(&path.join(format!("large-{index}.rs")), large.as_bytes())?;
    }
    Ok(())
}

fn write_fixture(path: &Path, source: &[u8]) -> Result<(), String> {
    match std::fs::read(path) {
        Ok(existing) if existing == source => Ok(()),
        Ok(_) => Err(format!(
            "Refusing to change an unexpected benchmark fixture: {}",
            path.display()
        )),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            std::fs::write(path, source).map_err(error)
        }
        Err(e) => Err(error(e)),
    }
}

/// Checks the fixture's hand-declared totals in both tools' CSV reports.
fn verify(programs: &Programs, corpus: &Path) -> Result<(), String> {
    for tool in [Tool::Ccm, Tool::Scc] {
        let totals = totals(tool, programs, corpus)?;
        if (totals.code, totals.comments, totals.blank, totals.files)
            != (206_008, 2004, 2004, 2004)
        {
            return Err(format!(
                "Counting conformance failed for {}: {totals:?}",
                tool.name()
            ));
        }
    }
    Ok(())
}

/// A tool's totals over a corpus. cloc reports no bytes, so its `bytes` is `None`.
#[derive(Debug, PartialEq)]
struct Totals {
    files: u64,
    code: u64,
    comments: u64,
    blank: u64,
    bytes: Option<u64>,
}

impl Totals {
    fn lines(&self) -> u64 {
        self.code + self.comments + self.blank
    }
}

/// Requires that the tools count the same files and physical lines, and CCM and scc
/// the same bytes. Their classes may differ; we print them beside the number of
/// whitespace-only lines, which is what a blank count should equal.
fn verify_project(programs: &Programs, corpus: &Path) -> Result<(), String> {
    let mut rows = Vec::new();
    for tool in programs.tools() {
        rows.push((tool, totals(tool, programs, corpus)?));
    }
    let reference = &rows[0].1;
    for (tool, totals) in &rows {
        let same_bytes = totals
            .bytes
            .is_none_or(|bytes| Some(bytes) == reference.bytes);
        if totals.files != reference.files || totals.lines() != reference.lines() || !same_bytes {
            return Err(format!(
                "Input coverage differs for {}: {totals:?}",
                tool.name()
            ));
        }
    }
    println!(
        "Equivalent input: {} files, {} physical lines, {} bytes.",
        reference.files,
        reference.lines(),
        reference.bytes.unwrap_or(0)
    );
    println!("Whitespace-only lines: {}", whitespace_lines(corpus)?);
    for (tool, totals) in &rows {
        println!(
            "  {:<4} code {}, comments {}, blank {}",
            tool.name(),
            totals.code,
            totals.comments,
            totals.blank
        );
    }
    Ok(())
}

/// Reads a tool's CSV report of `corpus` and adds up its rows.
fn totals(tool: Tool, programs: &Programs, corpus: &Path) -> Result<Totals, String> {
    let mut command = tool.command(programs, corpus, false);
    match tool {
        Tool::Ccm | Tool::Scc => command.args(["--format", "csv"]),
        Tool::Cloc => command.args(["--csv", "--csv-delimiter=\t"]),
    };
    let output = command.output().map_err(error)?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).into_owned());
    }
    let text = String::from_utf8(output.stdout).map_err(error)?;
    let mut totals = Totals {
        files: 0,
        code: 0,
        comments: 0,
        blank: 0,
        bytes: (tool != Tool::Cloc).then_some(0),
    };
    for row in text.lines().filter(|row| !row.trim().is_empty()).skip(1) {
        let fields = match tool {
            Tool::Cloc => row.split('\t').map(str::to_owned).collect(),
            _ => csv_fields(row)?,
        };
        let number = |index: usize| -> Result<u64, String> {
            fields
                .get(index)
                .ok_or_else(|| format!("Missing CSV column {index} in {row}"))?
                .parse()
                .map_err(error)
        };
        // CCM writes one row per file; scc and cloc write one per language.
        let (files, code, comments, blank, bytes) = match tool {
            Tool::Ccm => (1, number(5)?, number(6)?, number(7)?, number(3)?),
            Tool::Scc => (number(7)?, number(2)?, number(3)?, number(4)?, number(6)?),
            Tool::Cloc if fields.get(1).is_some_and(|name| name == "SUM") => continue,
            Tool::Cloc => (number(0)?, number(4)?, number(3)?, number(2)?, 0),
        };
        totals.files += files;
        totals.code += code;
        totals.comments += comments;
        totals.blank += blank;
        totals.bytes = totals.bytes.map(|total| total + bytes);
    }
    Ok(totals)
}

/// Counts the lines of every file under `corpus` that hold only whitespace, without any
/// counter's rules: a line ends at `\n`, a nonempty last line without one still counts, and
/// a byte-order mark at the start of a file is not content.
fn whitespace_lines(corpus: &Path) -> Result<u64, String> {
    let mut count = 0;
    let mut stack = vec![corpus.to_path_buf()];
    while let Some(path) = stack.pop() {
        let kind = std::fs::symlink_metadata(&path).map_err(error)?.file_type();
        if kind.is_dir() {
            if path.file_name().is_some_and(|name| name == ".git") {
                continue;
            }
            for entry in std::fs::read_dir(&path).map_err(error)? {
                stack.push(entry.map_err(error)?.path());
            }
        } else if kind.is_file() {
            let bytes = std::fs::read(&path).map_err(error)?;
            let text = String::from_utf8_lossy(&bytes);
            let text = text.strip_prefix('\u{feff}').unwrap_or(&text);
            let text = text.strip_suffix('\n').unwrap_or(text);
            if !text.is_empty() || bytes.ends_with(b"\n") {
                count += text
                    .split('\n')
                    .filter(|line| line.trim().is_empty())
                    .count() as u64;
            }
        }
    }
    Ok(count)
}

fn median(values: &mut [f64]) -> f64 {
    values.sort_by(f64::total_cmp);
    let middle = values.len() / 2;
    if values.len() % 2 == 0 {
        (values[middle - 1] + values[middle]) / 2.0
    } else {
        values[middle]
    }
}

fn bootstrap(ratios: &[f64]) -> (f64, f64) {
    let mut state = 0x43434d_u64;
    let mut sample = Vec::with_capacity(ratios.len());
    let mut estimates = Vec::with_capacity(5000);
    for _ in 0..5000 {
        sample.clear();
        for _ in ratios {
            state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
            sample.push(ratios[(state >> 32) as usize % ratios.len()]);
        }
        estimates.push(median(&mut sample));
    }
    estimates.sort_by(f64::total_cmp);
    (estimates[125], estimates[4874])
}

fn error(error: impl std::fmt::Display) -> String {
    error.to_string()
}

/// Splits one CSV row, honoring double-quoted fields with doubled quotes inside.
pub(super) fn csv_fields(row: &str) -> Result<Vec<String>, String> {
    let mut result = Vec::new();
    let mut field = String::new();
    let mut quoted = false;
    let mut chars = row.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '"' {
            if quoted && chars.peek() == Some(&'"') {
                field.push('"');
                chars.next();
            } else {
                quoted = !quoted;
            }
        } else if c == ',' && !quoted {
            result.push(std::mem::take(&mut field));
        } else {
            field.push(c);
        }
    }
    if quoted {
        return Err("Unclosed benchmark CSV field".into());
    }
    result.push(field);
    Ok(result)
}
