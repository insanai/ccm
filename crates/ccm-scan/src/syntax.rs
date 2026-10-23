use crate::Language;

#[derive(Clone, Copy)]
pub(super) struct Syntax {
    pub lines: &'static [&'static [u8]],
    pub block: Option<(&'static [u8], &'static [u8])>,
    pub nested: bool,
    pub single: bool,
    pub triple: bool,
    pub backtick: bool,
    pub literal_single: bool,
}

impl Default for Syntax {
    fn default() -> Self {
        Self {
            lines: &[],
            block: None,
            nested: false,
            single: true,
            triple: false,
            backtick: false,
            literal_single: false,
        }
    }
}

pub(super) fn syntax(language: Language) -> Syntax {
    use Language::*;
    let mut syntax = Syntax::default();
    match language {
        Catalog(_) => {}
        Rust | C | Cpp | CSharp | Java | JavaScript | TypeScript | Go | Swift | Kotlin | Scala
        | Protobuf | Dart | Odin | Typst | Scss | Jsonc => {
            syntax.lines = &[b"//"];
            syntax.block = Some((b"/*", b"*/"));
            syntax.nested = matches!(language, Rust | Swift | Kotlin | Scala | Dart | Odin);
            syntax.triple = matches!(language, Java | Kotlin | Scala | Swift | Dart | CSharp);
            syntax.backtick = matches!(language, Go | JavaScript | TypeScript);
        }
        Python | Ruby | Shell | Toml | Yaml | Perl | R | PowerShell | Dockerfile | Makefile
        | CMake | Graphql | Hcl | Nix | Elixir | Julia | Gettext => syntax.lines = &[b"#"],
        Css => syntax.block = Some((b"/*", b"*/")),
        Sql => {
            syntax.lines = &[b"--"];
            syntax.block = Some((b"/*", b"*/"));
        }
        Lua => syntax.lines = &[b"--"],
        Html | Xml | Markdown => {
            syntax.block = Some((b"<!--", b"-->"));
            syntax.single = false;
        }
        Haskell => {
            syntax.lines = &[b"--"];
            syntax.block = Some((b"{-", b"-}"));
        }
        OCaml | FSharp => {
            syntax.block = Some((b"(*", b"*)"));
        }
        Clojure | Scheme => syntax.lines = &[b";"],
        Erlang | TeX => syntax.lines = &[b"%"],
        Fortran => syntax.lines = &[b"!"],
        Ini => syntax.lines = &[b";", b"#"],
        Zig => syntax.lines = &[b"//"],
        Php => {
            syntax.lines = &[b"//", b"#"];
            syntax.block = Some((b"/*", b"*/"));
        }
        Json => syntax.single = false,
        Dot => {
            syntax.lines = &[b"//", b"#"];
            syntax.block = Some((b"/*", b"*/"));
        }
        Text
        | ReStructuredText
        | Diff
        | Vue
        | Svelte
        | Svg
        | Jinja
        | Csv
        | TypeScriptDeclaration
        | CargoLock
        | JsonLock => {}
    }
    if matches!(language, Hcl | Nix) {
        syntax.block = Some((b"/*", b"*/"));
    }
    if language == PowerShell {
        syntax.block = Some((b"<#", b"#>"));
    }
    if language == Julia {
        syntax.block = Some((b"#=", b"=#"));
    }
    if matches!(language, Python | Toml | Elixir | Julia | Graphql) {
        syntax.triple = true;
    }
    if matches!(language, Julia | Haskell | OCaml | FSharp) {
        syntax.nested = true;
    }
    if language == Typst {
        syntax.single = false;
    }
    syntax.literal_single = matches!(language, Shell | Toml | Yaml | PowerShell);
    syntax
}
