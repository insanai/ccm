//! Typst counting with a stack of syntactic modes.
//!
//! Typst nests three modes: markup, code, and math. A `"` opens a string only in code and
//! math, and `//` opens a comment everywhere except in strings, raw text, and links. We
//! track the mode stack the way the Typst lexer does. We parse embedded code only as far
//! as we must to know where it ends; we never evaluate or fully parse an expression.
use crate::coverage;
use ccm_core::{Counts, Error};

/// The deepest delimiter or comment nesting we accept before reporting a coverage error.
const MAX_DEPTH: usize = 256;

/// Keywords whose embedded statement runs to a newline or `;` outside any delimiter.
const STATEMENTS: [&[u8]; 5] = [b"let", b"set", b"show", b"import", b"include"];

/// Keywords whose embedded expression ends after a body block not followed by `else`.
const CONTROLS: [&[u8]; 3] = [b"if", b"for", b"while"];

/// A syntactic region that is still open.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Frame {
    /// A content block `[...]`, read in markup mode.
    Content,
    /// A code block `{...}` or group `(...)`, with its closing byte.
    Code(u8),
    /// An equation `$...$`, read in math mode.
    Math,
    /// An expression that `#` embeds in markup or math.
    Embed(Embed),
}

/// How far an embedded expression has progressed, which decides where it ends.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Embed {
    /// Just after `#` or `context`: the next token decides the kind.
    Start,
    /// `#let`, `#set`, and similar: code up to a newline or `;`.
    Statement,
    /// `#if`, `#for`, `#while`: code up to the end of a body block.
    Control,
    /// After a control body: only `else` continues the expression.
    AfterBody,
    /// Identifiers, fields, calls, and content arguments without spaces between them.
    Chain,
}

/// A token that may span lines and suspends the mode stack while it is open.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum Lexeme {
    #[default]
    None,
    /// A string in code or math.
    String,
    /// Raw text opened by `ticks` backticks; `run` counts the current run of backticks.
    Raw { ticks: usize, run: usize },
    /// A block comment and its nesting depth.
    Comment(usize),
}

/// What a step consumed, for the line classification.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Class {
    Content,
    Comment,
}

/// The result of one step: bytes consumed and their class. Zero bytes means that the
/// step closed a frame and the same byte must be read again in the enclosing mode.
type Step = (usize, Class);

#[derive(Default)]
struct Scanner {
    frames: Vec<Frame>,
    lexeme: Lexeme,
}

/// Counts Typst lines. Fails only on an unterminated string, raw text, or block comment,
/// or on nesting deeper than `MAX_DEPTH`.
pub(super) fn count(source: &str) -> Result<Counts, Error> {
    let mut scanner = Scanner::default();
    let mut counts = Counts::default();
    for line in source.split_inclusive('\n') {
        let (content, comment) = scanner.line(line.as_bytes())?;
        if content {
            counts.code += 1;
        } else if comment {
            counts.comments += 1;
        } else {
            counts.blank += 1;
        }
    }
    match scanner.lexeme {
        Lexeme::None => Ok(counts),
        Lexeme::String => Err(coverage("Unterminated Typst string")),
        Lexeme::Raw { .. } => Err(coverage("Unterminated Typst raw text")),
        Lexeme::Comment(_) => Err(coverage("Unterminated Typst block comment")),
    }
}

impl Scanner {
    /// Reads one physical line, including its newline, and reports whether it holds
    /// visible content and whether it holds a comment.
    fn line(&mut self, line: &[u8]) -> Result<(bool, bool), Error> {
        let (mut content, mut comment) = (false, false);
        let mut index = 0;
        while index < line.len() {
            let previous = index.checked_sub(1).map(|i| line[i]);
            let (length, class) = self.step(&line[index..], previous)?;
            let consumed = &line[index..index + length];
            match class {
                Class::Content => content |= consumed.iter().any(|b| !b.is_ascii_whitespace()),
                Class::Comment => comment = true,
            }
            index += length;
        }
        Ok((content, comment))
    }

