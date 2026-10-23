use crate::report::Buffer;
use crate::{Args, Bundle, text};
use ccm_core::Counts;
use std::fmt::Write;

pub(crate) fn render(bundle: &Bundle, args: &Args) -> Result<String, ccm_core::Error> {
    let mut out = Buffer::new(args.max_report_bytes);
    out.push_str("{\n  \"schema\":\"ccm.report.v3\",\n");
    let _ = writeln!(
        out,
        "  \"complete\":{},\n  \"count_policy\":{},",
        bundle.current.complete,
        text::json(ccm_scan::COUNT_POLICY)
    );
    selection(args, &mut out);
    let _ = writeln!(
        out,
        "  \"totals\":{},\n  \"unsupported_files\":{},",
        counts(bundle.current.totals),
        bundle.current.unsupported_files
    );
    // `source_lines` includes generated files; `program_lines` is the one figure the
    // terminal and HTML reports show, and it equals the default AI rebuild scope.
    let _ = writeln!(
        out,
        concat!(
            "  \"program_lines\":{},\"generated_source_lines\":{},",
            "\"source_lines\":{},\"documentation_lines\":{},\"data_lines\":{},"
        ),
        bundle.current.program_lines(),
        bundle.current.generated_source_lines(),
        bundle.current.content_lines(ccm_scan::FileKind::Source),
        bundle
            .current
            .content_lines(ccm_scan::FileKind::Documentation),
        bundle.current.content_lines(ccm_scan::FileKind::Data)
    );
    let r = &bundle.current;
    let _ = writeln!(
        out,
        concat!(
            "  \"skipped\":{{\"symlinks\":{},\"binary\":{},\"generated\":{},",
            "\"oversized\":{},\"ignored_entries\":{}}},"
        ),
        r.skipped_symlinks,
        r.skipped_binary,
        r.skipped_generated,
        r.skipped_oversized,
        r.ignored_entries
    );
    files(bundle, &mut out);
    out.push_str("],\n  \"notices\":[");
    for (index, notice) in bundle.current.notices.iter().enumerate() {
        if out.failed() {
            break;
        }
        comma(&mut out, index);
        let _ = write!(
            out,
            "{{\"path\":{},\"message\":{}}}",
            text::json(&notice.path),
            text::json(&notice.message)
        );
    }
    out.push_str("],\n  \"comparison\":");
    comparison(bundle, &mut out);
    out.push_str(",\n  \"change_model\":");
    model(bundle, &mut out);
    out.push_str(",\n  \"economic_scenario\":");
    if args.no_cost {
        out.push_str("null");
    } else {
        economic(bundle, args, &mut out);
    }
    out.push_str(",\n  \"ai_development\":");
    if args.no_cost {
        out.push_str("null");
    } else {
        crate::budget::json(bundle, args, &mut out);
    }
    out.push_str(",\n  \"batch_interaction\":");
    interaction(bundle, &mut out);
    out.push_str(",\n  \"behavioral_ambiguity\":");
    ambiguity(bundle, &mut out);
    out.push_str(",\n  \"semantic_guarantee\":null\n}\n");
    out.finish()
}

fn selection(args: &Args, out: &mut Buffer) {
    let _ = write!(
        out,
        "  \"selection\":{{\"symlinks\":\"skip\",\"gitignore\":{},\"exclusions\":[",
        args.scan.respect_ignore
    );
    list_strings(out, args.scan.exclude.iter().map(String::as_str));
    let _ = writeln!(
        out,
        "],\"max_files\":{},\"max_file_bytes\":{},\"max_total_bytes\":{}}},",
        args.scan.max_files, args.scan.max_file_bytes, args.scan.max_total_bytes
    );
}

fn counts(c: Counts) -> String {
    format!(
        "{{\"bytes\":{},\"lines\":{},\"code\":{},\"comments\":{},\"blank\":{}}}",
        c.bytes,
        c.lines(),
        c.code,
        c.comments,
        c.blank
    )
}

fn comparison(bundle: &Bundle, out: &mut Buffer) {
    let Some(changes) = &bundle.changes else {
        out.push_str("null");
        return;
    };
    let _ = write!(
        out,
        "{{\"kind\":\"exact-file-bytes\",\"baseline_totals\":{},\"files\":[",
        bundle
            .baseline
            .as_ref()
            .map_or("null".into(), |r| counts(r.totals))
    );
    for (index, change) in changes.iter().enumerate() {
        if out.failed() {
            break;
        }
        comma(out, index);
        let _ = write!(
            out,
            "{{\"path\":{},\"kind\":{},\"before\":{},\"after\":{}}}",
            text::json(&change.name),
            text::json(change.kind),
            change.before.map_or("null".into(), counts),
            change.after.map_or("null".into(), counts)
        );
    }
    out.push_str("]}");
}

