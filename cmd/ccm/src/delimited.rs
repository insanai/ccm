use crate::report::Buffer;
use crate::{Args, Bundle, json, text};
use std::fmt::Write;

pub(crate) fn csv(bundle: &Bundle, args: &Args) -> Result<String, ccm_core::Error> {
    let mut out = Buffer::new(args.max_report_bytes);
    out.push_str("path,language,kind,bytes,lines,code,comments,blank,generated\r\n");
    for file in &bundle.current.files {
        if out.failed() {
            break;
        }
        let c = file.counts;
        let kind = match file.language.kind() {
            ccm_scan::FileKind::Source => "source",
            ccm_scan::FileKind::Documentation => "documentation",
            ccm_scan::FileKind::Data => "data",
        };
        let _ = write!(
            out,
            "{},{},{kind},{},{},{},{},{},{}\r\n",
            csv_field(&file.name),
            csv_field(file.language.name()),
            c.bytes,
            c.lines(),
            c.code,
            c.comments,
            c.blank,
            file.generated
        );
    }
    out.finish()
}

pub(crate) fn ndjson(bundle: &Bundle, args: &Args) -> Result<String, ccm_core::Error> {
    let mut out = Buffer::new(args.max_report_bytes);
    for file in &bundle.current.files {
        if out.failed() {
            break;
        }
        let _ = writeln!(
            out,
            concat!(
                "{{\"type\":\"file\",\"path\":{},\"language\":{},\"code\":{},",
                "\"comments\":{},\"blank\":{},\"bytes\":{}}}"
            ),
            text::json(&file.name),
            text::json(file.language.name()),
            file.counts.code,
            file.counts.comments,
            file.counts.blank,
            file.counts.bytes
        );
    }
    let summary = json::render(bundle, args)?;
    out.push_str("{\"type\":\"report\",\"report\":");
    // Whitespace outside strings can be removed without changing embedded source names.
    let mut quoted = false;
    let mut escaped = false;
    for c in summary.chars() {
        if out.failed() {
            break;
        }
        if quoted || !c.is_ascii_whitespace() {
            out.push(c);
        }
        if escaped {
            escaped = false;
        } else if c == '\\' && quoted {
            escaped = true;
        } else if c == '"' {
            quoted = !quoted;
        }
    }
    out.push_str("}\n");
    out.finish()
}

fn csv_field(value: &str) -> String {
    let value = if value.starts_with(['=', '+', '-', '@', '\t', '\r']) {
        format!("'{value}")
    } else {
        value.into()
    };
    format!("\"{}\"", value.replace('"', "\"\""))
}
