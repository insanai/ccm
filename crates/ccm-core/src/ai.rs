//! Versioned resource-price scenarios; empirical calibration is not a semantic theorem.
use crate::Error;

/// Editable AI development priors, frozen independently of provider APIs.
#[derive(Clone, Debug)]
pub struct AiOptions {
    /// Publication year of the calibration profile.
    pub year: u16,
    /// Selected price card, for presentation only.
    pub model: String,
    /// USD per million uncached input tokens.
    pub input_price: f64,
    /// USD per million output tokens, including reasoning.
    pub output_price: f64,
    /// Inferred input tokens per delivered program line.
    pub input_per_line: f64,
    /// Inferred output tokens per delivered program line before language adjustment.
    pub output_per_line: f64,
    /// Extra token budget per pending declared check policy unit.
    pub check_tokens: f64,
    /// USD per hosted 1 GB, 20-minute sandbox session.
    pub session_price: f64,
    /// Assumed delivered lines per sandbox session.
    pub lines_per_session: f64,
    /// Explicit workload multiplier; neither productivity nor correctness is measured.
    pub workload: f64,
    /// Output-token multipliers; unspecified languages use 1.
    pub languages: Vec<(String, f64)>,
}

impl Default for AiOptions {
    fn default() -> Self {
        // Span's 2026 examples imply $0.089 and $0.138 per merged line.
        // Their midpoint is our prior. The 90:10 mix is an unverified assumption.
        let tokens = 0.1135 * 1_000_000.0 / 14.0;
        Self {
            year: 2026,
            model: "gpt-6-astra (2026-10-23 standard)".into(),
            input_price: 10.0,
            output_price: 50.0,
            input_per_line: tokens * 0.9,
            output_per_line: tokens * 0.1,
            check_tokens: 1000.0,
            session_price: 0.03,
            lines_per_session: 50.0,
            workload: 1.0,
            languages: vec![
                ("Java".into(), 1.24),
                ("Rust".into(), 1.26),
                ("OCaml".into(), 1.29),
            ],
        }
    }
}

impl AiOptions {
    /// Returns the explicit output multiplier, or the neutral unmeasured-language prior.
    pub fn language_factor(&self, language: &str) -> f64 {
        self.languages
            .iter()
            .find(|(name, _)| name == language)
            .map_or(1.0, |(_, n)| *n)
    }

    /// Rejects nonfinite, negative, excessive, or degenerate resource assumptions.
    pub fn validate(&self) -> Result<(), Error> {
        let values = [
            self.input_price,
            self.output_price,
            self.input_per_line,
            self.output_per_line,
            self.check_tokens,
            self.session_price,
            self.workload,
        ];
        if values
            .iter()
            .chain(self.languages.iter().map(|(_, v)| v))
            .any(|v| !v.is_finite() || !(0.0..=1e9).contains(v))
            || !self.lines_per_session.is_finite()
            || !(1.0..=1e9).contains(&self.lines_per_session)
            || !(2026..=9999).contains(&self.year)
        {
            return Err(Error::new(
                "ai-assumptions",
                "Invalid AI cost profile",
                "Use finite values from 0 to 1e9, lines/session >= 1, and year >= 2026.",
            ));
        }
        Ok(())
    }
}

/// AI resources for a selected implementation scope and residual acceptance work.
#[derive(Clone, Copy, Debug, Default)]
pub struct AiEstimate {
    /// Physical program lines selected for a counterfactual rebuild.
    pub lines: u64,
    /// Language-weighted output line equivalents; not semantic complexity.
    pub output_lines: f64,
    /// Inferred input tokens for implementation and generic agent iteration.
    pub input_tokens: f64,
    /// Inferred output tokens, including reasoning, for the implementation scope.
    pub output_tokens: f64,
    /// USD for implementation and generic agent iteration, per attempt.
    pub implementation_usd: f64,
    /// Additional tokens for residual author-declared acceptance checks.
    pub check_tokens: f64,
    /// Additional USD for residual declared acceptance checks, per attempt.
    pub checks_usd: f64,
    /// Assumed hosted sandbox sessions, per attempt.
    pub sessions: f64,
    /// Hosted sandbox USD, per attempt.
    pub tools_usd: f64,
}

impl AiEstimate {
    /// Returns all modeled AI and sandbox spending for one attempt.
    pub fn usd(&self) -> f64 {
        self.implementation_usd + self.checks_usd + self.tools_usd
    }
}

/// Prices a supplied source workload and residual policy units without filesystem access.
/// `output_lines` already includes explicit language factors.
/// Missing check demand is excluded; callers must preserve that uncertainty in reports.
pub fn estimate_ai(
    lines: u64,
    output_lines: f64,
    pending_units: Option<u64>,
    options: &AiOptions,
) -> Result<AiEstimate, Error> {
    options.validate()?;
    if !output_lines.is_finite() || output_lines < 0.0 || output_lines > u64::MAX as f64 * 1e9 {
        return Err(Error::new(
            "ai-workload",
            "Invalid source workload",
            "Use finite source counts.",
        ));
    }
    let input_tokens = lines as f64 * options.input_per_line * options.workload;
    let output_tokens = output_lines * options.output_per_line * options.workload;
    let check_tokens = pending_units.unwrap_or(0) as f64 * options.check_tokens * options.workload;
    let total_per_line = options.input_per_line + options.output_per_line;
    let input_share = if total_per_line == 0.0 {
        0.9
    } else {
        options.input_per_line / total_per_line
    };
    let checks_usd = check_tokens
        * (input_share * options.input_price + (1.0 - input_share) * options.output_price)
        / 1_000_000.0;
    let sessions = (lines as f64 * options.workload / options.lines_per_session).ceil();
    Ok(AiEstimate {
        lines,
        output_lines,
        input_tokens,
        output_tokens,
        implementation_usd: (input_tokens * options.input_price
            + output_tokens * options.output_price)
            / 1_000_000.0,
        check_tokens,
        checks_usd,
        sessions,
        tools_usd: sessions * options.session_price,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calibration_reconciles_and_repricing_changes_only_prices() {
        let mut profile = AiOptions::default();
        let base = estimate_ai(1000, 1000.0, None, &profile).unwrap();
        assert!((base.implementation_usd - 113.5).abs() < 1e-8);
        profile.input_price *= 0.4;
        profile.output_price *= 0.4;
        let cheap = estimate_ai(1000, 1000.0, None, &profile).unwrap();
        assert_eq!(base.input_tokens, cheap.input_tokens);
        assert!((cheap.implementation_usd - 45.4).abs() < 1e-8);
    }

    #[test]
    fn language_adjustment_and_residual_checks_price_distinct_resources() {
        let profile = AiOptions::default();
        let python = estimate_ai(100, 100.0, Some(4), &profile).unwrap();
        let rust = estimate_ai(100, 126.0, Some(0), &profile).unwrap();
        assert_eq!(python.input_tokens, rust.input_tokens);
        assert!(rust.implementation_usd > python.implementation_usd);
        assert!(python.checks_usd > rust.checks_usd);
        assert_eq!(rust.checks_usd, 0.0);
        assert!(estimate_ai(1, f64::NAN, None, &profile).is_err());
        assert!(
            AiOptions {
                workload: f64::INFINITY,
                ..profile
            }
            .validate()
            .is_err()
        );
    }
}
