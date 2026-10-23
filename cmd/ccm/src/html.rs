use crate::report::Buffer;
use crate::{Args, Bundle, text};
use std::fmt::Write;

pub(crate) fn render(bundle: &Bundle, args: &Args) -> Result<String, ccm_core::Error> {
    let mut out = Buffer::new(args.max_report_bytes);
    out.push_str(include_str!("report-head.html"));
    out.push_str("<header><div class=brand-mark>");
    out.push_str(include_str!("../../../assets/ccm-logo.svg"));
    out.push_str("</div><span class=eyebrow>CODE COUNTER MACHINE</span>");
    out.push_str("<h1>Source &amp; change intelligence</h1>");
    out.push_str(
        "<p>A clear inventory. An explicit acceptance plan. A budget we can inspect.</p></header>",
    );
    out.push_str("<main><section class=metrics aria-label='Overview'>");
    card(
        &mut out,
        "Selected files",
        &text::number(bundle.current.files.len() as u64),
    );
    card(
        &mut out,
        "Program lines",
        &text::number(bundle.current.content_lines(ccm_scan::FileKind::Source)),
    );
    card(
        &mut out,
        if args.no_cost {
            "Documentation content"
        } else {
            "Budget scenario (USD)"
        },
        &if args.no_cost {
            text::number(
                bundle
                    .current
                    .content_lines(ccm_scan::FileKind::Documentation),
            )
        } else {
            text::money(crate::economics::total(bundle))
        },
    );
    out.push_str("</section>");
    if bundle.model.is_some() {
        model(bundle, &mut out, args.technical);
    }
    if !args.no_cost {
        crate::budget::html(bundle, args, &mut out);
    }
    out.push_str("<section><h2>Source inventory</h2>");
    out.push_str("<div class=scroll><table><thead><tr><th>File</th><th>Language</th>");
    out.push_str("<th>Code</th><th>Comments</th><th>Blank</th></tr></thead><tbody>");
    for file in &bundle.current.files {
        if out.failed() {
            break;
        }
        let _ = write!(
            out,
            "<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>",
            text::html(&file.name),
            text::html(file.language.name()),
            text::number(file.counts.code),
            text::number(file.counts.comments),
            text::number(file.counts.blank)
        );
    }
    out.push_str("</tbody></table></div></section>");
    if let Some(changes) = &bundle.changes {
        out.push_str("<section><h2>Changed files</h2><ul>");
        for change in changes {
            if out.failed() {
                break;
            }
            let _ = write!(
                out,
                "<li><span class=badge>{}</span> {}</li>",
                change.kind,
                text::html(&change.name)
            );
        }
        out.push_str("</ul></section>");
    }
    coverage(bundle, &mut out);
    let _ = write!(
        out,
        "</main><footer>CCM {} &middot; Local report &middot; No scripts or remote assets",
        env!("CARGO_PKG_VERSION")
    );
    out.push_str("</footer></body></html>\n");
    out.finish()
}

fn coverage(bundle: &Bundle, out: &mut Buffer) {
    let _ = write!(
        out,
        "<section><h2>Coverage</h2><p>{} unsupported files excluded.</p>",
        bundle.current.unsupported_files
    );
    if !bundle.current.complete {
        out.push_str("<p class=warning>Incomplete: some source files were not counted.</p>");
    }
    for notice in &bundle.current.notices {
        if out.failed() {
            break;
        }
        let _ = write!(
            out,
            "<p class=warning>{}: {}</p>",
            text::html(&notice.path),
            text::html(&notice.message)
        );
    }
    out.push_str("<p>Lexical policy v2. Mixed lines count as code. Symlinks are skipped. ");
    out.push_str("Selection follows the configured ignore rules and exclusions.</p>");
    out.push_str("<p>Source counting does not establish semantic coverage. ");
    out.push_str("Task outcomes require declared cases.</p></section>");
}

fn card(out: &mut Buffer, label: &str, value: &str) {
    let _ = write!(
        out,
        "<div class=metric><span>{label}</span><strong>{value}</strong></div>"
    );
}

fn model(bundle: &Bundle, out: &mut Buffer, technical: bool) {
    out.push_str("<section><h2>Acceptance plan</h2>");
    if let (Some(model), Some(change)) = (&bundle.model, &bundle.change) {
        let _ = write!(
            out,
            "<p>{}</p><p>{} affected parts; {} checks still need evidence. ",
            text::html(&model.context),
            change.affected.len(),
            change.pending.len()
        );
        let _ = write!(
            out,
            "{} previous results need rechecking.</p><ul>",
            change.invalidated.len()
        );
        for &q in &change.active {
            if out.failed() {
                break;
            }
            let status = if change.failed.binary_search(&q).is_ok() {
                "Failed check"
            } else if change.pending.binary_search(&q).is_ok() {
                "Needs evidence"
            } else {
                "Supported"
            };
            let _ = write!(
                out,
                "<li><span class=badge>{status}</span> {}</li>",
                text::html(&model.obligations[q].name)
            );
        }
        out.push_str("</ul>");
        if let Some(b) = &change.behavior {
            let _ = write!(
                out,
                "<p>Modeled outcomes still possible: {}. Consistent evidence: {}.</p>",
                b.observations, b.consistent
            );
        }
        out.push_str("<p>Exact within the author-declared model. ");
        out.push_str(
            "Evidence acceptance and source binding are supplied by the model author.</p>",
        );
        if technical {
            if let Some(i) = &bundle.interaction {
                let _ = write!(
                    out,
                    "<p>First seed versus remaining seeds: {} shared checks, ",
                    i.shared.len()
                );
                let _ = write!(
                    out,
                    "{} additional joint checks; net {:+} policy units.</p>",
                    i.joint.len(),
                    i.net_units
                );
            }
            out.push_str("<h3>Dependency witnesses</h3><ul>");
            for id in &change.affected {
                if out.failed() {
                    break;
                }
                let previous = change.predecessor[id.0].map(|p| model.artifacts[p.0].as_str());
                let _ = write!(
                    out,
                    "<li>{} &rarr; {}</li>",
                    text::html(previous.unwrap_or("Change seed")),
                    text::html(&model.artifacts[id.0])
                );
            }
            out.push_str("</ul>");
        }
    } else {
        out.push_str(
            "<p>Change impact and required checks are unavailable without a task model.</p>",
        );
    }
    out.push_str("</section>");
}
