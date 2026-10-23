//! CCM command-line application.
mod args;
mod budget;
mod color;
mod delimited;
mod economics;
mod html;
mod inventory;
mod json;
mod language_report;
mod notices;
mod output;
mod report;
mod terminal;
mod text;

use args::{Args, Format};
use ccm_core::{ArtifactId, ChangeReport, CostEstimate, Error, Model, estimate_cost, parse_model};
use ccm_project::{FileChange, Report, compare_reports, measure_paths, read_source};

struct Bundle {
    current: Report,
    baseline: Option<Report>,
    changes: Option<Vec<FileChange>>,
    model: Option<Model>,
    change: Option<ChangeReport>,
    cost: CostEstimate,
    development: economics::Development,
    seeds: Vec<ArtifactId>,
    interaction: Option<ccm_core::InteractionReport>,
}

fn main() -> std::process::ExitCode {
    let arguments = std::env::args_os()
        .skip(1)
        .map(|word| {
            word.into_string().map_err(|_| {
                Error::new(
                    "argument-encoding",
                    "A command-line argument is not UTF-8",
                    "Use UTF-8 paths and option values.",
                )
            })
        })
        .collect::<Result<Vec<_>, _>>();
    match arguments.and_then(run) {
        Ok(code) => std::process::ExitCode::from(code),
        Err(error) => {
            if let Some(location) = &error.location {
                eprintln!(
                    "-- {} --- {}:{}:{}\n\n{} | {}\n    ^\n\n{}\n\nHint: {}",
                    error.code.to_ascii_uppercase(),
                    text::terminal(location.path.as_deref().unwrap_or("input")),
                    location.line,
                    location.column,
                    location.line,
                    text::terminal(&location.excerpt.replace('\t', "    ")),
                    text::terminal(&error.message),
                    error.hint
                );
                return std::process::ExitCode::from(1);
            }
            eprintln!(
                "-- {} ----------------------------------------\n\n{}\n\nHint: {}",
                error.code.to_ascii_uppercase(),
                text::terminal(&error.message),
                error.hint
            );
            std::process::ExitCode::from(1)
        }
    }
}

fn run(words: Vec<String>) -> Result<u8, Error> {
    let args = args::parse(words.into_iter())?;
    if args.help {
        output::write(&args, args::HELP)?;
        return Ok(0);
    }
    if args.version {
        output::write(&args, &format!("ccm {}\n", env!("CARGO_PKG_VERSION")))?;
        return Ok(0);
    }
    if args.licenses {
        output::write(
            &args,
            include_str!("../../../crates/ccm-scan/LICENSE-language-data"),
        )?;
        return Ok(0);
    }
    if args.languages {
        output::write(&args, &language_report::render(&args)?)?;
        return Ok(0);
    }
    let bundle = measure(&args)?;
    let color = color::enabled(&args.color, args.output.is_some());
    let output = match args.format {
        Format::Terminal => terminal::render(&bundle, &args, color),
        Format::Json => json::render(&bundle, &args),
        Format::Html => html::render(&bundle, &args),
        Format::Csv => delimited::csv(&bundle, &args),
        Format::Ndjson => delimited::ndjson(&bundle, &args),
    }?;
    output::write(&args, &output)?;
    Ok(if !bundle.current.complete {
        2
    } else if args.fail_on_pending
        && bundle.change.as_ref().is_some_and(|c| {
            !c.pending.is_empty() || c.behavior.as_ref().is_some_and(|b| !b.consistent)
        })
    {
        3
    } else {
        0
    })
}

fn measure(args: &Args) -> Result<Bundle, Error> {
    // Validate assumptions before doing any filesystem work.
    estimate_cost(0, None, args.cost)?;
    args.ai.validate()?;
    let current = measure_paths(&args.paths, &args.scan)?;
    let baseline = args
        .compare
        .as_ref()
        .map(|path| measure_paths(std::slice::from_ref(path), &args.scan))
        .transpose()?;
    let changes = baseline
        .as_ref()
        .map(|before| compare_reports(before, &current, &args.scan))
        .transpose()?;
    let model = args
        .model
        .as_ref()
        .map(|path| {
            let mut buffer = Vec::new();
            read_source(path, 4 * 1024 * 1024, &mut buffer)?;
            let source = std::str::from_utf8(&buffer).map_err(|_| {
                Error::new(
                    "model-encoding",
                    "Model is not UTF-8",
                    "Save the model as UTF-8.",
                )
            })?;
            parse_model(source).map_err(|mut error| {
                if let Some(location) = &mut error.location {
                    location.path = Some(path.display().to_string());
                }
                error
            })
        })
        .transpose()?;
    let seeds = seeds(args, model.as_ref(), changes.as_deref())?;
    let change = model.as_ref().map(|m| m.evaluate(&seeds)).transpose()?;
    let interaction = if seeds.len() > 1 {
        model
            .as_ref()
            .map(|m| m.compare_batches(&seeds[..1], &seeds[1..]))
            .transpose()?
    } else {
        None
    };
    let review_count = change
        .as_ref()
        .map(|c| c.affected.len())
        .unwrap_or_else(|| changes.as_ref().map_or(current.files.len(), |c| c.len()));
    let cost = estimate_cost(
        review_count,
        change.as_ref().map(|c| c.pending_units),
        args.cost,
    )?;
    let development = economics::measure(
        &current,
        changes.as_deref(),
        model.as_ref(),
        &seeds,
        change.as_ref().map(|c| c.pending_units),
        args,
    )?;
    Ok(Bundle {
        current,
        development,
        baseline,
        changes,
        model,
        change,
        cost,
        seeds,
        interaction,
    })
}

fn seeds(
    args: &Args,
    model: Option<&Model>,
    changes: Option<&[FileChange]>,
) -> Result<Vec<ArtifactId>, Error> {
    let Some(model) = model else {
        return Ok(Vec::new());
    };
    let names: Vec<_> = if args.changed.is_empty() {
        changes
            .unwrap_or_default()
            .iter()
            .map(|c| c.name.as_str())
            .collect()
    } else {
        args.changed.iter().map(String::as_str).collect()
    };
    let mut seeds = Vec::new();
    for name in names {
        let index = model.artifacts.iter().position(|n| n == name).ok_or_else(|| Error::new(
            "unknown-change", format!("No model artifact is named {name}"),
            "Declare changed paths as artifacts or supply explicit --changed artifact names."))?;
        seeds.push(ArtifactId(index));
    }
    seeds.sort_unstable();
    seeds.dedup();
    Ok(seeds)
}
