//! The data island of the HTML report: one compact JSON document that the page script
//! renders. Files are rows of numbers that refer to shared language and directory tables,
//! so a tree of many thousand files stays a few bytes per file.

use super::escape::{json, json_string, number};
use crate::{Args, Bundle};
use ccm_core::Counts;
use ccm_project::FileReport;
use std::collections::BTreeMap;
use std::fmt::Write;

/// Schema name of the data island. The page script checks it before rendering.
pub(super) const SCHEMA: &str = "ccm.html-data.v1";

/// Builds the data island for `bundle`. Fails with the report-limit error as soon as the
/// text grows beyond `limit` bytes, so a huge tree never builds an unbounded string.
pub(super) fn render(
    bundle: &Bundle,
    args: &Args,
    limit: usize,
) -> Result<String, ccm_core::Error> {
    let mut out = String::new();
    let r = &bundle.current;
    let _ = write!(
        out,
        "{{\"schema\":{},\"version\":{},\"complete\":{},\"count_policy\":{},",
        json(SCHEMA),
        json(env!("CARGO_PKG_VERSION")),
        r.complete,
        json(ccm_scan::COUNT_POLICY)
    );
    let _ = write!(
        out,
        "\"no_cost\":{},\"technical\":{},\"verbose\":{},",
        args.no_cost, args.technical, args.verbose
    );
    selection(args, &mut out);
    let _ = write!(
        out,
        concat!(
            "\"totals\":{},\"lines\":{{\"program\":{},\"generated_source\":{},",
            "\"source\":{},\"documentation\":{},\"data\":{}}},"
        ),
        counts(r.totals),
        r.program_lines(),
        r.generated_source_lines(),
        r.content_lines(ccm_scan::FileKind::Source),
        r.content_lines(ccm_scan::FileKind::Documentation),
        r.content_lines(ccm_scan::FileKind::Data)
    );
    let _ = write!(
        out,
        concat!(
            "\"unsupported_files\":{},\"skipped\":{{\"symlinks\":{},\"binary\":{},",
            "\"generated\":{},\"oversized\":{},\"ignored_entries\":{}}},\"unknown_extensions\":["
        ),
        r.unsupported_files,
        r.skipped_symlinks,
        r.skipped_binary,
        r.skipped_generated,
        r.skipped_oversized,
        r.ignored_entries
    );
    for (index, (extension, count)) in r.unknown_extensions.iter().enumerate() {
        comma(&mut out, index);
        let _ = write!(out, "[{},{count}]", json(extension));
    }
    out.push_str("],");
    files(&r.files, &mut out, limit)?;
    notices(bundle, &mut out);
    comparison(bundle, &mut out);
    super::model_data::render(bundle, &mut out);
    economics(bundle, args, &mut out, limit)?;
    out.push('}');
    if out.len() > limit {
        return Err(crate::report::limit_error());
    }
    Ok(out)
}

fn selection(args: &Args, out: &mut String) {
    let s = &args.scan;
    let _ = write!(
        out,
        concat!(
            "\"selection\":{{\"symlinks\":\"skip\",\"gitignore\":{},\"default_exclusions\":{},",
            "\"skip_generated\":{},\"collect_work\":{},\"max_files\":{},\"max_file_bytes\":{},",
            "\"max_total_bytes\":{},\"max_depth\":{},\"max_report_bytes\":{},\"paths\":"
        ),
        s.respect_ignore,
        s.default_exclusions,
        s.skip_generated,
        s.collect_work,
        s.max_files,
        s.max_file_bytes,
        s.max_total_bytes,
        s.max_depth,
        args.max_report_bytes
    );
    let paths: Vec<String> = args.paths.iter().map(|p| p.display().to_string()).collect();
    strings(out, paths.iter().map(String::as_str));
    out.push_str(",\"exclusions\":");
    strings(out, s.exclude.iter().map(String::as_str));
    out.push_str(",\"include_extensions\":");
    strings(out, s.include_extensions.iter().map(String::as_str));
    out.push_str(",\"exclude_extensions\":");
    strings(out, s.exclude_extensions.iter().map(String::as_str));
    out.push_str(",\"count_as\":[");
    for (index, (extension, language)) in s.count_as.iter().enumerate() {
        comma(out, index);
        let _ = write!(out, "[{},{}]", json(extension), json(language.name()));
    }
    let compare = args.compare.as_ref().map(|p| p.display().to_string());
    let model = args.model.as_ref().map(|p| p.display().to_string());
    let _ = write!(
        out,
        "],\"compare\":{},\"model\":{},\"changed\":",
        compare.as_deref().map_or("null".into(), json),
        model.as_deref().map_or("null".into(), json)
    );
    strings(out, args.changed.iter().map(String::as_str));
    out.push_str("},");
}

