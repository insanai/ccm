//! The server-rendered part of the HTML report. It is complete without scripts: headline
//! numbers, the language table, the largest files and the notices. The page script
//! replaces it with the interactive report.

use super::escape::text;
use crate::report::Buffer;
use crate::{Args, Bundle};
use ccm_core::Counts;
use ccm_scan::FileKind;
use std::collections::BTreeMap;
use std::fmt::Write;

/// Number of files the static report lists, largest first.
const LARGEST_FILES: usize = 25;
/// Number of notices the static report lists.
const NOTICES: usize = 100;

/// Writes the static overview and tables into `out`.
pub(super) fn render(bundle: &Bundle, args: &Args, out: &mut Buffer) {
    let r = &bundle.current;
    out.push_str("<section id=overview-static><h2>Overview</h2><div class=metrics>");
    card(out, "Selected files", &group(r.files.len() as u64));
    card(out, "Program lines", &group(r.program_lines()));
    card(out, "Total lines", &group(r.totals.lines()));
    card(out, "Code", &group(r.totals.code));
    card(out, "Comments", &group(r.totals.comments));
    card(out, "Blank", &group(r.totals.blank));
    card(out, "Bytes", &group(r.totals.bytes));
    if !args.no_cost {
        card(
            out,
            "Budget scenario (USD)",
            &crate::text::money(crate::economics::total(bundle)),
        );
    }
    out.push_str("</div>");
    let generated = r.generated_source_lines();
    if generated > 0 {
        let _ = write!(
            out,
            "<p>{} generated source lines are listed but excluded from program lines.</p>",
            group(generated)
        );
    }
    if !r.complete {
        out.push_str("<p class=warning>Incomplete: some source files were not counted.</p>");
    }
    let _ = write!(
        out,
        "<p>Count policy {}. {} unsupported files excluded. {} notices.</p></section>",
        text(ccm_scan::COUNT_POLICY),
        group(r.unsupported_files as u64),
        group(r.notices.len() as u64)
    );
    languages(bundle, out);
    if !args.no_cost {
        crate::budget::html(bundle, args, out);
    }
    largest(bundle, out);
    notices(bundle, out);
}

fn card(out: &mut Buffer, label: &str, value: &str) {
    let _ = write!(
        out,
        "<div class=metric><span>{label}</span><strong>{value}</strong></div>"
    );
}

fn languages(bundle: &Bundle, out: &mut Buffer) {
    let mut rows: BTreeMap<(u8, &str), (u64, Counts)> = BTreeMap::new();
    for file in &bundle.current.files {
        let kind = file.language.kind();
        let row = rows.entry((rank(kind), file.language.name())).or_default();
        row.0 += 1;
        row.1.add(file.counts);
    }
    out.push_str("<section id=languages-static><h2>Languages</h2><div class=scroll><table>");
    out.push_str("<thead><tr><th>Language</th><th>Files</th><th>Code</th><th>Comments</th>");
    out.push_str("<th>Blank</th><th>Bytes</th></tr></thead><tbody>");
    for kind in [FileKind::Source, FileKind::Documentation, FileKind::Data] {
        let group_rows: Vec<_> = rows.iter().filter(|((k, _), _)| *k == rank(kind)).collect();
        if group_rows.is_empty() {
            continue;
        }
        let mut subtotal = (0, Counts::default());
        for (_, (files, counts)) in &group_rows {
            subtotal.0 += files;
            subtotal.1.add(*counts);
        }
        row(out, "group", label(kind), subtotal.0, subtotal.1);
        for ((_, name), (files, counts)) in group_rows {
            row(out, "", name, *files, *counts);
        }
    }
    out.push_str("</tbody></table></div></section>");
}

fn row(out: &mut Buffer, class: &str, name: &str, files: u64, c: Counts) {
    let _ = write!(
        out,
        concat!(
            "<tr class='{}'><td>{}</td><td>{}</td><td>{}</td>",
            "<td>{}</td><td>{}</td><td>{}</td></tr>"
        ),
        class,
        text(name),
        group(files),
        group(c.code),
        group(c.comments),
        group(c.blank),
        group(c.bytes)
    );
}

fn largest(bundle: &Bundle, out: &mut Buffer) {
    let mut files: Vec<_> = bundle.current.files.iter().collect();
    files.sort_by(|a, b| {
        b.counts
            .lines()
            .cmp(&a.counts.lines())
            .then(a.name.cmp(&b.name))
    });
    let _ = write!(
        out,
        "<section id=files-static><h2>Largest files</h2><p>The {} largest of {} files.</p>",
        files.len().min(LARGEST_FILES),
        group(files.len() as u64)
    );
    out.push_str("<div class=scroll><table><thead><tr><th>File</th><th>Language</th>");
    out.push_str("<th>Code</th><th>Comments</th><th>Blank</th></tr></thead><tbody>");
    for file in files.into_iter().take(LARGEST_FILES) {
        let _ = write!(
            out,
            "<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>",
            text(&file.name),
            text(file.language.name()),
            group(file.counts.code),
            group(file.counts.comments),
            group(file.counts.blank)
        );
    }
    out.push_str("</tbody></table></div></section>");
}

fn notices(bundle: &Bundle, out: &mut Buffer) {
    let notices = &bundle.current.notices;
    if notices.is_empty() {
        return;
    }
    out.push_str("<section id=notices-static><h2>Notices</h2><ul>");
    for notice in notices.iter().take(NOTICES) {
        if out.failed() {
            break;
        }
        let _ = write!(
            out,
            "<li class=warning>{}: {}</li>",
            text(&notice.path),
            text(&notice.message)
        );
    }
    out.push_str("</ul></section>");
}

fn rank(kind: FileKind) -> u8 {
    match kind {
        FileKind::Source => 0,
        FileKind::Documentation => 1,
        FileKind::Data => 2,
    }
}

fn label(kind: FileKind) -> &'static str {
    match kind {
        FileKind::Source => "Program source",
        FileKind::Documentation => "Documentation",
        FileKind::Data => "Data and configuration",
    }
}

/// Formats an integer with thousands separators, such as `12,345`.
pub(super) fn group(value: u64) -> String {
    let digits = value.to_string();
    let mut out = String::with_capacity(digits.len() + digits.len() / 3);
    for (index, digit) in digits.chars().enumerate() {
        if index > 0 && (digits.len() - index) % 3 == 0 {
            out.push(',');
        }
        out.push(digit);
    }
    out
}

#[cfg(test)]
mod tests {
    #[test]
    fn numbers_are_grouped_in_thousands() {
        assert_eq!(super::group(0), "0");
        assert_eq!(super::group(999), "999");
        assert_eq!(super::group(1000), "1,000");
        assert_eq!(super::group(1234567), "1,234,567");
    }
}
