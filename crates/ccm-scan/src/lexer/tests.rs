use crate::{Language, measure_source};

#[test]
fn rust_nested_comments_raw_strings_lifetimes_and_line_endings() {
    let text = concat!(
        "\u{feff}// first\r\n/* outer\n/* inner */\n",
        "*/let x = r##\"/*\n\"##;\nfn f<'a>() {}\n\n"
    )
    .as_bytes();
    let counts = measure_source(text, Language::Rust).unwrap();
    assert_eq!((counts.code, counts.comments, counts.blank), (3, 3, 1));
    assert_eq!(counts.lines(), 7);
}

#[test]
fn physical_lines_and_literals_partition_input() {
    assert_eq!(measure_source(b"", Language::Rust).unwrap().lines(), 0);
    assert_eq!(measure_source(b"\n", Language::Rust).unwrap().blank, 1);
    assert_eq!(measure_source(b"x", Language::Rust).unwrap().code, 1);
    assert_eq!(
        measure_source(b"x = '# hi' # c", Language::Python)
            .unwrap()
            .code,
        1
    );
    assert_eq!(
        measure_source(b"'''doc\n\nstring'''", Language::Python)
            .unwrap()
            .code,
        3
    );
    assert!(measure_source(b"/*", Language::Rust).is_err());
    assert_eq!(
        measure_source(b"R\"tag(/*\n*/)tag\"; // comment", Language::Cpp)
            .unwrap()
            .code,
        2
    );
}

#[test]
fn shell_heredocs_hash_in_words_and_zig_multiline_literals() {
    let source = b"# heading\ncat <<-'EOF'\n\t# data\n\tEOF\necho word#fragment\n";
    let counts = measure_source(source, Language::Shell).unwrap();
    assert_eq!((counts.code, counts.comments), (4, 1));
    let counts =
        measure_source(b"const s =\n    \\\\// data\n; // comment\n", Language::Zig).unwrap();
    assert_eq!(counts.code, 3);
}

#[test]
fn extended_delimiters_and_nested_comments() {
    for (language, text, expected) in [
        (
            Language::Lua,
            "--[=[ comment\n]=]\nx = [==[--data]==]",
            (1, 2),
        ),
        (Language::Go, "// comment\nvar s = `/*\n*/`", (2, 1)),
        (
            Language::Haskell,
            "{- outer\n{- inner -}\n-}\nx = 1",
            (1, 3),
        ),
        (
            Language::OCaml,
            "(* outer\n(* inner *)\n*)\nlet x = 1",
            (1, 3),
        ),
        (
            Language::Sql,
            "-- note\nselect '/* text */';\n/* comment */",
            (1, 2),
        ),
        (
            Language::Html,
            "<!-- comment\ncontinued -->\n<p>text</p>",
            (1, 2),
        ),
    ] {
        let c = measure_source(text.as_bytes(), language).unwrap();
        assert_eq!((c.code, c.comments), expected, "{}", language.name());
    }
}

#[test]
fn every_policy_has_deterministic_disjoint_line_counts() {
    for entry in crate::languages() {
        let source = b"x\n\n  \r\ny";
        let c = measure_source(source, entry.language).unwrap();
        assert_eq!(c.lines(), 4, "{}", entry.name);
        assert_eq!(c.bytes, source.len() as u64);
        assert_eq!(c, measure_source(source, entry.language).unwrap());
        let path = entry
            .filenames
            .first()
            .map(|s| (*s).to_owned())
            .or_else(|| entry.extensions.first().map(|s| format!("x.{s}")));
        if let Some(path) = path {
            assert!(
                Language::from_path(std::path::Path::new(&path)).is_some(),
                "{}",
                entry.name
            );
        }
    }
}

#[test]
fn every_registered_comment_profile_preserves_mixed_lines_and_quoted_delimiters() {
    for entry in crate::languages() {
        if matches!(
            entry.language,
            Language::Html | Language::Xml | Language::Markdown
        ) {
            continue;
        }
        let syntax = crate::syntax::syntax(entry.language);
        for token in syntax.lines {
            let token = std::str::from_utf8(token).unwrap();
            let source =
                format!("{token} comment\nx = 1; {token} trailing\nx = \"{token} literal\"\n");
            let c = measure_source(source.as_bytes(), entry.language).unwrap();
            assert_eq!((c.code, c.comments), (2, 1), "{}", entry.name);
        }
        if let Some((open, close)) = syntax.block {
            let source = format!(
                "{} comment\n{}\nx = 1\n",
                std::str::from_utf8(open).unwrap(),
                std::str::from_utf8(close).unwrap()
            );
            let c = measure_source(source.as_bytes(), entry.language).unwrap();
            assert_eq!((c.code, c.comments), (1, 2), "{}", entry.name);
        }
    }
}

#[test]
fn regex_character_classes_do_not_open_strings_or_comments() {
    let source = br#"const quote = /['"]/g; // comment
const url = /https?:\/\//;
// final comment
"#;
    let counts = measure_source(source, Language::JavaScript).unwrap();
    assert_eq!((counts.code, counts.comments), (2, 1));
}

#[test]
fn heredoc_tab_stripping_here_strings_and_arithmetic_shifts_are_distinct() {
    let source = b"cat <<-END\n END\n# literal data\nEND\nvalue=$((1 << 2))\ncat <<< 'data'\n";
    let counts = measure_source(source, Language::Shell).unwrap();
    assert_eq!((counts.code, counts.comments), (6, 0));
}

#[test]
fn source_work_tokens_ignore_comments_spacing_and_literal_payloads() {
    let a = crate::measure(b"let answer = 42;", Language::Rust, true).unwrap();
    let b = crate::measure(
        b"/* note */ let\nanswer=42; // note\n",
        Language::Rust,
        true,
    )
    .unwrap();
    assert_eq!(a.source_tokens, Some(5));
    assert_eq!(a.source_tokens, b.source_tokens);
    let a = crate::measure(b"let s = r#\"short\"#;", Language::Rust, true).unwrap();
    let b = crate::measure(
        b"let s = r#\"long\n// not a comment\"#;",
        Language::Rust,
        true,
    )
    .unwrap();
    assert_eq!(a.source_tokens, Some(5));
    assert_eq!(a.source_tokens, b.source_tokens);
    assert_eq!(
        crate::measure(b"x", Language::Rust, false)
            .unwrap()
            .source_tokens,
        None
    );
    assert_eq!(
        crate::measure(b"prose", Language::Text, true)
            .unwrap()
            .source_tokens,
        None
    );
}
