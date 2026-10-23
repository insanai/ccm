use crate::syntax::{Syntax, syntax};
use crate::{Language, coverage};
use ccm_core::{Counts, Error};
use std::collections::VecDeque;

#[derive(Default)]
enum State {
    #[default]
    Normal,
    Block(usize),
    Quote(u8, bool),
    Delimited(Vec<u8>, bool),
}

#[derive(Default)]
struct Lexer {
    state: State,
    heredocs: VecDeque<(Vec<u8>, bool)>,
    continued_comment: bool,
    arithmetic_depth: usize,
    work: crate::work::Counter,
}

pub(super) fn count(
    source: &str,
    language: Language,
    work: bool,
) -> Result<(Counts, Option<u64>), Error> {
    classify(source, language, work, |_, _| {})
}

pub(super) fn classify(
    source: &str,
    language: Language,
    work: bool,
    mut emit: impl FnMut(bool, bool),
) -> Result<(Counts, Option<u64>), Error> {
    let mut lexer = Lexer {
        work: crate::work::Counter::new(work),
        ..Default::default()
    };
    let mut counts = Counts::default();
    let policy = syntax(language);
    let special = special_bytes(language, policy);
    for line in source.split_inclusive('\n') {
        let line = line.strip_suffix('\n').unwrap_or(line);
        let line = line.strip_suffix('\r').unwrap_or(line);
        let (code, comment) = if matches!(
            language,
            Language::Text | Language::ReStructuredText | Language::Diff | Language::Csv
        ) {
            (!line.trim().is_empty(), false)
        } else {
            lexer.line(line.as_bytes(), language, policy, &special)?
        };
        emit(code, comment);
        lexer.work.boundary();
        if code {
            counts.code += 1;
        } else if comment {
            counts.comments += 1;
        } else {
            counts.blank += 1;
        }
    }
    if !matches!(lexer.state, State::Normal)
        || !lexer.heredocs.is_empty()
        || lexer.arithmetic_depth > 0
    {
        return Err(coverage("Unterminated string, heredoc, or block comment"));
    }
    Ok((counts, work.then_some(lexer.work.tokens)))
}

impl Lexer {
    fn line(
        &mut self,
        line: &[u8],
        language: Language,
        syntax: Syntax,
        special: &[bool; 256],
    ) -> Result<(bool, bool), Error> {
        if let Some((delimiter, tabs)) = self.heredocs.front() {
            let content = if *tabs {
                &line[line.iter().take_while(|&&byte| byte == b'\t').count()..]
            } else {
                line
            };
            if content == delimiter {
                self.heredocs.pop_front();
            }
            return Ok((true, false));
        }
        if self.continued_comment {
            self.continued_comment = line.ends_with(b"\\");
            return Ok((false, true));
        }
        let mut code = matches!(self.state, State::Quote(..) | State::Delimited(_, false));
        let mut comment = matches!(self.state, State::Block(_) | State::Delimited(_, true));
        let mut index = 0;
        while index < line.len() {
            let rest = &line[index..];
            if matches!(self.state, State::Normal) {
                let length = rest
                    .iter()
                    .position(|&byte| special[byte as usize])
                    .unwrap_or(rest.len());
                if length > 0 {
                    if !code {
                        code = rest[..length].iter().any(|b| !b.is_ascii_whitespace());
                    }
                    self.work.source(&rest[..length]);
                    index += length;
                    continue;
                }
            }
            match &mut self.state {
                State::Block(depth) => {
                    comment = true;
                    let (open, close) = syntax.block.unwrap_or((b"/*", b"*/"));
                    if rest.starts_with(close) {
                        *depth -= 1;
                        index += close.len();
                        if *depth == 0 {
                            self.state = State::Normal;
                        }
                    } else if syntax.nested && rest.starts_with(open) {
                        *depth += 1;
                        index += open.len();
                        if *depth > 256 {
                            return Err(coverage("Comment nesting exceeds 256"));
                        }
                    } else {
                        index += 1;
                    }
                }
                State::Quote(quote, escapes) => {
                    code = true;
                    if rest[0] == b'\\' && *escapes {
                        index += rest.len().min(2);
                    } else if rest[0] == *quote {
                        self.state = State::Normal;
                        index += 1;
                    } else {
                        index += 1;
                    }
                }
                State::Delimited(close, is_comment) => {
                    if *is_comment {
                        comment = true;
                    } else {
                        code = true;
                    }
                    if rest.starts_with(close) {
                        index += close.len();
                        self.state = State::Normal;
                    } else {
                        index += 1;
                    }
                }
                State::Normal => {
                    let (advance, has_code, has_comment, stop) = if self.work.enabled {
                        self.normal_work(line, index, language, syntax)?
                    } else {
                        self.normal(line, index, language, syntax)?
                    };
                    code |= has_code;
                    comment |= has_comment;
                    index += advance;
                    if stop {
                        break;
                    }
                }
            }
        }
        Ok((code, comment))
    }

