use std::fmt::Write;

pub(super) fn render(source: &str) -> String {
    let mut out = String::from(concat!(
        "<!doctype html><html lang=en><meta charset=utf-8>",
        "<meta name=viewport content='width=device-width,initial-scale=1'>",
        "<title>CCM live terminal capture</title><style>",
        "*{box-sizing:border-box}body{margin:0;padding:30px;background:#fff6f0;color:#e9dfef}",
        ".terminal{background:#251c30;border:1px solid #513a60;border-radius:14px;",
        "box-shadow:0 18px 50px #54316122;overflow:hidden}",
        "header{display:flex;gap:8px;padding:16px 20px;border-bottom:1px solid #493451;",
        "font:12px system-ui;color:#bfaccb}i{width:9px;height:9px;border-radius:50%;",
        "background:#ef82ae}i:nth-child(2){background:#bba0e8}i:nth-child(3){background:#f7b175}",
        "header span{margin-left:12px}pre{margin:0;padding:12px 18px 22px;",
        "font:15px/1.55 'DejaVu Sans Mono',monospace;white-space:pre-wrap}",
        ".bold{font-weight:bold}.dim{color:#ac98b9}.c31{color:#ff8e8e}",
        ".c32{color:#b6dfb3}.c33{color:#ffb77b}.c35,.c95{color:#f58dbd}",
        ".c36{color:#c5a5f2}.c94{color:#b7a5fb}</style><body><div class=terminal>",
        "<header><i></i><i></i><i></i><span>CCM / local measurement</span></header><pre>"
    ));
    let mut remaining = source;
    let mut spans = 0;
    while let Some(index) = remaining.find('\x1b') {
        escape(&mut out, &remaining[..index]);
        remaining = &remaining[index + 1..];
        let Some(end) = remaining.find('m') else {
            break;
        };
        let codes = remaining[..end].trim_start_matches('[');
        for _ in 0..spans {
            out.push_str("</span>");
        }
        spans = 0;
        for code in codes.split(';') {
            let class = match code {
                "1" => Some("bold".into()),
                "2" => Some("dim".into()),
                "31" | "32" | "33" | "35" | "36" | "94" | "95" => Some(format!("c{code}")),
                _ => None,
            };
            if let Some(class) = class {
                let _ = write!(out, "<span class={class}>");
                spans += 1;
            }
        }
        remaining = &remaining[end + 1..];
    }
    escape(&mut out, remaining);
    for _ in 0..spans {
        out.push_str("</span>");
    }
    out.push_str("</pre></div></body></html>");
    out
}

fn escape(out: &mut String, text: &str) {
    for c in text.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            _ => out.push(c),
        }
    }
}
