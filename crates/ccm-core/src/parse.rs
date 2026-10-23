use crate::{ArtifactId, Certificate, Error, Model, Obligation};
use std::collections::BTreeMap;

const MAX_RECORDS: usize = 100_000;

/// Parses version 1 tab-separated finite models, with a 4 MiB byte limit.
/// References must name earlier declarations. Returns owned data or a located diagnostic.
pub fn parse_model(source: &str) -> Result<Model, Error> {
    if source.len() > 4 * 1024 * 1024 {
        return Err(problem(0, "Model exceeds 4 MiB"));
    }
    let mut model = Model::default();
    let mut artifacts = BTreeMap::new();
    let mut checks = BTreeMap::new();
    let mut evidence = BTreeMap::new();
    let mut cases = BTreeMap::new();
    let mut header = false;
    let mut version = 1;
    let mut records = 0usize;
    for (index, line) in source.lines().enumerate() {
        let line_number = index + 1;
        if line.trim().is_empty() || line.starts_with('#') {
            continue;
        }
        if !header {
            if !matches!(line, "ccm-model\t1" | "ccm-model\t2") {
                return Err(problem(
                    line_number,
                    "Expected ccm-model<TAB>1 or ccm-model<TAB>2",
                ));
            }
            version = if line.ends_with('2') { 2 } else { 1 };
            model.input_version = Some(version);
            header = true;
            continue;
        }
        // The cap counts record lines; comments, blank lines, and the header are free.
        records += 1;
        if records > MAX_RECORDS {
            return Err(problem(line_number, "Model exceeds 100,000 records"));
        }
        let fields: Vec<_> = line.split('\t').collect();
        if parse_behavior(&fields, &mut model, &evidence, &mut cases)
            .map_err(|message| problem(line_number, message).at_source(line_number, 1, line))?
        {
            continue;
        }
        parse_record(
            &fields,
            &mut model,
            &mut artifacts,
            &mut checks,
            &mut evidence,
            version,
        )
        .map_err(|message| problem(line_number, message).at_source(line_number, 1, line))?;
    }
    if !header || model.context.is_empty() {
        return Err(problem(
            1,
            "A version header and a context identity are required",
        ));
    }
    Ok(model)
}

fn parse_record(
    fields: &[&str],
    model: &mut Model,
    artifacts: &mut BTreeMap<String, usize>,
    checks: &mut BTreeMap<String, usize>,
    evidence: &mut BTreeMap<String, usize>,
    version: u32,
) -> Result<(), String> {
    match fields {
        ["context", context] if model.context.is_empty() && !context.is_empty() => {
            model.context = (*context).into();
        }
        ["artifact", name] => {
            declare(artifacts, name, model.artifacts.len())?;
            model.artifacts.push((*name).into());
        }
        ["edge", from, to] => {
            model.edges.push((
                ArtifactId(resolve(artifacts, from)?),
                ArtifactId(resolve(artifacts, to)?),
            ));
        }
        ["check", name, weight, triggers] => {
            declare(checks, name, model.obligations.len())?;
            let triggers = if *triggers == "!" {
                Vec::new()
            } else {
                triggers
                    .split(';')
                    .map(|set| artifact_set(artifacts, set))
                    .collect::<Result<_, _>>()?
            };
            model.obligations.push(Obligation {
                name: (*name).into(),
                weight: weight.parse().map_err(|_| "Invalid check weight")?,
                triggers,
            });
        }
        ["evidence", name, class, status, support, discharges] => {
            if !matches!(*class, "test" | "review" | "proof" | "assertion")
                || !matches!(*status, "accepted" | "failed" | "unverified")
            {
                return Err("Unknown evidence class or acceptance status".into());
            }
            if *class == "assertion"
                && (*status == "accepted" || (version >= 2 && *status == "failed"))
            {
                return Err("An assertion cannot be accepted as checked evidence".into());
            }
            declare(evidence, name, model.certificates.len())?;
            model.certificates.push(Certificate {
                name: (*name).into(),
                class: (*class).into(),
                accepted: *status == "accepted" || (version >= 2 && *status == "failed"),
                conclusion: match *status {
                    "accepted" => crate::Conclusion::Passed,
                    "failed" => crate::Conclusion::Failed,
                    _ => crate::Conclusion::Unverified,
                },
                support: artifact_set(artifacts, support)?,
                discharges: names(discharges)
                    .map(|q| resolve(checks, q))
                    .collect::<Result<_, _>>()?,
            });
        }
        _ => return Err("Unknown record, wrong field count, or duplicate context".into()),
    }
    Ok(())
}

