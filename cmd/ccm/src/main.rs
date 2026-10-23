//! CCM command-line application.
mod args;
mod budget;
mod color;
mod delimited;
mod diagnostic;
mod economics;
mod estimate;
mod html;
mod inventory;
mod json;
mod language_report;
mod notices;
mod output;
mod report;
mod style;
mod terminal;
mod text;

use args::{Args, Format};
use ccm_core::{ArtifactId, ChangeReport, CostEstimate, Error, Model, estimate_cost, parse_model};
use ccm_project::{FileChange, Report, compare_reports, measure_paths, read_source};

/// CCM's own license, shown before the notices of the bundled language data.
const LICENSE_NOTICE: &str = "\
CCM, the Code Counter Machine
Copyright (C) 2026 Vikrant Rathore, with Ronak Rathore and Kanak Rathore

CCM is free software: you can redistribute it and/or modify it under the terms of the \
GNU Lesser General Public License as published by the Free Software Foundation, either \
version 3 of the License, or (at your option) any later version. It is distributed WITHOUT \
ANY WARRANTY. See COPYING.LESSER and COPYING in the source distribution, or \
<https://www.gnu.org/licenses/>.

CCM bundles language data that is distributed under the following license.
";

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
            eprint!("{}", diagnostic::render(&error));
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
        let notices = format!(
            "{LICENSE_NOTICE}\n{}",
            include_str!("../../../crates/ccm-scan/LICENSE-language-data")
        );
        output::write(&args, &notices)?;
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
    let review_count = change.as_ref().map_or_else(
        || review_files(&current, baseline.as_ref(), changes.as_deref()),
        |c| c.affected.len(),
    );
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

/// Counts the files a human reviews without a task model: program files only, matching
/// the program-lines definition. With a comparison, these are the changed program files,
/// judged by their current version or, for a removed file, by their baseline version.
fn review_files(
    current: &Report,
    baseline: Option<&Report>,
    changes: Option<&[FileChange]>,
) -> usize {
    let Some(changes) = changes else {
        return current.files.iter().filter(|f| f.is_program()).count();
    };
    let (now, before) = (programs(Some(current)), programs(baseline));
    changes
        .iter()
        .filter(|c| {
            let side = if c.after.is_some() { &now } else { &before };
            side.contains(c.name.as_str())
        })
        .count()
}

fn programs(report: Option<&Report>) -> std::collections::BTreeSet<&str> {
    report
        .into_iter()
        .flat_map(|r| &r.files)
        .filter(|f| f.is_program())
        .map(|f| f.name.as_str())
        .collect()
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
