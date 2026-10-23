use ccm_core::Error;
use std::fmt;
use std::ops::Deref;

/// Keeps presentation work within a user-visible output budget before writing any report.
pub(crate) struct Buffer {
    text: String,
    limit: usize,
    failed: bool,
}

impl Buffer {
    pub fn new(limit: usize) -> Self {
        Self {
            text: String::new(),
            limit,
            failed: false,
        }
    }

    pub fn push_str(&mut self, text: &str) {
        let _ = fmt::Write::write_str(self, text);
    }

    pub fn push(&mut self, character: char) {
        let mut bytes = [0; 4];
        self.push_str(character.encode_utf8(&mut bytes));
    }

    pub fn fail(&mut self) {
        self.failed = true;
    }

    pub fn failed(&self) -> bool {
        self.failed
    }

    pub fn finish(self) -> Result<String, Error> {
        if self.failed {
            Err(limit_error())
        } else {
            Ok(self.text)
        }
    }
}

impl fmt::Write for Buffer {
    fn write_str(&mut self, text: &str) -> fmt::Result {
        if self.failed
            || text.len() > self.limit.saturating_sub(self.text.len())
            || self.text.try_reserve(text.len()).is_err()
        {
            self.failed = true;
            return Err(fmt::Error);
        }
        self.text.push_str(text);
        Ok(())
    }
}

impl Deref for Buffer {
    type Target = str;
    fn deref(&self) -> &str {
        &self.text
    }
}

pub(crate) fn limit_error() -> Error {
    Error::new(
        "report-limit",
        "The rendered report exceeds its byte or allocation budget",
        "Narrow the selected paths or increase --max-report-bytes explicitly.",
    )
}
