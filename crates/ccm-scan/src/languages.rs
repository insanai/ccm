use std::path::Path;

/// A registered lexical counting policy, independent of semantic analysis.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Language {
    /// Rust, including nested comments and raw strings.
    Rust,
    /// C and header files.
    C,
    /// C++, including delimited raw strings.
    Cpp,
    /// C# lexical source.
    CSharp,
    /// Java lexical source.
    Java,
    /// JavaScript lexical source.
    JavaScript,
    /// TypeScript lexical source.
    TypeScript,
    /// Go, including backtick strings.
    Go,
    /// Python, including triple-quoted strings.
    Python,
    /// Ruby lexical source.
    Ruby,
    /// POSIX shell and Bash, including heredocs.
    Shell,
    /// TOML configuration.
    Toml,
    /// JSON data, without comments.
    Json,
    /// JSON with comments.
    Jsonc,
    /// YAML configuration.
    Yaml,
    /// Markdown documentation.
    Markdown,
    /// reStructuredText documentation.
    ReStructuredText,
    /// Plain text, with nonblank lines counted as content.
    Text,
    /// HTML markup, including HTML comments.
    Html,
    /// XML markup, including XML comments.
    Xml,
    /// CSS with block comments.
    Css,
    /// SCSS with block and line comments.
    Scss,
    /// SQL with dash and block comments.
    Sql,
    /// Lua, including long strings and long comments.
    Lua,
    /// Perl lexical source.
    Perl,
    /// PHP lexical source.
    Php,
    /// R lexical source.
    R,
    /// Swift, including nested block comments.
    Swift,
    /// Kotlin, including nested block comments.
    Kotlin,
    /// Scala, including nested block comments.
    Scala,
    /// PowerShell, including block comments and here-strings.
    PowerShell,
    /// Docker build instructions.
    Dockerfile,
    /// Make build instructions.
    Makefile,
    /// CMake build instructions.
    CMake,
    /// Protocol Buffers definitions.
    Protobuf,
    /// GraphQL definitions.
    Graphql,
    /// Terraform and HCL configuration.
    Hcl,
    /// Nix expressions.
    Nix,
    /// Elixir, including triple-quoted strings.
    Elixir,
    /// Erlang source.
    Erlang,
    /// Haskell, including nested block comments.
    Haskell,
    /// OCaml, including nested block comments.
    OCaml,
    /// F#, including nested block comments.
    FSharp,
    /// Clojure source.
    Clojure,
    /// Scheme source.
    Scheme,
    /// Dart source.
    Dart,
    /// Zig source.
    Zig,
    /// Odin source.
    Odin,
    /// Julia, including nested block comments.
    Julia,
    /// Fortran free-form source.
    Fortran,
    /// INI and editor configuration.
    Ini,
    /// TeX source.
    TeX,
    /// Typst source.
    Typst,
    /// gettext translation catalogs.
    Gettext,
    /// Unified patches; content is counted without interpreting embedded languages.
    Diff,
    /// Vue single-file components.
    Vue,
    /// Svelte components with script and style blocks.
    Svelte,
    /// SVG XML markup.
    Svg,
    /// Graphviz DOT source.
    Dot,
    /// Jinja templates with HTML and template comments.
    Jinja,
    /// CSV data, counted as physical text lines.
    Csv,
    /// TypeScript declaration files.
    TypeScriptDeclaration,
    /// Cargo dependency lock data.
    CargoLock,
    /// JSON dependency lock data.
    JsonLock,
    /// Starlark, the Python dialect of Bazel `BUILD` and `.bzl` files.
    Starlark,
    /// Roff documents: troff and nroff papers and manual pages.
    Roff,
    /// Assembly in the GNU as, NASM, MASM, and armasm dialects, preprocessed or not.
    Assembly,
    /// A declarative policy in the bundled compatibility catalog.
    Catalog(CatalogId),
}

/// An opaque identifier for an immutable bundled language definition.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct CatalogId(pub(crate) u16);

/// The purpose of a file, used to keep documentation and data out of program-size totals.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FileKind {
    /// Program source and build instructions.
    Source,
    /// Prose, markup, and typesetting.
    Documentation,
    /// Data and configuration.
    Data,
}

impl FileKind {
    /// Returns a stable machine-readable purpose name.
    pub fn name(self) -> &'static str {
        match self {
            Self::Source => "source",
            Self::Documentation => "documentation",
            Self::Data => "data",
        }
    }
}

