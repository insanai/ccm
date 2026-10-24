//! The human terminal report. Sections appear from summary to detail: the scope header,
//! the language table, changes, the acceptance plan, the estimate, technical details,
//! and notices last. Every section shares one `Style` and stays within its width.
use crate::report::Buffer;
use crate::style::Style;
use crate::{Args, Bundle, text};
use std::fmt::Write;

/// Changed files listed before `--verbose`, `--technical`, or `--by-file` asks for all.
const SHOWN_CHANGES: usize = 12;

/// Renders the terminal report; `color` enables ANSI styling.
///
/// # Errors
///
/// Returns `report-limit` when the report exceeds `--max-report-bytes`.
pub(crate) fn render(
    bundle: &Bundle,
    args: &Args,
    color: bool,
) -> Result<String, ccm_core::Error> {
    let mut out = Buffer::new(args.max_report_bytes);
    let width = text::terminal_width();
    let table = crate::inventory::Table::new(bundle, args);
    let style = Style::new(color, args.plain, width, table.natural_width());
    header(bundle, &mut out, &style);
    crate::inventory::render(&table, &mut out, &style);
    if let Some(changes) = &bundle.changes {
        change_overview(bundle, args, changes, &mut out, &style);
    }
    acceptance(bundle, &mut out, &style, args.technical);
    if !args.no_cost {
        crate::estimate::render(bundle, args, &mut out, &style);
    }
    if args.technical || args.by_file {
        technical(bundle, &mut out, &style);
    }
    crate::notices::render(bundle, args, &mut out, &style);
    out.push('\n');
    let report = out.finish()?;
    // A safety net only: every section already fits the width.
    let wrapped = text::wrap_terminal(&report, width);
    if wrapped.len() > args.max_report_bytes {
        return Err(crate::report::limit_error());
    }
    Ok(wrapped)
}

/// Labels program lines, and names generated source lines separately so that the
/// header agrees with the development estimate, which never prices generated files.
pub(crate) fn program_lines(report: &ccm_project::Report) -> String {
    let program = text::number(report.program_lines());
    match report.generated_source_lines() {
        0 if report.program_lines() == 1 => format!("{program} program line"),
        0 => format!("{program} program lines"),
        generated => format!(
            "{program} program lines (+{} generated)",
            text::number(generated)
        ),
    }
}

/// Writes the tool name and the scope in one line each: files, program lines, size.
fn header(bundle: &Bundle, out: &mut Buffer, style: &Style) {
    let report = &bundle.current;
    let tagline = "source, change, and acceptance intelligence";
    let _ = write!(out, "\n  {}", style.bold(&style.accent("CCM")));
    if 2 + 3 + 2 + tagline.len() <= style.width {
        let _ = write!(out, "  {}", style.dim(tagline));
    }
    out.push('\n');
    let files = report.files.len();
    let parts = [
        style.bold(&program_lines(report)),
        style.dim(&format!(
            "{} {}",
            text::number(files as u64),
            if files == 1 { "file" } else { "files" }
        )),
        style.dim(&text::bytes(report.totals.bytes)),
    ];
    let dot = style.dim(style.dot());
    for line in text::pack(&parts, &dot, style.width.saturating_sub(2)) {
        let _ = writeln!(out, "  {line}");
    }
}

