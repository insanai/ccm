use super::Record;
use crate::{Language, coverage};
use ccm_core::{Counts, Error};

#[derive(Default)]
struct Scanner {
    block: Option<(&'static str, &'static str, usize)>,
    quote: Option<(&'static str, bool)>,
}

pub(crate) fn count(source: &str, language: Language) -> Result<Counts, Error> {
    let Language::Catalog(id) = language else {
        return Err(coverage("A declarative profile was not selected"));
    };
    let record = &super::records()[usize::from(id.0)];
    let mut scanner = Scanner::default();
    let mut counts = Counts::default();
    for line in source.split_inclusive('\n') {
        let (code, comment) = scanner.line(line.as_bytes(), record)?;
        if line.trim().is_empty() {
            counts.blank += 1;
        } else if code {
            counts.code += 1;
        } else if comment {
            counts.comments += 1;
        } else {
            counts.blank += 1;
        }
    }
    if scanner
        .block
        .is_some_and(|(_, close, _)| !close.is_empty() && close != "\n")
        || scanner.quote.is_some()
    {
        return Err(coverage(
            "Unterminated literal or comment in declarative language profile",
        ));
    }
    Ok(counts)
}

impl Scanner {
    fn line(&mut self, line: &[u8], r: &Record) -> Result<(bool, bool), Error> {
        let mut code = self.quote.is_some();
        let mut comment = self.block.is_some();
        let mut index = 0;
        while index < line.len() {
            let rest = &line[index..];
            if let Some((open, close, depth)) = self.block {
                comment = true;
                if starts(rest, close, r.insensitive) {
                    self.block = (depth > 1).then_some((open, close, depth - 1));
                    index += close.len();
                } else if r.nested && starts(rest, open, r.insensitive) {
                    if depth >= 256 {
                        return Err(coverage("Comment nesting exceeds 256"));
                    }
                    self.block = Some((open, close, depth + 1));
                    index += open.len();
                } else {
                    index += 1;
                }
            } else if let Some((close, escapes)) = self.quote {
                code = true;
                if starts(rest, close, false) {
                    self.quote = None;
                    index += close.len();
                } else if escapes && rest[0] == b'\\' {
                    index += rest.len().min(2);
                } else {
                    index += 1;
                }
            } else if let Some(&(open, close)) = r
                .blocks
                .iter()
                .find(|(open, _)| starts(rest, open, r.insensitive))
            {
                self.block = Some((open, close, 1));
                comment = true;
                index += open.len();
            } else if r.lines.iter().any(|open| starts(rest, open, r.insensitive)) {
                comment = true;
                break;
            } else if let Some(&(open, close, escapes)) = r
                .quotes
                .iter()
                .find(|(open, _, _)| starts(rest, open, false))
            {
                self.quote = Some((close, escapes));
                code = true;
                index += open.len();
            } else {
                code |= !rest[0].is_ascii_whitespace();
                index += 1;
            }
        }
        Ok((code, comment))
    }
}

fn starts(source: &[u8], token: &str, insensitive: bool) -> bool {
    !token.is_empty()
        && source.get(..token.len()).is_some_and(|prefix| {
            if insensitive {
                prefix.eq_ignore_ascii_case(token.as_bytes())
            } else {
                prefix == token.as_bytes()
            }
        })
}

#[cfg(test)]
mod tests {
    use crate::{languages, measure_source};

    #[test]
    fn representative_declarative_families_protect_comments_and_literals() {
        for (name, source, expected) in [
            ("Ada", "-- note\nX := 1;\n", (1, 1)),
            ("APL", "⍝ note\nX ← '⍝ literal'\n", (1, 1)),
            ("Visual Basic", "' note\nDim s = \"' literal\"\n", (1, 1)),
            (
                "Standard ML",
                "(* outer\n(* inner *) *)\nval x = 1\n",
                (1, 2),
            ),
            ("WebAssembly", ";; note\n(module)\n", (1, 1)),
            ("ANTLR Grammar", "// note\ngrammar Example;\n", (1, 1)),
            ("ATS", "val x = 1\n////\ncomment through EOF\n", (1, 2)),
        ] {
            let language = languages()
                .iter()
                .find(|entry| entry.name == name)
                .unwrap()
                .language;
            let c = measure_source(source.as_bytes(), language).unwrap();
            assert_eq!((c.code, c.comments), expected, "{name}");
        }
    }
}
