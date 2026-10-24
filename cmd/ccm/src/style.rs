//! Presentation primitives of the human terminal report: color, glyphs, headings, bars.
//!
//! The report draws with Unicode box and block glyphs on UTF-8 terminals and falls back to
//! ASCII for `--format plain` and non-UTF-8 locales. Color is a second cue only: every
//! colored element also differs by glyph, word, or position.
use crate::report::Buffer;
use crate::text;
use std::fmt::Write;

/// The width of the report on wider terminals: a calm column of classic terminal width is
/// easier to scan than lines that span a wide window. A table that needs more widens the
/// report as far as the terminal allows; see `Style::new`.
pub(crate) const MAX_WIDTH: usize = 80;

/// Terminal presentation choices shared by every section of the report.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Style {
    /// Whether ANSI color and weight sequences are emitted.
    pub color: bool,
    /// Whether box, block, and arrow glyphs are drawn instead of ASCII.
    pub unicode: bool,
    /// Visible columns available to each line, indentation included.
    pub width: usize,
}

impl Style {
    /// Returns the style for a terminal of `terminal` columns; `plain` forces ASCII. The
    /// report is `MAX_WIDTH` wide, or as wide as `needed` when a table needs more, but
    /// never wider than the terminal. Every section shares that width, so the section
    /// rules line up with the table.
    pub fn new(color: bool, plain: bool, terminal: usize, needed: usize) -> Self {
        Self {
            color: color && !plain,
            unicode: !plain && utf8_locale(),
            width: MAX_WIDTH.max(needed).min(terminal),
        }
    }

    /// Wraps `value` in the SGR sequence `code` when color is enabled.
    pub fn paint(&self, code: &str, value: &str) -> String {
        if self.color && !value.is_empty() {
            format!("\x1b[{code}m{value}\x1b[0m")
        } else {
            value.to_owned()
        }
    }

    /// Returns `value` unchanged; the neutral paint of list items.
    pub fn plain(&self, value: &str) -> String {
        value.to_owned()
    }

    /// Returns `value` in bold: headings and totals.
    pub fn bold(&self, value: &str) -> String {
        self.paint("1", value)
    }

    /// Returns `value` dimmed: secondary text.
    pub fn dim(&self, value: &str) -> String {
        self.paint("2", value)
    }

    /// Returns `value` in the single accent color: bars and the tool name.
    pub fn accent(&self, value: &str) -> String {
        self.paint("36", value)
    }

    /// Returns `value` in the warning color.
    pub fn warn(&self, value: &str) -> String {
        self.paint("33", value)
    }

    /// Returns `value` in the error color.
    pub fn error(&self, value: &str) -> String {
        self.paint("1;31", value)
    }

    /// Chooses the Unicode or the ASCII spelling of one glyph.
    pub fn glyph(&self, unicode: &'static str, ascii: &'static str) -> &'static str {
        if self.unicode { unicode } else { ascii }
    }

    /// Returns the separator between items of one summary line.
    pub fn dot(&self) -> &'static str {
        self.glyph(" \u{b7} ", " | ")
    }

    /// Returns the arrow of flow sketches.
    pub fn arrow(&self) -> &'static str {
        self.glyph(" \u{2500}\u{25b6} ", " -> ")
    }

    /// Returns a horizontal rule of `count` cells.
    pub fn line(&self, count: usize) -> String {
        self.glyph("\u{2500}", "-").repeat(count)
    }

    /// Writes a section heading: a bold title, a dim rule, and an optional dim note that
    /// ends at the layout width. A blank line separates it from the previous section.
    pub fn heading(&self, out: &mut Buffer, title: &str, note: &str) {
        let used = 2 + title.chars().count();
        let note_width = note.chars().count();
        out.push('\n');
        let _ = write!(out, "  {}", self.bold(title));
        if used + 2 + note_width + 4 <= self.width {
            let note_cells = if note.is_empty() { 0 } else { 1 + note_width };
            let fill = self.width - used - 1 - note_cells;
            let _ = write!(out, " {}", self.dim(&self.line(fill)));
            if !note.is_empty() {
                let _ = write!(out, " {}", self.dim(note));
            }
        } else if !note.is_empty() && used + 2 + note_width <= self.width {
            let _ = write!(out, "  {}", self.dim(note));
        }
        out.push('\n');
    }

    /// Returns a horizontal bar of `cells` cells filled to `fraction`, with eighth blocks
    /// on Unicode terminals and `#` otherwise. A nonzero fraction always shows a mark.
    pub fn bar(&self, fraction: f64, cells: usize) -> String {
        let fraction = fraction.clamp(0.0, 1.0);
        if !self.unicode {
            let filled = (fraction * cells as f64).round() as usize;
            let filled = if fraction > 0.0 { filled.max(1) } else { 0 };
            return format!("{:<cells$}", "#".repeat(filled));
        }
        const EIGHTHS: [&str; 8] = [
            "", "\u{258f}", "\u{258e}", "\u{258d}", "\u{258c}", "\u{258b}", "\u{258a}", "\u{2589}",
        ];
        let eighths = (fraction * (cells * 8) as f64).round() as usize;
        let eighths = if fraction > 0.0 { eighths.max(1) } else { 0 };
        let mut bar = "\u{2588}".repeat(eighths / 8);
        bar.push_str(EIGHTHS[eighths % 8]);
        let shown = eighths / 8 + usize::from(eighths % 8 > 0);
        bar.push_str(&" ".repeat(cells.saturating_sub(shown)));
        bar
    }

    /// Writes `body` as a list item: a marker, then text wrapped with a hanging indent so
    /// that continuation lines align with the first word. Long words such as paths are
    /// broken at the layout width.
    pub fn item(&self, out: &mut Buffer, marker: &str, body: &str, indent: usize) {
        self.item_painted(out, marker, body, indent, Self::plain);
    }

    /// Writes a list item like `item`, painting each wrapped line of `body` with `paint`.
    pub fn item_painted(
        &self,
        out: &mut Buffer,
        marker: &str,
        body: &str,
        indent: usize,
        paint: fn(&Self, &str) -> String,
    ) {
        let marker_width = text::visible_width(marker);
        let hang = indent + marker_width + 1;
        let available = self.width.saturating_sub(hang).max(12);
        for (index, line) in text::wrap_words(body, available).iter().enumerate() {
            if index == 0 {
                let _ = writeln!(out, "{:indent$}{marker} {}", "", paint(self, line));
            } else {
                let _ = writeln!(out, "{:hang$}{}", "", paint(self, line));
            }
        }
    }
}