fn parse_behavior(
    fields: &[&str],
    model: &mut Model,
    evidence: &BTreeMap<String, usize>,
    cases: &mut BTreeMap<String, usize>,
) -> Result<bool, String> {
    match fields {
        ["case", name, observation] => {
            let behavior = model.behavior.get_or_insert_with(Default::default);
            declare(cases, name, behavior.cases.len())?;
            behavior.cases.push(crate::BehaviorCase {
                name: (*name).into(),
                observation: (*observation).into(),
            });
        }
        ["allows", certificate, compatible] => {
            let behavior = model.behavior.get_or_insert_with(Default::default);
            behavior.filters.push(crate::CaseFilter {
                certificate: resolve(evidence, certificate)?,
                compatible: names(compatible)
                    .map(|name| resolve(cases, name))
                    .collect::<Result<_, _>>()?,
            });
        }
        _ => return Ok(false),
    }
    Ok(true)
}

fn names(text: &str) -> impl Iterator<Item = &str> {
    text.split(',').filter(move |_| text != "-")
}

fn artifact_set(map: &BTreeMap<String, usize>, text: &str) -> Result<Vec<ArtifactId>, String> {
    names(text)
        .map(|name| resolve(map, name).map(ArtifactId))
        .collect()
}

fn declare(map: &mut BTreeMap<String, usize>, name: &str, index: usize) -> Result<(), String> {
    if name.len() > 256
        || name.is_empty()
        || name.chars().any(|c| c.is_control() || ",;!".contains(c))
        || name == "-"
    {
        return Err("Names must be nonempty and cannot contain controls or , ; !".into());
    }
    if map.insert(name.into(), index).is_some() {
        return Err(format!("Duplicate declaration: {name}"));
    }
    Ok(())
}

fn resolve(map: &BTreeMap<String, usize>, name: &str) -> Result<usize, String> {
    map.get(name)
        .copied()
        .ok_or_else(|| format!("Unknown reference: {name}"))
}

fn problem(line: usize, message: impl AsRef<str>) -> Error {
    Error::new(
        "model-input",
        format!("line {line}: {}", message.as_ref()),
        "Use the versioned tab-separated format described in docs/model.rst.",
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_unbound_references_and_unverified_assertions() {
        assert!(parse_model("ccm-model\t1\ncontext\tx\nedge\ta\tb").is_err());
        assert!(
            parse_model(concat!(
                "ccm-model\t1\ncontext\tx\n",
                "evidence\tguess\tassertion\taccepted\t-\t-"
            ))
            .is_err()
        );
        assert!(parse_model("ccm-model\t2").is_err());
    }

    #[test]
    fn record_cap_ignores_comments_and_blank_lines() {
        let mut source = String::from("ccm-model\t1\n");
        source.push_str(&"# comment\n\n".repeat(MAX_RECORDS));
        source.push_str("context\tx\n");
        assert!(parse_model(&source).is_ok());
        let mut source = String::from("ccm-model\t1\ncontext\tx\n");
        for index in 0..MAX_RECORDS {
            source.push_str(&format!("artifact\ta{index}\n"));
        }
        let error = parse_model(&source).unwrap_err();
        assert!(
            error.message.contains("100,000 records"),
            "{}",
            error.message
        );
    }
}