fn model(bundle: &Bundle, out: &mut Buffer) {
    let (Some(model), Some(change)) = (&bundle.model, &bundle.change) else {
        out.push_str("null");
        return;
    };
    let _ = write!(
        out,
        "{{\"schema\":\"ccm-model-v{}\",\"context\":{},",
        model.input_version.unwrap_or(2),
        text::json(&model.context)
    );
    out.push_str(
        "\"fidelity\":\"exact-declared-model\",\"source_binding\":\"author-declared\",
        \"semantic_coverage\":null,\"seeds\":[",
    );
    list_strings(
        out,
        bundle.seeds.iter().map(|id| model.artifacts[id.0].as_str()),
    );
    out.push_str("],\"affected\":[");
    for (index, id) in change.affected.iter().enumerate() {
        if out.failed() {
            break;
        }
        comma(out, index);
        let _ = write!(
            out,
            "{{\"artifact\":{},\"predecessor\":{}}}",
            text::json(&model.artifacts[id.0]),
            change.predecessor[id.0].map_or("null".into(), |p| text::json(&model.artifacts[p.0]))
        );
    }
    let _ = write!(
        out,
        "],\"required_units\":{},\"pending_units\":{},\"checks\":[",
        change.required_units, change.pending_units
    );
    for (index, check) in model.obligations.iter().enumerate() {
        if out.failed() {
            break;
        }
        comma(out, index);
        let _ = write!(
            out,
            "{{\"name\":{},\"policy_units\":{},\"active\":{},\"pending\":{},\"failed\":{}}}",
            text::json(&check.name),
            check.weight,
            change.active.binary_search(&index).is_ok(),
            change.pending.binary_search(&index).is_ok(),
            change.failed.binary_search(&index).is_ok()
        );
    }
    out.push_str("],\"evidence\":[");
    for (index, evidence) in model.certificates.iter().enumerate() {
        if out.failed() {
            break;
        }
        comma(out, index);
        let status = if change.invalidated.binary_search(&index).is_ok() {
            "invalidated"
        } else if change.surviving.binary_search(&index).is_ok() {
            "surviving"
        } else {
            "not-accepted"
        };
        let _ = write!(
            out,
            "{{\"name\":{},\"class\":{},\"status\":{},\"conclusion\":{}}}",
            text::json(&evidence.name),
            text::json(&evidence.class),
            text::json(status),
            text::json(evidence.conclusion.name())
        );
    }
    out.push_str("]}");
}

fn economic(bundle: &Bundle, args: &Args, out: &mut Buffer) {
    let c = bundle.cost;
    let a = args.cost;
    let _ = write!(
        out,
        "{{\"model\":\"ccc-resource-cost-v2\",\"currency\":\"USD\",
        \"status\":\"illustrative-unvalidated-scenario\",\"verification_included\":{},
        \"review_artifacts\":{},\"pending_policy_units\":{},\"hours_per_attempt\":{},
        \"expected_attempts\":{},\"expected_cost\":{},
        \"human_and_override_expected_cost\":{},\"assumptions\":{{
        \"hourly_cost\":{},\"review_hours_per_artifact\":{},\"hours_per_check_unit\":{},
        \"implementation_hours\":{},\"agent_cost_per_attempt\":{},\"rework_probability\":{}}}}}",
        c.pending_units.is_some(),
        c.review_artifacts,
        c.pending_units.map_or("null".into(), |n| n.to_string()),
        c.hours_per_attempt,
        c.expected_attempts,
        crate::economics::total(bundle),
        c.usd,
        a.hourly_cost,
        a.review_hours,
        a.check_hours,
        a.implementation_hours,
        a.agent_cost,
        a.rework_probability
    );
}

fn list_strings<'a>(out: &mut Buffer, items: impl Iterator<Item = &'a str>) {
    for (index, item) in items.enumerate() {
        if out.failed() {
            break;
        }
        comma(out, index);
        out.push_str(&text::json(item));
    }
}

fn comma(out: &mut Buffer, index: usize) {
    if index > 0 {
        out.push(',');
    }
}

fn ambiguity(bundle: &Bundle, out: &mut Buffer) {
    if let Some(b) = bundle.change.as_ref().and_then(|c| c.behavior.as_ref()) {
        let _ = write!(
            out,
            "{{\"fidelity\":\"exact-declared-universe\",\"consistent\":{},
            \"hypotheses\":{},\"observations\":{},\"distinction_bits\":{}}}",
            b.consistent,
            b.hypotheses,
            b.observations,
            b.distinction_bits.map_or("null".into(), |v| v.to_string())
        );
    } else {
        out.push_str("null");
    }
}

fn interaction(bundle: &Bundle, out: &mut Buffer) {
    if let Some(i) = &bundle.interaction {
        let _ = write!(
            out,
            "{{\"split\":\"first-seed-vs-remaining\",
            \"shared_check_indices\":{:?},
            \"joint_check_indices\":{:?},\"shared_units\":{},
            \"joint_units\":{},\"net_units\":{}}}",
            i.shared, i.joint, i.shared_units, i.joint_units, i.net_units
        );
    } else {
        out.push_str("null");
    }
}

fn files(bundle: &Bundle, out: &mut Buffer) {
    out.push_str("  \"files\":[");
    for (index, file) in bundle.current.files.iter().enumerate() {
        if out.failed() {
            break;
        }
        comma(out, index);
        let _ = write!(
            out,
            concat!(
                "{{\"path\":{},\"language\":{},\"kind\":{},\"generated\":{},",
                "\"source_work_tokens\":{},\"language_policy\":{},\"counts\":{}}}"
            ),
            text::json(&file.name),
            text::json(file.language.name()),
            text::json(file.language.kind().name()),
            file.generated,
            file.source_tokens.map_or("null".into(), |n| n.to_string()),
            text::json(file.language.policy()),
            counts(file.counts)
        );
    }
}