/// An immutable language registry entry.
#[derive(Clone, Copy, Debug)]
pub struct LanguageInfo {
    /// Language selected by this entry.
    pub language: Language,
    /// Stable display name.
    pub name: &'static str,
    /// Recognized filename extensions, without dots.
    pub extensions: &'static [&'static str],
    /// Exact filenames taking priority over extensions.
    pub filenames: &'static [&'static str],
    /// File purpose; this does not imply semantic validity.
    pub kind: FileKind,
}

/// Returns the versioned, immutable language registry.
pub fn languages() -> &'static [LanguageInfo] {
    static ALL: std::sync::LazyLock<Vec<LanguageInfo>> = std::sync::LazyLock::new(|| {
        let mut all = REGISTRY.to_vec();
        for record in crate::catalog::records() {
            if !all.iter().any(|info| info.name == record.info.name) {
                all.push(record.info);
            }
        }
        all
    });
    &ALL
}

/// Extensions that two formats share, the signature that selects the second format, and
/// that format's catalog name.
const SIGNATURES: [(&str, &[u8], &str); 1] = [("ccm", b"ccm-model", "CCM model")];

/// Returns true when the first line of `header` with content starts with `//`, `/*`,
/// `#`, or `@`, as Objective-C files do. MATLAB, Mercury, and Mathematica files, which
/// share the `.m` extension, start with `%`, `:-`, `(*`, or code, never with these.
fn opens_like_c(header: &[u8]) -> bool {
    let first = header
        .split(|&byte| byte == b'\n')
        .map(<[u8]>::trim_ascii)
        .find(|line| !line.is_empty())
        .unwrap_or_default();
    [b"//".as_slice(), b"/*", b"#", b"@"]
        .iter()
        .any(|opener| first.starts_with(opener))
}

/// Returns true for a manual section extension, `1` to `9`.
fn is_manual_section(extension: &str) -> bool {
    matches!(extension.as_bytes(), [b'1'..=b'9'])
}

/// Returns true when the first line of `header` with content is a roff request or comment:
/// a `.` or `'` control character followed by a letter or a comment `\"`, as in `.TH`,
/// `.\"`, or `'\"`. Prose and code never begin that way.
fn opens_like_roff(header: &[u8]) -> bool {
    let first = header
        .split(|&byte| byte == b'\n')
        .find(|line| !line.trim_ascii().is_empty())
        .unwrap_or_default();
    match first {
        [b'.' | b'\'', b'\\', b'"', ..] => true,
        [b'.' | b'\'', next, ..] => next.is_ascii_alphabetic(),
        _ => false,
    }
}

/// Returns true when `header` holds a Jinja statement or comment tag, `{%` or `{#`. Go
/// templates, which share `.tmpl`, write both as `{{...}}`.
fn has_jinja_tags(header: &[u8]) -> bool {
    header.windows(2).any(|pair| pair == b"{%" || pair == b"{#")
}

/// Recognizes variants of conventional notice files, such as `LICENSE-MIT` or
/// `COPYING_v2`. Variants with an extension, such as `LICENSE.md`, follow the extension.
fn is_notice_variant(name: &str) -> bool {
    ["LICENSE", "LICENCE", "COPYING", "NOTICE", "AUTHORS"]
        .iter()
        .any(|stem| {
            name.strip_prefix(stem)
                .is_some_and(|rest| rest.starts_with(['-', '_']) && !rest.contains('.'))
        })
}

