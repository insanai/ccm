/// Counts versioned source work tokens without assigning semantic difficulty or LLM tokens.
#[derive(Default)]
pub(super) struct Counter {
    pub enabled: bool,
    pub tokens: u64,
    word: bool,
}

impl Counter {
    pub fn new(enabled: bool) -> Self {
        Self {
            enabled,
            ..Self::default()
        }
    }

    pub fn source(&mut self, bytes: &[u8]) {
        if !self.enabled {
            return;
        }
        for &byte in bytes {
            if byte.is_ascii_alphanumeric() || byte == b'_' || byte >= 128 {
                if !self.word {
                    self.tokens += 1;
                }
                self.word = true;
            } else {
                self.word = false;
                if !byte.is_ascii_whitespace() {
                    self.tokens += 1;
                }
            }
        }
    }

    pub fn literal(&mut self) {
        if self.enabled {
            self.tokens += 1;
        }
        self.boundary();
    }

    pub fn boundary(&mut self) {
        self.word = false;
    }
}
