//! Bounded, deterministic lexical counting. This is not a language parser.
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
/// Returns a diagnostic for invalid encoding, binary data, or detected lexical failure.
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
    let text = std::str::from_utf8(source).map_err(|_| coverage("Input is not UTF-8"))?;
    if source.contains(&0) {
        return Err(coverage("Input contains binary NUL bytes"));
    }
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

fn coverage(message: &str) -> Error {
    Error::new(
        "lexical-coverage",
        message,
        "Inspect the skipped file; see the supported language policies in docs/counting.rst.",
    )
}