/// Writes the compared files: a marker and kind word, the path, and code lines before
/// and after with the signed difference.
fn change_overview(
    bundle: &Bundle,
    args: &Args,
    changes: &[ccm_project::FileChange],
    out: &mut Buffer,
    style: &Style,
) {
    let note = args
        .compare
        .as_ref()
        .map(|path| {
            let path = text::terminal(&path.display().to_string());
            let room = (style.width / 2).saturating_sub(3);
            format!("vs {}", text::truncate_left(&path, room, style.unicode))
        })
        .unwrap_or_default();
    style.heading(out, "CHANGES", &note);
    let mut kinds = Vec::new();
    for kind in ["modified", "added", "removed"] {
        let count = changes.iter().filter(|c| c.kind == kind).count();
        if count > 0 {
            kinds.push(format!(
                "{} {kind}",
                style.bold(&text::number(count as u64))
            ));
        }
    }
    if kinds.is_empty() {
        let _ = writeln!(out, "    {}", style.dim("No files differ."));
        return;
    }
    let review = bundle.cost.review_artifacts;
    let unit = match (bundle.model.is_some(), review == 1) {
        (true, true) => "part",
        (true, false) => "parts",
        (false, true) => "program file",
        (false, false) => "program files",
    };
    let _ = writeln!(
        out,
        "    {}{}{} {unit} to review",
        kinds.join(&style.dim(style.dot())),
        style.dim(style.arrow()),
        style.bold(&text::number(review as u64)),
    );
    let all = args.verbose || args.technical || args.by_file;
    let shown = if all {
        changes.len()
    } else {
        changes.len().min(SHOWN_CHANGES)
    };
    change_rows(&changes[..shown], out, style);
    if changes.len() > shown {
        let more = format!(
            "{} more changed files; --verbose lists all.",
            changes.len() - shown
        );
        let _ = writeln!(out, "    {}", style.dim(&more));
    }
}

fn change_rows(changes: &[ccm_project::FileChange], out: &mut Buffer, style: &Style) {
    let code = |c: Option<ccm_core::Counts>| c.map_or(0, |c| c.code);
    let figures: Vec<(String, String, String)> = changes
        .iter()
        .map(|c| {
            let (before, after) = (code(c.before), code(c.after));
            let delta = i128::from(after) - i128::from(before);
            let delta = if delta == 0 {
                "0".to_owned()
            } else {
                format!("{delta:+}")
            };
            (text::number(before), text::number(after), delta)
        })
        .collect();
    let widest =
        |pick: fn(&(String, String, String)) -> usize| figures.iter().map(pick).max().unwrap_or(0);
    let (bw, aw, dw) = (
        widest(|f| f.0.len()),
        widest(|f| f.1.len()),
        widest(|f| f.2.len()),
    );
    let arrow = style.glyph(" \u{2192} ", " -> ");
    let arrow_width = arrow.chars().count();
    let before_width = bw.max(4);
    let numbers = before_width + arrow_width + aw + 2 + dw.max(4);
    // indent, marker, space, kind, two spaces, path, two spaces, numbers
    let available = style
        .width
        .saturating_sub(4 + 2 + 8 + 2 + 2 + numbers)
        .max(8);
    let longest = changes
        .iter()
        .map(|c| text::terminal(&c.name).chars().count());
    let path_width = longest.max().unwrap_or(0).clamp(4, available);
    let _ = writeln!(
        out,
        "    {}",
        style.dim(&format!(
            "{:<10}  {:<path_width$}  {:>block$}  {:>dw$}",
            "",
            "File",
            "Code",
            "Diff",
            block = before_width + arrow_width + aw,
            dw = dw.max(4)
        ))
    );
    for (change, (before, after, delta)) in changes.iter().zip(&figures) {
        if out.failed() {
            break;
        }
        let marker = match change.kind {
            "added" => "+",
            "removed" => "-",
            _ => "~",
        };
        let path = text::truncate_left(&text::terminal(&change.name), path_width, style.unicode);
        let _ = writeln!(
            out,
            "    {} {}  {path:<path_width$}  {before:>before_width$}{}{after:>aw$}  {}",
            style.bold(marker),
            style.dim(&format!("{:<8}", change.kind)),
            style.dim(arrow),
            style.dim(&format!("{delta:>width$}", width = dw.max(4)))
        );
    }
}

