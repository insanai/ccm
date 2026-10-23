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
                    let double = bytes.get(index + 1) == Some(&b'*');
                    index += if double { 2 } else { 1 };
                    if double && bytes.get(index) == Some(&b'/') {
                        tokens.push(Token::Directories);
                        index += 1;
                    } else {
                        tokens.push(Token::Star(double));
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
            *budget = budget
                .checked_sub(text.len() as u64 + 1)
                .ok_or_else(|| problem("Ignore matching work budget exhausted"))?;
            return Ok(literal == text.as_bytes());
        }
        let work = (self.tokens.len() as u64 + 1).saturating_mul(text.len() as u64 + 1);
        *budget = budget
            .checked_sub(work)
            .ok_or_else(|| problem("Ignore matching work budget exhausted"))?;
        let bytes = text.as_bytes();
        if bytes.len() < self.prefix.len() + self.suffix.len()
            || !bytes.starts_with(&self.prefix)
            || !bytes.ends_with(&self.suffix)
        {
            return Ok(false);
        }
        if let Some(cross) = self.simple_star {
            return Ok(cross
                || !bytes[self.prefix.len()..bytes.len() - self.suffix.len()].contains(&b'/'));
        }
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
    let mut ranges = Vec::new();
    while index < bytes.len() && bytes[index] != b']' {
        if bytes[index..].starts_with(b"[:") {
            let end = bytes[index + 2..].windows(2).position(|w| w == b":]")? + index + 2;
            ranges.extend_from_slice(ascii_class(&bytes[index + 2..end])?);
            index = end + 2;
            continue;
        }
        let start = bytes[index];
        if bytes.get(index + 1) == Some(&b'-') && bytes.get(index + 2).is_some_and(|&b| b != b']')
        {
            ranges.push((start, bytes[index + 2]));
            index += 3;
        } else {
            ranges.push((start, start));
            index += 1;
        }
    }
    if ranges.is_empty() || bytes.get(index) != Some(&b']') {
        return None;
    }
    Some((Token::Class(ranges, negate), index + 1))
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
