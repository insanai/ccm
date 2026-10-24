use ccm_core::{CostOptions, Error};
use ccm_project::Options;
use std::path::PathBuf;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Format {
    Terminal,
    Json,
    Html,
    Csv,
    Ndjson,
}

pub(crate) struct Args {
    pub paths: Vec<PathBuf>,
    pub scan: Options,
    pub format: Format,
    pub technical: bool,
    pub color: String,
    /// Whether `--format plain` asked for ASCII-only, colorless terminal text.
    pub plain: bool,
    pub output: Option<PathBuf>,
    pub compare: Option<PathBuf>,
    pub model: Option<PathBuf>,
    pub changed: Vec<String>,
    pub cost: CostOptions,
    pub ai: ccm_core::AiOptions,
    pub agent_override: bool,
    /// Whether `--cost-profile` replaced the 2026 default calibration.
    pub custom_profile: bool,
    pub languages: bool,
    pub licenses: bool,
    pub help: bool,
    pub version: bool,
    pub by_file: bool,
    pub sort: String,
    pub force: bool,
    pub max_report_bytes: usize,
    pub no_cost: bool,
    pub fail_on_pending: bool,
    pub verbose: bool,
}

impl Default for Args {
    fn default() -> Self {
        Self {
            paths: Vec::new(),
            scan: Options::default(),
            format: Format::Terminal,
            technical: false,
            color: "auto".into(),
            plain: false,
            output: None,
            compare: None,
            model: None,
            changed: Vec::new(),
            cost: CostOptions::default(),
            ai: ccm_core::AiOptions::default(),
            agent_override: false,
            custom_profile: false,
            languages: false,
            licenses: false,
            help: false,
            version: false,
            by_file: false,
            sort: "code".into(),
            force: false,
            max_report_bytes: 256 * 1024 * 1024,
            no_cost: false,
            fail_on_pending: false,
            verbose: false,
        }
    }
}

pub(crate) fn parse(arguments: impl Iterator<Item = String>) -> Result<Args, Error> {
    let mut args = Args::default();
    let mut words = normalize(arguments).into_iter();
    while let Some(word) = words.next() {
        if word == "--" {
            args.paths.extend(words.map(PathBuf::from));
            break;
        }
        match word.as_str() {
            "--help" | "-h" => args.help = true,
            "--version" | "-V" => args.version = true,
            "--technical" => args.technical = true,
            "--languages" | "-l" => args.languages = true,
            "--licenses" => args.licenses = true,
            "--by-file" | "--files" => args.by_file = true,
            "--force" => args.force = true,
            "--no-cost" => args.no_cost = true,
            "--fail-on-pending" => args.fail_on_pending = true,
            "--verbose" | "-v" => args.verbose = true,
            flag if selection_option(flag, &mut words, &mut args)? => {}
            flag if crate::economics::option(flag, &mut words, &mut args)? => {}
            "--json" => args.format = Format::Json,
            "--html" => args.format = Format::Html,
            "--format" => {
                args.format = match value(&mut words, &word)?.as_str() {
                    "terminal" => Format::Terminal,
                    "plain" => {
                        args.color = "never".into();
                        args.plain = true;
                        Format::Terminal
                    }
                    "json" => Format::Json,
                    "html" => Format::Html,
                    "csv" => Format::Csv,
                    "ndjson" => Format::Ndjson,
                    _ => {
                        return Err(usage(
                            "Expected terminal, plain, json, html, csv, or ndjson",
                        ));
                    }
                }
            }
            "--color" => {
                args.color = value(&mut words, &word)?;
                if !matches!(args.color.as_str(), "auto" | "always" | "never") {
                    return Err(usage("Expected auto, always, or never for --color"));
                }
            }
            "-o" | "--output" => args.output = Some(value(&mut words, &word)?.into()),
            "--compare" => args.compare = Some(value(&mut words, &word)?.into()),
            "--model" => args.model = Some(value(&mut words, &word)?.into()),
            "--changed" => args.changed.push(value(&mut words, &word)?),
            "--exclude" => args.scan.exclude.push(value(&mut words, &word)?),
            "--max-report-bytes" => args.max_report_bytes = number(&mut words, &word)?,
            "--max-files" => args.scan.max_files = number(&mut words, &word)?,
            "--max-file-bytes" => args.scan.max_file_bytes = number(&mut words, &word)?,
            "--max-total-bytes" => args.scan.max_total_bytes = number(&mut words, &word)?,
            "--hourly-cost" => args.cost.hourly_cost = number(&mut words, &word)?,
            "--review-hours" => args.cost.review_hours = number(&mut words, &word)?,
            "--check-hours" => args.cost.check_hours = number(&mut words, &word)?,
            "--implementation-hours" => {
                args.cost.implementation_hours = number(&mut words, &word)?
            }
            "--rework-probability" => args.cost.rework_probability = number(&mut words, &word)?,
            option if option.starts_with('-') => {
                return Err(usage(format!("Unknown option: {option}")));
            }
            _ => args.paths.push(word.into()),
        }
    }
    validate(&mut args)?;
    Ok(args)
}