const REGISTRY: [LanguageInfo; 67] = [
    LanguageInfo {
        language: Language::Rust,
        name: "Rust",
        extensions: &["rs"],
        filenames: &[],
        kind: FileKind::Source,
    },
    LanguageInfo {
        language: Language::C,
        name: "C",
        extensions: &["c", "h"],
        filenames: &[],
        kind: FileKind::Source,
    },
    LanguageInfo {
        language: Language::Cpp,
        name: "C++",
        extensions: &["cc", "cpp", "cxx", "c++", "hpp", "hh", "hxx"],
        filenames: &[],
        kind: FileKind::Source,
    },
    LanguageInfo {
        language: Language::CSharp,
        name: "C#",
        extensions: &["cs", "csx"],
        filenames: &[],
        kind: FileKind::Source,
    },
    LanguageInfo {
        language: Language::Java,
        name: "Java",
        extensions: &["java"],
        filenames: &[],
        kind: FileKind::Source,
    },
    LanguageInfo {
        language: Language::JavaScript,
        name: "JavaScript",
        extensions: &["js", "mjs", "cjs", "jsx"],
        filenames: &[],
        kind: FileKind::Source,
    },
    LanguageInfo {
        language: Language::TypeScript,
        name: "TypeScript",
        extensions: &["ts", "mts", "cts", "tsx"],
        filenames: &[],
        kind: FileKind::Source,
    },
    LanguageInfo {
        language: Language::Go,
        name: "Go",
        extensions: &["go"],
        filenames: &[],
        kind: FileKind::Source,
    },
    LanguageInfo {
        language: Language::Python,
        name: "Python",
        extensions: &["py", "pyi", "pyw"],
        filenames: &["SConstruct", "SConscript"],
        kind: FileKind::Source,
    },
    LanguageInfo {
        language: Language::Ruby,
        name: "Ruby",
        extensions: &["rb", "rake", "gemspec"],
        filenames: &["Rakefile", "Gemfile"],
        kind: FileKind::Source,
    },
    LanguageInfo {
        language: Language::Shell,
        name: "Shell",
        extensions: &["sh", "bash", "zsh", "ksh"],
        filenames: &["configure", ".bashrc", ".zshrc", ".profile"],
        kind: FileKind::Source,
    },
    LanguageInfo {
        language: Language::Toml,
        name: "TOML",
        extensions: &["toml"],
        filenames: &["uv.lock"],
        kind: FileKind::Data,
    },
    LanguageInfo {
        language: Language::Json,
        name: "JSON",
        extensions: &["json"],
        filenames: &["composer.lock"],
        kind: FileKind::Data,
    },
    LanguageInfo {
        language: Language::Jsonc,
        name: "JSONC",
        extensions: &["jsonc"],
        filenames: &[],
        kind: FileKind::Data,
    },
    LanguageInfo {
        language: Language::Yaml,
        name: "YAML",
        extensions: &["yaml", "yml"],
        filenames: &[],
        kind: FileKind::Data,
    },
    LanguageInfo {
        language: Language::Markdown,
        name: "Markdown",
        extensions: &["md", "markdown", "mdx"],
        filenames: &[],
        kind: FileKind::Documentation,
    },
    LanguageInfo {
        language: Language::ReStructuredText,
        name: "reStructuredText",
        extensions: &["rst"],
        filenames: &[],
        kind: FileKind::Documentation,
    },
    LanguageInfo {
        language: Language::Text,
        name: "Text",
        extensions: &["txt", "text"],
        filenames: &[
            "LICENSE", "LICENCE", "COPYING", "NOTICE", "AUTHORS", "README",
        ],
        kind: FileKind::Documentation,
    },
    LanguageInfo {
        language: Language::Html,
        name: "HTML",
        extensions: &["html", "htm"],
        filenames: &[],
        kind: FileKind::Documentation,
    },
    LanguageInfo {
        language: Language::Xml,
        name: "XML",
        extensions: &["xml", "xsd", "xsl", "xslt", "plist"],
        filenames: &[],
        kind: FileKind::Documentation,
    },
    LanguageInfo {
        language: Language::Css,
        name: "CSS",
        extensions: &["css"],
        filenames: &[],
        kind: FileKind::Source,
    },
    LanguageInfo {
        language: Language::Scss,
        name: "SCSS",
        extensions: &["scss", "less"],
        filenames: &[],
        kind: FileKind::Source,
    },
    LanguageInfo {
        language: Language::Sql,
        name: "SQL",
        extensions: &["sql"],
        filenames: &[],
        kind: FileKind::Source,
    },
    LanguageInfo {
        language: Language::Lua,
        name: "Lua",
        extensions: &["lua"],
        filenames: &[],
        kind: FileKind::Source,
    },
    LanguageInfo {
        language: Language::Perl,
        name: "Perl",
        extensions: &["pl", "pm", "t"],
        filenames: &[],
        kind: FileKind::Source,
    },
    LanguageInfo {
        language: Language::Php,
        name: "PHP",
        extensions: &["php", "phtml"],
        filenames: &[],
        kind: FileKind::Source,
    },
    LanguageInfo {
        language: Language::R,
        name: "R",
        extensions: &["r", "R", "Rscript"],
        filenames: &[],
        kind: FileKind::Source,
    },
    LanguageInfo {
        language: Language::Swift,
        name: "Swift",
        extensions: &["swift"],
        filenames: &[],
        kind: FileKind::Source,
    },
    LanguageInfo {
        language: Language::Kotlin,
        name: "Kotlin",
        extensions: &["kt", "kts"],
        filenames: &[],
        kind: FileKind::Source,
    },
    LanguageInfo {
        language: Language::Scala,
        name: "Scala",
        extensions: &["scala", "sc"],
        filenames: &[],
        kind: FileKind::Source,
    },
    LanguageInfo {
        language: Language::PowerShell,
        name: "PowerShell",
        extensions: &["ps1", "psm1", "psd1"],
        filenames: &[],
        kind: FileKind::Source,
    },
    LanguageInfo {
        language: Language::Dockerfile,
        name: "Dockerfile",
        extensions: &["dockerfile"],
        filenames: &["Dockerfile", "Containerfile"],
        kind: FileKind::Source,
    },
    LanguageInfo {
        language: Language::Makefile,
        name: "Makefile",
        extensions: &["mk", "mak"],
        filenames: &["Makefile", "makefile", "GNUmakefile"],
        kind: FileKind::Source,
    },
    LanguageInfo {
        language: Language::CMake,
        name: "CMake",
        extensions: &["cmake"],
        filenames: &["CMakeLists.txt"],
        kind: FileKind::Source,
    },
    LanguageInfo {
        language: Language::Protobuf,
        name: "Protocol Buffers",
        extensions: &["proto"],
        filenames: &[],
        kind: FileKind::Source,
    },
    LanguageInfo {
        language: Language::Graphql,
        name: "GraphQL",
        extensions: &["graphql", "gql"],
        filenames: &[],
        kind: FileKind::Source,
    },
    LanguageInfo {
        language: Language::Hcl,
        name: "HCL",
        extensions: &["tf", "tfvars", "hcl"],
        filenames: &[],
        kind: FileKind::Data,
    },
    LanguageInfo {
        language: Language::Nix,
        name: "Nix",
        extensions: &["nix"],
        filenames: &[],
        kind: FileKind::Source,
    },
    LanguageInfo {
        language: Language::Elixir,
        name: "Elixir",
        extensions: &["ex", "exs"],
        filenames: &[],
        kind: FileKind::Source,
    },
    LanguageInfo {
        language: Language::Erlang,
        name: "Erlang",
        extensions: &["erl", "hrl", "escript"],
        filenames: &[],
        kind: FileKind::Source,
    },
    LanguageInfo {
        language: Language::Haskell,
        name: "Haskell",
        extensions: &["hs", "lhs"],
        filenames: &[],
        kind: FileKind::Source,
    },
    LanguageInfo {
        language: Language::OCaml,
        name: "OCaml",
        extensions: &["ml", "mli"],
        filenames: &[],
        kind: FileKind::Source,
    },
    LanguageInfo {
        language: Language::FSharp,
        name: "F#",
        extensions: &["fs", "fsi", "fsx"],
        filenames: &[],
        kind: FileKind::Source,
    },
    LanguageInfo {
        language: Language::Clojure,
        name: "Clojure",
        extensions: &["clj", "cljs", "cljc", "edn"],
        filenames: &[],
        kind: FileKind::Source,
    },
    LanguageInfo {
        language: Language::Scheme,
        name: "Scheme",
        extensions: &["scm", "ss", "rkt", "lisp", "cl"],
        filenames: &[],
        kind: FileKind::Source,
    },
    LanguageInfo {
        language: Language::Dart,
        name: "Dart",
        extensions: &["dart"],
        filenames: &[],
        kind: FileKind::Source,
    },
    LanguageInfo {
        language: Language::Zig,
        name: "Zig",
        extensions: &["zig", "zon"],
        filenames: &[],
        kind: FileKind::Source,
    },
    LanguageInfo {
        language: Language::Odin,
        name: "Odin",
        extensions: &["odin"],
        filenames: &[],
        kind: FileKind::Source,
    },
    LanguageInfo {
        language: Language::Julia,
        name: "Julia",
        extensions: &["jl"],
        filenames: &[],
        kind: FileKind::Source,
    },
    LanguageInfo {
        language: Language::Fortran,
        name: "Fortran",
        extensions: &["f90", "f95", "f03", "f08"],
        filenames: &[],
        kind: FileKind::Source,
    },
    LanguageInfo {
        language: Language::Ini,
        name: "INI",
        extensions: &["ini", "cfg", "conf", "properties"],
        filenames: &[".editorconfig"],
        kind: FileKind::Data,
    },
    LanguageInfo {
        language: Language::TeX,
        name: "TeX",
        extensions: &["tex", "sty", "cls"],
        filenames: &[],
        kind: FileKind::Documentation,
    },
    LanguageInfo {
        language: Language::Typst,
        name: "Typst",
        extensions: &["typ"],
        filenames: &[],
        kind: FileKind::Documentation,
    },
    LanguageInfo {
        language: Language::Gettext,
        name: "gettext",
        extensions: &["po", "pot"],
        filenames: &[],
        kind: FileKind::Data,
    },
    LanguageInfo {
        language: Language::Diff,
        name: "Diff",
        extensions: &["diff", "patch"],
        filenames: &[],
        kind: FileKind::Documentation,
    },
    LanguageInfo {
        language: Language::Vue,
        name: "Vue",
        extensions: &["vue"],
        filenames: &[],
        kind: FileKind::Source,
    },
    LanguageInfo {
        language: Language::Svelte,
        name: "Svelte",
        extensions: &["svelte"],
        filenames: &[],
        kind: FileKind::Source,
    },
    LanguageInfo {
        language: Language::Svg,
        name: "SVG",
        extensions: &["svg"],
        filenames: &[],
        kind: FileKind::Documentation,
    },
    LanguageInfo {
        language: Language::Dot,
        name: "DOT",
        extensions: &["dot", "gv"],
        filenames: &[],
        kind: FileKind::Source,
    },
    LanguageInfo {
        language: Language::Jinja,
        name: "Jinja",
        extensions: &["j2", "jinja", "jinja2"],
        filenames: &[],
        kind: FileKind::Source,
    },
    LanguageInfo {
        language: Language::Csv,
        name: "CSV",
        extensions: &["csv", "tsv"],
        filenames: &[],
        kind: FileKind::Data,
    },
    LanguageInfo {
        language: Language::TypeScriptDeclaration,
        name: "TypeScript types",
        extensions: &["d.ts", "d.mts", "d.cts"],
        filenames: &[],
        kind: FileKind::Source,
    },
    LanguageInfo {
        language: Language::CargoLock,
        name: "Cargo lock",
        extensions: &[],
        filenames: &["Cargo.lock"],
        kind: FileKind::Data,
    },
    LanguageInfo {
        language: Language::JsonLock,
        name: "JSON lock",
        extensions: &[],
        filenames: &["package-lock.json", "npm-shrinkwrap.json", "bun.lock"],
        kind: FileKind::Data,
    },
    LanguageInfo {
        language: Language::Assembly,
        name: "Assembly",
        extensions: &["s", "asm", "nasm", "a51"],
        filenames: &[],
        kind: FileKind::Source,
    },
    LanguageInfo {
        language: Language::Starlark,
        name: "Starlark",
        extensions: &["bzl", "bazel", "star", "sky"],
        filenames: &[
            "BUILD",
            "BUILD.bazel",
            "WORKSPACE",
            "WORKSPACE.bazel",
            "WORKSPACE.bzlmod",
            "MODULE.bazel",
        ],
        kind: FileKind::Source,
    },
    LanguageInfo {
        language: Language::Roff,
        name: "Roff",
        extensions: &["roff", "man", "mdoc", "ms", "me", "tmac"],
        filenames: &[],
        kind: FileKind::Documentation,
    },
];