/// Writes the acceptance plan: the flow from changed artifacts to pending checks, one
/// row per required check, and what the model can still tell apart.
fn acceptance(bundle: &Bundle, out: &mut Buffer, style: &Style, technical: bool) {
    let (Some(model), Some(change)) = (&bundle.model, &bundle.change) else {
        if technical {
            style.heading(out, "ACCEPTANCE PLAN", "");
            let _ = writeln!(
                out,
                "    {}",
                style.dim("No task model requested (--model).")
            );
        }
        return;
    };
    style.heading(out, "ACCEPTANCE PLAN", "");
    for line in text::wrap_words(&text::terminal(&model.context), style.width - 4) {
        let _ = writeln!(out, "    {}", style.dim(&line));
    }
    flow(bundle, change, out, style);
    out.push('\n');
    checks(model, change, out, style);
    out.push('\n');
    let facts = [format!(
        "Previous results: {} reusable{}{} need rechecking",
        change.surviving.len(),
        style.dot(),
        change.invalidated.len()
    )];
    for fact in facts {
        note(out, style, &fact);
    }
    outcomes(change, out, style, technical);
    if technical {
        model_metrics(bundle, change, out, style);
    }
    note(
        out,
        style,
        "Exact within the declared finite model; source coverage is not proven.",
    );
    if technical {
        witnesses(model, change, out, style);
    }
}

/// Draws `2 changed ─▶ 3 affected ─▶ 5 checks (4 pending)`.
fn flow(bundle: &Bundle, change: &ccm_core::ChangeReport, out: &mut Buffer, style: &Style) {
    let count = |n: usize| style.bold(&text::number(n as u64));
    let pending = change.pending.len();
    let pending_text = format!("{pending} pending");
    let pending_text = if pending > 0 {
        style.warn(&pending_text)
    } else {
        pending_text
    };
    let checks = if change.active.len() == 1 {
        "check"
    } else {
        "checks"
    };
    let steps = [
        format!("{} changed", count(bundle.seeds.len())),
        format!("{} affected", count(change.affected.len())),
        format!("{} {checks} ({pending_text})", count(change.active.len())),
    ];
    out.push('\n');
    let arrow = style.dim(style.arrow());
    for (index, line) in text::pack(&steps, &arrow, style.width.saturating_sub(7))
        .iter()
        .enumerate()
    {
        // A continued flow starts with its arrow, so the chain still reads left to right.
        let lead = if index == 0 {
            String::new()
        } else {
            style.dim(style.arrow().trim_start())
        };
        let _ = writeln!(out, "    {lead}{line}");
    }
}

fn checks(
    model: &ccm_core::Model,
    change: &ccm_core::ChangeReport,
    out: &mut Buffer,
    style: &Style,
) {
    for &q in &change.active {
        if out.failed() {
            break;
        }
        let failed = change.failed.binary_search(&q).is_ok();
        let pending = change.pending.binary_search(&q).is_ok();
        let (glyph, word) = if failed {
            (
                style.error(style.glyph("\u{2717}", "[!]")),
                style.error("failed   "),
            )
        } else if pending {
            (
                style.warn(style.glyph("\u{25cb}", "[ ]")),
                style.warn("pending  "),
            )
        } else {
            (
                style.accent(style.glyph("\u{25cf}", "[x]")),
                style.dim("supported"),
            )
        };
        let name = text::terminal(&model.obligations[q].name);
        let prefix = 4 + text::visible_width(&glyph) + 1 + 9 + 2;
        let name = text::truncate(&name, style.width.saturating_sub(prefix), style.unicode);
        let _ = writeln!(out, "    {glyph} {word}  {name}");
    }
}

fn outcomes(change: &ccm_core::ChangeReport, out: &mut Buffer, style: &Style, technical: bool) {
    let Some(b) = &change.behavior else {
        return;
    };
    note(
        out,
        style,
        &format!(
            "Modeled outcomes still possible: {} ({} compatible cases).",
            b.observations, b.hypotheses
        ),
    );
    if technical {
        let bits = b
            .distinction_bits
            .map_or("unavailable".into(), |v| format!("{v:.4}"));
        note(
            out,
            style,
            &format!("Distinction capacity: {bits} bits (log2 of exact outcome count)."),
        );
    }
    if !b.consistent {
        style.item(
            out,
            &style.warn("!"),
            "Evidence is inconsistent within the declared case universe.",
            4,
        );
    }
}

