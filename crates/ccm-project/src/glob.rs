// Bounded byte glob matching for file selection. No recursion or regular-expression engine.
use ccm_core::Error;

#[derive(Clone, Debug)]
enum Token {
    Byte(u8),
    Any,
    Star(bool),
    Directories,
    Class(Vec<(u8, u8)>, bool),
}

#[derive(Clone, Debug)]
pub(super) struct Glob {
    tokens: Vec<Token>,
    literal: Option<Vec<u8>>,
    prefix: Vec<u8>,
    suffix: Vec<u8>,
    simple_star: Option<bool>,
}

impl Glob {
    pub fn compile(pattern: &str) -> Result<Self, Error> {
        if pattern.len() > 1024 {
            return Err(problem("An ignore pattern exceeds 1024 bytes"));
        }
        let bytes = pattern.as_bytes();
        let mut tokens = Vec::new();
        let mut index = 0;
        while index < bytes.len() {
            match bytes[index] {
                b'\\' if index + 1 < bytes.len() => {
                    tokens.push(Token::Byte(bytes[index + 1]));
                    index += 2;
                }
                b'*' => {
                    let run = bytes[index..].iter().take_while(|&&b| b == b'*').count();
                    // Measured against `git check-ignore` 2.47: `**/` matches zero or more
                    // directories wherever it occurs, and a final `**` crosses slashes only
                    // as a whole segment. Other star runs behave like one ordinary star.
                    let segment_start = index == 0 || bytes[index - 1] == b'/';
                    index += run;
                    if run == 2 && bytes.get(index) == Some(&b'/') {
                        tokens.push(Token::Directories);
                        index += 1;
                    } else {
                        let whole = run == 2 && segment_start && index == bytes.len();
                        tokens.push(Token::Star(whole));
                    }
                }
                b'?' => {
                    tokens.push(Token::Any);
                    index += 1;
                }
                b'[' => {
                    if let Some((token, length)) = character_class(&bytes[index..]) {
                        tokens.push(token);
                        index += length;
                    } else {
                        tokens.push(Token::Byte(b'['));
                        index += 1;
                    }
                }
                byte => {
                    tokens.push(Token::Byte(byte));
                    index += 1;
                }
            }
        }
        let literal = tokens
            .iter()
            .map(|token| match token {
                Token::Byte(byte) => Some(*byte),
                _ => None,
            })
            .collect::<Option<Vec<_>>>();
        Ok(Self::from_tokens(tokens, literal))
    }

    fn from_tokens(tokens: Vec<Token>, literal: Option<Vec<u8>>) -> Self {
        let prefix = tokens
            .iter()
            .map_while(|t| match t {
                Token::Byte(b) => Some(*b),
                _ => None,
            })
            .collect();
        let mut suffix: Vec<_> = tokens
            .iter()
            .rev()
            .map_while(|t| match t {
                Token::Byte(b) => Some(*b),
                _ => None,
            })
            .collect();
        suffix.reverse();
        let stars: Vec<_> = tokens
            .iter()
            .filter_map(|t| match t {
                Token::Star(cross) => Some(*cross),
                _ => None,
            })
            .collect();
        let simple_star = if stars.len() == 1
            && tokens
                .iter()
                .all(|t| matches!(t, Token::Byte(_) | Token::Star(_)))
        {
            Some(stars[0])
        } else {
            None
        };
        Self {
            tokens,
            literal,
            prefix,
            suffix,
            simple_star,
        }
    }

