//! The closing section of the terminal report: coverage warnings, skipped inputs, and one
//! line that explains a nonzero exit status. Warnings come first, then information.
use crate::report::Buffer;
use crate::style::Style;
use crate::{Args, Bundle, text};
use std::fmt::Write;

/// Warnings shown before `--verbose` asks for all of them.
const SHOWN_WARNINGS: usize = 5;

/// Writes the notices section when there is anything to say.
pub(crate) fn render(bundle: &Bundle, args: &Args, out: &mut Buffer, style: &Style) {
    let report = &bundle.current;
    let info = information(bundle, args);
    let status = status(bundle, args);
    if report.notices.is_empty() && info.is_empty() && status.is_none() {
        return;
    }
    style.heading(out, "NOTICES", "");
    let shown = if args.verbose {
        report.notices.len()
    } else {
        report.notices.len().min(SHOWN_WARNINGS)
    };
    let warning = style.warn("!");
    for notice in report.notices.iter().take(shown) {
        if out.failed() {
            break;
        }
        let body = format!(
            "{}: {}",
            text::terminal(&notice.path),
            text::terminal(&notice.message)
        );
        style.item(out, &warning, &body, 2);
    }
    if report.notices.len() > shown {
        let hidden = report.notices.len() - shown;
        let body = format!(
            "{hidden} more {}; --verbose shows all.",
            plural(hidden, "warning")
        );
        style.item(out, &warning, &body, 2);
    }
    let marker = style.dim(style.glyph("\u{b7}", "-"));
    for line in info {
        style.item(out, &marker, &line, 2);
    }
    if let Some((failure, line)) = status {
        out.push('\n');
        let marker = if failure {
            style.error(style.glyph("\u{2717}", "x"))
        } else {
            style.warn("!")
        };
        style.item_painted(out, &marker, &line, 2, Style::bold);
    }
}

/// Returns the informational lines: skipped inputs and unrecognized file types.
fn information(bundle: &Bundle, args: &Args) -> Vec<String> {
    let report = &bundle.current;
    let skips = [
        (report.skipped_symlinks, "symlink", "symlinks"),
        (report.skipped_binary, "binary asset", "binary assets"),
        (
            report.skipped_generated,
            "generated file",
            "generated files",
        ),
        (
            report.skipped_oversized,
            "file over the per-file limit (--max-file-bytes)",
            "files over the per-file limit (--max-file-bytes)",
        ),
    ];
    let skipped: Vec<String> = skips
        .iter()
        .filter(|(count, _, _)| *count > 0)
        .map(|&(count, one, many)| format!("{count} {}", if count == 1 { one } else { many }))
        .collect();
    let mut lines = Vec::new();
    if !skipped.is_empty() {
        lines.push(format!("Skipped: {}.", skipped.join(", ")));
    }
    if report.unsupported_files > 0 {
        let count = report.unsupported_files;
        let mut line = format!("{count} unrecognized {}", plural(count, "file"));
        if args.verbose {
            let types: Vec<String> = report
                .unknown_extensions
                .iter()
                .map(|(extension, count)| {
                    let name = if extension.is_empty() {
                        "(no extension)".to_owned()
                    } else {
                        format!(".{}", text::terminal(extension))
                    };
                    format!("{name} {count}")
                })
                .collect();
            let _ = write!(line, ": {}.", types.join(", "));
        } else {
            line.push_str(" (--verbose lists types).");
        }
        lines.push(line);
    }
    lines
}

/// Explains a nonzero exit status in one line, with whether it is a failure.
fn status(bundle: &Bundle, args: &Args) -> Option<(bool, String)> {
    let report = &bundle.current;
    if !report.complete {
        let missing = report.notices.len();
        return Some((
            true,
            format!(
                "Incomplete: {missing} {} not counted (exit 2).",
                plural(missing, "file")
            ),
        ));
    }
    let change = bundle.change.as_ref()?;
    let inconsistent = change.behavior.as_ref().is_some_and(|b| !b.consistent);
    if !args.fail_on_pending || (change.pending.is_empty() && !inconsistent) {
        return None;
    }
    let pending = change.pending.len();
    let reason = if pending > 0 {
        format!(
            "{pending} {} still {} evidence",
            plural(pending, "check"),
            if pending == 1 { "needs" } else { "need" }
        )
    } else {
        "evidence is inconsistent".to_owned()
    };
    Some((
        false,
        format!("Pending: {reason} (exit 3, --fail-on-pending)."),
    ))
}

fn plural(count: usize, word: &str) -> String {
    if count == 1 {
        word.to_owned()
    } else {
        format!("{word}s")
    }
}
