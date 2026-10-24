//! Assembly counting across the common dialects: GNU as, NASM, MASM, and armasm.
//!
//! Assemblers disagree on comments. GNU as reads `#` on x86 and `@` on 32-bit ARM, NASM
//! and MASM read `;`, and a `.S` file first passes through the C preprocessor, which
//! removes `//` and `/* */` and keeps `#define` and its kin. One file never says which
//! assembler it targets, so we accept each comment form where it cannot be code:
//!
//! ```text
//!   // note          ; note          /* note */         every dialect
//!   # note           @ note                             at the start of a line
//!   mov r0, #1  # n  mov r0, r1  @ n                    after code, followed by space
//!   #define X 1      @@loop:         mov r0, #1         code: directive, label, immediate
//! ```
//!
//! Literals never span lines in assembly, so a quote that does not close ends at the
//! line end. Only a block comment can stay open, and it is the only coverage failure.
use crate::{bytes, coverage, is_blank};
use ccm_core::{Counts, Error};

/// C preprocessor directives. A `#` line that names one is code in a `.S` file; any
/// other `#` line is a GNU as comment, which the preprocessor passes through.
const DIRECTIVES: [&[u8]; 17] = [
    b"define",
    b"undef",
    b"include",
    b"include_next",
    b"import",
    b"if",
    b"ifdef",
    b"ifndef",
    b"elif",
    b"elifdef",
    b"elifndef",
    b"else",
    b"endif",
    b"error",
    b"warning",
    b"pragma",
    b"line",
];

/// Counts assembly source. Returns a coverage error only for a block comment that is
/// still open at the end of the file.
pub(crate) fn count(text: &str) -> Result<Counts, Error> {
    let mut counts = Counts::default();
    let mut opened_on = None;
    for (index, line) in bytes::lines(text).enumerate() {
        let (code, comment) = classify(line.as_bytes(), &mut opened_on, index + 1);
        if is_blank(line) {
            counts.blank += 1;
        } else if code {
            counts.code += 1;
        } else if comment {
            counts.comments += 1;
        } else {
            counts.blank += 1;
        }
    }
    match opened_on {
        Some(line) => Err(coverage(&format!(
            "Unterminated block comment that begins on line {line}"
        ))),
        None => Ok(counts),
    }
}

/// Returns whether `line` holds code and whether it holds a comment. `opened_on` is the
/// line number of the block comment open at the start of the line, if any; we update it
/// as comments open and close.
fn classify(line: &[u8], opened_on: &mut Option<usize>, number: usize) -> (bool, bool) {
    let (mut code, mut comment) = (false, opened_on.is_some());
    let mut index = 0;
    while index < line.len() {
        if opened_on.is_some() {
            let Some(end) = find_close(&line[index..]) else {
                return (code, true);
            };
            *opened_on = None;
            index += end + 2;
            continue;
        }
        let rest = &line[index..];
        let byte = rest[0];
        if byte.is_ascii_whitespace() {
            index += 1;
        } else if bytes::starts(rest, b"/*") {
            *opened_on = Some(number);
            comment = true;
            index += 2;
        } else if bytes::starts(rest, b"//")
            || byte == b';'
            || (matches!(byte, b'#' | b'@') && opens_comment(line, index, code))
        {
            return (code, true);
        } else {
            code = true;
            index += token_length(rest, line[..index].last().copied());
        }
    }
    (code, comment)
}

/// Returns true when the `#` or `@` at `line[index]` begins a comment. At the start of a
/// line it does, except in a preprocessor directive or a MASM `@@` label. After code it
/// does only between spaces, as in `mov r0, r1  @ note`, so that `#1`, `foo@PLT`, and
/// `.type f, @function` stay code.
fn opens_comment(line: &[u8], index: usize, code: bool) -> bool {
    let rest = &line[index..];
    if code {
        let spaced_before = index > 0 && line[index - 1].is_ascii_whitespace();
        return spaced_before && rest.get(1).is_none_or(u8::is_ascii_whitespace);
    }
    match rest[0] {
        b'#' => !is_directive(&rest[1..]),
        _ => rest.get(1) != Some(&b'@'),
    }
}

/// Returns the offset of the `*/` that closes a block comment in `rest`.
fn find_close(rest: &[u8]) -> Option<usize> {
    let mut start = 0;
    while let Some(offset) = bytes::find(&rest[start..], b'*') {
        let at = start + offset;
        if rest.get(at + 1) == Some(&b'/') {
            return Some(at);
        }
        start = at + 1;
    }
    None
}

/// Returns true when the text after a line-start `#` names a preprocessor directive,
/// with optional space between, as in `# define`.
fn is_directive(after_hash: &[u8]) -> bool {
    let text = after_hash.trim_ascii_start();
    let word_end = text
        .iter()
        .position(|byte| !(byte.is_ascii_alphanumeric() || *byte == b'_'))
        .unwrap_or(text.len());
    DIRECTIVES.contains(&&text[..word_end])
}

/// Returns the length of the code token at the start of `rest`. Strings and character
/// constants are skipped whole, so the comment bytes inside them are text. `before` is
/// the byte before the token, which tells a GNU as character constant from a string.
fn token_length(rest: &[u8], before: Option<u8>) -> usize {
    match rest[0] {
        quote @ (b'"' | b'`') => string_length(rest, quote, true),
        // GNU as writes a character constant as `$'a` or `#'a`, with no closing quote.
        b'\'' if matches!(before, Some(b'$' | b'#')) => {
            if rest.get(1) == Some(&b'\\') {
                rest.len().min(3)
            } else {
                rest.len().min(2)
            }
        }
        b'\'' => string_length(rest, b'\'', false),
        _ => 1,
    }
}

/// Returns the length of the string that opens `rest` with `quote`, up to its closing
/// quote or the line end. `escapes` says whether a backslash escapes the next byte.
fn string_length(rest: &[u8], quote: u8, escapes: bool) -> usize {
    let mut index = 1;
    while index < rest.len() {
        match rest[index] {
            b'\\' if escapes => index += 2,
            byte if byte == quote => return index + 1,
            _ => index += 1,
        }
    }
    rest.len()
}
