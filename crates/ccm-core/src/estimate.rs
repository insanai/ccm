/// Explicit prices and activity assumptions for an expected acceptance-cost scenario.
#[derive(Clone, Copy, Debug)]
pub struct CostOptions {
    /// Loaded human cost per hour in USD; not inferred from source.
    pub hourly_cost: f64,
    /// Assumed review hours per affected artifact (per source file without a model).
    pub review_hours: f64,
    /// Assumed hours per pending check policy unit.
    pub check_hours: f64,
    /// Explicit implementation hours per attempt; never derived from line count.
    pub implementation_hours: f64,
    /// Explicit agent/tool spending per attempt in USD.
    pub agent_cost: f64,
    /// Assumed independent repeat probability for a stationary attempt, from 0 to 0.95.
    pub rework_probability: f64,
}

impl Default for CostOptions {
    fn default() -> Self {
        Self {
            hourly_cost: 100.0,
            review_hours: 0.25,
            check_hours: 1.0,
            implementation_hours: 0.0,
            agent_cost: 0.0,
            rework_probability: 0.0,
        }
    }
}

/// Activity-based expected resource cost, distinct from value, price, and actual spending.
#[derive(Clone, Copy, Debug)]
pub struct CostEstimate {
    /// Count of artifacts priced for review.
    pub review_artifacts: usize,
    /// Weighted residual checks, or none when no task model is supplied.
    pub pending_units: Option<u64>,
    /// Review, verification, and explicit implementation hours in one attempt.
    pub hours_per_attempt: f64,
    /// Expected attempts under a declared stationary geometric repeat scenario.
    pub expected_attempts: f64,
    /// Expected human and agent/tool spending in USD.
    pub usd: f64,
}

/// Prices review and residual verification activities with explicit user assumptions.
/// Computes `(hours * hourly_cost + agent_cost) / (1 - rework_probability)`.
/// Missing verification demand stays unknown; its cost is excluded and the result is partial.
/// Rejects nonfinite, negative, and out-of-range assumptions.
pub fn estimate_cost(
    review_artifacts: usize,
    pending_units: Option<u64>,
    options: CostOptions,
) -> Result<CostEstimate, crate::Error> {
    let nonnegative = [
        options.hourly_cost,
        options.review_hours,
        options.check_hours,
        options.implementation_hours,
        options.agent_cost,
    ];
    if nonnegative
        .iter()
        .any(|v| !v.is_finite() || !(0.0..=1e9).contains(v))
        || !options.rework_probability.is_finite()
        || !(0.0..=0.95).contains(&options.rework_probability)
    {
        return Err(crate::Error::new(
            "cost-assumptions",
            "Cost assumptions are outside supported ranges",
            "Use prices/hours from 0 to 1e9 and a repeat probability from 0 to 0.95.",
        ));
    }
    let hours_per_attempt = review_artifacts as f64 * options.review_hours
        + pending_units.unwrap_or(0) as f64 * options.check_hours
        + options.implementation_hours;
    let expected_attempts = 1.0 / (1.0 - options.rework_probability);
    let usd = (hours_per_attempt * options.hourly_cost + options.agent_cost) * expected_attempts;
    Ok(CostEstimate {
        review_artifacts,
        pending_units,
        hours_per_attempt,
        expected_attempts,
        usd,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expected_cost_reconciles_with_activity_prices_and_repeat_assumption() {
        let options = CostOptions {
            hourly_cost: 100.0,
            review_hours: 0.5,
            check_hours: 2.0,
            implementation_hours: 3.0,
            agent_cost: 10.0,
            rework_probability: 0.2,
        };
        let estimate = estimate_cost(4, Some(2), options).unwrap();
        assert_eq!(estimate.hours_per_attempt, 9.0);
        assert!((estimate.usd - 1137.5).abs() < 1e-8);
        assert_eq!(estimate_cost(1, None, options).unwrap().pending_units, None);
        assert!(
            estimate_cost(
                1,
                None,
                CostOptions {
                    hourly_cost: f64::NAN,
                    ..options
                }
            )
            .is_err()
        );
    }
}
