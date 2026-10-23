//! Fast byte searches for the scanners' hot loops.
//!
//! We test eight bytes at a time in a `u64` ("SIMD within a register"). The test is plain
//! integer arithmetic, so it is safe and portable, and the first match is found from the
//! lowest set bit without a byte-by-byte loop. Long runs first skip 32-byte blocks with a
//! branch-free test that the compiler turns into vector compares.

/// The low seven bits of every byte.
const LOW: u64 = 0x7f7f_7f7f_7f7f_7f7f;

/// Bytes skipped per step by the block test; two 16-byte vectors on SSE2.
const BLOCK: usize = 32;

/// Returns a word whose byte `i` is 0x80 when byte `i` of `word` is zero, and 0 otherwise.
/// Unlike the shorter `(x - 0x01..) & !x & 0x80..`, this form has no false positives.
fn zero_bytes(word: u64) -> u64 {
    !(((word & LOW).wrapping_add(LOW)) | word | LOW)
}

/// Returns the position of the first byte equal to `a` or `b` in `bytes`, at most 8 long.
fn find_in_word(bytes: &[u8], a: u64, b: u64) -> Option<usize> {
    let mut padded = [0; 8];
    padded[..bytes.len()].copy_from_slice(bytes);
    let word = u64::from_le_bytes(padded);
    let mut hits = zero_bytes(word ^ a) | zero_bytes(word ^ b);
    // The padding may equal a needle, so only the bits of real bytes count.
    if bytes.len() < 8 {
        hits &= (1 << (8 * bytes.len())) - 1;
    }
    (hits != 0).then(|| hits.trailing_zeros() as usize / 8)
}

/// Returns the index of the first byte of `haystack` that equals `a` or `b`.
pub(crate) fn find_either(haystack: &[u8], a: u8, b: u8) -> Option<usize> {
    let mut start = 0;
    // Skip whole blocks without a match, as in a long comment.
    for block in haystack.chunks_exact(BLOCK) {
        if block
            .iter()
            .fold(false, |found, &byte| found | (byte == a) | (byte == b))
        {
            break;
        }
        start += BLOCK;
    }
    let (a, b) = (u64::from_le_bytes([a; 8]), u64::from_le_bytes([b; 8]));
    for word in haystack[start..].chunks(8) {
        if let Some(index) = find_in_word(word, a, b) {
            return Some(start + index);
        }
        start += word.len();
    }
    None
}

/// Returns the index of the first byte of `haystack` that equals `needle`.
pub(crate) fn find(haystack: &[u8], needle: u8) -> Option<usize> {
    find_either(haystack, needle, needle)
}

/// Splits text into physical lines like `str::lines`, but only at LF; a CR before the LF
/// is removed and a lone CR stays in the line. A final segment without LF is a line.
pub(crate) fn lines(text: &str) -> impl Iterator<Item = &str> {
    let mut rest = text;
    std::iter::from_fn(move || {
        if rest.is_empty() {
            return None;
        }
        let end = find(rest.as_bytes(), b'\n').map_or(rest.len(), |index| index + 1);
        let (line, tail) = rest.split_at(end);
        rest = tail;
        let line = line.strip_suffix('\n').unwrap_or(line);
        Some(line.strip_suffix('\r').unwrap_or(line))
    })
}

/// Returns true when `haystack` begins with `prefix`. Our delimiters are one to a few
/// bytes, so comparing them inline is cheaper than the library call behind `starts_with`.
pub(crate) fn starts(haystack: &[u8], prefix: &[u8]) -> bool {
    haystack.len() >= prefix.len() && haystack.iter().zip(prefix).all(|(a, b)| a == b)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn searches_agree_with_a_plain_scan() {
        let text: Vec<u8> = (0..200u8).map(|i| b'a' + i % 23).collect();
        for needle in *b"akw*\n" {
            for start in [0, 1, 31, 32, 33, 64, 150, 199, 200] {
                let haystack = &text[start..];
                let expected = haystack.iter().position(|&b| b == needle);
                assert_eq!(find(haystack, needle), expected, "{needle} at {start}");
                let either = haystack.iter().position(|&b| b == needle || b == b'q');
                assert_eq!(find_either(haystack, needle, b'q'), either);
            }
        }
        assert!(starts(b"*/ rest", b"*/"));
        assert!(!starts(b"*", b"*/"));
        assert!(starts(b"x", b""));
    }
}