/// Writes the batch interaction and the policy units of the plan.
fn model_metrics(
    bundle: &Bundle,
    change: &ccm_core::ChangeReport,
    out: &mut Buffer,
    style: &Style,
) {
    if let Some(i) = &bundle.interaction {
        note(
            out,
            style,
            &format!(
                "Combined edits: {} shared checks, {} additional joint checks.",
                i.shared.len(),
                i.joint.len()
            ),
        );
        note(
            out,
            style,
            &format!(
                "First seed vs remaining seeds: net {:+} policy units.",
                i.net_units
            ),
        );
    }
    note(
        out,
        style,
        &format!(
            "Policy units: required {}, pending {} (not time units).",
            change.required_units, change.pending_units
        ),
    );
}

/// Writes why each artifact is affected: the dependency it was reached through.
fn witnesses(
    model: &ccm_core::Model,
    change: &ccm_core::ChangeReport,
    out: &mut Buffer,
    style: &Style,
) {
    let _ = writeln!(out, "\n    {}", style.bold("Dependency witnesses"));
    for &id in &change.affected {
        if out.failed() {
            break;
        }
        let previous = change.predecessor[id.0].map(|p| model.artifacts[p.0].as_str());
        let witness = format!(
            "{}{}{}",
            text::terminal(previous.unwrap_or("change seed")),
            style.arrow(),
            text::terminal(&model.artifacts[id.0])
        );
        note(out, style, &witness);
    }
}

/// Writes dim secondary text wrapped at the section indentation.
fn note(out: &mut Buffer, style: &Style, body: &str) {
    for line in text::wrap_words(body, style.width.saturating_sub(4)) {
        let _ = writeln!(out, "    {}", style.dim(&line));
    }
}

/// Writes the counting policy and a per-file table with the path last, so that long
/// paths never disturb the numeric columns.
fn technical(bundle: &Bundle, out: &mut Buffer, style: &Style) {
    let files = &bundle.current.files;
    style.heading(out, "TECHNICAL DETAILS", "");
    for line in [
        format!(
            "Count policy: {}; mixed lines count as code; UTF-8 only.",
            ccm_scan::COUNT_POLICY
        ),
        "Nested ignore rules and explicit selection options determine the inventory.".into(),
        "Exact finite model calculations; no claim of semantic extraction completeness.".into(),
    ] {
        note(out, style, &line);
    }
    file_table(files, out, style);
}

/// Writes one row per file with the path last. Narrow terminals keep code and bytes only,
/// so that paths stay readable.
fn file_table(files: &[ccm_project::FileReport], out: &mut Buffer, style: &Style) {
    let all = ["Code", "Comments", "Blank", "Bytes"];
    let value = |f: &ccm_project::FileReport, column: &str| match column {
        "Code" => f.counts.code,
        "Comments" => f.counts.comments,
        "Blank" => f.counts.blank,
        _ => f.counts.bytes,
    };
    let width_of = |column: &str| {
        let widest = files.iter().map(|f| text::number(value(f, column)).len());
        widest.max().unwrap_or(0).max(column.len())
    };
    let mut columns: Vec<(&str, usize)> = all.iter().map(|&c| (c, width_of(c))).collect();
    let numbers = |columns: &[(&str, usize)]| columns.iter().map(|(_, w)| w + 2).sum::<usize>();
    if style.width.saturating_sub(4 + numbers(&columns)) < 24 {
        columns.retain(|(column, _)| matches!(*column, "Code" | "Bytes"));
    }
    let path_width = style.width.saturating_sub(4 + numbers(&columns)).max(8);
    let mut row = String::new();
    for (header, width) in &columns {
        let _ = write!(row, "{header:>width$}  ");
    }
    let _ = writeln!(out, "\n    {}", style.dim(&format!("{row}File")));
    for file in files {
        if out.failed() {
            break;
        }
        let mut row = String::new();
        for (column, width) in &columns {
            let _ = write!(row, "{:>width$}  ", text::number(value(file, column)));
        }
        let path = text::truncate_left(&text::terminal(&file.name), path_width, style.unicode);
        let _ = writeln!(out, "    {row}{path}");
    }
}
