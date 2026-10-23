use crate::bytes::{find, find_either, starts};
use crate::syntax::{Syntax, syntax};
use crate::{Language, coverage, is_blank};
use ccm_core::{Counts, Error};
use std::collections::VecDeque;

mod perl;

#[derive(Default)]
enum State {
    #[default]
    Normal,
    Block(usize),
    Quote(u8, bool),
    Delimited(Vec<u8>, bool),
    /// A Perl quote-like operator or pattern.
    Perl(perl::Quote),
}

/// Which leading whitespace a heredoc terminator line may carry.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Indent {
    None,
    /// Shell `<<-`: leading tabs.
    Tabs,
    /// Perl `<<~`: any leading whitespace.
    Any,
}

#[derive(Default)]
struct Lexer {
    state: State,
    heredocs: VecDeque<(Vec<u8>, Indent)>,
    continued_comment: bool,
    arithmetic_depth: usize,
    /// The line that ends an open Perl POD or Ruby `=begin` block.
    pod_end: Option<&'static [u8]>,
    /// After Perl or Ruby `__END__`, the rest of the file is data.
    data: bool,
    /// The current line number, and the line on which the open literal or comment began.
    line_number: usize,
    opened_on: usize,
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
    for line in crate::bytes::lines(source) {
        lexer.line_number += 1;
        let (code, comment) = if matches!(
            language,
            Language::Text | Language::ReStructuredText | Language::Diff | Language::Csv
        ) {
            (true, false)
        } else {
            lexer.line(line.as_bytes(), language, policy, &special)?
        };
        // A physically empty line is blank even inside a string, heredoc, or comment.
        let (code, comment) = if is_blank(line) {
            (false, false)
        } else {
            (code, comment)
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
        let what = match lexer.state {
            State::Block(_) | State::Delimited(_, true) => "block comment",
            State::Normal if lexer.arithmetic_depth > 0 => "arithmetic expression",
            State::Normal => "heredoc",
            _ => "string",
        };
        return Err(coverage(&format!(
            "Unterminated {what} that begins on line {}",
            lexer.opened_on
        )));
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
        if let Some((delimiter, indent)) = self.heredocs.front() {
            let content = match indent {
                Indent::None => line,
                Indent::Tabs => &line[line.iter().take_while(|&&b| b == b'\t').count()..],
                Indent::Any => line.trim_ascii_start(),
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
        if let Some(classes) = self.documentation_line(line, language) {
            return Ok(classes);
        }
        let mut code = matches!(
            self.state,
            State::Quote(..) | State::Delimited(_, false) | State::Perl(_)
        );
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
                    // Jump to the next byte that can begin a closing or nested delimiter.
                    let opening = if syntax.nested { open[0] } else { close[0] };
                    let Some(skip) = find_either(rest, close[0], opening) else {
                        break;
                    };
                    index += skip;
                    let rest = &line[index..];
                    if starts(rest, close) {
                        *depth -= 1;
                        index += close.len();
                        if *depth == 0 {
                            self.state = State::Normal;
                        }
                    } else if syntax.nested && starts(rest, open) {
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
                    let escape = if *escapes { b'\\' } else { *quote };
                    let Some(skip) = find_either(rest, *quote, escape) else {
                        break;
                    };
                    index += skip;
                    if line[index] == b'\\' && *escapes {
                        index += (line.len() - index).min(2);
                    } else {
                        self.state = State::Normal;
                        index += 1;
                    }
                }
                State::Delimited(close, is_comment) => {
                    if *is_comment {
                        comment = true;
                    } else {
                        code = true;
                    }
                    let Some(skip) = find(rest, close[0]) else {
                        break;
                    };
                    index += skip;
                    if starts(&line[index..], close) {
                        index += close.len();
                        self.state = State::Normal;
                    } else {
                        index += 1;
                    }
                }
                State::Perl(quote) => {
                    code = true;
                    let (length, closed) = quote.step(rest);
                    index += length;
                    if closed {
                        self.state = State::Normal;
                    }
                }
                State::Normal => {
                    let (heredocs, arithmetic) = (self.heredocs.len(), self.arithmetic_depth);
                    let (advance, has_code, has_comment, stop) = if self.work.enabled {
                        self.normal_work(line, index, language, syntax)?
                    } else {
                        self.normal(line, index, language, syntax)?
                    };
                    if !matches!(self.state, State::Normal)
                        || self.heredocs.len() > heredocs
                        || self.arithmetic_depth > arithmetic
                    {
                        self.opened_on = self.line_number;
                    }
                    code |= has_code;
                    comment |= has_comment;
                    index += advance;
                    if stop {
                        break;
                    }
                }
            }
        }
        if let State::Perl(quote) = self.state {
            if quote.is_bare() {
                self.state = State::Normal;
            }
        }
        // An ordinary literal cannot cross a line end without a backslash in these
        // languages. Closing it here recovers from stray quotes, as in `#error isn't`.
        if let State::Quote(b'\'' | b'"', _) = self.state {
            if syntax.single_line_quotes && !line.ends_with(b"\\") {
                self.state = State::Normal;
            }
        }
        Ok((code, comment))
    }

    /// Classifies Perl POD, Ruby `=begin` blocks, and data after `__END__`. Returns
    /// `None` for an ordinary line.
    fn documentation_line(&mut self, line: &[u8], language: Language) -> Option<(bool, bool)> {
        if !matches!(language, Language::Perl | Language::Ruby) {
            return None;
        }
        if let Some(end) = self.pod_end {
            if line.starts_with(end) {
                self.pod_end = None;
            }
            return Some((false, true));
        }
        if !matches!(self.state, State::Normal) {
            return None;
        }
        let pod = match language {
            Language::Perl => {
                line.first() == Some(&b'=') && line.get(1).is_some_and(u8::is_ascii_alphabetic)
            }
            _ => line.starts_with(b"=begin"),
        };
        if pod {
            self.pod_end = Some(if language == Language::Perl {
                b"=cut"
            } else {
                b"=end"
            });
            return Some((false, true));
        }
        if self.data {
            return Some((true, false));
        }
        let end = line.trim_ascii_end();
        if end == b"__END__" || (language == Language::Perl && end == b"__DATA__") {
            self.data = true;
            return Some((true, false));
        }
        None
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
            if matches!(
                self.state,
                State::Quote(..) | State::Delimited(_, false) | State::Perl(_)
            ) || self.heredocs.len() > heredocs
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
        if let Some(length) = self.special(rest, &line[..index], language)? {
            let comment = matches!(self.state, State::Delimited(_, true));
            return Ok((length, !comment, comment, false));
        }
        if let Some((open, _)) = syntax.block.filter(|(a, _)| starts(rest, a)) {
            self.state = State::Block(1);
            return Ok((open.len(), false, true, false));
        }
        if syntax.lines.iter().any(|token| starts(rest, token))
            && !(language == Language::Shell
                && index > 0
                && !b" \t;|&()".contains(&line[index - 1]))
            // `$#array` and `$#{list}` are Perl expressions, not comments.
            && !(language == Language::Perl && index > 0 && line[index - 1] == b'$')
            // A YAML comment must be separated from preceding content by whitespace.
            && !(language == Language::Yaml && index > 0 && !b" \t".contains(&line[index - 1]))
        {
            self.continued_comment =
                matches!(language, Language::C | Language::Cpp) && line.ends_with(b"\\");
            return Ok((rest.len(), false, true, true));
        }
        if language == Language::Yaml && index > 0 && !b" \t:[{,-?".contains(&line[index - 1]) {
            return Ok((1, !rest[0].is_ascii_whitespace(), false, false));
        }
        let line_start = line[..index].iter().all(u8::is_ascii_whitespace);
        let length = self.start_quote(rest, language, syntax, line_start);
        let docstring = matches!(self.state, State::Delimited(_, true));
        Ok((
            length,
            !docstring && !rest[0].is_ascii_whitespace(),
            docstring,
            false,
        ))
    }

    /// Opens a string at `rest`, if one starts there, and returns the bytes consumed.
    /// A triple-quoted Python or Julia string that begins a line is a docstring, which
    /// we count as a comment.
    fn start_quote(
        &mut self,
        rest: &[u8],
        language: Language,
        syntax: Syntax,
        line_start: bool,
    ) -> usize {
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
        if (quote == b'"' && syntax.double)
            || (quote == b'\'' && syntax.single)
            || (quote == b'`' && syntax.backtick)
        {
            if syntax.triple && rest.starts_with(&[quote; 3]) {
                let docstring =
                    line_start && matches!(language, Language::Python | Language::Julia);
                self.state = State::Delimited(vec![quote; 3], docstring);
                return 3;
            }
            self.state = State::Quote(
                quote,
                quote != b'`' && !(quote == b'\'' && syntax.literal_single),
            );
        }
        1
    }

    /// Opens a language-specific literal at `rest`; `prefix` is the line before it.
    fn special(
        &mut self,
        rest: &[u8],
        prefix: &[u8],
        language: Language,
    ) -> Result<Option<usize>, Error> {
        if language == Language::Rust && token_start(prefix) {
            if let Some((length, hashes)) = raw_start(rest) {
                let mut close = vec![b'"'];
                close.resize(1 + hashes, b'#');
                self.state = State::Delimited(close, false);
                return Ok(Some(length));
            }
        }
        if matches!(language, Language::C | Language::Cpp)
            && rest.starts_with(b"R\"")
            && raw_string_prefix(prefix)
        {
            // A delimiter has at most 16 characters and no spaces, parentheses, or
            // backslashes. Anything else is an ordinary string after an identifier `R`.
            let delimiter = rest[2..]
                .iter()
                .take(17)
                .position(|&b| b == b'(')
                .filter(|&end| !rest[2..2 + end].iter().any(|b| b" )\\\t\"".contains(b)));
            if let Some(end) = delimiter {
                let mut close = vec![b')'];
                close.extend_from_slice(&rest[2..2 + end]);
                close.push(b'"');
                self.state = State::Delimited(close, false);
                return Ok(Some(end + 3));
            }
        }
        if matches!(language, Language::Lua | Language::CMake) {
            // Lua long brackets `[==[` and `--[==[`; CMake bracket arguments and `#[==[`.
            let marker: &[u8] = if language == Language::Lua {
                b"--["
            } else {
                b"#["
            };
            let (prefix, comment) = if rest.starts_with(marker) {
                (marker.len() - 1, true)
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
        self.special_more(rest, prefix, language)
    }

    /// Continues `special` for the remaining languages with their own literals.
    fn special_more(
        &mut self,
        rest: &[u8],
        prefix: &[u8],
        language: Language,
    ) -> Result<Option<usize>, Error> {
        if language == Language::Shell && rest.starts_with(b"<<") && !rest.starts_with(b"<<<") {
            return self.heredoc(rest).map(Some);
        }
        if language == Language::Perl {
            // Punctuation variables such as `$'`, `$"`, and `$;` hold no quote or comment.
            if rest[0] == b'$' && rest.get(1).is_some_and(u8::is_ascii_punctuation) {
                return Ok(Some(2));
            }
            if let Some((length, quote)) = perl::start(rest, prefix) {
                self.state = State::Perl(quote);
                return Ok(Some(length));
            }
            if rest.starts_with(b"<<") {
                return self.perl_heredoc(rest);
            }
        }
        // Character literals: Scheme `#\"` and Clojure `\"` do not open strings.
        if language == Language::Scheme && rest.starts_with(b"#\\") {
            return Ok(Some(rest.len().min(3)));
        }
        if language == Language::Clojure && rest[0] == b'\\' {
            return Ok(Some(rest.len().min(2)));
        }
        // A TeX control symbol such as `\%`, or a shell or CMake escape such as `\"`
        // outside quotes, is text: it opens no comment or string.
        if matches!(language, Language::TeX | Language::Shell | Language::CMake)
            && rest[0] == b'\\'
        {
            return Ok(Some(rest.len().min(2)));
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

    /// Opens a Perl heredoc such as `<<EOF`, `<<"EOF"`, or `<<~'EOF'`. A space after
    /// `<<` means a left shift.
    fn perl_heredoc(&mut self, rest: &[u8]) -> Result<Option<usize>, Error> {
        let indent = rest.get(2) == Some(&b'~');
        let start = 2 + usize::from(indent);
        let next = rest.get(start).copied().unwrap_or(b' ');
        if !(next == b'"' || next == b'\'' || next.is_ascii_alphabetic() || next == b'_') {
            return Ok(Some(2));
        }
        let length =
            self.heredoc_at(rest, start, if indent { Indent::Any } else { Indent::None })?;
        Ok(Some(length))
    }

    fn heredoc(&mut self, rest: &[u8]) -> Result<usize, Error> {
        let tabs = rest.get(2) == Some(&b'-');
        let indent = if tabs { Indent::Tabs } else { Indent::None };
        self.heredoc_at(rest, if tabs { 3 } else { 2 }, indent)
    }

    /// Reads a heredoc delimiter that starts at `index` and queues it.
    fn heredoc_at(
        &mut self,
        rest: &[u8],
        mut index: usize,
        indent: Indent,
    ) -> Result<usize, Error> {
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
        // `<<\EOF` quotes the delimiter like `<<'EOF'`.
        if quote.is_none() && rest.get(index) == Some(&b'\\') {
            index += 1;
        }
        let start = index;
        while let Some(&byte) = rest.get(index) {
            if quote == Some(byte) || (quote.is_none() && b" \t;|&<>),".contains(&byte)) {
                break;
            }
            index += 1;
        }
        if index == start || index - start > 256 || self.heredocs.len() >= 16 {
            return Err(coverage("Invalid or oversized heredoc delimiter"));
        }
        self.heredocs
            .push_back((rest[start..index].to_vec(), indent));
        Ok(index + usize::from(quote.is_some()))
    }
}

fn special_bytes(language: Language, syntax: Syntax) -> [bool; 256] {
    let mut bytes = [false; 256];
    bytes[b'"' as usize] = syntax.double;
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
        Language::Shell => b"<()\\",
        Language::Zig | Language::TeX => b"\\",
        Language::PowerShell => b"@",
        Language::Perl => b"qmsty/<$",
        Language::CMake => b"[\\",
        Language::Scheme => b"#",
        Language::Clojure => b"\\",
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

/// Returns true when a literal prefix such as `r` or `R` starts a new token here.
fn token_start(prefix: &[u8]) -> bool {
    prefix
        .last()
        .is_none_or(|&b| !b.is_ascii_alphanumeric() && b != b'_')
}

/// Returns true when `R"` begins a C++ raw string: at a token start, or after one of
/// the encoding prefixes `L`, `u`, `U`, and `u8`.
fn raw_string_prefix(prefix: &[u8]) -> bool {
    let word = prefix
        .rsplit(|&b| !b.is_ascii_alphanumeric() && b != b'_')
        .next()
        .unwrap_or(prefix);
    matches!(word, b"" | b"L" | b"u" | b"U" | b"u8")
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