fn validate(args: &mut Args) -> Result<(), Error> {
    if args.help || args.version || args.languages || args.licenses {
        return Ok(());
    }
    if !(1..=64).contains(&args.scan.jobs) {
        return Err(usage("--jobs must be from 1 to 64"));
    }
    if !matches!(
        args.sort.as_str(),
        "name" | "code" | "files" | "lines" | "comments" | "blanks"
    ) {
        return Err(usage(
            "--sort expects name, code, files, lines, comments, or blanks",
        ));
    }
    args.scan.collect_work = args.technical && !args.no_cost;
    args.ai.validate()?;
    if args.paths.is_empty() {
        args.paths.push(".".into());
    }
    if args.compare.is_some() && args.paths.len() != 1 {
        return Err(usage(
            "--compare requires exactly one current snapshot path",
        ));
    }
    if (args.fail_on_pending || !args.changed.is_empty()) && args.model.is_none() {
        return Err(usage(
            "--changed and --fail-on-pending require an explicit --model",
        ));
    }
    Ok(())
}

fn normalize(arguments: impl Iterator<Item = String>) -> Vec<String> {
    let mut words = Vec::new();
    let mut positional = false;
    for word in arguments {
        if !positional && word.starts_with("--") && word.contains('=') {
            if let Some((flag, value)) = word.split_once('=') {
                words.push(flag.into());
                words.push(value.into());
            }
        } else {
            positional |= word == "--";
            words.push(word);
        }
    }
    words
}

fn selection_option(
    word: &str,
    words: &mut impl Iterator<Item = String>,
    args: &mut Args,
) -> Result<bool, Error> {
    match word {
        "--no-ignore" => args.scan.respect_ignore = false,
        "--no-default-ignore" => args.scan.default_exclusions = false,
        "--no-generated" => args.scan.skip_generated = true,
        "--jobs" | "-j" => args.scan.jobs = number(words, word)?,
        "--include-ext" | "-i" => args
            .scan
            .include_extensions
            .extend(value(words, word)?.split(',').map(str::to_owned)),
        "--exclude-ext" | "-x" => args
            .scan
            .exclude_extensions
            .extend(value(words, word)?.split(',').map(str::to_owned)),
        "--count-as" => {
            let mapping = value(words, word)?;
            let (extension, name) = mapping
                .split_once(':')
                .ok_or_else(|| usage("Use EXT:LANGUAGE"))?;
            let language = ccm_scan::languages()
                .iter()
                .find(|info| info.name.eq_ignore_ascii_case(name))
                .ok_or_else(|| usage("Unknown --count-as language; see --languages"))?
                .language;
            args.scan.count_as.push((extension.into(), language));
        }
        "--sort" | "-s" => args.sort = value(words, word)?,
        _ => return Ok(false),
    }
    Ok(true)
}

