use std::fmt::Write;

pub(super) fn render(kind: &str) -> String {
    let (width, height) = if kind == "sequence" {
        (940, 650)
    } else {
        (900, 460)
    };
    let mut s = format!(
        concat!(
            "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 {} {}\" ",
            "role=\"img\"><title>CCM {}</title><defs>",
            "<marker id=\"arrow\" markerWidth=\"8\" markerHeight=\"8\" refX=\"7\" refY=\"4\" ",
            "orient=\"auto\"><path d=\"M0 0L8 4L0 8Z\" fill=\"#79529d\"/></marker>",
            "</defs><rect width=\"100%\" height=\"100%\" rx=\"16\" fill=\"#fffcfa\"/>",
            "<g font-family=\"DejaVu Sans,sans-serif\" fill=\"#302638\">"
        ),
        width, height, kind
    );
    match kind {
        "architecture" => architecture(&mut s),
        "sequence" => sequence(&mut s),
        "change" => change(&mut s),
        "economics" => economics(&mut s),
        _ => {}
    }
    s.push_str("</g></svg>\n");
    s
}

fn text(s: &mut String, x: i32, y: i32, value: &str, size: i32, bold: bool) {
    let _ = write!(
        s,
        concat!("<text x=\"{}\" y=\"{}\" font-size=\"{}\" font-weight=\"{}\">{}</text>"),
        x,
        y,
        size,
        if bold { 700 } else { 400 },
        value
    );
}

fn box_label(s: &mut String, x: i32, y: i32, w: i32, title: &str, detail: &str, color: &str) {
    let _ = write!(
        s,
        concat!(
            "<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"68\" rx=\"10\" ",
            "fill=\"{}\" stroke=\"#cdbad9\"/>"
        ),
        x, y, w, color
    );
    text(s, x + 18, y + 27, title, 19, true);
    text(s, x + 18, y + 50, detail, 15, false);
}

fn arrow(s: &mut String, x: i32, y: i32, end_x: i32, end_y: i32, dashed: bool) {
    let _ = write!(
        s,
        concat!(
            "<path d=\"M{} {}L{} {}\" fill=\"none\" stroke=\"#79529d\" ",
            "stroke-width=\"1.6\" marker-end=\"url(#arrow)\" stroke-dasharray=\"{}\"/>"
        ),
        x,
        y,
        end_x,
        end_y,
        if dashed { "5 4" } else { "none" }
    );
}

fn architecture(s: &mut String) {
    text(
        s,
        40,
        35,
        "Four crates. Dependencies point downward.",
        20,
        true,
    );
    for (i, title, detail, color) in [
        (
            0,
            "cmd/ccm",
            "Arguments, report bundle, terminal / HTML / JSON",
            "#fbe4f0",
        ),
        (
            1,
            "ccm-project",
            "Selection, bounded workers, snapshots, byte comparison",
            "#eee4fa",
        ),
        (
            2,
            "ccm-scan",
            "Native lexers, mixed documents, declarative catalog",
            "#fbe9db",
        ),
        (
            3,
            "ccm-core",
            "Counts, errors, finite models, evidence, resource prices",
            "#f1eafa",
        ),
    ] {
        box_label(s, 185, 58 + i * 94, 565, title, detail, color);
        if i < 3 {
            arrow(s, 470, 126 + i * 94, 470, 150 + i * 94, false);
        }
    }
    text(
        s,
        40,
        441,
        "The CLI and project adapter also use ccm-core directly. The core never reads files.",
        13,
        false,
    );
}

fn sequence(s: &mut String) {
    text(
        s,
        35,
        32,
        "One invocation, from options to a report",
        20,
        true,
    );
    let xs = [80, 270, 460, 650, 840];
    for (x, label) in xs
        .into_iter()
        .zip(["CLI", "Project", "Workers", "Finite core", "Renderer"])
    {
        let _ = write!(
            s,
            concat!("<path d=\"M{} 95V610\" stroke=\"#d5cadf\" stroke-dasharray=\"4 5\"/>"),
            x
        );
        box_label(s, x - 65, 54, 130, label, "", "#f2eafa");
    }
    for (from, to, y, label, dashed) in [
        (0, 1, 155, "select paths and limits", false),
        (1, 2, 210, "bounded file queue", false),
        (2, 1, 265, "counts + diagnostics", true),
        (1, 0, 320, "stable source inventory", true),
        (0, 3, 375, "model + change seeds", false),
        (3, 0, 430, "closure, checks, evidence", true),
        (0, 4, 485, "bundle + explicit prices", false),
        (4, 0, 555, "bounded complete report", true),
    ] {
        arrow(s, xs[from], y, xs[to], y, dashed);
        text(s, xs[from].min(xs[to]) + 10, y - 10, label, 16, false);
    }
    text(
        s,
        35,
        630,
        "Selection overlaps workers. The core evaluates evidence; no checker runs.",
        13,
        false,
    );
}

fn change(s: &mut String) {
    text(s, 35, 38, "Two edit seeds. Three affected parts.", 22, true);
    box_label(s, 40, 95, 220, "pricing.rs", "changed seed", "#fbe0ef");
    box_label(s, 40, 260, 220, "inventory.rs", "changed seed", "#fbe0ef");
    box_label(
        s,
        350,
        177,
        230,
        "checkout.rs",
        "dependent / review scope",
        "#e8ddf6",
    );
    box_label(
        s,
        660,
        177,
        200,
        "payment.rs",
        "unchanged boundary",
        "#ffecd9",
    );
    arrow(s, 260, 129, 350, 201, false);
    arrow(s, 260, 294, 350, 221, false);
    text(s, 360, 282, "Affected does not mean rewritten.", 13, false);
    text(s, 635, 282, "No dependency edge is declared.", 13, false);
    text(
        s,
        40,
        391,
        "Pricing and checkout evidence is invalidated. Payment evidence survives.",
        16,
        true,
    );
    text(
        s,
        40,
        420,
        "Required check units: 7. Remaining check units: 6. Shared: 1. Additional joint: 3.",
        14,
        false,
    );
}

fn economics(s: &mut String) {
    text(
        s,
        35,
        38,
        "Price declared resources; preserve their units.",
        21,
        true,
    );
    box_label(
        s,
        40,
        85,
        350,
        "Source workload",
        "Program lines + output language factors",
        "#fbe0ef",
    );
    box_label(
        s,
        510,
        85,
        350,
        "Residual acceptance work",
        "CCC pending policy units",
        "#eee2fa",
    );
    box_label(
        s,
        40,
        215,
        350,
        "AI usage + hosted tools",
        "Editable 2026 workload and price profile",
        "#ffecd9",
    );
    box_label(
        s,
        510,
        215,
        350,
        "Human review + checks",
        "Explicit hours and loaded hourly price",
        "#ffecd9",
    );
    arrow(s, 215, 153, 215, 213, false);
    arrow(s, 685, 153, 685, 213, false);
    arrow(s, 510, 140, 390, 230, true);
    text(
        s,
        370,
        309,
        "Agent checks have a separate token budget.",
        13,
        false,
    );
    arrow(s, 215, 283, 375, 353, false);
    arrow(s, 685, 283, 525, 353, false);
    box_label(
        s,
        235,
        357,
        440,
        "Expected resource cost",
        "Sum disjoint activities; apply explicit repeat scenario",
        "#ede2f8",
    );
}