/// Writes the language table, the directory table and one row per file:
/// `[directory, name, language, generated, code, comments, blank, bytes, tokens]`.
fn files(files: &[FileReport], out: &mut String, limit: usize) -> Result<(), ccm_core::Error> {
    let mut languages = BTreeMap::new();
    let mut directories = BTreeMap::new();
    for file in files {
        let next = languages.len();
        languages
            .entry(file.language.name())
            .or_insert((next, file.language));
        let next = directories.len();
        directories.entry(split(&file.name).0).or_insert(next);
    }
    out.push_str("\"languages\":[");
    let mut by_index: Vec<_> = languages.values().collect();
    by_index.sort_by_key(|(index, _)| *index);
    for (position, (_, language)) in by_index.into_iter().enumerate() {
        comma(out, position);
        let _ = write!(
            out,
            "[{},{},{}]",
            json(language.name()),
            json(language.kind().name()),
            json(language.policy())
        );
    }
    out.push_str("],\"dirs\":[");
    let mut names: Vec<(&str, usize)> = directories.iter().map(|(n, i)| (*n, *i)).collect();
    names.sort_by_key(|(_, index)| *index);
    for (position, (name, _)) in names.into_iter().enumerate() {
        comma(out, position);
        json_string(out, name);
    }
    out.push_str("],\"files\":[");
    for (index, file) in files.iter().enumerate() {
        if out.len() > limit {
            return Err(crate::report::limit_error());
        }
        comma(out, index);
        let (directory, name) = split(&file.name);
        let c = file.counts;
        let _ = write!(
            out,
            "[{},{},{},{},{},{},{},{},{}]",
            directories.get(directory).copied().unwrap_or(0),
            json(name),
            languages.get(file.language.name()).map_or(0, |(i, _)| *i),
            u8::from(file.generated),
            c.code,
            c.comments,
            c.blank,
            c.bytes,
            file.source_tokens.map_or("null".into(), |n| n.to_string())
        );
    }
    out.push_str("],");
    Ok(())
}

/// Splits a report path into its directory and its final name at the last `/`.
fn split(path: &str) -> (&str, &str) {
    path.rsplit_once('/').unwrap_or(("", path))
}

fn notices(bundle: &Bundle, out: &mut String) {
    out.push_str("\"notices\":[");
    for (index, notice) in bundle.current.notices.iter().enumerate() {
        comma(out, index);
        let _ = write!(out, "[{},{}]", json(&notice.path), json(&notice.message));
    }
    out.push_str("],");
}

fn comparison(bundle: &Bundle, out: &mut String) {
    let (Some(changes), Some(baseline)) = (&bundle.changes, &bundle.baseline) else {
        out.push_str("\"comparison\":null,");
        return;
    };
    let _ = write!(
        out,
        "\"comparison\":{{\"kind\":\"exact-file-bytes\",\"baseline_totals\":{},",
        counts(baseline.totals)
    );
    let _ = write!(
        out,
        "\"baseline_files\":{},\"baseline_program_lines\":{},\"files\":[",
        baseline.files.len(),
        baseline.program_lines()
    );
    for (index, change) in changes.iter().enumerate() {
        comma(out, index);
        let _ = write!(
            out,
            "[{},{},{},{}]",
            json(&change.name),
            json(change.kind),
            change.before.map_or("null".into(), counts),
            change.after.map_or("null".into(), counts)
        );
    }
    out.push_str("]},");
}

