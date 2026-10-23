//! Escaping for the two contexts the HTML report writes untrusted text into: element text
//! and attribute values, and a JSON data island inside a `<script>` element.

use std::fmt::Write;

/// Returns `value` escaped for HTML text and quoted attribute values. Control characters and
/// bidirectional overrides become visible `\u{..}` escapes, so a path cannot hide or reorder
/// the text around it.
pub(super) fn text(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for c in value.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            c if is_hidden(c) => {
                let _ = write!(out, "\\u{{{:x}}}", c as u32);
            }
            _ => out.push(c),
        }
    }
    out
}

/// Appends `value` to `out` as a JSON string literal that is also safe inside a `<script>`
/// element: `<`, `>` and `&` are written as `\u` escapes, so the text can never close the
/// element or open a comment, and the line separators U+2028 and U+2029 are escaped too.
pub(super) fn json_string(out: &mut String, value: &str) {
    out.push('"');
    for c in value.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '<' | '>' | '&' | '\u{2028}' | '\u{2029}' => {
                let _ = write!(out, "\\u{:04x}", c as u32);
            }
            c if c.is_control() => {
                let _ = write!(out, "\\u{:04x}", c as u32);
            }
            _ => out.push(c),
        }
    }
    out.push('"');
}

/// Returns `value` as a script-safe JSON string literal; see `json_string`.
pub(super) fn json(value: &str) -> String {
    let mut out = String::with_capacity(value.len() + 2);
    json_string(&mut out, value);
    out
}

/// Appends JSON text produced elsewhere so that it is safe inside a `<script>` element.
/// In valid JSON these characters occur only inside string literals, where a `\u` escape
/// denotes the same character, so the data does not change.
pub(super) fn script_safe(out: &mut String, json: &str) {
    for c in json.chars() {
        match c {
            '<' | '>' | '&' | '\u{2028}' | '\u{2029}' => {
                let _ = write!(out, "\\u{:04x}", c as u32);
            }
            _ => out.push(c),
        }
    }
}

/// Writes a finite number as JSON, or `null` for a value JSON cannot represent.
pub(super) fn number(value: f64) -> String {
    if value.is_finite() {
        format!("{}", value + 0.0)
    } else {
        "null".into()
    }
}

fn is_hidden(c: char) -> bool {
    c.is_control() || matches!(c, '\u{202a}'..='\u{202e}' | '\u{2066}'..='\u{2069}')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn markup_and_script_terminators_are_neutralized() {
        assert_eq!(
            text("a</script><b>&'\""),
            "a&lt;/script&gt;&lt;b&gt;&amp;&#39;&quot;"
        );
        assert_eq!(text("x\u{202e}y\n"), "x\\u{202e}y\\u{a}");
        assert_eq!(json("</script><!--"), "\"\\u003c/script\\u003e\\u003c!--\"");
        let mut out = String::new();
        script_safe(&mut out, "{\"model\":\"</script>\"}");
        assert_eq!(out, "{\"model\":\"\\u003c/script\\u003e\"}");
        assert_eq!(number(f64::NAN), "null");
        assert_eq!(number(-0.0), "0");
        assert_eq!(number(1.5), "1.5");
    }
}
