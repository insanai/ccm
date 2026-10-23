use crate::report::Buffer;
use crate::{Args, Bundle, text};
use std::fmt::Write;

pub(crate) fn render(
    bundle: &Bundle,
    args: &Args,
    color: bool,
) -> Result<String, ccm_core::Error> {
    let mut out = Buffer::new(args.max_report_bytes);
    let accent = if color { "\x1b[1;35m" } else { "" };
    let reset = if color { "\x1b[0m" } else { "" };
    let muted = if color { "\x1b[2m" } else { "" };
    let _ = writeln!(out, "\n  {accent}CCM{reset}  Source & change intelligence");
    let _ = writeln!(
        out,
        "  {muted}{} files  |  {} program lines  |  {} bytes{reset}\n",
        text::number(bundle.current.files.len() as u64),
        text::number(bundle.current.content_lines(ccm_scan::FileKind::Source)),
        text::number(bundle.current.totals.bytes)
    );
    crate::inventory::render(bundle, args, &mut out, color);
    if let Some(changes) = &bundle.changes {
        let _ = writeln!(
            out,
            "  {accent}CHANGE OVERVIEW{reset}  {} changed files",
            changes.len()
        );
        for change in changes {
            if out.failed() {
                break;
            }
            let before = change.before.map_or(0, |c| c.code);
            let after = change.after.map_or(0, |c| c.code);
            let _ = writeln!(
                out,
                "    {:<8} {}  (code: {before} -> {after})",
                change.kind,
                text::terminal(&change.name)
            );
        }
        out.push('\n');
    }
    model_summary(bundle, &mut out, args.technical, color);
    if !args.no_cost {
        crate::budget::terminal(bundle, args, &mut out, accent, reset);
    }
    if args.technical || args.by_file {
        technical(bundle, &mut out);
    }
    crate::notices::render(bundle, args, &mut out);
    out.push('\n');
    let width = text::terminal_width();
    let plain = out.finish()?;
    let wrapped = text::wrap_terminal(&plain, width);
    if wrapped.len() > args.max_report_bytes {
        return Err(crate::report::limit_error());
    }
    Ok(wrapped)
}

fn model_summary(bundle: &Bundle, out: &mut Buffer, technical: bool, color: bool) {
    let (Some(model), Some(change)) = (&bundle.model, &bundle.change) else {
        if technical {
            out.push_str("  Change impact: no task model requested.\n\n");
        }
        return;
    };
    let _ = writeln!(out, "  ACCEPTANCE PLAN  {}", text::terminal(&model.context));
    let _ = writeln!(
        out,
        "    {} affected parts  |  {} required checks  |  {} still need evidence",
        change.affected.len(),
        change.active.len(),
        change.pending.len()
    );
    let _ = writeln!(
        out,
        "    Previous results: {} reusable  |  {} need rechecking",
        change.surviving.len(),
        change.invalidated.len()
    );
    for &q in &change.active {
        if out.failed() {
            break;
        }
        let pending = change.pending.binary_search(&q).is_ok();
        let failed = change.failed.binary_search(&q).is_ok();
        let status = if failed {
            "FAILED"
        } else if pending {
            "TODO"
        } else {
            "SUPPORTED"
        };
        let tint = if !color {
            ""
        } else if failed {
            "\x1b[31m"
        } else if pending {
            "\x1b[33m"
        } else {
            "\x1b[32m"
        };
        let reset = if color { "\x1b[0m" } else { "" };
        let _ = writeln!(
            out,
            "    {tint}[{status}]{reset} {}",
            text::terminal(&model.obligations[q].name)
        );
    }
    outcomes(change, out, technical);
    if technical {
        if let Some(i) = &bundle.interaction {
            let _ = writeln!(
                out,
                "    Combined edits: {} shared checks, {} additional joint checks.",
                i.shared.len(),
                i.joint.len()
            );
            let _ = writeln!(
                out,
                "    First seed vs remaining seeds: net {:+} policy units.",
                i.net_units
            );
        }
        for &id in &change.affected {
            if out.failed() {
                break;
            }
            let previous = change.predecessor[id.0].map(|p| model.artifacts[p.0].as_str());
            let _ = writeln!(
                out,
                "    Dependency witness: {} -> {}",
                text::terminal(previous.unwrap_or("change seed")),
                text::terminal(&model.artifacts[id.0])
            );
        }
        let _ = writeln!(
            out,
            "    Policy units: required {}, pending {} (not time units)",
            change.required_units, change.pending_units
        );
    }
    out.push_str("    Exact within the declared finite model; source coverage is not proven.\n\n");
}

fn outcomes(change: &ccm_core::ChangeReport, out: &mut Buffer, technical: bool) {
    if let Some(b) = &change.behavior {
        let _ = writeln!(
            out,
            "    Modeled outcomes still possible: {} ({} compatible cases).",
            b.observations, b.hypotheses
        );
        if technical {
            let _ = writeln!(
                out,
                "    Distinction capacity: {} bits (log2 of exact outcome count).",
                b.distinction_bits
                    .map_or("unavailable".into(), |v| format!("{v:.4}"))
            );
        }
        if !b.consistent {
            out.push_str("    Evidence is inconsistent within the declared case universe.\n");
        }
    }
}

fn technical(bundle: &Bundle, out: &mut Buffer) {
    out.push_str("  TECHNICAL DETAILS\n");
    out.push_str("    Count policy: lexical-v3; mixed lines count as code; UTF-8 only.\n");
    out.push_str(
        "    Nested ignore rules and explicit selection options determine the inventory.\n",
    );
    out.push_str(
        "    Exact finite model calculations; no claim of semantic extraction completeness.\n",
    );
    for file in &bundle.current.files {
        if out.failed() {
            break;
        }
        let _ = writeln!(
            out,
            "    {}: {} code, {} comment, {} blank, {} bytes",
            text::terminal(&file.name),
            file.counts.code,
            file.counts.comments,
            file.counts.blank,
            file.counts.bytes
        );
    }
    out.push('\n');
}
