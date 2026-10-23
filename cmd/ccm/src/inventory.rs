//! The language table of the terminal report.
//!
//! Rows are grouped by purpose, so the program-line figure of the header is the code
//! subtotal of the first group. Columns are sized to their content and dropped from the
//! least important upward until the table fits the terminal.
use crate::report::Buffer;
use crate::style::{self, Style};
use crate::{Args, Bundle, text};
use ccm_core::Counts;
use ccm_project::FileReport;
use ccm_scan::FileKind;
use std::collections::BTreeMap;
use std::fmt::Write;

/// The purpose groups of the table, in display order.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Group {
    Program,
    Generated,
    Documentation,
    Data,
}

impl Group {
    fn of(file: &FileReport) -> Self {
        match file.language.kind() {
            FileKind::Source if file.generated => Self::Generated,
            FileKind::Source => Self::Program,
            FileKind::Documentation => Self::Documentation,
            FileKind::Data => Self::Data,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Program => "Program source",
            Self::Generated => "Generated",
            Self::Documentation => "Documentation",
            Self::Data => "Data and config",
        }
    }
}

/// One table row: a language within a group, a group subtotal, or the total.
#[derive(Clone, Copy)]
struct Row<'a> {
    name: &'a str,
    files: usize,
    counts: Counts,
}

/// Numeric columns from the most to the least important.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Column {
    Code,
    Files,
    Lines,
    Comments,
    Blank,
}

impl Column {
    fn header(self) -> &'static str {
        match self {
            Self::Code => "Code",
            Self::Files => "Files",
            Self::Lines => "Lines",
            Self::Comments => "Comments",
            Self::Blank => "Blank",
        }
    }

    fn value(self, row: &Row) -> u64 {
        match self {
            Self::Code => row.counts.code,
            Self::Files => row.files as u64,
            Self::Lines => row.counts.lines(),
            Self::Comments => row.counts.comments,
            Self::Blank => row.counts.blank,
        }
    }
}

/// The fitted shape of the table.
struct Layout {
    name: usize,
    columns: Vec<(Column, usize)>,
    /// Bar cells of the share column, or zero when the column is hidden.
    bar: usize,
}

const SHARE_GAP: usize = 2;
const PERCENT: usize = 4;

impl Layout {
    fn width(&self) -> usize {
        let numbers: usize = self.columns.iter().map(|(_, width)| width + 2).sum();
        let share = if self.bar > 0 {
            SHARE_GAP + self.bar + 1 + PERCENT
        } else {
            0
        };
        2 + self.name + numbers + share
    }

    /// Fits the table into `width` columns: drop Blank, Comments, and Lines, narrow the
    /// bar, drop the share column, and finally shorten language names.
    fn fit(rows: &[Row], groups: &[Group], width: usize) -> Self {
        let longest = rows.iter().map(|r| r.name.chars().count() + 2);
        let labels = groups.iter().map(|g| g.label().chars().count());
        let name = longest.chain(labels).max().unwrap_or(0).clamp(8, 24);
        let order = [
            Column::Files,
            Column::Lines,
            Column::Blank,
            Column::Comments,
            Column::Code,
        ];
        let columns = order
            .iter()
            .map(|&column| {
                let widest = rows.iter().map(|r| text::number(column.value(r)).len());
                (column, widest.max().unwrap_or(0).max(column.header().len()))
            })
            .collect();
        let mut layout = Self {
            name,
            columns,
            bar: 10,
        };
        for drop in [Column::Blank, Column::Comments, Column::Lines] {
            if layout.width() <= width {
                return layout;
            }
            layout.columns.retain(|(column, _)| *column != drop);
        }
        while layout.width() > width && layout.bar > 5 {
            layout.bar -= 1;
        }
        let shrink = |layout: &mut Self, floor: usize| {
            let excess = layout.width().saturating_sub(width);
            layout.name = layout
                .name
                .saturating_sub(excess)
                .max(floor.min(layout.name));
        };
        shrink(&mut layout, 14);
        if layout.width() > width {
            layout.bar = 0;
        }
        shrink(&mut layout, 10);
        layout
    }
}

/// Writes the per-language table grouped by purpose, with subtotals and share bars.
pub(crate) fn render(bundle: &Bundle, args: &Args, out: &mut Buffer, style: &Style) {
    let mut languages: BTreeMap<(Group, &str), (usize, Counts)> = BTreeMap::new();
    for file in &bundle.current.files {
        let entry = languages
            .entry((Group::of(file), file.language.name()))
            .or_default();
        entry.0 += 1;
        entry.1.add(file.counts);
    }
    let mut groups: Vec<(Group, Row, Vec<Row>)> = Vec::new();
    for ((group, name), (files, counts)) in languages {
        if groups.last().is_none_or(|(last, _, _)| *last != group) {
            let subtotal = Row {
                name: group.label(),
                files: 0,
                counts: Counts::default(),
            };
            groups.push((group, subtotal, Vec::new()));
        }
        let (_, subtotal, rows) = groups.last_mut().expect("a group was just pushed");
        subtotal.files += files;
        subtotal.counts.add(counts);
        rows.push(Row {
            name,
            files,
            counts,
        });
    }
    for (_, _, rows) in &mut groups {
        sort(rows, &args.sort);
    }
    let total = Row {
        name: "Total",
        files: bundle.current.files.len(),
        counts: bundle.current.totals,
    };
    let every: Vec<Row> = groups
        .iter()
        .flat_map(|(_, subtotal, rows)| std::iter::once(subtotal).chain(rows))
        .chain(std::iter::once(&total))
        .copied()
        .collect();
    let kinds: Vec<Group> = groups.iter().map(|(group, _, _)| *group).collect();
    let layout = Layout::fit(&every, &kinds, style.width);
    let names: std::collections::BTreeSet<&str> = groups
        .iter()
        .flat_map(|(_, _, rows)| rows.iter().map(|r| r.name))
        .collect();
    let note = match names.len() {
        1 => "1 language".to_owned(),
        count => format!("{count} languages"),
    };
    style.heading(out, "LANGUAGES", &note);
    header(out, &layout, style);
    let whole = total.counts.code.max(1) as f64;
    // Bars are scaled to the largest language so that small differences stay visible;
    // the percentage beside each bar keeps the share of the whole.
    let largest = groups
        .iter()
        .flat_map(|(_, _, rows)| rows.iter().map(|r| r.counts.code))
        .max()
        .unwrap_or(0)
        .max(1) as f64;
    let scale = Scale { whole, largest };
    for (index, (_, subtotal, rows)) in groups.iter().enumerate() {
        if out.failed() {
            break;
        }
        if index > 0 {
            out.push('\n');
        }
        line(out, &layout, style, subtotal, Weight::Group, scale);
        for row in rows {
            line(out, &layout, style, row, Weight::Language, scale);
        }
    }
    let _ = writeln!(
        out,
        "  {}",
        style.dim(&style.line(layout.width().saturating_sub(2)))
    );
    line(out, &layout, style, &total, Weight::Total, scale);
}

