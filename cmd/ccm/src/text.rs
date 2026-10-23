use std::fmt::Write;

pub(crate) fn terminal(value: &str) -> String {
    let mut out = String::new();
    for c in value.chars() {
        if c.is_control() || matches!(c, '\u{202a}'..='\u{202e}' | '\u{2066}'..='\u{2069}') {
            let _ = write!(out, "\\u{{{:x}}}", c as u32);
        } else {
            out.push(c);
        }
    }
    out
}

pub(crate) fn html(value: &str) -> String {
    let mut out = String::new();
    for c in terminal(value).chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(c),
        }
    }
    out
}

pub(crate) fn json(value: &str) -> String {
    let mut out = String::from("\"");
    for c in value.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c.is_control() => {
                let _ = write!(out, "\\u{:04x}", c as u32);
            }
            _ => out.push(c),
        }
    }
    out.push('"');
    out
}

/// Formats USD with two decimals and thousands separators, such as `$1,234.50`.
/// A negative amount keeps its sign outside the digit groups; `-0.00` prints as `$0.00`.
pub(crate) fn money(amount: f64) -> String {
    // Adding +0.0 turns -0.0 into +0.0; rounding can still produce "-0.00" below.
    let formatted = format!("{:.2}", amount + 0.0);
    let (sign, unsigned) = match formatted.strip_prefix('-') {
        Some(rest) if rest.bytes().any(|b| b.is_ascii_digit() && b != b'0') => ("-", rest),
        Some(rest) => ("", rest),
        None => ("", formatted.as_str()),
    };
    let (digits, cents) = unsigned.split_once('.').unwrap_or((unsigned, "00"));
    let mut out = format!("{sign}$");
    for (index, digit) in digits.chars().enumerate() {
        if index > 0 && (digits.len() - index) % 3 == 0 {
            out.push(',');
        }
        out.push(digit);
    }
    out.push('.');
    out.push_str(cents);
    out
}

pub(crate) fn wrap_terminal(source: &str, width: usize) -> String {
    let mut out = String::with_capacity(source.len());
    for line in source.lines() {
        let mut remaining = line;
        let mut available = width;
        loop {
            let mut visible = 0;
            let mut escape = false;
            let mut last_space = None;
            let mut boundary = remaining.len();
            for (index, c) in remaining.char_indices() {
                if c == '\u{1b}' {
                    escape = true;
                    continue;
                }
                if escape {
                    if c == 'm' {
                        escape = false;
                    }
                    continue;
                }
                if visible >= available {
                    boundary = index;
                    break;
                }
                if c == ' ' && visible > 4 {
                    last_space = Some(index);
                }
                visible += 1;
            }
            if boundary == remaining.len() {
                out.push_str(remaining);
                out.push('\n');
                break;
            }
            let split = last_space.unwrap_or(boundary);
            out.push_str(&remaining[..split]);
            out.push_str("\n    ");
            remaining = remaining[split..].trim_start();
            available = width.saturating_sub(4).max(1);
        }
    }
    out
}

pub(crate) fn number(amount: u64) -> String {
    let digits = amount.to_string();
    let mut out = String::with_capacity(digits.len() + digits.len() / 3);
    for (index, digit) in digits.chars().enumerate() {
        if index > 0 && (digits.len() - index) % 3 == 0 {
            out.push(',');
        }
        out.push(digit);
    }
    out
}

/// Returns the terminal width in columns, clamped to 44..=160. `COLUMNS` wins; an
/// interactive terminal is asked with `stty size`; everything else gets 80 columns.
pub(crate) fn terminal_width() -> usize {
    std::env::var("COLUMNS")
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .or_else(tty_width)
        .unwrap_or(80)
        .clamp(44, 160)
}

/// Asks the controlling terminal for its width when standard output is that terminal.
/// We call the installed `stty` instead of a platform `ioctl`, which would need `unsafe`.
fn tty_width() -> Option<usize> {
    use std::io::IsTerminal;
    if !cfg!(unix) || !std::io::stdout().is_terminal() {
        return None;
    }
    let tty = std::fs::File::open("/dev/tty").ok()?;
    let output = std::process::Command::new("stty")
        .arg("size")
        .stdin(tty)
        .stderr(std::process::Stdio::null())
        .output()
        .ok()?;
    let size = String::from_utf8(output.stdout).ok()?;
    size.split_whitespace()
        .nth(1)?
        .parse()
        .ok()
        .filter(|&w| w > 0)
}

/// Returns the number of visible cells of `value`, skipping ANSI SGR sequences.
/// Every character counts as one cell; the report draws only narrow glyphs.
pub(crate) fn visible_width(value: &str) -> usize {
    let mut width = 0;
    let mut escape = false;
    for c in value.chars() {
        if c == '\u{1b}' {
            escape = true;
        } else if escape {
            escape = c != 'm';
        } else {
            width += 1;
        }
    }
    width
}

/// Shortens `value` to at most `width` characters, ending with an ellipsis when cut.
pub(crate) fn truncate(value: &str, width: usize, unicode: bool) -> String {
    if value.chars().count() <= width {
        return value.to_owned();
    }
    let ellipsis = if unicode { "\u{2026}" } else { "..." };
    let keep = width.saturating_sub(ellipsis.chars().count());
    let mut out: String = value.chars().take(keep).collect();
    out.push_str(ellipsis);
    out
}

