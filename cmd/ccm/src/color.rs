use std::io::IsTerminal;

pub(crate) fn enabled(mode: &str, output_file: bool) -> bool {
    match mode {
        "always" => true,
        "never" => false,
        _ => automatic(
            std::io::stdout().is_terminal() && !output_file,
            std::env::var("TERM").ok().as_deref(),
            std::env::var("NO_COLOR").ok().as_deref(),
            std::env::var("CLICOLOR").ok().as_deref(),
        ),
    }
}

fn automatic(
    terminal: bool,
    term: Option<&str>,
    no_color: Option<&str>,
    cli_color: Option<&str>,
) -> bool {
    terminal
        && term != Some("dumb")
        && !no_color.is_some_and(|value| !value.is_empty())
        && cli_color != Some("0")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn terminal_hosts_do_not_need_term_and_empty_no_color_does_not_disable_color() {
        assert!(automatic(true, None, None, None));
        assert!(automatic(true, Some("xterm-256color"), Some(""), None));
        assert!(!automatic(false, Some("xterm"), None, None));
        assert!(!automatic(true, Some("dumb"), None, None));
        assert!(!automatic(true, None, Some("1"), None));
        assert!(!automatic(true, None, None, Some("0")));
    }
}