pub(crate) fn value(
    words: &mut impl Iterator<Item = String>,
    flag: &str,
) -> Result<String, Error> {
    words
        .next()
        .ok_or_else(|| usage(format!("Missing value for {flag}")))
}

pub(crate) fn number<T: std::str::FromStr>(
    words: &mut impl Iterator<Item = String>,
    flag: &str,
) -> Result<T, Error> {
    value(words, flag)?
        .parse()
        .map_err(|_| usage(format!("Invalid number for {flag}")))
}

fn usage(message: impl Into<String>) -> Error {
    Error::new(
        "arguments",
        message,
        "Run ccm --help for options and examples.",
    )
}

pub(crate) const HELP: &str = "\
CCM - source, change, and acceptance intelligence

Usage: ccm [OPTIONS] [PATH ...]

Reports:
  --technical               Show per-file counts and dependency explanations
  --by-file                 Show per-file details
  --format FORMAT           terminal, plain, json, html, csv, ndjson
  --json / --html           Machine or standalone web report shortcuts
  -o, --output FILE         Save a report (existing files are protected)
  --force                   Replace an existing report atomically
  --color WHEN              auto, always, never (auto honors nonempty NO_COLOR)
  --sort COLUMN             code (default), name, files, lines, comments, blanks
  --verbose                 Show estimate assumptions and all coverage warnings

Selection:
  --languages               List bundled language policies and format aliases
  -j, --jobs N              Scanner workers, 1..64 (default: hardware threads, up to 16)
  -i, --include-ext LIST     Include comma-separated extensions (without dots)
  -x, --exclude-ext LIST     Exclude comma-separated extensions
  --count-as EXT:LANGUAGE    Apply a named policy to an extension (repeatable)
  --exclude NAME_OR_PATH    Exclude an exact name or root-relative path prefix
  --no-ignore               Disable .gitignore, .ignore, and .ccmignore rules
  --no-default-ignore       Include normally excluded build/vendor directories
  --no-generated            Exclude files declaring generated content in headers
  --max-report-bytes N      Output budget (default: 268435456)
  --max-files N             File budget (default: 1000000)
  --max-file-bytes N        Per-file budget (default: 67108864)
  --max-total-bytes N       Snapshot budget (default: 68719476736)

Changes and acceptance:
  --compare PATH            Compare an earlier snapshot with one current path
  --model FILE              Evaluate an explicit finite task/evidence model
  --changed ARTIFACT        Model seed (repeatable); comparison paths otherwise
  --fail-on-pending         Exit 3 for open checks or inconsistent case evidence

Budget assumptions (USD; scenarios, not measured effort):
  --no-cost                 Omit the economic scenario
  --hourly-cost N           Loaded human cost/hour (default: 100)
  --review-hours N          Review hours/affected part or file (default: 0.25)
  --check-hours N           Hours/pending check effort unit (default: 1)
  --implementation-hours N  Explicit implementation hours/attempt (default: 0)
  --agent-cost N            Override ALL automatic AI/tool spend per attempt
  --cost-profile FILE       Editable AI resource/pricing profile (2026 default)
  --ai-input-price N        USD per million input tokens (default: 10)
  --ai-output-price N       USD per million output tokens (default: 50)
  --ai-workload N           Multiply inferred AI workload (default: 1)
  --rework-probability N    Stationary repeat probability, 0..0.95 (default: 0)

  -h, --help                Show help
  -V, --version             Show version
  --licenses                Show bundled syntax-data license notices

Examples:
  ccm --color=always .
  ccm --compare ../before . --technical
  ccm --model examples/checkout.ccm --changed pricing --html -o report.html
  ccm --format=csv --include-ext=rs,py .

No source code or external tools are executed during measurement.
Symlinks are skipped without warnings. Unavailable semantics stay explicit.
";
