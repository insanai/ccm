use crate::Error;
use std::collections::VecDeque;

/// An artifact index, validated by `Model::evaluate` before use.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ArtifactId(pub usize);

/// A required acceptance check. Alternatives are OR; members of each trigger are AND.
#[derive(Clone, Debug, Default)]
pub struct Obligation {
    /// Human-readable check name.
    pub name: String,
    /// Nonnegative policy units, never minutes or defect probabilities.
    pub weight: u64,
    /// An empty trigger activates a standing check; no triggers never activates it.
    pub triggers: Vec<Vec<ArtifactId>>,
}

/// A checker's declared conclusion, distinct from whether its evidence is admissible.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Conclusion {
    /// Favorable evidence may discharge its declared checks when admissible and surviving.
    #[default]
    Passed,
    /// A checked counterexample leaves its declared checks open and marks them failed.
    Failed,
    /// An unchecked declaration cannot discharge checks.
    Unverified,
}

impl Conclusion {
    /// Returns the stable report spelling of the declared conclusion.
    pub fn name(self) -> &'static str {
        match self {
            Self::Passed => "passed",
            Self::Failed => "failed",
            Self::Unverified => "unverified",
        }
    }
}

/// A previously accepted evidence route under an explicitly supplied model policy.
#[derive(Clone, Debug, Default)]
pub struct Certificate {
    /// Human-readable evidence name.
    pub name: String,
    /// Declared provenance class, not an authenticity guarantee.
    pub class: String,
    /// Whether the importer declares this evidence accepted under the context policy.
    pub accepted: bool,
    /// Declared checker conclusion: passed, failed, or unverified.
    /// Admissible failures constrain cases without discharging checks.
    pub conclusion: Conclusion,
    /// Assumptions that must avoid the invalidation closure for transport to succeed.
    pub support: Vec<ArtifactId>,
    /// Check indices addressed by this route. Favorable admissible results discharge them.
    pub discharges: Vec<usize>,
}

/// A finite, explicitly declared model. Exact results apply only to this model.
#[derive(Clone, Debug, Default)]
pub struct Model {
    /// Parsed input schema, or none for a model constructed directly through the Rust API.
    pub input_version: Option<u32>,
    /// Snapshot, task, and policy identity supplied by the model author.
    pub context: String,
    /// Artifact names in stable index order.
    pub artifacts: Vec<String>,
    /// Edges point from changed assumptions to their dependents.
    pub edges: Vec<(ArtifactId, ArtifactId)>,
    /// Acceptance checks and activation policies.
    pub obligations: Vec<Obligation>,
    /// Imported evidence routes; no checker is executed by this kernel.
    pub certificates: Vec<Certificate>,
    /// Optional explicitly enumerated behavioral hypotheses and evidence predicates.
    pub behavior: Option<crate::BehaviorModel>,
}

/// A deterministic closure and transported-evidence report.
#[derive(Clone, Debug, Default)]
pub struct ChangeReport {
    /// Seeds and dependents reached in artifact order.
    pub affected: Vec<ArtifactId>,
    /// One predecessor per artifact; seeds and unreached artifacts have none.
    pub predecessor: Vec<Option<ArtifactId>>,
    /// Activated check indices, including standing checks.
    pub active: Vec<usize>,
    /// Active check indices without surviving accepted evidence.
    pub pending: Vec<usize>,
    /// Active checks with a surviving admissible negative result, even if another route passes.
    pub failed: Vec<usize>,
    /// Indices of accepted evidence routes invalidated by this change.
    pub invalidated: Vec<usize>,
    /// Indices of accepted evidence routes that survive transport.
    pub surviving: Vec<usize>,
    /// Total activated policy units.
    pub required_units: u64,
    /// Activated policy units still without supporting evidence.
    pub pending_units: u64,
    /// Exact finite observation counts after surviving evidence, when a universe is declared.
    pub behavior: Option<crate::BehaviorReport>,
}

