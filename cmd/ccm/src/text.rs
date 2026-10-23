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

pub(crate) fn money(amount: f64) -> String {
    let formatted = format!("{amount:.2}");
    let (digits, cents) = formatted.split_once('.').unwrap_or((&formatted, "00"));
    let mut out = String::from("$");
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

pub(crate) fn terminal_width() -> usize {
    std::env::var("COLUMNS")
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(80)
        .clamp(44, 160)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wrapping_preserves_colors_and_does_not_count_escape_bytes() {
        let source = "  \u{1b}[32mone two three four five six seven eight nine\u{1b}[0m\n";
        let wrapped = wrap_terminal(source, 24);
        assert!(wrapped.contains("\u{1b}[32m"));
        assert!(wrapped.contains("\u{1b}[0m"));
        assert!(wrapped.lines().count() > 1);
    }
}
