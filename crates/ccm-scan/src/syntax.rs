//! Comment and literal syntax for the native lexer, one table per concern.
use crate::Language;

/// What the native lexer must recognize in one language.
#[derive(Clone, Copy)]
pub(super) struct Syntax {
    /// Tokens that start a comment running to the end of the line.
    pub lines: &'static [&'static [u8]],
    /// The block comment delimiters, if any.
    pub block: Option<(&'static [u8], &'static [u8])>,
    /// Block comments nest.
    pub nested: bool,
    /// `"` opens a string; TeX, for one, has none.
    pub double: bool,
    /// `'` opens a string or character literal.
    pub single: bool,
    /// `"""` and `'''` open strings that end only at the same triple quote.
    pub triple: bool,
    /// A backtick opens a string that may span lines.
    pub backtick: bool,
    /// `'` strings take no backslash escapes.
    pub literal_single: bool,
    /// Ordinary `'` and `"` literals end at the line end unless a backslash continues it.
    pub single_line_quotes: bool,
}

impl Default for Syntax {
    fn default() -> Self {
        Self {
            lines: &[],
            block: None,
            nested: false,
            double: true,
            single: true,
            triple: false,
            backtick: false,
            literal_single: false,
            single_line_quotes: false,
        }
    }
}

/// Returns the comment and literal syntax of a native policy.
pub(super) fn syntax(language: Language) -> Syntax {
    let (lines, block) = comments(language);
    let syntax = Syntax {
        lines,
        block,
        nested: nested_comments(language),
        ..Syntax::default()
    };
    literals(language, syntax)
}

type Comments = (
    &'static [&'static [u8]],
    Option<(&'static [u8], &'static [u8])>,
);

fn comments(language: Language) -> Comments {
    use Language::*;
    const C_BLOCK: Option<(&[u8], &[u8])> = Some((b"/*", b"*/"));
    match language {
        Rust | C | Cpp | CSharp | Java | JavaScript | TypeScript | Go | Swift | Kotlin | Scala
        | Protobuf | Dart | Odin | Typst | Scss | Jsonc => (&[b"//"], C_BLOCK),
        Zig => (&[b"//"], None),
        Php | Dot => (&[b"//", b"#"], C_BLOCK),
        Hcl | Nix => (&[b"#"], C_BLOCK),
        PowerShell => (&[b"#"], Some((b"<#", b"#>"))),
        Julia => (&[b"#"], Some((b"#=", b"=#"))),
        Python | Starlark | Ruby | Shell | Toml | Yaml | Perl | R | Dockerfile | Makefile
        | CMake | Graphql | Elixir | Gettext => (&[b"#"], None),
        Css => (&[], C_BLOCK),
        Sql => (&[b"--"], C_BLOCK),
        Lua => (&[b"--"], None),
        Haskell => (&[b"--"], Some((b"{-", b"-}"))),
        OCaml | FSharp => (&[], Some((b"(*", b"*)"))),
        Clojure => (&[b";"], None),
        Scheme => (&[b";"], Some((b"#|", b"|#"))),
        Erlang | TeX => (&[b"%"], None),
        // A roff comment is `.\"` or `'\"` on a line of its own, or `\"` and groff's `\#`
        // after text.
        Roff => (&[b".\\\"", b"'\\\"", b"\\\"", b"\\#"], None),
        Fortran => (&[b"!"], None),
        Ini => (&[b";", b"#"], None),
        Html | Xml | Markdown => (&[], Some((b"<!--", b"-->"))),
        Json
        | Catalog(_)
        | Assembly
        | Text
        | ReStructuredText
        | Diff
        | Vue
        | Svelte
        | Svg
        | Jinja
        | Csv
        | TypeScriptDeclaration
        | CargoLock
        | JsonLock => (&[], None),
    }
}

fn nested_comments(language: Language) -> bool {
    use Language::*;
    matches!(
        language,
        Rust | Swift | Kotlin | Scala | Dart | Odin | Julia | Haskell | OCaml | FSharp | Scheme
    )
}

fn literals(language: Language, syntax: Syntax) -> Syntax {
    use Language::*;
    Syntax {
        // TeX and Roff have no string literals. Make and Docker pass quotes to the shell as plain
        // characters: a `#` starts a Make comment even between quotes, and a Dockerfile
        // comment is a line that starts with `#`, so quotes never hide one.
        double: !matches!(language, TeX | Roff | Makefile | Dockerfile),
        // In Lisps `'` quotes a form; in markup and CMake it is an apostrophe.
        single: !matches!(
            language,
            Html | Xml
                | Markdown
                | Json
                | Typst
                | Clojure
                | Scheme
                | TeX
                | CMake
                | Makefile
                | Dockerfile
                | Roff
        ),
        triple: matches!(
            language,
            Java | Kotlin
                | Scala
                | Swift
                | Dart
                | CSharp
                | Python
                | Starlark
                | Toml
                | Elixir
                | Julia
                | Graphql
        ),
        backtick: matches!(language, Go | JavaScript | TypeScript | Odin),
        literal_single: matches!(language, Shell | Toml | Yaml | PowerShell),
        single_line_quotes: matches!(
            language,
            C | Cpp
                | CSharp
                | Java
                | JavaScript
                | TypeScript
                | Go
                | Python
                | Starlark
                | Kotlin
                | Scala
                | Swift
                | Dart
                | Zig
                | Odin
                | Lua
                | Toml
                | Css
                | Scss
                | Json
                | Jsonc
                | Protobuf
                | Graphql
                | Hcl
                | Ini
                | Fortran
        ),
        ..syntax
    }
}