/// Shared and joint checks for two disjoint edit batches, excluding standing checks.
#[derive(Clone, Debug, Default)]
pub struct InteractionReport {
    /// Checks activated by both separate batches, counted once in their union.
    pub shared: Vec<usize>,
    /// Checks activated only by combining the batches.
    pub joint: Vec<usize>,
    /// Policy units shared between the separate plans.
    pub shared_units: u64,
    /// Additional policy units required by the combined plan.
    pub joint_units: u64,
    /// Joint minus shared policy units. Zero does not imply independence.
    pub net_units: i128,
}

impl Model {
    /// Explains combined activation for borrowed, disjoint batches; returns owned check indices.
    /// Performs four bounded model evaluations and rejects overlapping seeds or invalid models.
    pub fn compare_batches(
        &self,
        left: &[ArtifactId],
        right: &[ArtifactId],
    ) -> Result<InteractionReport, Error> {
        let mut ordered_right = right.to_vec();
        ordered_right.sort_unstable();
        if left
            .iter()
            .any(|id| ordered_right.binary_search(id).is_ok())
        {
            return Err(Error::new(
                "overlapping-batches",
                "Edit batches must be disjoint",
                "Assign each changed artifact to one batch.",
            ));
        }
        let mut union = left.to_vec();
        union.extend_from_slice(right);
        let combined = self.evaluate(&union)?.active;
        let standing = self.evaluate(&[])?.active;
        let left = self.evaluate(left)?.active;
        let right = self.evaluate(right)?.active;
        let mut result = InteractionReport::default();
        for index in 0..self.obligations.len() {
            if standing.binary_search(&index).is_ok() {
                continue;
            }
            let in_left = left.binary_search(&index).is_ok();
            let in_right = right.binary_search(&index).is_ok();
            if in_left && in_right {
                result.shared.push(index);
                result.shared_units =
                    add_weight(result.shared_units, self.obligations[index].weight)?;
            }
            if combined.binary_search(&index).is_ok() && !in_left && !in_right {
                result.joint.push(index);
                result.joint_units =
                    add_weight(result.joint_units, self.obligations[index].weight)?;
            }
        }
        result.net_units = i128::from(result.joint_units) - i128::from(result.shared_units);
        Ok(result)
    }
    /// Validates references and computes reachability, activation, and evidence transport.
    /// Borrows the model and seeds, returning owned results or a structured validation error.
    pub fn evaluate(&self, seeds: &[ArtifactId]) -> Result<ChangeReport, Error> {
        self.validate(seeds)?;
        let mut adjacency = vec![Vec::new(); self.artifacts.len()];
        for &(from, to) in &self.edges {
            adjacency[from.0].push(to);
        }
        for neighbors in &mut adjacency {
            neighbors.sort_unstable();
            neighbors.dedup();
        }
        let mut seen = vec![false; self.artifacts.len()];
        let mut queue = VecDeque::new();
        let mut report = ChangeReport {
            predecessor: vec![None; self.artifacts.len()],
            ..ChangeReport::default()
        };
        let mut ordered = seeds.to_vec();
        ordered.sort_unstable();
        ordered.dedup();
        for seed in ordered {
            seen[seed.0] = true;
            queue.push_back(seed);
        }
        while let Some(from) = queue.pop_front() {
            for &to in &adjacency[from.0] {
                if !seen[to.0] {
                    seen[to.0] = true;
                    report.predecessor[to.0] = Some(from);
                    queue.push_back(to);
                }
            }
        }
        report.affected = seen
            .iter()
            .enumerate()
            .filter(|(_, v)| **v)
            .map(|(i, _)| ArtifactId(i))
            .collect();
        self.evaluate_checks(&seen, &mut report)?;
        report.behavior = self
            .behavior
            .as_ref()
            .map(|b| b.measure(&report.surviving))
            .transpose()?;
        Ok(report)
    }

