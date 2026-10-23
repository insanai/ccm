use std::path::{Path, PathBuf};

pub(super) fn check(soft: bool) -> Result<(), String> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut stack = vec![root];
    let mut failures = Vec::new();
    while let Some(path) = stack.pop() {
        let metadata = std::fs::symlink_metadata(&path).map_err(|e| e.to_string())?;
        if metadata.file_type().is_symlink() {
            continue;
        }
        if metadata.is_dir() {
            if [".git", ".jj", "target", "_build", "docs"]
                .iter()
                .any(|n| path.ends_with(n))
            {
                continue;
            }
            for entry in std::fs::read_dir(path).map_err(|e| e.to_string())? {
                stack.push(entry.map_err(|e| e.to_string())?.path());
            }
        } else if matches!(
            path.extension().and_then(|s| s.to_str()),
            Some("rs" | "toml" | "html" | "css" | "js" | "sh")
        ) {
            check_file(&path, soft, &mut failures)?;
        }
    }
    if failures.is_empty() {
        Ok(())
    } else {
        Err(failures.join("\n"))
    }
}

fn check_file(path: &PathBuf, soft: bool, failures: &mut Vec<String>) -> Result<(), String> {
    let source = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    if source.lines().count() > 1408 {
        failures.push(format!("{}: exceeds 1408 lines", path.display()));
    }
    for (index, line) in source.lines().enumerate() {
        if line.chars().count() > if soft { 99 } else { 108 } {
            failures.push(format!(
                "{}:{}: line width exceeds limit",
                path.display(),
                index + 1
            ));
        }
    }
    if path.extension().is_some_and(|s| s == "rs") {
        function_limits(&source, path, failures);
    }
    Ok(())
}

fn function_limits(source: &str, path: &Path, failures: &mut Vec<String>) {
    let masked = mask_literals(source.as_bytes());
    let text = String::from_utf8_lossy(&masked);
    let mut pending = None;
    let mut depth = 0usize;
    let mut active: Vec<(usize, usize, usize)> = Vec::new();
    for (index, line) in text.lines().enumerate() {
        let tokens: Vec<_> = line
            .split(|c: char| !(c.is_alphanumeric() || c == '_'))
            .collect();
        if tokens.contains(&"fn") {
            pending = Some(index + 1);
        }
        let logical = line
            .chars()
            .any(|c| !c.is_whitespace() && !"{}[],;".contains(c));
        if logical {
            for (_, _, count) in &mut active {
                *count += 1;
            }
        }
        for byte in line.bytes() {
            if byte == b'{' {
                depth += 1;
                if let Some(start) = pending.take() {
                    active.push((depth, start, 0));
                }
            } else if byte == b'}' {
                if active.last().is_some_and(|(d, _, _)| *d == depth) {
                    if let Some((_, start, count)) = active.pop() {
                        if count > 70 {
                            failures.push(format!(
                                "{}:{start}: function has {count} logical lines",
                                path.display()
                            ));
                        }
                    }
                }
                depth = depth.saturating_sub(1);
            } else if byte == b';' {
                pending = None;
            }
        }
    }
}

// Preserve offsets/newlines while excluding strings, character literals, and nested comments.
fn mask_literals(source: &[u8]) -> Vec<u8> {
    let mut out = source.to_vec();
    let mut index = 0;
    while index < source.len() {
        let rest = &source[index..];
        let length = if rest.starts_with(b"//") {
            rest.iter().position(|&b| b == b'\n').unwrap_or(rest.len())
        } else if rest.starts_with(b"/*") {
            block_length(rest)
        } else if let Some(length) = raw_length(rest) {
            length
        } else if rest[0] == b'"' {
            quote_length(rest, b'"')
        } else if rest[0] == b'\'' && char_literal(rest) {
            quote_length(rest, b'\'')
        } else {
            index += 1;
            continue;
        };
        for byte in &mut out[index..index + length] {
            if *byte != b'\n' {
                *byte = b' ';
            }
        }
        index += length;
    }
    out
}

fn block_length(source: &[u8]) -> usize {
    let mut depth = 1;
    let mut index = 2;
    while index + 1 < source.len() {
        if source[index..].starts_with(b"/*") {
            depth += 1;
            index += 2;
        } else if source[index..].starts_with(b"*/") {
            depth -= 1;
            index += 2;
            if depth == 0 {
                return index;
            }
        } else {
            index += 1;
        }
    }
    source.len()
}

fn quote_length(source: &[u8], quote: u8) -> usize {
    let mut index = 1;
    while index < source.len() {
        if source[index] == b'\\' {
            index = (index + 2).min(source.len());
        } else if source[index] == quote {
            return index + 1;
        } else {
            index += 1;
        }
    }
    source.len()
}

fn char_literal(source: &[u8]) -> bool {
    if source.get(1) == Some(&b'\\') {
        return true;
    }
    let length = source.get(1).map_or(1, |&b| {
        if b < 128 {
            1
        } else if b < 224 {
            2
        } else if b < 240 {
            3
        } else {
            4
        }
    });
    source.get(1 + length) == Some(&b'\'')
}

fn raw_length(source: &[u8]) -> Option<usize> {
    if source[0] != b'r' {
        return None;
    }
    let hashes = source[1..].iter().take_while(|&&b| b == b'#').count();
    if source.get(1 + hashes) != Some(&b'"') {
        return None;
    }
    let start = hashes + 2;
    for index in start..source.len() {
        if source[index] == b'"' && source[index + 1..].starts_with(&vec![b'#'; hashes]) {
            return Some(index + 1 + hashes);
        }
    }
    Some(source.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lexer_ignores_delimiters_in_nested_comments_raw_and_escaped_strings() {
        let source = br###"fn f() { let s = r##" } fn fake() { "##; /* /* } */ } */
            let c = '}'; let s = "\\\"}"; }"###;
        let masked = mask_literals(source);
        let text = String::from_utf8(masked).unwrap();
        assert_eq!(text.matches("fn").count(), 1);
        assert_eq!(text.matches('{').count(), 1);
        assert_eq!(text.matches('}').count(), 1);
    }
}