    fn step(&mut self, rest: &[u8], previous: Option<u8>) -> Result<Step, Error> {
        match self.lexeme {
            Lexeme::String => return Ok((self.string(rest), Class::Content)),
            Lexeme::Raw { .. } => return Ok((self.raw(rest), Class::Content)),
            Lexeme::Comment(_) => return self.comment(rest),
            Lexeme::None => {}
        }
        if self.is_markup() && previous.is_none_or(|b| !is_ident(b)) {
            if let Some(length) = link_length(rest) {
                return Ok((length, Class::Content));
            }
        }
        if rest.starts_with(b"//") {
            return Ok((rest.len() - newline_length(rest), Class::Comment));
        }
        if rest.starts_with(b"/*") {
            self.lexeme = Lexeme::Comment(1);
            return Ok((2, Class::Comment));
        }
        match self.frames.last().copied() {
            None | Some(Frame::Content) => self.markup(rest),
            Some(Frame::Code(_)) => self.code(rest),
            Some(Frame::Math) => self.math(rest),
            Some(Frame::Embed(embed)) => self.embed(embed, rest),
        }
    }

    fn is_markup(&self) -> bool {
        matches!(self.frames.last(), None | Some(Frame::Content))
    }

    fn markup(&mut self, rest: &[u8]) -> Result<Step, Error> {
        match rest[0] {
            b'\\' => return Ok((rest.len().min(2), Class::Content)),
            b'`' => return Ok((self.open_raw(rest), Class::Content)),
            b'$' => self.push(Frame::Math)?,
            b'#' if rest.get(1).is_some_and(|&b| starts_expression(b)) => {
                self.push(Frame::Embed(Embed::Start))?;
            }
            // Brackets in markup are text, but they must balance inside a content block.
            b'[' => self.push(Frame::Content)?,
            b']' if self.frames.last() == Some(&Frame::Content) => self.pop(),
            _ => {}
        }
        Ok((1, Class::Content))
    }

    fn code(&mut self, rest: &[u8]) -> Result<Step, Error> {
        match rest[0] {
            b'"' => self.lexeme = Lexeme::String,
            b'`' => return Ok((self.open_raw(rest), Class::Content)),
            b'$' => self.push(Frame::Math)?,
            b'[' => self.push(Frame::Content)?,
            b'(' => self.push(Frame::Code(b')'))?,
            b'{' => self.push(Frame::Code(b'}'))?,
            close @ (b')' | b'}') if self.frames.last() == Some(&Frame::Code(close)) => {
                self.pop();
            }
            _ => {}
        }
        Ok((1, Class::Content))
    }

    fn math(&mut self, rest: &[u8]) -> Result<Step, Error> {
        match rest[0] {
            b'\\' => return Ok((rest.len().min(2), Class::Content)),
            b'"' => self.lexeme = Lexeme::String,
            b'$' => self.pop(),
            b'#' if rest.get(1).is_some_and(|&b| starts_expression(b)) => {
                self.push(Frame::Embed(Embed::Start))?;
            }
            _ => {}
        }
        Ok((1, Class::Content))
    }

    fn embed(&mut self, embed: Embed, rest: &[u8]) -> Result<Step, Error> {
        let byte = rest[0];
        match embed {
            Embed::Start => self.start(rest),
            Embed::Chain => self.chain(rest),
            Embed::Statement => match byte {
                b'\n' | b')' | b'}' | b']' => Ok(self.finish()),
                b';' => {
                    self.pop();
                    Ok((1, Class::Content))
                }
                _ => self.code(rest),
            },
            Embed::Control => match byte {
                b')' | b'}' | b']' => Ok(self.finish()),
                _ => self.code(rest),
            },
            Embed::AfterBody => {
                if byte == b' ' || byte == b'\t' {
                    Ok((1, Class::Content))
                } else if word(rest) == b"else" {
                    self.replace(Embed::Control);
                    Ok((4, Class::Content))
                } else {
                    Ok(self.finish())
                }
            }
        }
    }