fn sort(rows: &mut [Row], order: &str) {
    rows.sort_by(|a, b| {
        let primary = match order {
            "name" => a.name.cmp(b.name),
            "files" => b.files.cmp(&a.files),
            "lines" => b.counts.lines().cmp(&a.counts.lines()),
            "comments" => b.counts.comments.cmp(&a.counts.comments),
            "blanks" => b.counts.blank.cmp(&a.counts.blank),
            _ => b.counts.code.cmp(&a.counts.code),
        };
        primary.then_with(|| a.name.cmp(b.name))
    });
}

fn header(out: &mut Buffer, layout: &Layout, style: &Style) {
    let mut row = format!("{:<width$}", "Language", width = layout.name);
    for (column, width) in &layout.columns {
        let _ = write!(row, "  {:>width$}", column.header());
    }
    if layout.bar > 0 {
        let share = if layout.bar >= 8 {
            "Share of code"
        } else {
            "Share"
        };
        let cells = layout.bar + 1 + PERCENT;
        let _ = write!(row, "{:SHARE_GAP$}{share:<cells$}", "");
    }
    let _ = writeln!(out, "  {}", style.dim(row.trim_end()));
}

/// The denominators of the share column: the whole for percentages, the largest
/// language for bar lengths.
#[derive(Clone, Copy)]
struct Scale {
    whole: f64,
    largest: f64,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Weight {
    Group,
    Language,
    Total,
}

fn line(
    out: &mut Buffer,
    layout: &Layout,
    style: &Style,
    row: &Row,
    weight: Weight,
    scale: Scale,
) {
    let name = match weight {
        Weight::Language => format!(
            "  {}",
            text::truncate(row.name, layout.name - 2, style.unicode)
        ),
        _ => text::truncate(row.name, layout.name, style.unicode),
    };
    let padded = format!("{name:<width$}", width = layout.name);
    let mut cells = if weight == Weight::Language {
        padded
    } else {
        style.bold(&padded)
    };
    for &(column, width) in &layout.columns {
        let value = format!("{:>width$}", text::number(column.value(row)));
        let value = match (weight, column) {
            (Weight::Total, _) => style.bold(&value),
            (_, Column::Blank | Column::Comments) => style.dim(&value),
            _ => value,
        };
        let _ = write!(cells, "  {value}");
    }
    if layout.bar > 0 && weight != Weight::Total {
        let code = row.counts.code as f64;
        let fraction = code / scale.whole;
        let bar = if weight == Weight::Language {
            style.accent(&style.bar(code / scale.largest, layout.bar))
        } else {
            " ".repeat(layout.bar)
        };
        let share = format!("{:>PERCENT$}", style::percent(fraction));
        let share = if weight == Weight::Group {
            style.bold(&share)
        } else {
            style.dim(&share)
        };
        let _ = write!(cells, "{:SHARE_GAP$}{bar} {share}", "");
    }
    let _ = writeln!(out, "  {}", cells.trim_end());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fitted_tables_never_exceed_the_width_and_keep_code_and_files() {
        let counts = Counts {
            code: 1_234_567,
            comments: 12_345,
            blank: 9_999,
            bytes: 1,
        };
        let rows = [
            Row {
                name: "A language name longer than columns",
                files: 99_999,
                counts,
            },
            Row {
                name: "Rust",
                files: 1,
                counts,
            },
        ];
        for width in 44..=100 {
            let layout = Layout::fit(&rows, &[Group::Generated], width);
            assert!(layout.width() <= width, "{width}: {}", layout.width());
            let kept: Vec<_> = layout.columns.iter().map(|(c, _)| *c).collect();
            assert!(kept.contains(&Column::Code) && kept.contains(&Column::Files));
            let style = Style {
                color: true,
                unicode: true,
                width,
            };
            let mut out = Buffer::new(1 << 20);
            for (row, weight) in rows.iter().zip([Weight::Language, Weight::Group]) {
                let scale = Scale {
                    whole: 2_469_134.0,
                    largest: 1_234_567.0,
                };
                line(&mut out, &layout, &style, row, weight, scale);
            }
            for printed in out.lines() {
                assert!(text::visible_width(printed) <= width, "{printed:?}");
            }
        }
    }
}