    pub fn matches(&self, text: &str, budget: &mut u64) -> Result<bool, Error> {
        if let Some(literal) = &self.literal {
            // Slices of different lengths compare unequal without reading their bytes.
            let compared = if literal.len() == text.len() {
                text.len()
            } else {
                0
            };
            charge(budget, compared + 1)?;
            return Ok(literal == text.as_bytes());
        }
        // We charge the work each stage actually does. Most rules, such as `*.o`, are
        // decided by the literal prefix and suffix alone, so charging the full matching
        // table for them would exhaust the budget on a large tree that we match quickly.
        let bytes = text.as_bytes();
        charge(budget, self.prefix.len() + self.suffix.len() + 1)?;
        if bytes.len() < self.prefix.len() + self.suffix.len()
            || !bytes.starts_with(&self.prefix)
            || !bytes.ends_with(&self.suffix)
        {
            return Ok(false);
        }
        if let Some(cross) = self.simple_star {
            charge(budget, bytes.len())?;
            return Ok(cross
                || !bytes[self.prefix.len()..bytes.len() - self.suffix.len()].contains(&b'/'));
        }
        charge(
            budget,
            (self.tokens.len() + 1).saturating_mul(bytes.len() + 1),
        )?;
        let mut previous = vec![false; bytes.len() + 1];
        let mut next = vec![false; bytes.len() + 1];
        previous[0] = true;
        for token in &self.tokens {
            next.fill(false);
            if matches!(token, Token::Star(_) | Token::Directories) {
                next[0] = previous[0];
            }
            let mut directory_prefix = previous[0];
            for (index, &byte) in bytes.iter().enumerate() {
                next[index + 1] = match token {
                    Token::Byte(wanted) => previous[index] && byte == *wanted,
                    Token::Any => previous[index] && byte != b'/',
                    Token::Class(ranges, negate) => {
                        previous[index]
                            && byte != b'/'
                            && (ranges.iter().any(|&(a, b)| a <= byte && byte <= b) != *negate)
                    }
                    Token::Star(cross) => {
                        previous[index + 1] || (next[index] && (*cross || byte != b'/'))
                    }
                    Token::Directories => {
                        previous[index + 1] || (byte == b'/' && directory_prefix)
                    }
                };
                directory_prefix |= previous[index + 1];
            }
            std::mem::swap(&mut previous, &mut next);
        }
        Ok(previous[bytes.len()])
    }
}

fn character_class(bytes: &[u8]) -> Option<(Token, usize)> {
    let negate = matches!(bytes.get(1), Some(b'!' | b'^'));
    let mut index = if negate { 2 } else { 1 };
    let first = index;
    let mut ranges = Vec::new();
    // A `]` directly after the opening bracket (or its negation) is a literal member.
    while index < bytes.len() && (bytes[index] != b']' || index == first) {
        if bytes[index..].starts_with(b"[:") {
            let end = bytes[index + 2..].windows(2).position(|w| w == b":]")? + index + 2;
            ranges.extend_from_slice(ascii_class(&bytes[index + 2..end])?);
            index = end + 2;
            continue;
        }
        let (start, width) = class_member(&bytes[index..])?;
        index += width;
        let end = if bytes.get(index) == Some(&b'-') && bytes.get(index + 1) != Some(&b']') {
            let (end, width) = class_member(&bytes[index + 1..])?;
            index += 1 + width;
            end
        } else {
            start
        };
        ranges.push((start, end));
    }
    if ranges.is_empty() || bytes.get(index) != Some(&b']') {
        return None;
    }
    Some((Token::Class(ranges, negate), index + 1))
}

// Returns one class member byte and its encoded width; a backslash escapes the next byte.
fn class_member(bytes: &[u8]) -> Option<(u8, usize)> {
    match bytes {
        [b'\\', escaped, ..] => Some((*escaped, 2)),
        [byte, ..] => Some((*byte, 1)),
        [] => None,
    }
}

fn ascii_class(name: &[u8]) -> Option<&'static [(u8, u8)]> {
    Some(match name {
        b"alnum" => &[(b'0', b'9'), (b'A', b'Z'), (b'a', b'z')],
        b"alpha" => &[(b'A', b'Z'), (b'a', b'z')],
        b"blank" => &[(b'\t', b'\t'), (b' ', b' ')],
        b"cntrl" => &[(0, 31), (127, 127)],
        b"digit" => &[(b'0', b'9')],
        b"graph" => &[(33, 126)],
        b"lower" => &[(b'a', b'z')],
        b"print" => &[(32, 126)],
        b"punct" => &[(33, 47), (58, 64), (91, 96), (123, 126)],
        b"space" => &[(9, 13), (32, 32)],
        b"upper" => &[(b'A', b'Z')],
        b"xdigit" => &[(b'0', b'9'), (b'A', b'F'), (b'a', b'f')],
        _ => return None,
    })
}

/// Takes `work` units from the matching budget, or fails when it is exhausted.
fn charge(budget: &mut u64, work: usize) -> Result<(), Error> {
    *budget = budget
        .checked_sub(work as u64)
        .ok_or_else(|| problem("Ignore matching work budget exhausted"))?;
    Ok(())
}

