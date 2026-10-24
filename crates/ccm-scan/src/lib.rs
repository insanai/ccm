//! Bounded, deterministic lexical counting. This is not a language parser.
mod assembly;
mod bytes;
mod catalog;
mod languages;
mod lexer;
mod markup;
mod mixed;
mod syntax;
mod typst;
mod work;
pub use languages::{CatalogId, FileKind, Language, LanguageInfo, languages};

use ccm_core::{Counts, Error};

/// The version of the line classification rules that every report states.
/// Version 4 counts every physically empty line as blank, and docstrings as comments.
pub const COUNT_POLICY: &str = "lexical-v4";

/// The error code of `measure` and `measure_source` for input that contains NUL bytes.
pub const BINARY_INPUT: &str = "binary-input";

/// Physical counts and an optional implementation-work driver.
#[derive(Clone, Copy, Debug, Default)]
pub struct SourceMeasurement {
    /// Disjoint physical line counts and input bytes.
    pub counts: Counts,
    /// Source-work-v1 tokens, excluding comments and collapsing recognized literals.
    /// Unavailable for non-program policies or when work collection is disabled.
    pub source_tokens: Option<u64>,
}

/// Counts UTF-8 source under the lexical policy without collecting cost drivers.
/// Returns a diagnostic for binary data or a detected lexical failure. Text that is not
/// UTF-8 is counted by its ASCII structure.
pub fn measure_source(source: &[u8], language: Language) -> Result<Counts, Error> {
    Ok(measure(source, language, false)?.counts)
}

/// Counts source and optionally collects versioned work tokens in the same lexical pass.
/// Work tokens are lexical quantities.
/// They do not measure semantic complexity, human time, or model billing tokens.
pub fn measure(
    source: &[u8],
    language: Language,
    collect_work: bool,
) -> Result<SourceMeasurement, Error> {
    if is_binary(source) {
        return Err(Error::new(
            BINARY_INPUT,
            "Input contains binary NUL bytes",
            "CCM counts text; select the file only if it is text.",
        ));
    }
    // Every delimiter we recognize is ASCII, so replacing invalid sequences in legacy
    // encodings such as Latin-1 leaves each line's class unchanged.
    let decoded;
    let text = match std::str::from_utf8(source) {
        Ok(text) => text,
        Err(_) => {
            decoded = String::from_utf8_lossy(source);
            &decoded
        }
    };
    let text = text.strip_prefix('\u{feff}').unwrap_or(text);
    let language = match language {
        Language::TypeScriptDeclaration => Language::TypeScript,
        Language::CargoLock => Language::Toml,
        Language::JsonLock => Language::Json,
        _ => language,
    };
    let (mut counts, source_tokens) = if matches!(language, Language::Catalog(_)) {
        (catalog::count(text, language)?, None)
    } else if matches!(language, Language::Vue | Language::Svelte) {
        (mixed::count(text)?, None)
    } else if language == Language::Assembly {
        (assembly::count(text)?, None)
    } else if language == Language::Typst {
        (typst::count(text)?, None)
    } else if matches!(
        language,
        Language::Html | Language::Xml | Language::Svg | Language::Markdown | Language::Jinja
    ) {
        (markup::count(text, language)?, None)
    } else {
        lexer::count(
            text,
            language,
            collect_work && language.kind() == FileKind::Source,
        )?
    };
    counts.bytes = source.len() as u64;
    Ok(SourceMeasurement {
        counts,
        source_tokens,
    })
}

/// Returns true when `source` contains a NUL byte, which no text file we count contains.
pub fn is_binary(source: &[u8]) -> bool {
    bytes::find(source, 0).is_some()
}

/// Returns true when a line holds only whitespace. ASCII lines, by far the most common,
/// avoid the Unicode whitespace table.
fn is_blank(line: &str) -> bool {
    match line.bytes().find(|b| !b.is_ascii_whitespace()) {
        None => true,
        Some(byte) if byte < 0x80 => false,
        Some(_) => line.trim().is_empty(),
    }
}

fn coverage(message: &str) -> Error {
    Error::new(
        "lexical-coverage",
        message,
        "Inspect the skipped file; see the supported language policies in docs/counting.rst.",
    )
}
