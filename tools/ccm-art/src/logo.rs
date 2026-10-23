use std::fmt::Write;

const C: &str = "M88 14 Q75 0 50 0 H34 Q0 0 0 34 V66 Q0 100 34 100 H50 \
    Q75 100 88 86 L70 67 Q63 76 49 76 H38 Q25 76 25 63 V37 Q25 24 38 24 \
    H49 Q63 24 70 33 Z";
const M: &str = "M0 100 V0 H24 L52 43 80 0 H104 V100 H79 V43 L52 82 \
    25 43 V100 Z";

pub(super) fn render(mono: bool) -> String {
    let mut out = String::with_capacity(14000);
    out.push_str(concat!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 440 250\" ",
        "role=\"img\" aria-labelledby=\"ccm-logo-title ccm-logo-desc\">\n",
        "<title id=\"ccm-logo-title\">CCM</title>\n",
        "<desc id=\"ccm-logo-desc\">Three sculpted letters in pink, purple, and orange. ",
        "A measured view of code, change, and confidence.</desc>\n"
    ));
    if !mono {
        gradients(&mut out);
        out.push_str(concat!(
            "<ellipse cx=\"229\" cy=\"201\" rx=\"163\" ry=\"15\" ",
            "fill=\"#33204f\" opacity=\".07\"/>\n"
        ));
    }
    out.push_str("<g transform=\"matrix(1 -.10 -.32 .88 71 88)\">\n");
    for (index, (path, name, side)) in [
        (C, "pink", (151, 39, 103)),
        (C, "purple", (72, 36, 138)),
        (M, "orange", (176, 70, 27)),
    ]
    .into_iter()
    .enumerate()
    {
        let _ = writeln!(out, "<g transform=\"translate({} 0)\">", index * 109);
        if mono {
            let _ = writeln!(
                out,
                "<path d=\"{path}\" fill=\"none\" stroke=\"#241d2b\" stroke-width=\"1.8\"/>"
            );
        } else {
            depth(&mut out, path, side);
            let _ = writeln!(
                out,
                concat!(
                    "<path d=\"{}\" fill=\"url(#ccm-{})\" stroke=\"#fff\" ",
                    "stroke-opacity=\".38\" stroke-width=\".65\"/>"
                ),
                path, name
            );
        }
        out.push_str("</g>\n");
    }
    out.push_str("</g></svg>\n");
    out
}

fn gradients(out: &mut String) {
    out.push_str("<defs>\n");
    for (name, top, base, bottom) in [
        ("pink", "#ffb1d3", "#f05ba4", "#c93881"),
        ("purple", "#c9b1ff", "#9970ee", "#7041c1"),
        ("orange", "#ffdaa5", "#ffae62", "#f17e3d"),
    ] {
        let _ = writeln!(
            out,
            concat!(
                "<linearGradient id=\"ccm-{}\" x1=\"0\" y1=\"0\" x2=\".7\" y2=\"1\">",
                "<stop stop-color=\"{}\"/><stop offset=\".45\" stop-color=\"{}\"/>",
                "<stop offset=\"1\" stop-color=\"{}\"/></linearGradient>"
            ),
            name, top, base, bottom
        );
    }
    out.push_str("</defs>\n");
}

fn depth(out: &mut String, path: &str, side: (u16, u16, u16)) {
    for step in (1..=18).rev() {
        let (r, g, b) = (side.0 + step, side.1 + step, side.2 + step);
        let _ = writeln!(
            out,
            concat!(
                "<path d=\"{}\" transform=\"translate({:.2} {:.2})\" ",
                "fill=\"#{:02x}{:02x}{:02x}\"/>"
            ),
            path,
            step as f64 * 0.58,
            step as f64 * 0.92,
            r,
            g,
            b
        );
    }
}
