//! Deterministic counts and finite change measurements. No filesystem or presentation policy.
mod ai;
mod behavior;
mod estimate;
mod model;
mod parse;

pub use ai::*;
pub use behavior::*;
pub use estimate::*;
pub use model::*;
pub use parse::parse_model;

use std::fmt;

/// A recoverable measurement failure. Libraries never print it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Error {
    /// Stable category for machine consumers.
    pub code: &'static str,
    /// Plain explanation, including a location when available.
    pub message: String,
    /// Suggested corrective action.
    pub hint: &'static str,
    /// Optional source location for input diagnostics.
    pub location: Option<Location>,
}

/// A source excerpt and caret position for an input error.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Location {
    /// Optional input filename, attached by the I/O adapter.
    pub path: Option<String>,
    /// One-based physical line number.
    pub line: usize,
    /// One-based character column.
    pub column: usize,
    /// Bounded source excerpt; presentation must escape controls.
    pub excerpt: String,
}

impl Error {
    /// Creates an owned diagnostic from a borrowed category and hint.
    pub fn new(code: &'static str, message: impl Into<String>, hint: &'static str) -> Self {
        Self {
            code,
            message: message.into(),
            hint,
            location: None,
        }
    }

    /// Attaches a borrowed source excerpt, retaining at most 200 characters.
    pub fn at_source(mut self, line: usize, column: usize, source: &str) -> Self {
        self.location = Some(Location {
            path: None,
            line,
            column,
            excerpt: source.chars().take(200).collect(),
        });
        self
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}

impl std::error::Error for Error {}

/// Disjoint physical line classes. A line with code and a comment counts as code.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Counts {
    /// Input bytes, including line endings and a possible UTF-8 BOM.
    pub bytes: u64,
    /// Lines containing source text.
    pub code: u64,
    /// Lines containing only comments and whitespace.
    pub comments: u64,
    /// Whitespace-only lines outside strings and comments.
    pub blank: u64,
}

impl Counts {
    /// Returns physical lines, including a nonempty unterminated final line.
    pub fn lines(self) -> u64 {
        self.code + self.comments + self.blank
    }

    /// Adds bounded per-file counts to this total.
    pub fn add(&mut self, other: Self) {
        self.bytes += other.bytes;
        self.code += other.code;
        self.comments += other.comments;
        self.blank += other.blank;
    }
}
