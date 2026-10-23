use crate::{Args, Bundle};
use ccm_core::{AiEstimate, AiOptions, ArtifactId, Error, Model};
use ccm_project::{FileChange, Report};

pub(crate) struct Development {
    pub estimate: AiEstimate,
    pub scope_known: bool,
    pub scope: &'static str,
    pub languages: Vec<(String, u64, f64)>,
}

pub(crate) fn measure(
    current: &Report,
    changes: Option<&[FileChange]>,
    model: Option<&Model>,
    seeds: &[ArtifactId],
    pending: Option<u64>,
    args: &Args,
) -> Result<Development, Error> {
    let mut groups = std::collections::BTreeMap::<String, u64>::new();
    let names: Option<std::collections::BTreeSet<&str>> = if let Some(model) = model {
        Some(
            seeds
                .iter()
                .map(|id| model.artifacts[id.0].as_str())
                .collect(),
        )
    } else {
        changes.map(|changes| changes.iter().map(|c| c.name.as_str()).collect())
    };
    let paths: std::collections::BTreeSet<_> = if model.is_some() {
        current.files.iter().map(|f| f.name.as_str()).collect()
    } else {
        std::collections::BTreeSet::new()
    };
    let scope_known = model.is_none()
        || names
            .as_ref()
            .is_none_or(|names| names.iter().all(|name| paths.contains(name)));
    for file in &current.files {
        if !file.is_program() {
            continue;
        }
        if names
            .as_ref()
            .is_some_and(|names| !names.contains(&file.name.as_str()))
        {
            continue;
        }
        *groups.entry(file.language.name().into()).or_default() += file.counts.code;
    }
    let languages: Vec<_> = groups
        .into_iter()
        .map(|(name, lines)| {
            let factor = args.ai.language_factor(&name);
            (name, lines, factor)
        })
        .collect();
    let lines = languages.iter().map(|(_, lines, _)| lines).sum();
    // Fold from +0.0: an empty f64 `sum()` is -0.0, which reports would print as "-0".
    let weighted = languages.iter().fold(0.0, |total, (_, lines, factor)| {
        total + *lines as f64 * factor
    });
    let estimate = if args.agent_override || args.no_cost {
        AiEstimate {
            lines,
            output_lines: weighted,
            ..AiEstimate::default()
        }
    } else {
        ccm_core::estimate_ai(lines, weighted, pending, &args.ai)?
    };
    Ok(Development {
        estimate,
        scope_known,
        scope: if model.is_some() {
            "matched change-seed source files"
        } else if changes.is_some() {
            "whole changed source files (not a patch effort estimate)"
        } else {
            "rebuild selected non-generated program source"
        },
        languages,
    })
}

pub(crate) fn total(bundle: &Bundle) -> f64 {
    bundle.cost.usd + bundle.development.estimate.usd() * bundle.cost.expected_attempts
}

pub(crate) fn option(
    flag: &str,
    words: &mut impl Iterator<Item = String>,
    args: &mut Args,
) -> Result<bool, Error> {
    match flag {
        "--cost-profile" => {
            args.custom_profile = true;
            let path = crate::args::value(words, flag)?;
            let mut bytes = Vec::new();
            ccm_project::read_source(std::path::Path::new(&path), 65536, &mut bytes)?;
            let source =
                std::str::from_utf8(&bytes).map_err(|_| profile_error("Expected UTF-8"))?;
            load_profile(source, &mut args.ai)?;
        }
        "--ai-input-price" => args.ai.input_price = crate::args::number(words, flag)?,
        "--ai-output-price" => args.ai.output_price = crate::args::number(words, flag)?,
        "--ai-workload" => args.ai.workload = crate::args::number(words, flag)?,
        "--agent-cost" => {
            args.cost.agent_cost = crate::args::number(words, flag)?;
            args.agent_override = true;
        }
        _ => return Ok(false),
    }
    Ok(true)
}

fn load_profile(source: &str, profile: &mut AiOptions) -> Result<(), Error> {
    for (index, line) in source.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let result = line
            .split_once('=')
            .ok_or_else(|| profile_error("Expected key = value"))
            .and_then(|(key, value)| assign(profile, key.trim(), value.trim()));
        result.map_err(|error| error.at_source(index + 1, 1, line))?;
    }
    profile.validate()
}

fn assign(profile: &mut AiOptions, key: &str, value: &str) -> Result<(), Error> {
    if key == "model" {
        profile.model = value.trim_matches('"').into();
        return Ok(());
    }
    if key == "year" {
        profile.year = value.parse().map_err(|_| profile_error("Invalid year"))?;
        return Ok(());
    }
    let number: f64 = value
        .parse()
        .map_err(|_| profile_error("Expected a numeric value"))?;
    match key {
        "input_price" => profile.input_price = number,
        "output_price" => profile.output_price = number,
        "input_per_line" => profile.input_per_line = number,
        "output_per_line" => profile.output_per_line = number,
        "check_tokens" => profile.check_tokens = number,
        "session_price" => profile.session_price = number,
        "lines_per_session" => profile.lines_per_session = number,
        "workload" => profile.workload = number,
        _ => {
            let language = key
                .strip_prefix("language.")
                .filter(|name| ccm_scan::languages().iter().any(|info| info.name == *name))
                .ok_or_else(|| profile_error(format!("Unknown profile key: {key}")))?;
            profile.languages.retain(|(name, _)| name != language);
            profile.languages.push((language.into(), number));
        }
    }
    Ok(())
}

fn profile_error(message: impl Into<String>) -> Error {
    Error::new(
        "cost-profile",
        message,
        "See examples/ai-costs-2026.conf for supported profile keys.",
    )
}