impl Language {
    /// Returns immutable policy metadata for this language.
    pub fn info(self) -> &'static LanguageInfo {
        match self {
            Self::Rust => &REGISTRY[0],
            Self::C => &REGISTRY[1],
            Self::Cpp => &REGISTRY[2],
            Self::CSharp => &REGISTRY[3],
            Self::Java => &REGISTRY[4],
            Self::JavaScript => &REGISTRY[5],
            Self::TypeScript => &REGISTRY[6],
            Self::Go => &REGISTRY[7],
            Self::Python => &REGISTRY[8],
            Self::Ruby => &REGISTRY[9],
            Self::Shell => &REGISTRY[10],
            Self::Toml => &REGISTRY[11],
            Self::Json => &REGISTRY[12],
            Self::Jsonc => &REGISTRY[13],
            Self::Yaml => &REGISTRY[14],
            Self::Markdown => &REGISTRY[15],
            Self::ReStructuredText => &REGISTRY[16],
            Self::Text => &REGISTRY[17],
            Self::Html => &REGISTRY[18],
            Self::Xml => &REGISTRY[19],
            Self::Css => &REGISTRY[20],
            Self::Scss => &REGISTRY[21],
            Self::Sql => &REGISTRY[22],
            Self::Lua => &REGISTRY[23],
            Self::Perl => &REGISTRY[24],
            Self::Php => &REGISTRY[25],
            Self::R => &REGISTRY[26],
            Self::Swift => &REGISTRY[27],
            Self::Kotlin => &REGISTRY[28],
            Self::Scala => &REGISTRY[29],
            Self::PowerShell => &REGISTRY[30],
            Self::Dockerfile => &REGISTRY[31],
            Self::Makefile => &REGISTRY[32],
            Self::CMake => &REGISTRY[33],
            Self::Protobuf => &REGISTRY[34],
            Self::Graphql => &REGISTRY[35],
            Self::Hcl => &REGISTRY[36],
            Self::Nix => &REGISTRY[37],
            Self::Elixir => &REGISTRY[38],
            Self::Erlang => &REGISTRY[39],
            Self::Haskell => &REGISTRY[40],
            Self::OCaml => &REGISTRY[41],
            Self::FSharp => &REGISTRY[42],
            Self::Clojure => &REGISTRY[43],
            Self::Scheme => &REGISTRY[44],
            Self::Dart => &REGISTRY[45],
            Self::Zig => &REGISTRY[46],
            Self::Odin => &REGISTRY[47],
            Self::Julia => &REGISTRY[48],
            Self::Fortran => &REGISTRY[49],
            Self::Ini => &REGISTRY[50],
            Self::TeX => &REGISTRY[51],
            Self::Typst => &REGISTRY[52],
            Self::Gettext => &REGISTRY[53],
            Self::Diff => &REGISTRY[54],
            Self::Vue => &REGISTRY[55],
            Self::Svelte => &REGISTRY[56],
            Self::Svg => &REGISTRY[57],
            Self::Dot => &REGISTRY[58],
            Self::Jinja => &REGISTRY[59],
            Self::Csv => &REGISTRY[60],
            Self::TypeScriptDeclaration => &REGISTRY[61],
            Self::CargoLock => &REGISTRY[62],
            Self::JsonLock => &REGISTRY[63],
            Self::Assembly => &REGISTRY[64],
            Self::Starlark => &REGISTRY[65],
            Self::Roff => &REGISTRY[66],
            Self::Catalog(id) => &crate::catalog::records()[usize::from(id.0)].info,
        }
    }
    /// Returns the stable display name.
    pub fn name(self) -> &'static str {
        self.info().name
    }
    /// Returns the file purpose under this policy.
    pub fn kind(self) -> FileKind {
        self.info().kind
    }
    /// Returns the versioned lexical policy tier, not a semantic coverage guarantee.
    pub fn policy(self) -> &'static str {
        if matches!(self, Self::Catalog(_)) {
            "declarative-v1"
        } else {
            "native-lexical-v5"
        }
    }
    /// Selects by filename, compound filename, or extension; unknown types remain unknown.
    pub fn from_path(path: &Path) -> Option<Self> {
        let name = path.file_name()?.to_str().unwrap_or("");
        if let Some(info) = REGISTRY.iter().find(|info| info.filenames.contains(&name)) {
            return Some(info.language);
        }
        if let Some(language) = Self::from_name_convention(name) {
            return Some(language);
        }
        if is_notice_variant(name) {
            return Some(Self::Text);
        }
        if [".d.ts", ".d.mts", ".d.cts"]
            .iter()
            .any(|suffix| name.ends_with(suffix))
        {
            return Some(Self::TypeScriptDeclaration);
        }
        if let Some(language) = crate::catalog::lookup(name, true) {
            return Some(language);
        }
        for (index, _) in name.match_indices('.') {
            let suffix = &name[index + 1..];
            if suffix.contains('.') {
                if let Some(language) = crate::catalog::lookup(suffix, false) {
                    return Some(language);
                }
            }
        }
        let extension = path.extension()?.to_str()?;
        REGISTRY
            .iter()
            .find(|info| {
                info.extensions
                    .iter()
                    .any(|ext| ext.eq_ignore_ascii_case(extension))
            })
            .map(|info| info.language)
            .or_else(|| crate::catalog::lookup(extension, false))
    }

    /// Selects by a naming convention that outranks the extension: `Dockerfile.dev` is a
    /// Dockerfile; `Makefile.inc`, `Makefile.build`, and `Makefile.asm` are Make, unless
    /// the suffix names the file's own language, as in `Makefile.PL` or `makefile.py`; and
    /// a Bazel overlay such as `zlib.BUILD`, in capitals, is Starlark; and `meson.build`
    /// is Meson.
    fn from_name_convention(name: &str) -> Option<Self> {
        if name.starts_with("Dockerfile.") || name.starts_with("Containerfile.") {
            return Some(Self::Dockerfile);
        }
        if name.ends_with(".BUILD") || name.ends_with(".BUILD.bazel") {
            return Some(Self::Starlark);
        }
        if matches!(name, "meson.build" | "meson_options.txt" | "meson.options") {
            return crate::catalog::by_name("Meson");
        }
        let (stem, suffix) = name.split_once('.')?;
        if !matches!(stem, "Makefile" | "makefile" | "GNUmakefile") {
            return None;
        }
        // The suffix usually names what the makefile builds, as in `Makefile.asm`; only
        // these name the file's own language.
        let other = matches!(
            suffix,
            "PL" | "pl" | "py" | "m4" | "rst" | "md" | "txt" | "toml" | "json" | "yaml" | "yml"
        );
        (!other).then_some(Self::Makefile)
    }

    /// Returns true when the file's first bytes may change the choice of `from_path`,
    /// because two formats share the file's extension.
    pub fn needs_header(path: &Path) -> bool {
        let extension = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        matches!(extension, "m" | "tmpl" | "t")
            || is_manual_section(extension)
            || SIGNATURES
                .iter()
                .any(|(ext, _, _)| ext.eq_ignore_ascii_case(extension))
    }

    /// Selects like `from_path`, but first looks for a format signature at the start of
    /// `header`, the file's first bytes. A `.ccm` file that begins with `ccm-model` is a
    /// CCM model, not C++; a `.m` file that opens like C is Objective-C, not MATLAB; and a
    /// `.tmpl` file with a Jinja `{%` or `{#` is Jinja, not a Go template. A `.t` file is a
    /// script for its shebang's interpreter, or Roff when it opens like roff, as FreeBSD's
    /// papers do, and Perl otherwise; a manual page `.1` to `.9` counts only as Roff.
    /// Single-digit extensions also name rotated logs, so they need the roff opening.
    pub fn from_path_and_header(path: &Path, header: &[u8]) -> Option<Self> {
        let extension = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        let signature = SIGNATURES.iter().find(|(ext, signature, _)| {
            ext.eq_ignore_ascii_case(extension) && header.starts_with(signature)
        });
        let chosen = match (signature, extension) {
            (Some((_, _, name)), _) => crate::catalog::by_name(name),
            (None, "m") if opens_like_c(header) => crate::catalog::by_name("Objective C"),
            (None, "tmpl") if has_jinja_tags(header) => Some(Self::Jinja),
            (None, "t") => Self::from_shebang(header)
                .or_else(|| opens_like_roff(header).then_some(Self::Roff)),
            (None, section) if is_manual_section(section) && opens_like_roff(header) => {
                Some(Self::Roff)
            }
            _ => None,
        };
        chosen.or_else(|| Self::from_path(path))
    }

    /// Detects a supported script interpreter from the first line only.
    pub fn from_shebang(source: &[u8]) -> Option<Self> {
        let line = source.split(|&b| b == b'\n').next()?;
        let line = std::str::from_utf8(line.strip_prefix(b"#!")?).ok()?;
        let words: Vec<_> = line.split_whitespace().collect();
        let interpreter = words
            .iter()
            .find(|word| {
                let name = word.rsplit('/').next().unwrap_or(word);
                name != "env" && !name.starts_with('-') && !name.contains('=')
            })?
            .rsplit('/')
            .next()?;
        if interpreter.starts_with("python") {
            Some(Self::Python)
        } else if matches!(interpreter, "sh" | "bash" | "zsh" | "ksh" | "dash") {
            Some(Self::Shell)
        } else if interpreter == "ruby" {
            Some(Self::Ruby)
        } else if interpreter == "perl" {
            Some(Self::Perl)
        } else if matches!(interpreter, "node" | "nodejs") {
            Some(Self::JavaScript)
        } else {
            None
        }
    }
}