    fn normal_work(
        &mut self,
        line: &[u8],
        index: usize,
        language: Language,
        syntax: Syntax,
    ) -> Result<(usize, bool, bool, bool), Error> {
        let rest = &line[index..];
        let heredocs = self.heredocs.len();
        let regex = matches!(language, Language::JavaScript | Language::TypeScript)
            && rest[0] == b'/'
            && !rest.starts_with(b"//")
            && !rest.starts_with(b"/*")
            && regex_position(&line[..index]);
        let result = self.normal(line, index, language, syntax)?;
        if result.2 {
            self.work.boundary();
        } else if result.1 {
            if matches!(self.state, State::Quote(..) | State::Delimited(_, false))
                || self.heredocs.len() > heredocs
                || regex
                || (language == Language::Zig && rest.starts_with(b"\\\\"))
            {
                self.work.literal();
            } else {
                self.work.source(&rest[..result.0]);
            }
        } else {
            self.work.source(&rest[..result.0]);
        }
        Ok(result)
    }

    fn normal(
        &mut self,
        line: &[u8],
        index: usize,
        language: Language,
        syntax: Syntax,
    ) -> Result<(usize, bool, bool, bool), Error> {
        let rest = &line[index..];
        if language == Language::Shell {
            if rest.starts_with(b"((") {
                self.arithmetic_depth += 1;
                if self.arithmetic_depth > 256 {
                    return Err(coverage("Arithmetic nesting exceeds 256"));
                }
                return Ok((2, true, false, false));
            }
            if rest.starts_with(b"))") && self.arithmetic_depth > 0 {
                self.arithmetic_depth -= 1;
                return Ok((2, true, false, false));
            }
            if rest.starts_with(b"<<<") {
                return Ok((3, true, false, false));
            }
            if rest.starts_with(b"<<") && self.arithmetic_depth > 0 {
                return Ok((2, true, false, false));
            }
        }
        if matches!(language, Language::JavaScript | Language::TypeScript)
            && rest[0] == b'/'
            && !rest.starts_with(b"//")
            && !rest.starts_with(b"/*")
            && regex_position(&line[..index])
        {
            return Ok((regex_length(rest)?, true, false, false));
        }
        if let Some(length) = self.special(rest, language)? {
            let comment = matches!(self.state, State::Delimited(_, true));
            return Ok((length, !comment, comment, false));
        }
        if let Some((open, _)) = syntax.block.filter(|(a, _)| rest.starts_with(a)) {
            self.state = State::Block(1);
            return Ok((open.len(), false, true, false));
        }
        if syntax.lines.iter().any(|token| rest.starts_with(token))
            && !(language == Language::Shell
                && index > 0
                && !b" \t;|&()".contains(&line[index - 1]))
        {
            self.continued_comment =
                matches!(language, Language::C | Language::Cpp) && line.ends_with(b"\\");
            return Ok((rest.len(), false, true, true));
        }
        if language == Language::Yaml && index > 0 && !b" \t:[{,-?".contains(&line[index - 1]) {
            return Ok((1, !rest[0].is_ascii_whitespace(), false, false));
        }
        Ok((
            self.start_quote(rest, language, syntax),
            !rest[0].is_ascii_whitespace(),
            false,
            false,
        ))
    }

    fn start_quote(&mut self, rest: &[u8], language: Language, syntax: Syntax) -> usize {
        if language == Language::Rust && rest[0] == b'\'' && !is_character(rest) {
            return 1;
        }
        if matches!(
            language,
            Language::OCaml | Language::Haskell | Language::FSharp
        ) && rest[0] == b'\''
            && !is_character(rest)
        {
            return 1;
        }
        let quote = rest[0];
        if quote == b'"' || (quote == b'\'' && syntax.single) || (quote == b'`' && syntax.backtick)
        {
            if syntax.triple && rest.starts_with(&[quote; 3]) {
                self.state = State::Delimited(vec![quote; 3], false);
                return 3;
            }
            self.state = State::Quote(
                quote,
                quote != b'`' && !(quote == b'\'' && syntax.literal_single),
            );
        }
        1
    }