    /// Decides the kind of an embedded expression from its first token.
    fn start(&mut self, rest: &[u8]) -> Result<Step, Error> {
        let name = word(rest);
        if !name.is_empty() {
            let kind = if name == b"context" {
                Embed::Start
            } else if STATEMENTS.contains(&name) {
                Embed::Statement
            } else if CONTROLS.contains(&name) {
                Embed::Control
            } else {
                Embed::Chain
            };
            self.replace(kind);
            return Ok((name.len(), Class::Content));
        }
        match rest[0] {
            // Only `context` leaves us here, and a space may follow it.
            b' ' | b'\t' => Ok((1, Class::Content)),
            b'{' | b'[' | b'(' | b'"' => {
                self.replace(Embed::Chain);
                self.code(rest)
            }
            _ => Ok(self.finish()),
        }
    }

    /// Continues a chain such as `#text(red)[body].len()`, or ends it.
    fn chain(&mut self, rest: &[u8]) -> Result<Step, Error> {
        match rest[0] {
            b'(' | b'[' => self.code(rest),
            b'.' if rest.get(1).is_some_and(|&b| starts_ident(b)) => Ok((1, Class::Content)),
            byte if is_ident(byte) => Ok((1, Class::Content)),
            _ => Ok(self.finish()),
        }
    }

    fn string(&mut self, rest: &[u8]) -> usize {
        match rest[0] {
            b'\\' => rest.len().min(2),
            b'"' => {
                self.lexeme = Lexeme::None;
                1
            }
            _ => 1,
        }
    }

    /// Opens raw text, except that exactly two backticks are an empty raw span.
    fn open_raw(&mut self, rest: &[u8]) -> usize {
        let ticks = rest.iter().take_while(|&&b| b == b'`').count();
        if ticks != 2 {
            self.lexeme = Lexeme::Raw { ticks, run: 0 };
        }
        ticks
    }

    /// Raw text ends at the first run of as many backticks as opened it.
    fn raw(&mut self, rest: &[u8]) -> usize {
        if let Lexeme::Raw { ticks, run } = self.lexeme {
            let run = if rest[0] == b'`' { run + 1 } else { 0 };
            self.lexeme = if run == ticks {
                Lexeme::None
            } else {
                Lexeme::Raw { ticks, run }
            };
        }
        1
    }

    fn comment(&mut self, rest: &[u8]) -> Result<Step, Error> {
        let Lexeme::Comment(depth) = self.lexeme else {
            return Ok((1, Class::Comment));
        };
        if rest.starts_with(b"*/") {
            self.lexeme = if depth == 1 {
                Lexeme::None
            } else {
                Lexeme::Comment(depth - 1)
            };
            Ok((2, Class::Comment))
        } else if rest.starts_with(b"/*") {
            if depth >= MAX_DEPTH {
                return Err(coverage("Typst comment nesting exceeds 256"));
            }
            self.lexeme = Lexeme::Comment(depth + 1);
            Ok((2, Class::Comment))
        } else {
            let class = if rest[0].is_ascii_whitespace() {
                Class::Content
            } else {
                Class::Comment
            };
            Ok((1, class))
        }
    }

    fn push(&mut self, frame: Frame) -> Result<(), Error> {
        if self.frames.len() >= MAX_DEPTH {
            return Err(coverage("Typst delimiter nesting exceeds 256"));
        }
        self.frames.push(frame);
        Ok(())
    }

    /// Closes the innermost frame. A body block that closes directly inside a control
    /// expression moves that expression on to look for `else`.
    fn pop(&mut self) {
        let closed = self.frames.pop();
        let body = matches!(closed, Some(Frame::Content | Frame::Code(b'}')));
        if body && self.frames.last() == Some(&Frame::Embed(Embed::Control)) {
            self.replace(Embed::AfterBody);
        }
    }

    /// Ends the innermost embedded expression without consuming the current byte.
    fn finish(&mut self) -> Step {
        self.frames.pop();
        (0, Class::Content)
    }

    fn replace(&mut self, embed: Embed) {
        if let Some(last) = self.frames.last_mut() {
            *last = Frame::Embed(embed);
        }
    }
}

/// Returns the identifier at the start of `rest`, which may be empty.
fn word(rest: &[u8]) -> &[u8] {
    if !rest.first().is_some_and(|&b| starts_ident(b)) {
        return &[];
    }
    let length = rest.iter().take_while(|&&b| is_ident(b)).count();
    &rest[..length]
}