    fn evaluate_checks(&self, seen: &[bool], report: &mut ChangeReport) -> Result<(), Error> {
        let mut covered = vec![false; self.obligations.len()];
        let mut failed = vec![false; self.obligations.len()];
        for (index, certificate) in self.certificates.iter().enumerate() {
            if !certificate.accepted || certificate.conclusion == Conclusion::Unverified {
                continue;
            }
            if certificate.support.iter().any(|id| seen[id.0]) {
                report.invalidated.push(index);
            } else {
                report.surviving.push(index);
                for &check in &certificate.discharges {
                    match certificate.conclusion {
                        Conclusion::Passed => covered[check] = true,
                        Conclusion::Failed => failed[check] = true,
                        Conclusion::Unverified => {}
                    }
                }
            }
        }
        for (index, check) in self.obligations.iter().enumerate() {
            if !check
                .triggers
                .iter()
                .any(|trigger| trigger.iter().all(|id| seen[id.0]))
            {
                continue;
            }
            report.active.push(index);
            report.required_units = add_weight(report.required_units, check.weight)?;
            if failed[index] {
                report.failed.push(index);
            }
            if !covered[index] || failed[index] {
                report.pending.push(index);
                report.pending_units = add_weight(report.pending_units, check.weight)?;
            }
        }
        Ok(())
    }

    fn validate(&self, seeds: &[ArtifactId]) -> Result<(), Error> {
        if let Some(behavior) = &self.behavior {
            behavior.validate(self.certificates.len())?;
        }
        if self
            .certificates
            .iter()
            .any(|e| e.class == "assertion" && e.accepted)
        {
            return Err(Error::new(
                "evidence-policy",
                "Assertions cannot discharge acceptance checks",
                "Supply accepted test, review, or proof evidence under an explicit policy.",
            ));
        }
        let memberships = self
            .obligations
            .iter()
            .flat_map(|q| &q.triggers)
            .map(Vec::len)
            .chain(
                self.certificates
                    .iter()
                    .map(|e| e.support.len() + e.discharges.len()),
            )
            .try_fold(0usize, |sum, size| sum.checked_add(size));
        if self.artifacts.len() > 100_000
            || self.edges.len() > 1_000_000
            || self.obligations.len() > 100_000
            || self.certificates.len() > 100_000
            || seeds.len() > 100_000
            || memberships.is_none_or(|count| count > 1_000_000)
        {
            return Err(Error::new(
                "model-limit",
                "Finite model exceeds the evaluation budget",
                "Use at most 100,000 objects per kind and 1,000,000 edges/memberships.",
            ));
        }
        let valid = |id: &ArtifactId| id.0 < self.artifacts.len();
        let references_valid = seeds.iter().all(valid)
            && self.edges.iter().all(|(a, b)| valid(a) && valid(b))
            && self
                .obligations
                .iter()
                .flat_map(|q| &q.triggers)
                .flatten()
                .all(valid)
            && self.certificates.iter().all(|e| {
                e.support.iter().all(valid)
                    && e.discharges.iter().all(|&q| q < self.obligations.len())
            });
        if !references_valid {
            return Err(Error::new(
                "model-reference",
                "Unknown artifact or check index",
                "Declare every referenced artifact and check before evaluation.",
            ));
        }
        Ok(())
    }
}

