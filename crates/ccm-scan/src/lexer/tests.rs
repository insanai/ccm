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
    // A docstring is documentation; an empty line inside any string is blank.
    let source = b"'''doc\n\nstring'''\nx = '''a\n\nb'''";
    let c = measure_source(source, Language::Python).unwrap();
    assert_eq!((c.code, c.comments, c.blank), (2, 2, 2));
    let source = b"def f():\n    \"\"\"Doc.\"\"\"\n    f(\"\"\"x\"\"\")";
    let c = measure_source(source, Language::Python).unwrap();
    assert_eq!((c.code, c.comments), (2, 1));
    // Odin raw strings use backticks, so an apostrophe inside one is text.
    let c = measure_source(b"s := `\nthe message's ID\n\n`\n", Language::Odin).unwrap();
    assert_eq!((c.code, c.comments, c.blank), (3, 0, 1));
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

#[test]
fn notice_variants_and_shared_extensions_resolve_by_name_and_signature() {
    use std::path::Path;
    for name in ["LICENSE-MIT", "LICENSE-APACHE", "COPYING_v2", "LICENCE"] {
        assert_eq!(Language::from_path(Path::new(name)), Some(Language::Text));
    }
    assert_eq!(
        Language::from_path(Path::new("LICENSE.md")),
        Some(Language::Markdown)
    );
    assert_eq!(Language::from_path(Path::new("LICENSES")), None);
    let path = Path::new("checkout.ccm");
    assert!(Language::needs_header(path));
    let model = Language::from_path_and_header(path, b"ccm-model\t2\n").unwrap();
    assert_eq!(model.name(), "CCM model");
    let module = Language::from_path_and_header(path, b"export module m;\n").unwrap();
    assert_eq!(module, Language::from_path(path).unwrap());
    assert!(!Language::needs_header(Path::new("main.rs")));
}

#[test]
fn yaml_hash_inside_a_plain_scalar_is_not_a_comment() {
    let c = measure_source(b"url: a#b\n# note\nkey: v # trailing\n", Language::Yaml).unwrap();
    assert_eq!((c.code, c.comments), (2, 1));
}

#[test]
fn stray_quotes_in_single_line_literal_languages_end_at_the_line() {
    let source = b"#error lacks clocks; isn't \\\nsupported\n/* real */\nint x;\n";
    let c = measure_source(source, Language::C).unwrap();
    assert_eq!((c.code, c.comments), (3, 1));
    // An identifier that ends in `R` before a string is not a raw string.
    let source = b"#define P LUA_LDIR\"?.lua;\" LUA_CDIR\"?\\\\init.lua;\"\n// note\n";
    let c = measure_source(source, Language::Cpp).unwrap();
    assert_eq!((c.code, c.comments), (1, 1));
    let c = measure_source(b"auto s = u8R\"x(/*)x\";\n// note\n", Language::Cpp).unwrap();
    assert_eq!((c.code, c.comments), (1, 1));
    // Rust strings may span lines, so the quote stays open.
    assert!(measure_source(b"let s = \"a\nb\";\n", Language::Rust).is_ok());
    assert!(measure_source(b"let s = \"a\n", Language::Rust).is_err());
}

#[test]
fn assembly_character_constants_do_not_open_a_string() {
    let language = crate::languages()
        .iter()
        .find(|entry| entry.name == "Assembly")
        .unwrap()
        .language;
    let c = measure_source(b"\tmovb $'a, %al\n; note\n/* block */\n", language).unwrap();
    assert_eq!((c.code, c.comments), (1, 2));
}

#[test]
fn legacy_encodings_count_by_ascii_structure_and_nul_bytes_stay_binary() {
    let c = measure_source(b"/* caf\xe9 */\nint x; // \xff\n", Language::C).unwrap();
    assert_eq!((c.code, c.comments, c.bytes), (1, 1, 23));
    assert!(measure_source(b"a\0b", Language::C).is_err());
}

#[test]
fn tex_has_no_strings_and_escaped_percent_is_text() {
    let source =
        b"title = {CRYPTO '87},\nauthor = {Mart{\\'i}n \"Q},\n50\\% done % note\n% only\n";
    let c = measure_source(source, Language::TeX).unwrap();
    assert_eq!((c.code, c.comments), (3, 1));
}

#[test]
fn perl_pod_quote_like_operators_patterns_and_heredocs() {
    let source = concat!(
        "my $n = $#list; # count\n",
        "$s =~ s/'/\\\\'/g;\n",
        "my @w = qw(it's done);\n",
        "if ($x =~ /\"#/) { print q{a 'b'}; }\n",
        "my $t = <<~\"EOF\";\n",
        "  it's # text\n",
        "  EOF\n",
        "\n",
        "=head1 NAME\n",
        "It's documentation.\n",
        "=cut\n",
        "my $r = $a / $b; # ratio\n",
        "__END__\n",
        "data with ' quote\n",
    );
    let c = measure_source(source.as_bytes(), Language::Perl).unwrap();
    assert_eq!((c.code, c.comments, c.blank), (10, 3, 1));
}

#[test]
fn ruby_begin_blocks_and_lisp_quotes() {
    let c = measure_source(b"=begin\nit's a note\n=end\nx = 1\n", Language::Ruby).unwrap();
    assert_eq!((c.code, c.comments), (1, 3));
    let source = b"(error 'wrong-type \"x\") ; note\n#| block\n|#\n(char #\\\")\n";
    let c = measure_source(source, Language::Scheme).unwrap();
    assert_eq!((c.code, c.comments), (2, 2));
    let c = measure_source(
        b"(def q 'sym) (str \\\" \"a\")\n; note\n",
        Language::Clojure,
    )
    .unwrap();
    assert_eq!((c.code, c.comments), (1, 1));
}

#[test]
fn heredoc_tags_inside_calls_and_quoted_with_a_backslash() {
    let source = b"f(<<EOF, 'x');\nit's\nEOF\n$p = $' . $\"; # note\n";
    let c = measure_source(source, Language::Perl).unwrap();
    assert_eq!((c.code, c.comments), (4, 0));
    let c = measure_source(b"cat <<\\EOF\nit's\nEOF\n# note\n", Language::Shell).unwrap();
    assert_eq!((c.code, c.comments), (3, 1));
    // A line-initial `/` that does not close is a division continued from above.
    let c = measure_source(b"my $x = $a\n  / $b; # note\n", Language::Perl).unwrap();
    assert_eq!((c.code, c.comments), (2, 0));
}

#[test]
fn cmake_brackets_and_apostrophes() {
    let source = b"#[[ block\ncomment ]]\nset(x [=[ it's ]=])\nmessage(\"don't\") # note\n";
    let c = measure_source(source, Language::CMake).unwrap();
    assert_eq!((c.code, c.comments), (2, 2));
}

#[test]
fn escaped_quotes_outside_strings_in_shell_and_cmake() {
    let c = measure_source(b"echo \\\"hi # note\n# only\n", Language::Shell).unwrap();
    assert_eq!((c.code, c.comments), (1, 1));
    let source = b"add(X=\\\"a.h\\\")\nmessage(\"b\") # note\n";
    let c = measure_source(source, Language::CMake).unwrap();
    assert_eq!((c.code, c.comments), (2, 0));
}

#[test]
fn blank_lines_include_unicode_whitespace() {
    assert!(crate::is_blank(" \t\r"));
    assert!(crate::is_blank("\u{a0}\u{3000}"));
    assert!(!crate::is_blank("  x"));
    assert!(!crate::is_blank(" \u{e9}"));
}
