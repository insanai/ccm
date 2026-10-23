use crate::Error;
use std::collections::BTreeSet;

/// A finite hypothesis and its declared task observation, without a probability assignment.
#[derive(Clone, Debug)]
pub struct BehaviorCase {
    /// Stable hypothesis name.
    pub name: String,
    /// Observable task outcome; equal labels represent indistinguishable outcomes.
    pub observation: String,
}

/// Hypotheses compatible with one accepted evidence item.
#[derive(Clone, Debug)]
pub struct CaseFilter {
    /// Index into the enclosing model's certificates.
    pub certificate: usize,
    /// Indices of compatible hypotheses; an empty set expresses inconsistency.
    pub compatible: Vec<usize>,
}

/// An explicitly enumerated, task-relative hypothesis universe.
#[derive(Clone, Debug, Default)]
pub struct BehaviorModel {
    /// Finite hypotheses; they are supplied, not inferred from program syntax.
    pub cases: Vec<BehaviorCase>,
    /// Evidence compatibility predicates. Selected predicates are intersected.
    pub filters: Vec<CaseFilter>,
}

/// Exact cardinalities and a rounded log-cardinality display, distinct from Shannon entropy.
#[derive(Clone, Debug, Default)]
pub struct BehaviorReport {
    /// Whether at least one hypothesis survives. False is inconsistency, not zero uncertainty.
    pub consistent: bool,
    /// Compatible hypothesis count.
    pub hypotheses: usize,
    /// Distinct compatible task observations, not the number of hypotheses.
    pub observations: usize,
    /// Log2 of positive observation count, or unavailable for an inconsistent universe.
    pub distinction_bits: Option<f64>,
}

impl BehaviorModel {
    /// Intersects predicates for borrowed evidence indices and counts unique observations.
    /// Work is capped at 20 million case-filter steps; malformed references return errors.
    pub fn measure(&self, evidence: &[usize]) -> Result<BehaviorReport, Error> {
        if self.cases.len() > 100_000 || self.filters.len() > 100_000 {
            return Err(problem("Behavior model exceeds 100,000 cases or filters"));
        }
        let mut selected = evidence.to_vec();
        selected.sort_unstable();
        let mut survivors = vec![true; self.cases.len()];
        let mut allowed = vec![false; self.cases.len()];
        let mut work = 0usize;
        for filter in &self.filters {
            if selected.binary_search(&filter.certificate).is_err() {
                continue;
            }
            work = work
                .saturating_add(self.cases.len())
                .saturating_add(filter.compatible.len());
            if work > 20_000_000 {
                return Err(problem("Behavior case-filter work exceeds 20 million"));
            }
            allowed.fill(false);
            for &case in &filter.compatible {
                let slot = allowed
                    .get_mut(case)
                    .ok_or_else(|| problem("Unknown behavior case index"))?;
                *slot = true;
            }
            for (survives, compatible) in survivors.iter_mut().zip(&allowed) {
                *survives &= compatible;
            }
        }
        let mut observations = BTreeSet::new();
        let mut hypotheses = 0;
        for (case, survives) in self.cases.iter().zip(survivors) {
            if survives {
                hypotheses += 1;
                observations.insert(case.observation.as_str());
            }
        }
        Ok(BehaviorReport {
            consistent: hypotheses > 0,
            hypotheses,
            observations: observations.len(),
            distinction_bits: (!observations.is_empty())
                .then(|| (observations.len() as f64).log2()),
        })
    }

