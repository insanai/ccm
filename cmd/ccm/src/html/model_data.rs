//! The change-model part of the data island: artifacts, edges, closure, checks, evidence,
//! behavioral cases and the batch interaction. Artifacts, checks, evidence and cases are
//! referred to by their index in the model's own tables.

use super::data::{comma, strings};
use super::escape::{json, number};
use crate::Bundle;
use ccm_core::{ArtifactId, ChangeReport, Model};
use std::fmt::Write;

/// Writes `"model"`, `"behavior"` and `"interaction"`, each `null` when absent, followed
/// by a comma.
pub(super) fn render(bundle: &Bundle, out: &mut String) {
    match (&bundle.model, &bundle.change) {
        (Some(model), Some(change)) => {
            closure(model, change, &bundle.seeds, out);
            checks(model, change, out);
            evidence(model, change, out);
            cases(model, out);
            out.push_str("},");
        }
        _ => out.push_str("\"model\":null,"),
    }
    out.push_str("\"behavior\":");
    match bundle.change.as_ref().and_then(|c| c.behavior.as_ref()) {
        Some(b) => {
            let _ = write!(
                out,
                concat!(
                    "{{\"fidelity\":\"exact-declared-universe\",\"consistent\":{},",
                    "\"hypotheses\":{},\"observations\":{},\"distinction_bits\":{}}},"
                ),
                b.consistent,
                b.hypotheses,
                b.observations,
                b.distinction_bits.map_or("null".into(), number)
            );
        }
        None => out.push_str("null,"),
    }
    out.push_str("\"interaction\":");
    match &bundle.interaction {
        Some(i) => {
            let _ = write!(
                out,
                concat!(
                    "{{\"split\":\"first-seed-vs-remaining\",\"shared\":{},\"joint\":{},",
                    "\"shared_units\":{},\"joint_units\":{},\"net_units\":{}}},"
                ),
                indices(&i.shared),
                indices(&i.joint),
                i.shared_units,
                i.joint_units,
                i.net_units
            );
        }
        None => out.push_str("null,"),
    }
}

fn closure(model: &Model, change: &ChangeReport, seeds: &[ArtifactId], out: &mut String) {
    let _ = write!(
        out,
        concat!(
            "\"model\":{{\"schema\":{},\"context\":{},\"fidelity\":\"exact-declared-model\",",
            "\"source_binding\":\"author-declared\",\"artifacts\":"
        ),
        json(&format!("ccm-model-v{}", model.input_version.unwrap_or(2))),
        json(&model.context)
    );
    strings(out, model.artifacts.iter().map(String::as_str));
    out.push_str(",\"edges\":[");
    for (index, (from, to)) in model.edges.iter().enumerate() {
        comma(out, index);
        let _ = write!(out, "[{},{}]", from.0, to.0);
    }
    let seeds: Vec<usize> = seeds.iter().map(|id| id.0).collect();
    let _ = write!(out, "],\"seeds\":{},\"affected\":[", indices(&seeds));
    for (index, id) in change.affected.iter().enumerate() {
        comma(out, index);
        let previous = change.predecessor.get(id.0).copied().flatten();
        let _ = write!(
            out,
            "[{},{}]",
            id.0,
            previous.map_or("null".into(), |p| p.0.to_string())
        );
    }
    let _ = write!(
        out,
        "],\"required_units\":{},\"pending_units\":{},",
        change.required_units, change.pending_units
    );
}

/// Writes checks as `[name, units, active, pending, failed, triggers]`, where triggers
/// are alternatives of artifact conjunctions.
fn checks(model: &Model, change: &ChangeReport, out: &mut String) {
    out.push_str("\"checks\":[");
    for (index, check) in model.obligations.iter().enumerate() {
        comma(out, index);
        let _ = write!(
            out,
            "[{},{},{},{},{},[",
            json(&check.name),
            check.weight,
            change.active.binary_search(&index).is_ok(),
            change.pending.binary_search(&index).is_ok(),
            change.failed.binary_search(&index).is_ok()
        );
        for (position, trigger) in check.triggers.iter().enumerate() {
            comma(out, position);
            let members: Vec<usize> = trigger.iter().map(|id| id.0).collect();
            out.push_str(&indices(&members));
        }
        out.push_str("]]");
    }
    out.push_str("],");
}

/// Writes evidence as `[name, class, accepted, status, conclusion, support, discharges]`.
fn evidence(model: &Model, change: &ChangeReport, out: &mut String) {
    out.push_str("\"evidence\":[");
    for (index, item) in model.certificates.iter().enumerate() {
        comma(out, index);
        let status = if change.invalidated.binary_search(&index).is_ok() {
            "invalidated"
        } else if change.surviving.binary_search(&index).is_ok() {
            "surviving"
        } else {
            "not-accepted"
        };
        let support: Vec<usize> = item.support.iter().map(|id| id.0).collect();
        let _ = write!(
            out,
            "[{},{},{},{},{},{},{}]",
            json(&item.name),
            json(&item.class),
            item.accepted,
            json(status),
            json(item.conclusion.name()),
            indices(&support),
            indices(&item.discharges)
        );
    }
    out.push(']');
}

/// Writes behavioral cases as `[name, observation]` and filters as
/// `[evidence index, compatible case indices]`.
fn cases(model: &Model, out: &mut String) {
    let Some(behavior) = &model.behavior else {
        out.push_str(",\"cases\":[],\"filters\":[]");
        return;
    };
    out.push_str(",\"cases\":[");
    for (index, case) in behavior.cases.iter().enumerate() {
        comma(out, index);
        let _ = write!(out, "[{},{}]", json(&case.name), json(&case.observation));
    }
    out.push_str("],\"filters\":[");
    for (index, filter) in behavior.filters.iter().enumerate() {
        comma(out, index);
        let _ = write!(
            out,
            "[{},{}]",
            filter.certificate,
            indices(&filter.compatible)
        );
    }
    out.push(']');
}

fn indices(values: &[usize]) -> String {
    let mut out = String::from("[");
    for (index, value) in values.iter().enumerate() {
        comma(&mut out, index);
        let _ = write!(out, "{value}");
    }
    out.push(']');
    out
}
