//! Typst markup keeps prose quotes and raw examples out of the code-string lexer.
use crate::coverage;
use ccm_core::{Counts, Error};

#[derive(Default)]
struct Scanner {
    block: usize,
    raw: usize,
    quoted: bool,
    code: bool,
    statement: bool,
    // Delimiters preserve the mode to return to after code or content blocks.
    frames: Vec<(u8, bool)>,
}

pub(super) fn count(source: &str) -> Result<Counts, Error> {
    let mut scanner = Scanner::default();
    let mut counts = Counts::default();
    for line in source.split_inclusive('\n') {
        let (code, comment) = scanner.line(line.as_bytes())?;
        if line.trim().is_empty() {
            counts.blank += 1;
        } else if code {
            counts.code += 1;
        } else if comment {
            counts.comments += 1;
        } else {
            counts.blank += 1;
        }
        if !scanner.quoted {
            scanner.code = scanner
                .frames
                .last()
                .is_some_and(|(close, _)| *close != b']');
            scanner.statement = false;
        }
    }
    if scanner.block > 0 || scanner.raw > 0 || scanner.quoted {
        return Err(coverage(
            "Unterminated Typst raw text, code string, or block comment",
        ));
    }
    Ok(counts)
}

impl Scanner {
    fn line(&mut self, line: &[u8]) -> Result<(bool, bool), Error> {
        let mut index = 0;
        let mut code = self.raw > 0 || self.quoted;
        let mut comment = self.block > 0;
        while index < line.len() {
            let rest = &line[index..];
            if self.block > 0 {
                comment = true;
                index += self.comment(rest)?;
            } else if self.raw > 0 {
                code = true;
                let ticks = rest.iter().take_while(|&&b| b == b'`').count();
                if ticks == self.raw {
                    self.raw = 0;
                }
                index += ticks.max(1);
            } else if self.quoted {
                code = true;
                if rest[0] == b'"' {
                    self.quoted = false;
                }
                index += if rest[0] == b'\\' {
                    rest.len().min(2)
                } else {
                    1
                };
            } else if rest.starts_with(b"//") {
                comment = true;
                break;
            } else if rest.starts_with(b"/*") {
                self.block = 1;
                comment = true;
                index += 2;
            } else {
                code |= !rest[0].is_ascii_whitespace();
                index += self.normal(rest)?;
            }
        }
        Ok((code, comment))
    }

    fn comment(&mut self, rest: &[u8]) -> Result<usize, Error> {
        if rest.starts_with(b"*/") {
            self.block -= 1;
            Ok(2)
        } else if rest.starts_with(b"/*") {
            self.block += 1;
            if self.block > 256 {
                return Err(coverage("Typst comment nesting exceeds 256"));
            }
            Ok(2)
        } else {
            Ok(1)
        }
    }

    fn normal(&mut self, rest: &[u8]) -> Result<usize, Error> {
        match rest[0] {
            b'\\' => return Ok(rest.len().min(2)),
            b'`' => {
                let ticks = rest.iter().take_while(|&&b| b == b'`').count();
                // Two adjacent backticks are an empty raw span, not an open fence.
                if ticks != 2 {
                    self.raw = ticks;
                }
                return Ok(ticks);
            }
            b'#' => {
                self.code = true;
                self.statement = [
                    b"let".as_slice(),
                    b"set",
                    b"show",
                    b"import",
                    b"include",
                    b"if",
                    b"for",
                    b"while",
                    b"context",
                ]
                .iter()
                .any(|word| {
                    rest[1..].starts_with(word)
                        && rest
                            .get(word.len() + 1)
                            .is_none_or(|b| !b.is_ascii_alphanumeric() && *b != b'-')
                });
            }
            b'"' if self.code => self.quoted = true,
            b'(' | b'{' if self.code => self.push(rest[0], true)?,
            b'[' => {
                self.push(b'[', false)?;
                self.code = false;
            }
            b')' | b'}' | b']'
                if self
                    .frames
                    .last()
                    .is_some_and(|(close, _)| *close == rest[0]) =>
            {
                if let Some((_, code)) = self.frames.pop() {
                    self.code = code;
                    if !self.statement
                        && self.frames.last().is_none_or(|(close, _)| *close == b']')
                    {
                        self.code = false;
                    }
                }
            }
            b if b.is_ascii_whitespace()
                && !self.statement
                && self.frames.last().is_none_or(|(close, _)| *close == b']') =>
            {
                self.code = false
            }
            _ => {}
        }
        Ok(1)
    }

    fn push(&mut self, open: u8, code: bool) -> Result<(), Error> {
        if self.frames.len() >= 256 {
            return Err(coverage("Typst delimiter nesting exceeds 256"));
        }
        self.frames.push((
            match open {
                b'(' => b')',
                b'{' => b'}',
                _ => b']',
            },
            self.code,
        ));
        self.code = code;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prose_raw_examples_and_nested_comments_preserve_line_classes() {
        let source = concat!(
            "A lone \" prose quote.\n",
            "`views/*.vue` and `https://example.com`\n",
            "```rs\n/* unfinished example\nlet x = \";\n```\n",
            "/* outer\n/* inner */\nend */\n",
            "#let url = \"https://example.com/*literal*/\"\n",
            "#text[An unmatched \" quote.]\n\n"
        );
        let c = count(source).unwrap();
        assert_eq!((c.code, c.comments, c.blank), (8, 3, 1));
        assert!(count("```rs\nunclosed").is_err());
        assert!(count("/* unclosed").is_err());
        assert!(count("#box[\n#text(\"hi\")\nA lone \" prose quote.\n]").is_ok());
        assert!(count("#text(\"hi\") A lone \" prose quote.").is_ok());
    }
}
