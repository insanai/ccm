use crate::args::Format;
use crate::report::Buffer;
use crate::{Args, Error, text};
use std::fmt::Write;

pub(crate) fn render(args: &Args) -> Result<String, Error> {
    let mut out = Buffer::new(args.max_report_bytes);
    if args.format == Format::Json {
        out.push_str("{\"schema\":\"ccm.languages.v2\",\"languages\":[");
    }
    if args.format == Format::Csv {
        out.push_str("language,kind,extensions,filenames\r\n");
    }
    if args.format == Format::Html {
        out.push_str("<!doctype html><html lang=en><meta charset=utf-8><pre>");
    }
    for (index, info) in ccm_scan::languages().iter().enumerate() {
        if out.failed() {
            break;
        }
        match args.format {
            Format::Json | Format::Ndjson => {
                if args.format == Format::Json && index > 0 {
                    out.push(',');
                }
                let extensions = info
                    .extensions
                    .iter()
                    .map(|s| text::json(s))
                    .collect::<Vec<_>>()
                    .join(",");
                let filenames = info
                    .filenames
                    .iter()
                    .map(|s| text::json(s))
                    .collect::<Vec<_>>()
                    .join(",");
                let _ = write!(
                    out,
                    concat!(
                        "{{\"name\":{},\"kind\":{},\"policy\":{},",
                        "\"extensions\":[{}],\"filenames\":[{}]}}"
                    ),
                    text::json(info.name),
                    text::json(info.kind.name()),
                    text::json(info.language.policy()),
                    extensions,
                    filenames
                );
                if args.format == Format::Ndjson {
                    out.push('\n');
                }
            }
            Format::Csv => {
                let _ = write!(
                    out,
                    "\"{}\",{},\"{}\",\"{}\"\r\n",
                    info.name,
                    info.kind.name(),
                    info.extensions.join(","),
                    info.filenames.join(",")
                );
            }
            Format::Terminal | Format::Html => {
                let line = format!(
                    "{:<18} {}  {}",
                    info.name,
                    info.extensions.join(","),
                    info.filenames.join(",")
                );
                if args.format == Format::Html {
                    out.push_str(&text::html(&line));
                    out.push('\n');
                } else {
                    out.push_str(&line);
                    out.push('\n');
                }
            }
        }
    }
    if args.format == Format::Json {
        out.push_str("]}\n");
    }
    if args.format == Format::Html {
        out.push_str("</pre></html>\n");
    }
    out.finish()
}
