//! Generates the Homebrew formula from the published archive checksums.
use std::collections::BTreeMap;
use std::path::Path;

const PACKAGES: [&str; 4] = ["linux-amd64", "linux-arm64", "macos-amd64", "macos-arm64"];

pub(super) fn run(arguments: &[String]) -> Result<(), String> {
    if arguments.len() != 2 {
        return Err("Usage: cargo xtask homebrew SHA256SUMS OUTPUT".into());
    }
    let text = std::fs::read_to_string(&arguments[0]).map_err(error)?;
    let sums = parse(&text)?;
    let mut formula =
        include_str!("homebrew/ccm.rb.in").replace("@VERSION@", env!("CARGO_PKG_VERSION"));
    for package in PACKAGES {
        let name = format!("ccm-{package}.tar.gz");
        let digest = sums
            .get(&name)
            .ok_or_else(|| format!("Missing checksum for {name}"))?;
        formula = formula.replace(&format!("@SHA256_{package}@"), digest);
    }
    let output = Path::new(&arguments[1]);
    if let Some(parent) = output.parent().filter(|p| !p.as_os_str().is_empty()) {
        std::fs::create_dir_all(parent).map_err(error)?;
    }
    std::fs::write(output, formula).map_err(error)?;
    println!(
        "Wrote {} from verified release checksum entries.",
        output.display()
    );
    Ok(())
}

fn parse(text: &str) -> Result<BTreeMap<String, String>, String> {
    if text.len() > 65_536 {
        return Err("The release checksum manifest exceeds 64 KiB".into());
    }
    let mut sums = BTreeMap::new();
    for (index, line) in text.lines().enumerate() {
        let fields: Vec<_> = line.split_whitespace().collect();
        if fields.len() != 2
            || fields[0].len() != 64
            || !fields[0].bytes().all(|b| b.is_ascii_hexdigit())
            || !fields[1]
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"-._".contains(&b))
        {
            return Err(format!(
                "Invalid release checksum entry on line {}",
                index + 1
            ));
        }
        if sums
            .insert(fields[1].into(), fields[0].to_ascii_lowercase())
            .is_some()
        {
            return Err(format!("Duplicate release checksum for {}", fields[1]));
        }
    }
    Ok(sums)
}

fn error(value: impl std::fmt::Display) -> String {
    value.to_string()
}

#[cfg(test)]
mod tests {
    use super::parse;

    #[test]
    fn malformed_or_ambiguous_release_manifests_are_rejected() {
        let digest = "a".repeat(64);
        let row = format!("{digest}  ccm-linux-amd64.tar.gz\n");
        assert!(parse(&row).is_ok());
        assert!(parse(&format!("{row}{row}")).is_err());
        assert!(parse(&format!("{digest}  ../archive.tar.gz\n")).is_err());
        assert!(parse("abc archive.tar.gz\n").is_err());
        assert!(parse(&"x".repeat(65_537)).is_err());
    }
}
