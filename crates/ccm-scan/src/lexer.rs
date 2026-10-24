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
    /// Shell substitutions opened inside a double-quoted string, innermost last, each with
    /// its parenthesis depth and whether it is arithmetic, `$((...))`, where `<<` is a
    /// shift. Closing one returns to the enclosing string.
    substitutions: Vec<(usize, bool)>,
    /// JavaScript template substitutions, `${...}`, innermost last, each with its brace
    /// depth. Closing one returns to the enclosing template literal.
    templates: Vec<usize>,
    /// The indentation of the line that opened a YAML block scalar, `key: |`. Deeper
    /// indented lines that follow are literal text.
    yaml_block: Option<usize>,
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
        || !lexer.substitutions.is_empty()
        || !lexer.templates.is_empty()
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
        if let Some(classes) = self.yaml_block_text(line) {
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
                State::Quote(..) => {
                    code = true;
                    let Some(next) = self.quoted(line, index, language)? else {
                        break;
                    };
                    index = next;
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
                        || (arithmetic == 0 && self.arithmetic_depth > 0)
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
        if language == Language::Yaml
            && matches!(self.state, State::Normal)
            && opens_yaml_block(line)
        {
            self.yaml_block = Some(indentation(line));
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

    /// Classifies a line inside an open YAML block scalar, where quotes are text. A line
    /// that starts with `#` counts as a comment: such blocks usually embed a script, as in
    /// a CI `run: |` step, and other counters agree. Returns `None` for a line that is not
    /// indented deeper than the line that opened the block, which ends it. Whitespace-only
    /// lines stay inside, and the caller counts them as blank.
    fn yaml_block_text(&mut self, line: &[u8]) -> Option<(bool, bool)> {
        let parent = self.yaml_block?;
        let text = line.trim_ascii();
        if text.is_empty() || indentation(line) > parent {
            let comment = text.first() == Some(&b'#');
            return Some((!comment, comment));
        }
        self.yaml_block = None;
        None
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
                || (regex && result.0 > 1)
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
            if let Some(length) = self.shell_arithmetic(rest)? {
                return Ok((length, true, false, false));
            }
            if let Some(length) = self.shell_substitution(rest)? {
                return Ok((length, true, false, false));
            }
        }
        if let Some(length) = self.template_brace(rest[0]) {
            return Ok((length, true, false, false));
        }
        if matches!(language, Language::JavaScript | Language::TypeScript)
            && rest[0] == b'/'
            && !rest.starts_with(b"//")
            && !rest.starts_with(b"/*")
            && regex_position(&line[..index])
        {
            // A regular-expression literal cannot span lines, so a `/` that does not close
            // on its line is a division or part of JSX markup, as in `<Item />`.
            return Ok((regex_length(rest).unwrap_or(1), true, false, false));
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
        if language == Language::Yaml
            && matches!(rest[0], b'\'' | b'"')
            && !yaml_scalar_start(&line[..index])
        {
            return Ok((1, true, false, false));
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

    /// Advances through the quoted string that is open at `index` and returns the next
    /// index, or `None` when the string continues past the end of the line. In a Shell
    /// double-quoted string, `$(` opens a command substitution in which quoting starts
    /// afresh, as in `"$(echo "$i" | wc -c)"`.
    fn quoted(
        &mut self,
        line: &[u8],
        index: usize,
        language: Language,
    ) -> Result<Option<usize>, Error> {
        let State::Quote(quote, escapes) = self.state else {
            return Ok(Some(index));
        };
        let rest = &line[index..];
        let substitutions = language == Language::Shell && quote == b'"';
        let template =
            quote == b'`' && matches!(language, Language::JavaScript | Language::TypeScript);
        // PowerShell escapes with a backtick, so `"C:\"` is a whole string.
        let escape = if language == Language::PowerShell {
            b'`'
        } else {
            b'\\'
        };
        let found = if substitutions || template {
            rest.iter()
                .position(|&byte| byte == quote || matches!(byte, b'\\' | b'$'))
        } else {
            find_either(rest, quote, if escapes { escape } else { quote })
        };
        let Some(skip) = found else {
            return Ok(None);
        };
        let index = index + skip;
        let rest = &line[index..];
        if rest[0] == escape && escapes {
            return Ok(Some(index + rest.len().min(2)));
        }
        if rest[0] == b'$' && template {
            if !rest.starts_with(b"${") {
                return Ok(Some(index + 1));
            }
            if self.templates.len() >= 256 {
                return Err(coverage("Template substitution nesting exceeds 256"));
            }
            self.templates.push(1);
            self.state = State::Normal;
            return Ok(Some(index + 2));
        }
        if rest[0] == b'$' {
            if !rest.starts_with(b"$(") {
                return Ok(Some(index + 1));
            }
            if self.substitutions.len() >= 256 {
                return Err(coverage("Command substitution nesting exceeds 256"));
            }
            self.substitutions.push((1, rest.starts_with(b"$((")));
            self.state = State::Normal;
            return Ok(Some(index + 2));
        }
        self.state = State::Normal;
        Ok(Some(index + 1))
    }

    /// Tracks the braces of a JavaScript template substitution, `${...}`, which may hold
    /// any expression, nested templates included. Its closing `}` returns to the template.
    /// Returns the bytes consumed, or `None` when `byte` is not a brace we track.
    fn template_brace(&mut self, byte: u8) -> Option<usize> {
        let depth = self.templates.last_mut()?;
        match byte {
            // Each frame began with depth 1, so 256 frames of 256 braces bound the work.
            b'{' => *depth = (*depth + 1).min(256),
            b'}' => {
                *depth -= 1;
                if *depth == 0 {
                    self.templates.pop();
                    self.state = State::Quote(b'`', true);
                }
            }
            _ => return None,
        }
        Some(1)
    }

    /// Tracks the parentheses of a Shell command substitution that began inside a
    /// double-quoted string. Its closing `)` returns to that string. Returns the bytes
    /// consumed, or `None` when `rest` does not start with a parenthesis we track.
    fn shell_substitution(&mut self, rest: &[u8]) -> Result<Option<usize>, Error> {
        let Some((depth, arithmetic)) = self.substitutions.last_mut() else {
            return Ok(None);
        };
        if *arithmetic && rest.starts_with(b"<<") {
            return Ok(Some(2));
        }
        match rest[0] {
            b'(' if *depth >= 256 => return Err(coverage("Parenthesis nesting exceeds 256")),
            b'(' => *depth += 1,
            b')' => {
                *depth -= 1;
                if *depth == 0 {
                    self.substitutions.pop();
                    self.state = State::Quote(b'"', true);
                }
            }
            _ => return Ok(None),
        }
        Ok(Some(1))
    }

    /// Tracks Shell arithmetic, `((...))` and `$((...))`, by parenthesis depth, so that
    /// `$(( (a & (b << c))) ))` and nested subshells such as `((( cmd ) ))` balance as
    /// they do in the shell. Inside arithmetic `<<` is a shift, not a heredoc. Returns
    /// the bytes consumed, or `None` when `rest` starts no arithmetic token.
    fn shell_arithmetic(&mut self, rest: &[u8]) -> Result<Option<usize>, Error> {
        if self.arithmetic_depth == 0 {
            if rest.starts_with(b"((") {
                self.arithmetic_depth = 2;
                return Ok(Some(2));
            }
            return Ok(rest.starts_with(b"<<<").then_some(3));
        }
        match rest[0] {
            b'(' => {
                self.arithmetic_depth += 1;
                if self.arithmetic_depth > 256 {
                    return Err(coverage("Arithmetic nesting exceeds 256"));
                }
                Ok(Some(1))
            }
            b')' => {
                self.arithmetic_depth -= 1;
                Ok(Some(1))
            }
            _ if rest.starts_with(b"<<") => Ok(Some(2)),
            _ => Ok(None),
        }
    }

    /// Opens a string at `rest`, if one starts there, and returns the bytes consumed.
    /// A triple-quoted Python, Starlark, or Julia string that begins a line is a docstring,
    /// which we count as a comment.
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
                let docstring = line_start
                    && matches!(
                        language,
                        Language::Python | Language::Starlark | Language::Julia
                    );
                self.state = State::Delimited(vec![quote; 3], docstring);
                return 3;
            }
            // Go and Odin backtick strings are raw; JavaScript templates have escapes.
            let escapes = match quote {
                b'`' => matches!(language, Language::JavaScript | Language::TypeScript),
                b'\'' => !syntax.literal_single,
                _ => true,
            };
            self.state = State::Quote(quote, escapes);
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
        // ANSI-C quoting, `$'...'`, honours backslash escapes such as `\'`.
        if language == Language::Shell && rest.starts_with(b"$'") {
            self.state = State::Quote(b'\'', true);
            return Ok(Some(2));
        }
        // BuildKit heredocs, `RUN <<EOF` and `COPY <<'EOF' file`, follow the shell's rules.
        if language == Language::Dockerfile && rest.starts_with(b"<<") {
            return self.heredoc(rest).map(Some);
        }
        if language == Language::Ruby && rest.starts_with(b"<<") {
            return self.ruby_heredoc(rest).map(Some);
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

    /// Opens a Ruby heredoc: `<<~ID` and `<<-ID` end at an indented `ID`, `<<ID` at one
    /// in the first column, and the delimiter may be quoted. A bare `<<ID` needs an
    /// upper-case `ID`, and `<<` before a space is the append operator, as in `a << b`.
    /// An unquoted delimiter ends at the identifier, as in `<<~EOS.strip`.
    fn ruby_heredoc(&mut self, rest: &[u8]) -> Result<usize, Error> {
        let indented = matches!(rest.get(2), Some(b'~' | b'-'));
        let start = 2 + usize::from(indented);
        let indent = if indented { Indent::Any } else { Indent::None };
        let identifier = |byte: &u8| byte.is_ascii_alphanumeric() || *byte == b'_';
        match rest.get(start) {
            Some(b'"' | b'\'') => self.heredoc_at(rest, start, indent),
            Some(byte) if byte.is_ascii_uppercase() || (indented && identifier(byte)) => {
                let end = rest[start..]
                    .iter()
                    .position(|byte| !identifier(byte))
                    .map_or(rest.len(), |length| start + length);
                if end - start > 256 || self.heredocs.len() >= 16 {
                    return Err(coverage("Invalid or oversized heredoc delimiter"));
                }
                self.heredocs.push_back((rest[start..end].to_vec(), indent));
                Ok(end)
            }
            _ => Ok(2),
        }
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

/// Returns the number of leading spaces and tabs of `line`.
fn indentation(line: &[u8]) -> usize {
    line.iter()
        .take_while(|&&byte| byte == b' ' || byte == b'\t')
        .count()
}

/// Returns true when a YAML line ends with a block scalar indicator, as in `key: |`,
/// `- >-`, or `text: |2 # comment`.
fn opens_yaml_block(line: &[u8]) -> bool {
    // A comment begins at a `#` that follows whitespace.
    let end = line
        .windows(2)
        .position(|pair| pair[0].is_ascii_whitespace() && pair[1] == b'#')
        .unwrap_or(line.len());
    let text = line[..end].trim_ascii_end();
    let start = text
        .iter()
        .rposition(u8::is_ascii_whitespace)
        .map_or(0, |index| index + 1);
    let (head, token) = (text[..start].trim_ascii_end(), &text[start..]);
    let indicator = matches!(token.first(), Some(b'|' | b'>'))
        && token.len() <= 3
        && token[1..]
            .iter()
            .all(|&byte| matches!(byte, b'+' | b'-' | b'0'..=b'9'));
    indicator
        && (head.is_empty() || head.ends_with(b":") || head.ends_with(b"-") || head == b"---")
}

/// Returns true when a YAML scalar can begin after `prefix`, the line before a quote. A
/// quote starts a string only there: at the start of a value, after `key: `, `- `, or
/// `? `, or inside a flow collection after `[`, `{`, `,`, or `:`. Elsewhere it is text in
/// a plain scalar, as in `auto-negotiation events':' Page received`.
fn yaml_scalar_start(prefix: &[u8]) -> bool {
    let before = prefix.trim_ascii_end();
    let spaced = before.len() < prefix.len();
    let Some((&last, earlier)) = before.split_last() else {
        return true;
    };
    let in_flow = before.iter().any(|&byte| matches!(byte, b'[' | b'{'));
    match last {
        b'[' | b'{' | b',' => true,
        b':' => spaced || in_flow,
        // An indicator stands alone: `- 'x'` starts an item, `a - 'b'` is plain text.
        b'-' | b'?' => spaced && earlier.last().is_none_or(u8::is_ascii_whitespace),
        _ => false,
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
        Language::Shell => b"<()\\$",
        Language::Ruby | Language::Dockerfile => b"<",
        Language::JavaScript | Language::TypeScript => b"{}",
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
    // A `!` after an operand is TypeScript's non-null assertion, as in `ratio! / 2`, and
    // an operand ends there, so `/` divides.
    if let [.., before, b'!'] = prefix {
        if before.is_ascii_alphanumeric() || matches!(before, b'_' | b')' | b']') {
            return false;
        }
    }
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

/// Returns the length of the regular-expression literal that starts `source`, or `None`
/// when it does not close on this line.
fn regex_length(source: &[u8]) -> Option<usize> {
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
            b'/' if !class => return Some(index + 1),
            _ => index += 1,
        }
    }
    None
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