fn starts_ident(byte: u8) -> bool {
    byte.is_ascii_alphabetic() || byte == b'_' || byte >= 0x80
}

fn is_ident(byte: u8) -> bool {
    starts_ident(byte) || byte.is_ascii_digit() || byte == b'-'
}

fn starts_expression(byte: u8) -> bool {
    starts_ident(byte) || b"{[(\"".contains(&byte)
}

fn newline_length(rest: &[u8]) -> usize {
    if rest.ends_with(b"\r\n") {
        2
    } else {
        usize::from(rest.ends_with(b"\n"))
    }
}

/// Returns the length of a markup link such as `https://example.com/a_(b)`. Like Typst,
/// we stop at whitespace, at an unbalanced closing bracket, and at `<`, `>`, or `"`.
fn link_length(rest: &[u8]) -> Option<usize> {
    let prefix = [b"https://".as_slice(), b"http://"]
        .into_iter()
        .find(|prefix| rest.starts_with(prefix))?;
    let (mut parens, mut brackets) = (0usize, 0usize);
    let mut length = prefix.len();
    for &byte in &rest[prefix.len()..] {
        match byte {
            b'(' => parens += 1,
            b'[' => brackets += 1,
            b')' if parens == 0 => break,
            b']' if brackets == 0 => break,
            b')' => parens -= 1,
            b']' => brackets -= 1,
            b'<' | b'>' | b'"' => break,
            _ if byte.is_ascii_whitespace() => break,
            _ => {}
        }
        length += 1;
    }
    Some(length)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lines(source: &str) -> (u64, u64, u64) {
        let c = count(source).unwrap();
        (c.code, c.comments, c.blank)
    }

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
        assert_eq!(lines(source), (8, 3, 1));
        assert!(count("```rs\nunclosed").is_err());
        assert!(count("/* unclosed").is_err());
        assert!(count("#box[\n#text(\"hi\")\nA lone \" prose quote.\n]").is_ok());
        assert!(count("#text(\"hi\") A lone \" prose quote.").is_ok());
    }

    #[test]
    fn multiline_statement_headers_stay_in_code() {
        let source = concat!(
            "#let book(title: \"CCM\",\n",
            "  version: \"\", body) = {\n",
            "  // The trim is fixed.\n",
            "  let ink = rgb(\"#26202e\")\n",
            "  page[#text(fill: ink)[Code, change,\\\n  and \"confidence.]]\n",
            "}\n",
            "Prose with a \" quote after the block.\n"
        );
        assert_eq!(lines(source), (7, 1, 0));
    }

    #[test]
    fn embedded_expressions_end_where_typst_ends_them() {
        // After the chain or the control body, the quote is prose again.
        assert!(count("#text(red)[hi] and a \" quote\n").is_ok());
        assert!(count("#if x { a } else [b] then a \" quote\n").is_ok());
        assert!(count("#for x in (1, 2) [#x] a \" quote\n").is_ok());
        assert!(count("#context counter(page).display() a \" quote\n").is_ok());
        assert!(count("#set text(red); a \" quote\n").is_ok());
        // Inside a statement the quote opens a string that may span lines.
        assert!(count("#let s = \"a\nb\"\n").is_ok());
        assert!(count("#let s = \"a\n").is_err());
    }

    #[test]
    fn links_math_and_raw_runs_are_not_comments() {
        assert_eq!(lines("See https://example.com/a//b for more.\n"), (1, 0, 0));
        assert_eq!(lines("[https://example.com] // note\n"), (1, 0, 0));
        assert_eq!(lines("$ \"text // no\" $ // yes\n"), (1, 0, 0));
        assert_eq!(lines("// only a comment\n"), (0, 1, 0));
        assert_eq!(lines("```\n`` inside\n```\n"), (3, 0, 0));
        assert_eq!(lines("``  empty raw, then // comment\n"), (1, 0, 0));
    }

    #[test]
    fn excessive_nesting_is_a_coverage_error() {
        assert!(count(&"(".repeat(300)).is_ok());
        assert!(count(&format!("#{}", "(".repeat(300))).is_err());
        assert!(count(&"/*".repeat(300)).is_err());
    }
}
