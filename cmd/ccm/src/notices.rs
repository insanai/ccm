use crate::report::Buffer;
use crate::{Args, Bundle, text};
use std::fmt::Write;

pub(crate) fn render(bundle: &Bundle, args: &Args, out: &mut Buffer) {
    let report = &bundle.current;
    if report.skipped_symlinks + report.skipped_binary + report.skipped_generated > 0 {
        let mut skipped = Vec::new();
        for (count, label) in [
            (report.skipped_symlinks, "symlinks"),
            (report.skipped_binary, "binary assets"),
            (report.skipped_generated, "generated files"),
        ] {
            if count > 0 {
                skipped.push(format!("{count} {label}"));
            }
        }
        let _ = writeln!(out, "  Skipped: {}.", skipped.join(", "));
    }
    if report.unsupported_files > 0 {
        let _ = writeln!(
            out,
            "  {} unrecognized files{}.",
            report.unsupported_files,
            if args.verbose {
                ""
            } else {
                " (--verbose lists types)"
            }
        );
    }
    if args.verbose {
        for (extension, count) in &report.unknown_extensions {
            if out.failed() {
                break;
            }
            let _ = writeln!(
                out,
                "    {}: {count} files",
                text::terminal(if extension.is_empty() {
                    "(no extension)"
                } else {
                    extension
                })
            );
        }
    }
    let shown = if args.verbose {
        report.notices.len()
    } else {
        report.notices.len().min(5)
    };
    for notice in report.notices.iter().take(shown) {
        if out.failed() {
            break;
        }
        let _ = writeln!(
            out,
            "  Warning: {}: {}",
            text::terminal(&notice.path),
            text::terminal(&notice.message)
        );
    }
    if report.notices.len() > shown {
        let _ = writeln!(
            out,
            "  {} more warnings; --verbose shows all.",
            report.notices.len() - shown
        );
    }
    if !report.complete {
        out.push_str("  INCOMPLETE: some text files could not be counted (exit 2).\n");
    }
}