/// Shortens a path to at most `width` characters by dropping leading characters, since
/// the file name at the end is what we recognize.
pub(crate) fn truncate_left(value: &str, width: usize, unicode: bool) -> String {
    let count = value.chars().count();
    if count <= width {
        return value.to_owned();
    }
    let ellipsis = if unicode { "\u{2026}" } else { "..." };
    let keep = width.saturating_sub(ellipsis.chars().count());
    let mut out = ellipsis.to_owned();
    out.extend(value.chars().skip(count - keep));
    out
}

/// Breaks plain text (no escape sequences) into lines of at most `width` characters at
/// spaces. A word longer than a line, such as a path, is split across lines.
pub(crate) fn wrap_words(value: &str, width: usize) -> Vec<String> {
    let width = width.max(1);
    let mut lines = Vec::new();
    let mut line = String::new();
    let mut used = 0;
    for word in value.split_whitespace() {
        let mut word: Vec<char> = word.chars().collect();
        if used > 0 && used + 1 + word.len() > width {
            lines.push(std::mem::take(&mut line));
            used = 0;
        }
        // Only an empty line remains here when the word is too long for one line.
        while word.len() > width {
            // Break a path after a separator when one leaves a reasonably full line.
            let split = word[..width]
                .iter()
                .rposition(|&c| c == '/' || c == '\\')
                .map(|index| index + 1)
                .filter(|&index| index > width / 3)
                .unwrap_or(width);
            lines.push(word.drain(..split).collect());
        }
        if !word.is_empty() {
            if used > 0 {
                line.push(' ');
                used += 1;
            }
            used += word.len();
            line.extend(word);
        }
    }
    if !line.is_empty() || lines.is_empty() {
        lines.push(line);
    }
    lines
}

/// Packs `parts` into lines of at most `width` visible cells, joined by `separator`,
/// breaking only between parts. Parts may carry ANSI styling.
pub(crate) fn pack(parts: &[String], separator: &str, width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut line = String::new();
    for part in parts {
        let used = visible_width(&line);
        if line.is_empty() {
            line.push_str(part);
        } else if used + visible_width(separator) + visible_width(part) > width {
            lines.push(std::mem::replace(&mut line, part.clone()));
        } else {
            line.push_str(separator);
            line.push_str(part);
        }
    }
    if !line.is_empty() {
        lines.push(line);
    }
    lines
}

/// Formats a byte count with decimal units, such as `198 B`, `12.4 KB`, or `3.98 MB`.
pub(crate) fn bytes(amount: u64) -> String {
    const UNITS: [&str; 5] = ["KB", "MB", "GB", "TB", "PB"];
    if amount < 1000 {
        return format!("{amount} B");
    }
    let mut value = amount as f64 / 1000.0;
    let mut unit = 0;
    while value >= 999.5 && unit + 1 < UNITS.len() {
        value /= 1000.0;
        unit += 1;
    }
    let decimals = if value < 10.0 {
        2
    } else if value < 100.0 {
        1
    } else {
        0
    };
    format!("{value:.decimals$} {}", UNITS[unit])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn money_groups_digits_without_misplacing_signs() {
        assert_eq!(money(1234567.891), "$1,234,567.89");
        assert_eq!(money(123.0), "$123.00");
        assert_eq!(money(-123.0), "-$123.00");
        assert_eq!(money(-1234.5), "-$1,234.50");
        assert_eq!(money(-0.0), "$0.00");
        assert_eq!(money(-0.001), "$0.00");
    }

    #[test]
    fn wrapping_preserves_colors_and_does_not_count_escape_bytes() {
        let source = "  \u{1b}[32mone two three four five six seven eight nine\u{1b}[0m\n";
        let wrapped = wrap_terminal(source, 24);
        assert!(wrapped.contains("\u{1b}[32m"));
        assert!(wrapped.contains("\u{1b}[0m"));
        assert!(wrapped.lines().count() > 1);
    }

    #[test]
    fn byte_counts_use_decimal_units_with_three_significant_digits() {
        assert_eq!(bytes(198), "198 B");
        assert_eq!(bytes(12_400), "12.4 KB");
        assert_eq!(bytes(3_977_736), "3.98 MB");
        assert_eq!(bytes(999_999), "1.00 MB");
    }

    #[test]
    fn word_wrapping_respects_the_width_and_splits_long_paths() {
        let lines = wrap_words("see a/very/long/path/name.rs for details", 10);
        assert!(
            lines.iter().all(|line| line.chars().count() <= 10),
            "{lines:?}"
        );
        assert_eq!(
            lines.concat().replace(' ', ""),
            "seea/very/long/path/name.rsfordetails"
        );
        assert_eq!(truncate("reStructuredText", 8, false), "reStr...");
        assert_eq!(
            truncate_left("src/deep/main.rs", 10, true),
            "\u{2026}p/main.rs"
        );
        assert_eq!(visible_width("\u{1b}[1mab\u{1b}[0m"), 2);
    }
}