fn add_weight(total: u64, weight: u64) -> Result<u64, Error> {
    total.checked_add(weight).ok_or_else(|| {
        Error::new(
            "weight-overflow",
            "Check weights exceed the supported integer range",
            "Use smaller policy weights.",
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn combined_changes_activate_joint_checks_and_invalidate_routes() {
        let model = Model {
            artifacts: vec!["a".into(), "b".into(), "c".into()],
            edges: vec![
                (ArtifactId(0), ArtifactId(2)),
                (ArtifactId(1), ArtifactId(2)),
            ],
            obligations: vec![
                Obligation {
                    name: "shared".into(),
                    weight: 1,
                    triggers: vec![vec![ArtifactId(0)], vec![ArtifactId(1)]],
                },
                Obligation {
                    name: "joint".into(),
                    weight: 3,
                    triggers: vec![vec![ArtifactId(0), ArtifactId(1)]],
                },
            ],
            certificates: vec![Certificate {
                name: "old test".into(),
                accepted: true,
                support: vec![ArtifactId(2)],
                discharges: vec![0],
                ..Certificate::default()
            }],
            ..Model::default()
        };
        let single = model.evaluate(&[ArtifactId(0)]).unwrap();
        assert_eq!(single.affected, vec![ArtifactId(0), ArtifactId(2)]);
        assert_eq!(single.pending_units, 1);
        let both = model.evaluate(&[ArtifactId(1), ArtifactId(0)]).unwrap();
        assert_eq!(both.pending_units, 4);
        assert_eq!(both.invalidated, vec![0]);
        assert_eq!(both.predecessor[2], Some(ArtifactId(0)));
    }

    #[test]
    fn standing_checks_empty_support_and_missing_routes_are_distinct() {
        let model = Model {
            obligations: vec![Obligation {
                weight: 2,
                triggers: vec![vec![]],
                ..Obligation::default()
            }],
            ..Model::default()
        };
        assert_eq!(model.evaluate(&[]).unwrap().pending_units, 2);
        let mut covered = model;
        covered.certificates.push(Certificate {
            accepted: true,
            discharges: vec![0],
            ..Certificate::default()
        });
        assert_eq!(covered.evaluate(&[]).unwrap().pending_units, 0);
        assert!(covered.evaluate(&[ArtifactId(0)]).is_err());
    }
}

#[cfg(test)]
mod properties {
    use super::*;

    #[test]
    fn closure_agrees_with_fixed_point_for_all_three_artifact_graphs() {
        for graph in 0..512u16 {
            let mut model = Model {
                artifacts: vec!["a".into(), "b".into(), "c".into()],
                ..Model::default()
            };
            for from in 0..3 {
                for to in 0..3 {
                    if graph & (1 << (from * 3 + to)) != 0 {
                        model.edges.push((ArtifactId(from), ArtifactId(to)));
                    }
                }
            }
            for seed_mask in 0..8u8 {
                let seeds: Vec<_> = (0..3)
                    .filter(|i| seed_mask & (1 << i) != 0)
                    .map(ArtifactId)
                    .collect();
                let mut expected = seed_mask;
                loop {
                    let old = expected;
                    for &(from, to) in &model.edges {
                        if expected & (1 << from.0) != 0 {
                            expected |= 1 << to.0;
                        }
                    }
                    if expected == old {
                        break;
                    }
                }
                let actual = model
                    .evaluate(&seeds)
                    .unwrap()
                    .affected
                    .iter()
                    .fold(0u8, |mask, id| mask | (1 << id.0));
                assert_eq!(actual, expected);
            }
        }
    }

    #[test]
    fn zero_interaction_retains_shared_and_joint_terms_and_ignores_standing_work() {
        let model = Model {
            artifacts: vec!["a".into(), "b".into()],
            obligations: vec![
                Obligation {
                    weight: 1,
                    triggers: vec![vec![]],
                    ..Obligation::default()
                },
                Obligation {
                    weight: 2,
                    triggers: vec![vec![ArtifactId(0)], vec![ArtifactId(1)]],
                    ..Obligation::default()
                },
                Obligation {
                    weight: 2,
                    triggers: vec![vec![ArtifactId(0), ArtifactId(1)]],
                    ..Obligation::default()
                },
            ],
            ..Model::default()
        };
        let result = model
            .compare_batches(&[ArtifactId(0)], &[ArtifactId(1)])
            .unwrap();
        assert_eq!(result.net_units, 0);
        assert_eq!(result.shared, vec![1]);
        assert_eq!(result.joint, vec![2]);
        assert!(
            model
                .compare_batches(&[ArtifactId(0)], &[ArtifactId(0)])
                .is_err()
        );
    }
}