fn economics(
    bundle: &Bundle,
    args: &Args,
    out: &mut String,
    limit: usize,
) -> Result<(), ccm_core::Error> {
    if args.no_cost {
        out.push_str("\"economic\":null,\"ai\":null,\"budget\":null");
        return Ok(());
    }
    let c = bundle.cost;
    let a = args.cost;
    let _ = write!(
        out,
        concat!(
            "\"economic\":{{\"model\":\"ccc-resource-cost-v2\",\"currency\":\"USD\",",
            "\"status\":\"illustrative-unvalidated-scenario\",\"verification_included\":{},",
            "\"review_artifacts\":{},\"pending_policy_units\":{},\"hours_per_attempt\":{},",
            "\"expected_attempts\":{},\"expected_cost\":{},",
            "\"human_and_override_expected_cost\":{},\"assumptions\":{{",
            "\"hourly_cost\":{},\"review_hours_per_artifact\":{},\"hours_per_check_unit\":{},",
            "\"implementation_hours\":{},\"agent_cost_per_attempt\":{},",
            "\"rework_probability\":{}}}}},"
        ),
        c.pending_units.is_some(),
        c.review_artifacts,
        c.pending_units.map_or("null".into(), |n| n.to_string()),
        number(c.hours_per_attempt),
        number(c.expected_attempts),
        number(crate::economics::total(bundle)),
        number(c.usd),
        number(a.hourly_cost),
        number(a.review_hours),
        number(a.check_hours),
        number(a.implementation_hours),
        number(a.agent_cost),
        number(a.rework_probability)
    );
    // The JSON report's own writer produces the AI profile, so both reports agree.
    let mut ai = crate::report::Buffer::new(limit);
    crate::budget::json(bundle, args, &mut ai);
    out.push_str("\"ai\":");
    super::escape::script_safe(out, &ai.finish()?);
    budget(bundle, args, out);
    Ok(())
}

/// Writes the budget components with their expected-attempt multiplier applied, in the
/// order the terminal report lists them, and the total.
fn budget(bundle: &Bundle, args: &Args, out: &mut String) {
    let a = &args.cost;
    let ai = bundle.development.estimate;
    let mut rows: Vec<(&str, &str, f64)> = Vec::new();
    if args.agent_override {
        rows.push(("agent", "AI and tools (manual)", a.agent_cost));
    } else {
        let name = if bundle.development.scope_known {
            "AI implementation"
        } else {
            "Mapped AI implementation"
        };
        rows.push(("ai-implementation", name, ai.implementation_usd));
        rows.push(("ai-checks", "AI acceptance checks", ai.checks_usd));
        rows.push(("tools", "Hosted tools", ai.tools_usd));
    }
    let review = bundle.cost.review_artifacts as f64 * a.review_hours * a.hourly_cost;
    let checks = bundle.cost.pending_units.unwrap_or(0) as f64 * a.check_hours * a.hourly_cost;
    rows.push(("review", "Human review", review));
    rows.push(("human-checks", "Human acceptance checks", checks));
    let implementation = a.implementation_hours * a.hourly_cost;
    rows.push((
        "human-implementation",
        "Human implementation",
        implementation,
    ));
    out.push_str(",\"budget\":{\"components\":[");
    for (index, (key, name, value)) in rows.iter().enumerate() {
        comma(out, index);
        let _ = write!(
            out,
            "[{},{},{}]",
            json(key),
            json(name),
            number(value * bundle.cost.expected_attempts)
        );
    }
    let _ = write!(
        out,
        "],\"total\":{},\"partial_scope\":{}}}",
        number(crate::economics::total(bundle)),
        !bundle.development.scope_known || !bundle.current.complete
    );
}

/// Writes counts as `{bytes, lines, code, comments, blank}`, like the JSON report.
pub(super) fn counts(c: Counts) -> String {
    format!(
        "{{\"bytes\":{},\"lines\":{},\"code\":{},\"comments\":{},\"blank\":{}}}",
        c.bytes,
        c.lines(),
        c.code,
        c.comments,
        c.blank
    )
}

/// Writes a JSON array of strings.
pub(super) fn strings<'a>(out: &mut String, items: impl Iterator<Item = &'a str>) {
    out.push('[');
    for (index, item) in items.enumerate() {
        comma(out, index);
        json_string(out, item);
    }
    out.push(']');
}

/// Writes a separating comma before every item but the first.
pub(super) fn comma(out: &mut String, index: usize) {
    if index > 0 {
        out.push(',');
    }
}
