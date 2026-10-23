//! Perl quote-like operators and pattern literals, whose delimiters the author chooses.
//!
//! `q{...}`, `qq(...)`, `qw[...]`, `qr<...>`, `m!...!`, `s/a/b/`, `tr(a)(b)`, and a bare
//! `/pattern/` may contain quotes and `#` freely, so we must skip them as a whole.

/// Quote-like operators and the number of delimited parts each one takes.
const OPERATORS: [(&[u8], u8); 8] = [
    (b"q", 1),
    (b"qq", 1),
    (b"qw", 1),
    (b"qr", 1),
    (b"m", 1),
    (b"s", 2),
    (b"tr", 2),
    (b"y", 2),
];

/// Words after which a `/` starts a pattern rather than a division.
const PATTERN_WORDS: [&[u8]; 12] = [
    b"split", b"grep", b"map", b"join", b"if", b"unless", b"and", b"or", b"not", b"return",
    b"when", b"while",
];

/// An open quote-like literal.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Quote {
    open: u8,
    close: u8,
    /// Nesting depth of bracketing delimiters inside the current part.
    depth: usize,
    /// Parts still to close; `s` and `tr` have two.
    parts: u8,
    /// Between the bracketed parts of `s{a}{b}`, waiting for the second opening delimiter.
    between: bool,
    /// A bare `/pattern/`, which we expect to close on its own line.
    bare: bool,
}

impl Quote {
    fn new(open: u8, parts: u8) -> Self {
        Self {
            open,
            close: closing(open),
            depth: 0,
            parts,
            between: false,
            bare: false,
        }
    }

    /// Returns true for a bare `/pattern/`. Perl decides between a pattern and a division
    /// from the previous token, which may sit on an earlier line. When such a pattern
    /// does not close on its line, the `/` was most likely a division.
    pub(super) fn is_bare(self) -> bool {
        self.bare
    }

    /// Consumes bytes from `rest` and returns how many, and whether the literal closed.
    pub(super) fn step(&mut self, rest: &[u8]) -> (usize, bool) {
        let byte = rest[0];
        if self.between {
            if !byte.is_ascii_whitespace() {
                *self = Self {
                    between: false,
                    ..Self::new(byte, self.parts)
                };
            }
            return (1, false);
        }
        if byte == b'\\' {
            return (rest.len().min(2), false);
        }
        if self.open != self.close && byte == self.open {
            self.depth += 1;
        } else if byte == self.close {
            if self.depth > 0 {
                self.depth -= 1;
            } else {
                self.parts -= 1;
                if self.parts == 0 {
                    return (1, true);
                }
                // `s/a/b/` shares its middle delimiter; `s{a}{b}` opens a second pair.
                self.between = self.open != self.close;
            }
        }
        (1, false)
    }
}

/// Recognizes a quote-like operator or a pattern at `rest`. `prefix` is the line before
/// it. Returns the length of the opening and the open literal.
pub(super) fn start(rest: &[u8], prefix: &[u8]) -> Option<(usize, Quote)> {
    if !starts_token(prefix) {
        return None;
    }
    if rest[0] == b'/' {
        let quote = Quote {
            bare: true,
            ..Quote::new(b'/', 1)
        };
        return pattern_position(prefix).then_some((1, quote));
    }
    let length = rest
        .iter()
        .take_while(|&&b| b.is_ascii_alphanumeric() || b == b'_')
        .count();
    let parts = OPERATORS
        .iter()
        .find(|(word, _)| *word == &rest[..length])?
        .1;
    let spaces = rest[length..]
        .iter()
        .take_while(|&&b| b == b' ' || b == b'\t')
        .count();
    let open = *rest.get(length + spaces)?;
    let after = rest.get(length + spaces + 1).copied().unwrap_or(b' ');
    // After a space, `#` starts a comment. A `,` or `=` delimiter must touch the word, as
    // in `s,a,b,`; otherwise the word is a plain name, as in `s => 1` or `y, 2`.
    let delimiter = match open {
        b',' => spaces == 0 && !after.is_ascii_whitespace(),
        b'=' => spaces == 0 && !after.is_ascii_whitespace() && !b"=>".contains(&after),
        b'#' => spaces == 0,
        _ => open.is_ascii_punctuation() && !b";)]}>".contains(&open),
    };
    delimiter.then(|| (length + spaces + 1, Quote::new(open, parts)))
}

/// A sigil, `->`, `::`, or `-` (a file test such as `-s`) makes the next word a name, and
/// a word right after a closing delimiter is a modifier, as in `/x/s` or `s{a}{b}ms`.
fn starts_token(prefix: &[u8]) -> bool {
    prefix
        .last()
        .is_none_or(|&b| !(b.is_ascii_alphanumeric() || b"_$@%&*->:'/})]".contains(&b)))
}

fn pattern_position(prefix: &[u8]) -> bool {
    let prefix = prefix.trim_ascii_end();
    if prefix.last().is_none_or(|b| b"=(,;!?[{&|~".contains(b)) {
        return true;
    }
    let word = prefix
        .rsplit(|b| !b.is_ascii_alphanumeric() && *b != b'_')
        .next()
        .unwrap_or(prefix);
    PATTERN_WORDS.contains(&word)
}

fn closing(open: u8) -> u8 {
    match open {
        b'(' => b')',
        b'[' => b']',
        b'{' => b'}',
        b'<' => b'>',
        other => other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(source: &[u8]) -> Option<usize> {
        let (mut index, mut quote) = start(source, b"")?;
        while index < source.len() {
            let (length, closed) = quote.step(&source[index..]);
            index += length;
            if closed {
                return Some(index);
            }
        }
        None
    }

    #[test]
    fn quote_like_operators_span_their_delimiters() {
        assert_eq!(run(b"q{a {b} c}d"), Some(10));
        assert_eq!(run(b"s/'/\\//g"), Some(7));
        assert_eq!(run(b"s{a} {b}x"), Some(8));
        assert_eq!(run(b"tr#a#b#"), Some(7));
        assert_eq!(run(b"qw(it's)"), Some(8));
        assert_eq!(run(b"/\"#/"), Some(4));
        assert!(start(b"s => 1", b"").is_none());
        assert!(start(b"y, 2", b"").is_none());
        assert!(start(b"q #c", b"").is_none());
        assert!(start(b"s/a/b/", b"$").is_none());
        assert!(start(b"s/a/b/", b"-").is_none());
        assert!(start(b"/ 2", b"$x ").is_none());
        assert_eq!(run(b"s,/+,__,g"), Some(8));
        assert_eq!(run(b"s=\\\\=/=g"), Some(7));
        assert!(start(b"s ? a : b", b"/x/").is_none());
        assert!(start(b"s = 1", b"").is_none());
    }
}