/// Formats a share of a whole as a percentage of at most four cells: `46%`, `<1%`, `0%`.
pub(crate) fn percent(fraction: f64) -> String {
    if fraction <= 0.0 {
        "0%".into()
    } else if fraction < 0.005 {
        "<1%".into()
    } else {
        format!("{:.0}%", fraction * 100.0)
    }
}

/// Returns whether the locale promises UTF-8 output. The first nonempty variable of
/// `LC_ALL`, `LC_CTYPE`, and `LANG` decides, as in POSIX; no locale at all means UTF-8,
/// which is the default of every current terminal.
fn utf8_locale() -> bool {
    let locale = ["LC_ALL", "LC_CTYPE", "LANG"]
        .iter()
        .filter_map(|name| std::env::var(name).ok())
        .find(|value| !value.is_empty());
    locale.is_none_or(|value| {
        let value = value.to_ascii_lowercase();
        value.contains("utf-8") || value.contains("utf8")
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_wide_table_widens_the_report_only_as_far_as_the_terminal() {
        let width = |terminal, needed| Style::new(false, true, terminal, needed).width;
        assert_eq!(width(160, 60), MAX_WIDTH);
        assert_eq!(width(160, 96), 96);
        assert_eq!(width(90, 96), 90);
        assert_eq!(width(72, 60), 72);
    }

    fn style(unicode: bool) -> Style {
        Style {
            color: false,
            unicode,
            width: 80,
        }
    }

    #[test]
    fn bars_keep_their_width_and_show_nonzero_shares() {
        for unicode in [false, true] {
            for fraction in [0.0, 0.001, 0.37, 0.5, 1.0] {
                let bar = style(unicode).bar(fraction, 10);
                assert_eq!(bar.chars().count(), 10, "{bar:?}");
                assert_eq!(bar.trim().is_empty(), fraction == 0.0, "{bar:?}");
            }
        }
        assert_eq!(style(true).bar(0.5, 4), "\u{2588}\u{2588}  ");
        assert_eq!(style(false).bar(0.5, 4), "##  ");
    }

    #[test]
    fn percentages_never_round_a_present_share_to_zero() {
        assert_eq!(percent(0.0), "0%");
        assert_eq!(percent(0.001), "<1%");
        assert_eq!(percent(0.456), "46%");
        assert_eq!(percent(1.0), "100%");
    }

    #[test]
    fn headings_fill_the_layout_width_exactly() {
        let mut out = Buffer::new(1 << 16);
        style(true).heading(&mut out, "LANGUAGES", "185 files");
        let line = out.lines().nth(1).unwrap();
        assert_eq!(line.chars().count(), 80, "{line:?}");
        assert!(line.ends_with(" 185 files"));
    }

    #[test]
    fn items_hang_continuation_lines_under_the_first_word() {
        let mut out = Buffer::new(1 << 16);
        let narrow = Style {
            width: 30,
            ..style(false)
        };
        narrow.item(&mut out, "!", "one two three four five six seven eight", 2);
        let lines: Vec<_> = out.lines().collect();
        assert!(lines.len() > 1);
        assert!(lines[0].starts_with("  ! one"));
        assert!(lines[1].starts_with("    ") && !lines[1].starts_with("     "));
        assert!(lines.iter().all(|line| line.chars().count() <= 30));
    }
}
