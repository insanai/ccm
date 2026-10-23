//! Embedded script/style regions contribute to the same physical line partition.
use crate::{Language, coverage, lexer, markup};
use ccm_core::{Counts, Error};

struct Block {
    body: usize,
    tag: &'static [u8],
    language: Language,
}

pub(super) fn count(source: &str) -> Result<Counts, Error> {
    let lines = source.split_inclusive('\n').count();
    let mut classes = Vec::<u8>::new();
    classes
        .try_reserve(lines)
        .map_err(|_| coverage("Cannot reserve mixed-language line map"))?;
    classes.resize(lines, 0);
    let mut cursor = 0;
    let mut base = 0;
    while let Some(Block {
        body,
        tag,
        language,
    }) = opening(source, cursor)?
    {
        overlay(
            &source[cursor..body],
            Language::Html,
            &mut classes,
            &mut base,
        )?;
        let close = closing(source, body, tag)
            .ok_or_else(|| coverage("Unterminated embedded script or style block"))?;
        overlay(&source[body..close], language, &mut classes, &mut base)?;
        // Consume the close tag as markup; it must not be reconsidered as an opening tag.
        let end = tag_end(source.as_bytes(), close)?;
        overlay(&source[close..end], Language::Html, &mut classes, &mut base)?;
        cursor = end;
    }
    overlay(&source[cursor..], Language::Html, &mut classes, &mut base)?;
    let mut counts = Counts::default();
    for class in classes {
        if class & 1 != 0 {
            counts.code += 1;
        } else if class & 2 != 0 {
            counts.comments += 1;
        } else {
            counts.blank += 1;
        }
    }
    Ok(counts)
}

fn overlay(
    source: &str,
    language: Language,
    classes: &mut [u8],
    base: &mut usize,
) -> Result<(), Error> {
    let mut index = *base;
    let mut emit = |code: bool, comment: bool| {
        if let Some(class) = classes.get_mut(index) {
            *class |= u8::from(code) | (u8::from(comment) << 1);
        }
        index += 1;
    };
    if language == Language::Html {
        markup::classify(source, language, &mut emit)?;
    } else {
        lexer::classify(source, language, false, &mut emit)?;
    }
    *base += source.bytes().filter(|&b| b == b'\n').count();
    Ok(())
}

fn opening(source: &str, mut index: usize) -> Result<Option<Block>, Error> {
    let bytes = source.as_bytes();
    while index < bytes.len() {
        if bytes[index..].starts_with(b"<!--") {
            let length = bytes[index + 4..]
                .windows(3)
                .position(|w| w == b"-->")
                .ok_or_else(|| coverage("Unterminated component HTML comment"))?;
            index += length + 7;
            continue;
        }
        if bytes[index] != b'<' {
            index += 1;
            continue;
        }
        for (tag, default) in [
            (b"script".as_slice(), Language::JavaScript),
            (b"style", Language::Css),
        ] {
            let name_end = index + tag.len() + 1;
            if bytes
                .get(index + 1..name_end)
                .is_some_and(|name| name.eq_ignore_ascii_case(tag))
                && bytes
                    .get(name_end)
                    .is_some_and(|b| b.is_ascii_whitespace() || *b == b'>')
            {
                let end = tag_end(bytes, name_end)?;
                let attributes = &source[name_end..end - 1];
                let language = embedded_language(attributes, default)?;
                return Ok(Some(Block {
                    body: end,
                    tag,
                    language,
                }));
            }
        }
        // Skip a complete markup tag so quoted attribute examples cannot open a block.
        if bytes.get(index + 1).is_some_and(u8::is_ascii_alphabetic) {
            index = tag_end(bytes, index + 1)?;
        } else {
            index += 1;
        }
    }
    Ok(None)
}

fn tag_end(bytes: &[u8], mut index: usize) -> Result<usize, Error> {
    let mut quote = 0;
    while let Some(&byte) = bytes.get(index) {
        if quote != 0 {
            if byte == quote {
                quote = 0;
            }
        } else if matches!(byte, b'\'' | b'"') {
            quote = byte;
        } else if byte == b'>' {
            return Ok(index + 1);
        }
        index += 1;
    }
    Err(coverage("Unterminated component tag"))
}

fn closing(source: &str, start: usize, tag: &[u8]) -> Option<usize> {
    let bytes = source.as_bytes();
    bytes[start..]
        .windows(tag.len() + 2)
        .enumerate()
        .find_map(|(offset, window)| {
            let end = start + offset + window.len();
            (window.starts_with(b"</")
                && window[2..].eq_ignore_ascii_case(tag)
                && bytes
                    .get(end)
                    .is_some_and(|b| *b == b'>' || b.is_ascii_whitespace()))
            .then_some(start + offset)
        })
}

fn embedded_language(attributes: &str, default: Language) -> Result<Language, Error> {
    let mut words = attributes.split(|c: char| c.is_ascii_whitespace() || c == '=');
    while let Some(word) = words.next() {
        if word == "lang" {
            let value = words
                .find(|word| !word.is_empty())
                .unwrap_or("")
                .trim_matches(['\'', '"']);
            return match value {
                "ts" | "typescript" => Ok(Language::TypeScript),
                "js" | "javascript" => Ok(Language::JavaScript),
                "css" => Ok(Language::Css),
                "scss" => Ok(Language::Scss),
                _ => Err(coverage("Unsupported embedded language in component")),
            };
        }
    }
    Ok(default)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_languages_merge_shared_lines_and_preserve_literal_comments() {
        let source = concat!(
            "<!-- top comment -->\n<template><p>\" prose</p></template>\n",
            "<script setup lang=\"ts\">\n// script comment\n",
            "const url = 'https://example.com';\n</script><style>/* style */</style>\n\n"
        );
        let c = count(source).unwrap();
        assert_eq!((c.code, c.comments, c.blank), (4, 2, 1));
        assert!(count("<script lang=\"unknown\">x</script>").is_err());
        assert!(count("<script>unclosed").is_err());
        let c = count("<!-- <script> -->\n<p data-x=\"<script>\">ok</p>").unwrap();
        assert_eq!((c.code, c.comments), (1, 1));
    }
}