    /// Computes the signed complementarity of two additional evidence items in distinction bits.
    /// Positive means joint gain exceeds separate gains; inconsistent intersections return none.
    /// The interaction is defined only for two distinct items outside the borrowed background;
    /// any other pair is an `evidence-interaction` error.
    pub fn evidence_interaction(
        &self,
        background: &[usize],
        first: usize,
        second: usize,
    ) -> Result<Option<f64>, Error> {
        if first == second || background.contains(&first) || background.contains(&second) {
            return Err(Error::new(
                "evidence-interaction",
                "Evidence interaction needs two distinct items outside the background",
                "Choose two different evidence items that the background does not contain.",
            ));
        }
        let mut left = background.to_vec();
        left.push(first);
        let mut right = background.to_vec();
        right.push(second);
        let mut both = left.clone();
        both.push(second);
        let values = [
            self.measure(background)?,
            self.measure(&left)?,
            self.measure(&right)?,
            self.measure(&both)?,
        ];
        Ok(match values.map(|v| v.distinction_bits) {
            [Some(base), Some(left), Some(right), Some(both)] => Some(left + right - base - both),
            _ => None,
        })
    }

    pub(crate) fn validate(&self, certificates: usize) -> Result<(), Error> {
        if self.cases.len() > 100_000 || self.filters.len() > 100_000 {
            return Err(problem("Behavior model exceeds 100,000 cases or filters"));
        }
        for filter in &self.filters {
            if filter.certificate >= certificates
                || filter.compatible.iter().any(|&c| c >= self.cases.len())
            {
                return Err(problem("Unknown evidence or behavior case reference"));
            }
        }
        Ok(())
    }
}

fn problem(message: &str) -> Error {
    Error::new(
        "behavior-model",
        message,
        "Use a bounded, explicitly declared case and evidence universe.",
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn model() -> BehaviorModel {
        BehaviorModel {
            cases: ["00", "01", "10", "11"]
                .iter()
                .map(|name| BehaviorCase {
                    name: (*name).into(),
                    observation: (*name).into(),
                })
                .collect(),
            filters: vec![
                CaseFilter {
                    certificate: 0,
                    compatible: vec![0, 1, 2],
                },
                CaseFilter {
                    certificate: 1,
                    compatible: vec![0, 2, 3],
                },
                CaseFilter {
                    certificate: 2,
                    compatible: vec![3],
                },
            ],
        }
    }

    #[test]
    fn joint_evidence_gain_can_be_positive_or_negative() {
        let model = model();
        assert!(model.evidence_interaction(&[], 0, 1).unwrap().unwrap() > 0.0);
        // Two distinct filters that exclude the same case are redundant: negative interaction.
        let mut redundant = model.clone();
        redundant.filters[1].compatible = vec![0, 1, 2];
        assert!(redundant.evidence_interaction(&[], 0, 1).unwrap().unwrap() < 0.0);
        let inconsistent = model.measure(&[0, 2]).unwrap();
        assert!(!inconsistent.consistent);
        assert_eq!(inconsistent.distinction_bits, None);
    }

    #[test]
    fn interaction_rejects_repeated_items_and_items_in_the_background() {
        let model = model();
        assert!(model.evidence_interaction(&[], 0, 0).is_err());
        assert!(model.evidence_interaction(&[0], 0, 1).is_err());
        assert!(model.evidence_interaction(&[1], 0, 1).is_err());
        assert!(model.evidence_interaction(&[2], 0, 1).is_ok());
    }

    #[test]
    fn observed_classes_not_case_count_determine_ambiguity() {
        let mut model = model();
        model.cases[1].observation = "00".into();
        let report = model.measure(&[0]).unwrap();
        assert_eq!((report.hypotheses, report.observations), (3, 2));
        assert_eq!(report.distinction_bits, Some(1.0));
        assert!(
            BehaviorModel::default()
                .measure(&[])
                .unwrap()
                .distinction_bits
                .is_none()
        );
    }

    #[test]
    fn evidence_never_increases_observation_count_in_all_four_case_filters() {
        for first in 0..16u8 {
            for second in 0..16u8 {
                let mut model = model();
                model.filters[0].compatible = (0..4).filter(|i| first & (1 << i) != 0).collect();
                model.filters[1].compatible = (0..4).filter(|i| second & (1 << i) != 0).collect();
                let a = model.measure(&[0]).unwrap();
                let both = model.measure(&[0, 1]).unwrap();
                assert!(both.observations <= a.observations);
            }
        }
    }
}
