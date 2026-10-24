use crate::{Language, coverage};
use ccm_core::{Counts, Error};

pub(super) fn count(source: &str, language: Language) -> Result<Counts, Error> {
    classify(source, language, |_, _| {})
}

pub(super) fn classify(
    source: &str,
    language: Language,
    mut emit: impl FnMut(bool, bool),
) -> Result<Counts, Error> {
    let mut counts = Counts::default();
    let mut comment: Option<&[u8]> = None;
    let mut cdata = false;
    let mut fence: Option<(u8, usize)> = None;
    for line in source.split_inclusive('\n') {
        let trimmed = line.trim();
        if language == Language::Markdown && comment.is_none() {
            if let Some((marker, length)) = fence {
                if trimmed.bytes().take_while(|&b| b == marker).count() >= length {
                    fence = None;
                }
                emit(!trimmed.is_empty(), false);
                if trimmed.is_empty() {
                    counts.blank += 1;
                } else {
                    counts.code += 1;
                }
                continue;
            }
            let first = trimmed.as_bytes().first().copied().unwrap_or(0);
            let length = trimmed.bytes().take_while(|&b| b == first).count();
            if matches!(first, b'`' | b'~') && length >= 3 {
                fence = Some((first, length));
                counts.code += 1;
                emit(true, false);
                continue;
            }
        }
        let (code, has_comment) = line_counts(line.as_bytes(), &mut comment, &mut cdata, language);
        // A physically empty line is blank even inside a comment or CDATA section.
        let (code, has_comment) = if trimmed.is_empty() {
            (false, false)
        } else {
            (code, has_comment)
        };
        emit(code, has_comment);
        if code {
            counts.code += 1;
        } else if has_comment {
            counts.comments += 1;
        } else {
            counts.blank += 1;
        }
    }
    if comment.is_some() || cdata {
        return Err(coverage("Unterminated markup comment or CDATA section"));
    }
    Ok(counts)
}

fn line_counts(
    line: &[u8],
    comment: &mut Option<&'static [u8]>,
    cdata: &mut bool,
    language: Language,
) -> (bool, bool) {
    let mut code = *cdata;
    let mut has_comment = comment.is_some();
    let mut index = 0;
    while index < line.len() {
        let rest = &line[index..];
        if let Some(close) = *comment {
            has_comment = true;
            if rest.starts_with(close) {
                *comment = None;
                index += close.len();
            } else {
                index += 1;
            }
        } else if *cdata {
            code = true;
            if rest.starts_with(b"]]>") {
                *cdata = false;
                index += 3;
            } else {
                index += 1;
            }
        } else if let Some(span) = code_span(rest).filter(|_| language == Language::Markdown) {
            code = true;
            index += span;
        } else if rest.starts_with(b"<!--") {
            *comment = Some(b"-->");
            has_comment = true;
            index += 4;
        } else if language == Language::Jinja && rest.starts_with(b"{#") {
            *comment = Some(b"#}");
            has_comment = true;
            index += 2;
        } else if rest.starts_with(b"<![CDATA[") {
            *cdata = true;
            code = true;
            index += 9;
        } else {
            code |= !rest[0].is_ascii_whitespace();
            index += 1;
        }
    }
    (code, has_comment)
}

/// Returns the length of the Markdown code span that opens `rest`, such as `` `<!--` ``:
/// a run of backticks, its text, and a run of the same length on the same line. Markup in
/// a code span is text, so `<!--` there opens no comment.
fn code_span(rest: &[u8]) -> Option<usize> {
    let run = rest.iter().take_while(|&&byte| byte == b'`').count();
    if run == 0 {
        return None;
    }
    let mut index = run;
    while index < rest.len() {
        let length = rest[index..]
            .iter()
            .take_while(|&&byte| byte == b'`')
            .count();
        if length == run {
            return Some(index + length);
        }
        index += length.max(1);
    }
    None
}

#[cfg(test)]
mod tests {
    use crate::{Language, measure_source};

    #[test]
    fn prose_quotes_xml_text_and_fenced_examples_do_not_start_source_strings() {
        for language in [Language::Markdown, Language::Xml, Language::Html] {
            let c =
                measure_source(b"<p>one unmatched \" quote</p>\n<!-- note -->", language).unwrap();
            assert_eq!((c.code, c.comments), (1, 1));
        }
        let c = measure_source(
            b"```html\n<!-- example -->\n```\n<!-- note -->",
            Language::Markdown,
        )
        .unwrap();
        assert_eq!((c.code, c.comments), (3, 1));
    }

    #[test]
    fn markdown_code_spans_hold_markup_as_text() {
        let source = b"Escapes `<!--` and ``<![CDATA[``.\nA lone ` tick\n<!-- note -->\n";
        let c = measure_source(source, Language::Markdown).unwrap();
        assert_eq!((c.code, c.comments), (2, 1));
    }
}