fn problem(message: &str) -> Error {
    Error::new(
        "ignore-limit",
        message,
        "Simplify ignore patterns or narrow the selected paths.",
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn budget_counts_the_work_actually_done() {
        let path = "drivers/gpu/drm/amd/display/dc/core/dc_resource.c";
        let suffix = Glob::compile("*.o").unwrap();
        let mut budget = 1_000;
        assert!(!suffix.matches(path, &mut budget).unwrap());
        // Rejected by its two-byte suffix, not by a table over the whole path.
        assert!(1_000 - budget <= 3, "charged {}", 1_000 - budget);
        let literal = Glob::compile("vmlinux").unwrap();
        let mut budget = 1_000;
        assert!(!literal.matches(path, &mut budget).unwrap());
        assert_eq!(1_000 - budget, 1);
        // A general pattern still pays for its table and fails cleanly when out of budget.
        let general = Glob::compile("dr*/[a-z]*/*.c").unwrap();
        let mut budget = 10;
        assert!(general.matches(path, &mut budget).is_err());
    }

    #[test]
    fn patterns_support_directory_wildcards_classes_and_literal_escapes() {
        for (pattern, text, expected) in [
            ("**/test?.[ch]", "test1.c", true),
            ("**/test?.[ch]", "a/b/test2.h", true),
            ("*.rs", "a/b.rs", false),
            ("a/**/b", "a/b", true),
            ("a/**/b", "a/x/y/b", true),
            ("[!a-c].txt", "d.txt", true),
            ("[!a-c].txt", "b.txt", false),
            (r"\!important", "!important", true),
            ("[[:digit:]].rs", "3.rs", true),
            ("[[:digit:]].rs", "x.rs", false),
            ("a?", "a/", false),
            ("**/a", "ba", false),
        ] {
            assert_eq!(
                Glob::compile(pattern)
                    .unwrap()
                    .matches(text, &mut 1_000_000)
                    .unwrap(),
                expected,
                "{pattern}: {text}"
            );
        }
        assert!(Glob::compile("*").unwrap().matches("abc", &mut 1).is_err());
    }

    #[test]
    fn double_stars_not_followed_by_a_slash_are_ordinary_stars_as_in_git() {
        for (pattern, text, expected) in [
            ("a**b", "axb", true),
            ("a**b", "a/x/b", false),
            ("foo**/bar", "foox/bar", true),
            ("foo**/bar", "foo/x/bar", true),
            ("foo**/bar", "foobar", true),
            ("**a", "x/a", false),
            ("y/**q", "y/r/q", false),
            ("y/**q", "y/rq", true),
            ("a/**", "a/x/y", true),
            ("**", "x/y", true),
        ] {
            assert_eq!(
                Glob::compile(pattern)
                    .unwrap()
                    .matches(text, &mut 1_000_000)
                    .unwrap(),
                expected,
                "{pattern}: {text}"
            );
        }
    }

    #[test]
    fn classes_accept_a_leading_bracket_and_escaped_members() {
        for (pattern, text, expected) in [
            ("[]a]", "]", true),
            ("[]a]", "a", true),
            ("[!]a]", "b", true),
            ("[!]a]", "]", false),
            (r"[\]]x", "]x", true),
            ("[a-]", "-", true),
            ("[]", "[]", true),
        ] {
            assert_eq!(
                Glob::compile(pattern)
                    .unwrap()
                    .matches(text, &mut 1_000_000)
                    .unwrap(),
                expected,
                "{pattern}: {text}"
            );
        }
    }

    #[test]
    fn fast_rejections_and_single_stars_agree_with_the_full_state_machine() {
        for pattern in [
            "a",
            "a*a",
            "*.b",
            "a/**/b",
            "**/*.b",
            "a?b",
            "[ab]*",
            "[!a]",
            "a/*/b",
            "*a*",
            "**",
            "a**b",
            "a\\*b",
            "[[:digit:]]*",
        ] {
            let fast = Glob::compile(pattern).unwrap();
            let oracle = Glob {
                literal: None,
                prefix: Vec::new(),
                suffix: Vec::new(),
                simple_star: None,
                tokens: fast.tokens.clone(),
            };
            for length in 0..=5 {
                for mut encoded in 0..4usize.pow(length) {
                    let mut text = String::new();
                    for _ in 0..length {
                        text.push(b"ab/."[encoded % 4] as char);
                        encoded /= 4;
                    }
                    let mut fast_budget = u64::MAX;
                    let mut oracle_budget = u64::MAX;
                    assert_eq!(
                        fast.matches(&text, &mut fast_budget).unwrap(),
                        oracle.matches(&text, &mut oracle_budget).unwrap(),
                        "{pattern}: {text}"
                    );
                }
            }
        }
    }
}
