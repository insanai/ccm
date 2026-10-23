//! Elm-style rendering of structured errors for standard error.
use crate::text;
use ccm_core::Error;

/// Width of the title rule, in columns.
const RULE_WIDTH: usize = 80;

/// Renders a title rule with the location, the message, an optional excerpt with a caret
/// under the reported column, and the hint. Untrusted text is escaped for terminals.
pub(crate) fn render(error: &Error) -> String {
    let title = error.code.to_ascii_uppercase();
    let Some(location) = &error.location else {
        return format!(
            "{}\n\n{}\n\nHint: {}\n",
            title_rule(&title, ""),
            text::terminal(&error.message),
            error.hint
        );
    };
    let place = format!(
        "{}:{}:{}",
        text::terminal(location.path.as_deref().unwrap_or("input")),
        location.line,
        location.column
    );
    let gutter = format!("{}| ", location.line);
    let excerpt = text::terminal(&location.excerpt.replace('\t', "    "));
    // The caret sits under the reported character after tab expansion and escaping.
    let before: String = location
        .excerpt
        .chars()
        .take(location.column.saturating_sub(1))
        .collect();
    let offset = gutter.chars().count()
        + text::terminal(&before.replace('\t', "    "))
            .chars()
            .count();
    format!(
        "{}\n\n{}\n\n{gutter}{excerpt}\n{}^\n\nHint: {}\n",
        title_rule(&title, &place),
        text::terminal(&error.message),
        " ".repeat(offset),
        error.hint
    )
}

fn title_rule(title: &str, place: &str) -> String {
    let used = title.chars().count() + place.chars().count() + 5;
    let dashes = "-".repeat(RULE_WIDTH.saturating_sub(used).max(3));
    if place.is_empty() {
        format!("-- {title} {dashes}")
    } else {
        format!("-- {title} {dashes} {place}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn caret_follows_the_gutter_width_and_reported_column() {
        let mut error =
            Error::new("model-input", "line 12: Bad", "Fix it.").at_source(12, 3, "ab\tcd");
        if let Some(location) = &mut error.location {
            location.path = Some("m.ccm".into());
        }
        let text = render(&error);
        let lines: Vec<_> = text.lines().collect();
        assert!(lines[0].starts_with("-- MODEL-INPUT --"));
        assert!(lines[0].ends_with(" m.ccm:12:3"));
        assert_eq!(lines[0].chars().count(), RULE_WIDTH);
        assert_eq!(lines[4], "12| ab    cd");
        assert_eq!(lines[5].find('^'), lines[4].find('\t').or(Some(6)));
        assert!(text.ends_with("Hint: Fix it.\n"));
    }

    #[test]
    fn errors_without_a_location_have_a_plain_title_rule() {
        let text = render(&Error::new(
            "arguments",
            "Unknown option: --x",
            "Run ccm --help.",
        ));
        assert!(text.starts_with("-- ARGUMENTS ---"));
        assert!(text.contains("\n\nUnknown option: --x\n\nHint: Run ccm --help.\n"));
    }
}
