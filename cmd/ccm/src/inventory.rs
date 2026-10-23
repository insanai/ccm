use crate::report::Buffer;
use crate::{Args, Bundle, text};
use ccm_core::Counts;
use std::collections::BTreeMap;
use std::fmt::Write;

pub(crate) fn render(bundle: &Bundle, args: &Args, out: &mut Buffer, color: bool) {
    let mut groups: BTreeMap<&str, (usize, Counts)> = BTreeMap::new();
    for file in &bundle.current.files {
        let group = groups.entry(file.language.name()).or_default();
        group.0 += 1;
        group.1.add(file.counts);
    }
    let mut groups: Vec<_> = groups.into_iter().collect();
    groups.sort_by(|(a, (ac, av)), (b, (bc, bv))| {
        let order = match args.sort.as_str() {
            "name" => a.cmp(b),
            "files" => bc.cmp(ac),
            "lines" => bv.lines().cmp(&av.lines()),
            "comments" => bv.comments.cmp(&av.comments),
            "blanks" => bv.blank.cmp(&av.blank),
            _ => bv.code.cmp(&av.code),
        };
        order.then_with(|| a.cmp(b))
    });
    let narrow = text::terminal_width() < 72;
    rule(out, narrow, color);
    if narrow {
        out.push_str("  Language          Files    Lines     Code\n");
    } else {
        out.push_str(
            "  Language             Files       Lines     Blank   Comments        Code\n",
        );
    }
    rule(out, narrow, color);
    for (name, (files, counts)) in groups {
        if out.failed() {
            break;
        }
        row(out, name, files, counts, narrow, color);
    }
    rule(out, narrow, color);
    row(
        out,
        "Total",
        bundle.current.files.len(),
        bundle.current.totals,
        narrow,
        color,
    );
    rule(out, narrow, color);
    out.push('\n');
}

fn row(out: &mut Buffer, name: &str, files: usize, counts: Counts, narrow: bool, color: bool) {
    let cyan = if color { "\x1b[95m" } else { "" };
    let green = if color { "\x1b[33m" } else { "" };
    let dim = if color { "\x1b[2m" } else { "" };
    let reset = if color { "\x1b[0m" } else { "" };
    if narrow {
        let name: String = name.chars().take(14).collect();
        let _ = writeln!(
            out,
            "  {name:<14} {:>6} {:>8} {cyan}{:>8}{reset}",
            text::number(files as u64),
            text::number(counts.lines()),
            text::number(counts.code)
        );
    } else {
        let name = if name.chars().count() > 19 {
            format!("{}...", name.chars().take(16).collect::<String>())
        } else {
            name.into()
        };
        let _ = writeln!(
            out,
            "  {name:<19} {:>6} {:>11} {dim}{:>9}{reset} {green}{:>10}{reset} {cyan}{:>11}{reset}",
            text::number(files as u64),
            text::number(counts.lines()),
            text::number(counts.blank),
            text::number(counts.comments),
            text::number(counts.code)
        );
    }
}

fn rule(out: &mut Buffer, narrow: bool, color: bool) {
    let tint = if color { "\x1b[2m" } else { "" };
    let reset = if color { "\x1b[0m" } else { "" };
    let _ = writeln!(
        out,
        "  {tint}{}{reset}",
        "-".repeat(if narrow { 42 } else { 73 })
    );
}
