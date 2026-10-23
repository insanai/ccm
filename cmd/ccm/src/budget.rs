use crate::report::Buffer;
use crate::{Args, Bundle, economics, text};
use std::fmt::Write;

const SPAN: &str = "https://www.span.app/research/agent-effectiveness-july2026";
const LANGUAGE: &str = "https://arxiv.org/html/2607.22807v1";
const PRICES: &str = "https://developers.openai.com/api/docs/pricing";
// Span's 2026 per-merged-line costs in USD and the midpoint the default profile uses.
pub(crate) const SPAN_LOW: f64 = 0.089;
pub(crate) const SPAN_HIGH: f64 = 0.138;
pub(crate) const SPAN_MIDPOINT: f64 = 0.1135;

/// Writes the plain estimate explanation that the HTML report embeds in its details.
fn explanation(bundle: &Bundle, args: &Args, out: &mut Buffer, tint: &str, reset: &str) {
    let ai = bundle.development.estimate;
    let _ = writeln!(
        out,
        "  {tint}DEVELOPMENT ESTIMATE{reset}  USD  |  {} research profile",
        args.ai.year
    );
    for (name, value) in components(bundle, args) {
        cost_row(out, name, value);
    }
    if !bundle.development.scope_known {
        out.push_str("    Some implementation work is not mapped to source files.\n");
    }
    let _ = writeln!(
        out,
        "    {tint}{:<29} {:>16}{reset}",
        "Total",
        text::money(economics::total(bundle))
    );
    if bundle.change.is_none() {
        out.push_str("    Project-specific acceptance work is not modeled.\n");
    }
    let _ = writeln!(
        out,
        "    Planning estimate. Assumptions: docs/economics.rst ({}).",
        args.ai.year
    );
    if args.verbose {
        let p = &args.ai;
        let _ = writeln!(
            out,
            "    Priced scope: {} program lines; {}.",
            text::number(ai.lines),
            bundle.development.scope
        );
        let _ = writeln!(
            out,
            "    AI prices ${:.2} input / ${:.2} output per M tokens.",
            p.input_price, p.output_price
        );
    }
    if args.technical {
        details(bundle, args, out);
        uncertainty(bundle, out);
    }
    out.push('\n');
}

/// Returns the named cost components of the estimate in display order, each multiplied by
/// the expected attempts. Zero human components are left out.
pub(crate) fn components(bundle: &Bundle, args: &Args) -> Vec<(&'static str, f64)> {
    let a = &args.cost;
    let ai = bundle.development.estimate;
    let multiplier = bundle.cost.expected_attempts;
    let mut rows = Vec::new();
    if args.agent_override {
        rows.push(("AI and tools (manual)", a.agent_cost));
    } else {
        rows.push((
            if bundle.development.scope_known {
                "AI implementation"
            } else {
                "Mapped AI implementation"
            },
            ai.implementation_usd,
        ));
        if ai.checks_usd > 0.0 {
            rows.push(("AI acceptance checks", ai.checks_usd));
        }
        if ai.tools_usd > 0.0 {
            rows.push(("Hosted tools", ai.tools_usd));
        }
    }
    for (name, value) in [
        (
            "Human review",
            bundle.cost.review_artifacts as f64 * a.review_hours * a.hourly_cost,
        ),
        (
            "Human acceptance checks",
            bundle.cost.pending_units.unwrap_or(0) as f64 * a.check_hours * a.hourly_cost,
        ),
        (
            "Human implementation",
            a.implementation_hours * a.hourly_cost,
        ),
    ] {
        if value > 0.0 {
            rows.push((name, value));
        }
    }
    for (_, value) in &mut rows {
        *value *= multiplier;
    }
    rows
}

fn cost_row(out: &mut Buffer, name: &str, value: f64) {
    let _ = writeln!(out, "    {name:<29} {:>16}", text::money(value));
}