    fn special(&mut self, rest: &[u8], language: Language) -> Result<Option<usize>, Error> {
        if language == Language::Rust {
            if let Some((length, hashes)) = raw_start(rest) {
                let mut close = vec![b'"'];
                close.resize(1 + hashes, b'#');
                self.state = State::Delimited(close, false);
                return Ok(Some(length));
            }
        }
        if matches!(language, Language::C | Language::Cpp) && rest.starts_with(b"R\"") {
            if let Some(end) = rest[2..].iter().take(17).position(|&b| b == b'(') {
                let mut close = vec![b')'];
                close.extend_from_slice(&rest[2..2 + end]);
                close.push(b'"');
                self.state = State::Delimited(close, false);
                return Ok(Some(end + 3));
            }
            return Err(coverage("Invalid C++ raw string delimiter"));
        }
        if language == Language::Lua {
            let (prefix, comment) = if rest.starts_with(b"--[") {
                (2, true)
            } else {
                (0, false)
            };
            if rest.get(prefix) == Some(&b'[') {
                let equals = rest[prefix + 1..]
                    .iter()
                    .take_while(|&&b| b == b'=')
                    .count();
                if equals > 255 {
                    return Err(coverage("Lua delimiter exceeds 255 equals signs"));
                }
                if rest.get(prefix + 1 + equals) == Some(&b'[') {
                    let mut close = vec![b']'];
                    close.resize(equals + 1, b'=');
                    close.push(b']');
                    self.state = State::Delimited(close, comment);
                    return Ok(Some(prefix + equals + 2));
                }
            }
        }
        if language == Language::Shell && rest.starts_with(b"<<") && !rest.starts_with(b"<<<") {
            return self.heredoc(rest).map(Some);
        }
        if language == Language::Zig && rest.starts_with(b"\\\\") {
            self.state = State::Normal;
            // Zig multiline strings terminate at the physical line boundary.
            return Ok(Some(rest.len()));
        }
        if language == Language::PowerShell
            && (rest.starts_with(b"@\"") || rest.starts_with(b"@'"))
        {
            self.state = State::Delimited(vec![rest[1], b'@'], false);
            return Ok(Some(2));
        }
        Ok(None)
    }

    fn heredoc(&mut self, rest: &[u8]) -> Result<usize, Error> {
        let tabs = rest.get(2) == Some(&b'-');
        let mut index = if tabs { 3 } else { 2 };
        while rest.get(index).is_some_and(u8::is_ascii_whitespace) {
            index += 1;
        }
        let quote = rest
            .get(index)
            .copied()
            .filter(|b| *b == b'\'' || *b == b'"');
        if quote.is_some() {
            index += 1;
        }
        let start = index;
        while let Some(&byte) = rest.get(index) {
            if quote == Some(byte) || (quote.is_none() && b" \t;|&<>".contains(&byte)) {
                break;
            }
            index += 1;
        }
        if index == start || index - start > 256 || self.heredocs.len() >= 16 {
            return Err(coverage("Invalid or oversized heredoc delimiter"));
        }
        self.heredocs.push_back((rest[start..index].to_vec(), tabs));
        Ok(index + usize::from(quote.is_some()))
    }
}

fn special_bytes(language: Language, syntax: Syntax) -> [bool; 256] {
    let mut bytes = [false; 256];
    bytes[b'"' as usize] = true;
    if syntax.single {
        bytes[b'\'' as usize] = true;
    }
    if syntax.backtick {
        bytes[b'`' as usize] = true;
    }
    for token in syntax.lines {
        bytes[token[0] as usize] = true;
    }
    if let Some((open, _)) = syntax.block {
        bytes[open[0] as usize] = true;
    }
    let extras: &[u8] = match language {
        Language::Rust => b"rbc",
        Language::C | Language::Cpp => b"R",
        Language::Lua => b"[",
        Language::Shell => b"<()",
        Language::Zig => b"\\",
        Language::PowerShell => b"@",
        _ => b"",
    };
    for &byte in extras {
        bytes[byte as usize] = true;
    }
    bytes
}

fn regex_position(prefix: &[u8]) -> bool {
    let prefix = prefix.trim_ascii_end();
    if prefix.last().is_none_or(|b| b"=(:,;!?[{}&|".contains(b)) {
        return true;
    }
    let word = prefix
        .rsplit(|b| !b.is_ascii_alphanumeric() && *b != b'_')
        .next()
        .unwrap_or(prefix);
    [
        b"return".as_slice(),
        b"case",
        b"throw",
        b"yield",
        b"typeof",
        b"void",
        b"delete",
    ]
    .contains(&word)
}

fn regex_length(source: &[u8]) -> Result<usize, Error> {
    let mut index = 1;
    let mut class = false;
    while index < source.len() {
        match source[index] {
            b'\\' => index += 2,
            b'[' => {
                class = true;
                index += 1;
            }
            b']' => {
                class = false;
                index += 1;
            }
            b'/' if !class => return Ok(index + 1),
            _ => index += 1,
        }
    }
    Err(coverage("Unterminated regular-expression literal"))
}

fn raw_start(rest: &[u8]) -> Option<(usize, usize)> {
    let offset = if rest.starts_with(b"br") || rest.starts_with(b"cr") {
        2
    } else if rest.starts_with(b"r") {
        1
    } else {
        return None;
    };
    let hashes = rest[offset..].iter().take_while(|&&b| b == b'#').count();
    if hashes > 255 || rest.get(offset + hashes) != Some(&b'"') {
        return None;
    }
    Some((offset + hashes + 1, hashes))
}

fn is_character(rest: &[u8]) -> bool {
    if rest.len() < 3 {
        return false;
    }
    if rest[1] == b'\\' {
        return rest[2..].iter().take(12).any(|&c| c == b'\'');
    }
    let length = if rest[1] < 128 {
        1
    } else if rest[1] < 224 {
        2
    } else if rest[1] < 240 {
        3
    } else {
        4
    };
    rest.get(1 + length) == Some(&b'\'')
}

#[cfg(test)]
mod tests;
