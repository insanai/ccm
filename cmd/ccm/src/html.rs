//! The standalone HTML report: one offline file with inline styles and a small script.
//!
//! The page has three layers. A static layer, written here, holds the headline numbers and
//! tables and needs no script. A data island holds every figure of the report as JSON.
//! The script reads the island and renders the interactive report: sortable tables, a
//! directory explorer, paginated files, the budget breakdown and the change closure.

mod data;
mod escape;
mod fallback;
mod model_data;

use crate::report::Buffer;
use crate::{Args, Bundle};
use std::fmt::Write;

/// Applies a theme the reader chose earlier, before the first paint.
const THEME: &str = "try{var t=localStorage.getItem('guidedog-theme');\
    if(t==='dark'||t==='light')document.documentElement.dataset.theme=t}catch(e){}";

/// Renders the HTML report for `bundle` within `--max-report-bytes`.
/// Fails with the report-limit error when the page would exceed that budget.
pub(crate) fn render(bundle: &Bundle, args: &Args) -> Result<String, ccm_core::Error> {
    let island = data::render(bundle, args, args.max_report_bytes)?;
    let mut out = Buffer::new(args.max_report_bytes);
    out.push_str(include_str!("report-head.html"));
    out.push_str("<style>");
    out.push_str(include_str!("html/report.css"));
    out.push_str("</style><script>");
    out.push_str(THEME);
    out.push_str("</script></head><body>");
    header(bundle, &mut out);
    out.push_str("<nav id=nav class=nav aria-label='Report sections'></nav>");
    out.push_str("<main id=app>");
    out.push_str(
        "<noscript><p class=note>Scripts are off. This is the static summary; \
         enable scripts for the explorer, file search and drill-down.</p></noscript>",
    );
    fallback::render(bundle, args, &mut out);
    out.push_str("</main>");
    let _ = write!(
        out,
        "<footer><span>CCM {} &middot; Local report &middot; No remote assets</span>",
        env!("CARGO_PKG_VERSION")
    );
    out.push_str("<span>Counts are lexical. Estimates are planning scenarios.</span></footer>");
    out.push_str("<script type='application/json' id=ccm-data>");
    out.push_str(&island);
    out.push_str("</script><script>");
    out.push_str(include_str!("html/core.js"));
    out.push_str(include_str!("html/inventory.js"));
    out.push_str(include_str!("html/change.js"));
    out.push_str(include_str!("html/main.js"));
    out.push_str("</script></body></html>\n");
    out.finish()
}

fn header(bundle: &Bundle, out: &mut Buffer) {
    out.push_str("<header class=masthead><div class=brand><div class=brand-mark>");
    out.push_str(include_str!("html/ccm-logo.svg"));
    out.push_str("</div><div><span class=eyebrow>CODE COUNTER MACHINE</span>");
    out.push_str("<h1>Source &amp; change intelligence</h1></div></div>");
    out.push_str("<p class=lede>A clear inventory. An explicit acceptance plan. ");
    out.push_str("A budget we can inspect.</p><div class=chips>");
    let status = if bundle.current.complete {
        "<span class='chip ok'>Complete selection</span>"
    } else {
        "<span class='chip warn'>Incomplete selection</span>"
    };
    out.push_str(status);
    let _ = write!(
        out,
        "<span class=chip>Count policy {}</span><span class=chip>CCM {}</span>",
        escape::text(ccm_scan::COUNT_POLICY),
        env!("CARGO_PKG_VERSION")
    );
    if bundle.model.is_some() {
        out.push_str("<span class=chip>Change model</span>");
    }
    if bundle.changes.is_some() {
        out.push_str("<span class=chip>Comparison</span>");
    }
    out.push_str("</div></header>");
}