/// Writes the caveats of a partial or generic estimate, one per line.
pub(crate) fn uncertainty(bundle: &Bundle, out: &mut Buffer) {
    if !bundle.development.scope_known {
        out.push_str(concat!(
            "    Partial: some model seeds do not match source paths; ",
            "their implementation is unpriced.\n"
        ));
    }
    if bundle.change.is_none() {
        out.push_str(concat!(
            "    Generic agent iteration included; ",
            "project-specific acceptance checks are unknown.\n"
        ));
    }
    if bundle.current.unsupported_files > 0 {
        out.push_str(
            "    Unknown file types may contain additional implementation work; excluded.\n",
        );
    }
    if !bundle.current.complete {
        out.push_str("    Partial: source coverage is incomplete; excluded work is unpriced.\n");
    }
}

/// Writes the price card, token priors, and research sources behind the estimate.
pub(crate) fn details(bundle: &Bundle, args: &Args, out: &mut Buffer) {
    let p = &args.ai;
    let a = bundle.development.estimate;
    let _ = writeln!(out, "    Price card: {}", text::terminal(&p.model));
    let _ = writeln!(
        out,
        "    Input ${:.2}/M; output/reasoning ${:.2}/M; uncached standard prices.",
        p.input_price, p.output_price
    );
    let _ = writeln!(
        out,
        "    Inferred implementation tokens: {:.0} input + {:.0} output/reasoning.",
        a.input_tokens, a.output_tokens
    );
    let _ = writeln!(
        out,
        "    Prior per program line: {:.2} input + {:.2} output; workload x{:.2}.",
        p.input_per_line, p.output_per_line, p.workload
    );
    let _ = writeln!(
        out,
        "    Additional declared checks: {:.0} tokens; {:.0} tokens/pending policy unit.",
        a.check_tokens, p.check_tokens
    );
    let _ = writeln!(
        out,
        "    Tools: {:.0} assumed sessions x ${:.2}; {:.0} lines/20-minute 1 GB session.",
        a.sessions, p.session_price, p.lines_per_session
    );
    let _ = writeln!(
        out,
        "    Human: {} parts x {:.2} review hours + {} pending units x {:.2} hours; ${:.2}/hour.",
        bundle.cost.review_artifacts,
        args.cost.review_hours,
        bundle
            .cost
            .pending_units
            .map_or("unknown".into(), |n| n.to_string()),
        args.cost.check_hours,
        args.cost.hourly_cost
    );
    for (language, lines, factor) in &bundle.development.languages {
        if out.failed() {
            break;
        }
        let usd = (*lines as f64 * p.input_per_line * p.input_price
            + *lines as f64 * factor * p.output_per_line * p.output_price)
            * p.workload
            / 1_000_000.0;
        let _ = writeln!(
            out,
            "    {language}: {lines} lines; output x{factor:.2}; {} implementation.",
            text::money(usd)
        );
    }
    out.push_str(
        "    Span 2026: $4.45/$6.90 per 50 merged AI lines; chosen midpoint $0.1135/line.\n",
    );
    out.push_str(concat!(
        "    Default token conversion: 90% input at reference $10/$50 per M.\n",
        "    This is an inferred conversion, not measured tokens per line.\n"
    ));
    out.push_str(concat!(
        "    Language factors transfer July 2026 task-level results.\n",
        "    Unmeasured languages default to x1; profile overrides are allowed.\n"
    ));
    out.push_str(
        "    Check token budget and session throughput are R&D guesses, independently editable.\n",
    );
    out.push_str(concat!(
        "    Specific acceptance work is additional to generic iteration.\n",
        "    Calibrate those budgets separately to avoid overlapping work.\n"
    ));
    let _ = writeln!(
        out,
        concat!(
            "    Calibration: {SPAN}\n    Languages: {LANGUAGE}\n",
            "    Default price snapshot (2026-10-23): {PRICES}"
        ),
        SPAN = SPAN,
        LANGUAGE = LANGUAGE,
        PRICES = PRICES
    );
}

pub(crate) fn html(bundle: &Bundle, args: &Args, out: &mut Buffer) {
    out.push_str("<section><h2>Development budget</h2><p>2026 research profile. ");
    out.push_str("Inferred resource use with unverified R&amp;D assumptions.</p>");
    if !bundle.development.scope_known || !bundle.current.complete {
        out.push_str(concat!(
            "<p class=warning>Partial scope: some implementation work ",
            "is unpriced. We can inspect the scope below.</p>"
        ));
    }
    out.push_str("<table>");
    let mut rows = components(bundle, args);
    rows.push(("Total", economics::total(bundle)));
    for (name, value) in rows {
        let _ = write!(
            out,
            "<tr><th>{name}</th><td>{} USD</td></tr>",
            text::money(value)
        );
    }
    out.push_str("</table><details><summary>Scope, assumptions, and research</summary><pre>");
    let mut plain = Buffer::new(args.max_report_bytes);
    explanation(bundle, args, &mut plain, "", "");
    if !args.technical {
        details(bundle, args, &mut plain);
    }
    match plain.finish() {
        Ok(explanation) => {
            for line in explanation.lines() {
                if out.failed() {
                    break;
                }
                out.push_str(&text::html(line));
                out.push('\n');
            }
        }
        Err(_) => out.fail(),
    }
    out.push_str(
        "</pre></details><p>Not measured spending, asset value, or a quote.</p></section>",
    );
}

pub(crate) fn json(bundle: &Bundle, args: &Args, out: &mut Buffer) {
    let ai = bundle.development.estimate;
    let _ = write!(
        out,
        concat!(
            "{{\"profile\":\"ai-development-2026-v1\",\"year\":{},\"price_date\":\"2026-10-23\",",
            "\"status\":\"inferred-unvalidated-rd-scenario\",\"scope\":{},\"scope_known\":{},",
            "\"program_lines\":{},\"input_tokens_inferred\":{},\"output_tokens_inferred\":{},",
            "\"implementation_usd_per_attempt\":{},\"checks_usd_per_attempt\":{},",
            "\"check_tokens_inferred\":{},\"tools_usd_per_attempt\":{},\"sessions_assumed\":{},",
            "\"manual_override\":{},\"overall_expected_usd\":{},\"assumptions\":{{"
        ),
        args.ai.year,
        text::json(bundle.development.scope),
        bundle.development.scope_known,
        ai.lines,
        ai.input_tokens,
        ai.output_tokens,
        ai.implementation_usd,
        ai.checks_usd,
        ai.check_tokens,
        ai.tools_usd,
        ai.sessions,
        args.agent_override,
        economics::total(bundle)
    );
    // The sensitivity band rescales the default calibration midpoint to Span's low and
    // high per-line costs. It means nothing for a custom profile, so we report null.
    let sensitivity = if args.custom_profile {
        "null".to_owned()
    } else {
        format!(
            "[{},{}]",
            ai.implementation_usd * SPAN_LOW / SPAN_MIDPOINT,
            ai.implementation_usd * SPAN_HIGH / SPAN_MIDPOINT
        )
    };
    let _ = write!(
        out,
        concat!(
            "\"acceptance_scope_known\":{},\"workload_driver\":\"program-lines-v1\",",
            "\"implementation_sensitivity_usd\":{},\"excluded_generated\":true,"
        ),
        bundle.change.is_some(),
        sensitivity
    );
    let p = &args.ai;
    let _ = write!(
        out,
        concat!(
            "\"model\":{},\"input_usd_per_million\":{},\"output_usd_per_million\":{},",
            "\"input_per_line\":{},\"output_per_line\":{},\"check_tokens_per_unit\":{},",
            "\"usd_per_session\":{},\"lines_per_session\":{},\"workload_multiplier\":{}}},",
            "\"language_workload\":["
        ),
        text::json(&p.model),
        p.input_price,
        p.output_price,
        p.input_per_line,
        p.output_per_line,
        p.check_tokens,
        p.session_price,
        p.lines_per_session,
        p.workload
    );
    for (index, (language, lines, factor)) in bundle.development.languages.iter().enumerate() {
        if out.failed() {
            break;
        }
        if index > 0 {
            out.push(',');
        }
        let _ = write!(
            out,
            "{{\"language\":{},\"lines\":{},\"output_factor\":{}}}",
            text::json(language),
            lines,
            factor
        );
    }
    let _ = write!(
        out,
        "],\"sources\":[{},{},{}]}}",
        text::json(SPAN),
        text::json(LANGUAGE),
        text::json(PRICES)
    );
}
