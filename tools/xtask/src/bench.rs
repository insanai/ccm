use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::Instant;

pub(super) fn run(arguments: &[String]) -> Result<(), String> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .map_err(error)?;
    let ccm = root.join("target/release/ccm");
    let corpus = if let Some(path) = arguments.get(1) {
        Path::new(path).canonicalize().map_err(error)?
    } else {
        root.join("target/bench-fixture-v1")
    };
    if arguments.len() < 2 {
        create_corpus(&corpus)?;
        verify(&ccm, &corpus)?;
    } else {
        verify_project(&ccm, &corpus)?;
    }
    let runs = arguments
        .first()
        .map_or(Ok(30), |s| s.parse::<usize>().map_err(error))?;
    if !(10..=500).contains(&runs) {
        return Err("Use 10 to 500 benchmark pairs".into());
    }
    let path = root.join("target/bench-paired.csv");
    let mut csv = std::fs::File::create(&path).map_err(error)?;
    writeln!(csv, "pair,ccm_seconds,scc_seconds,speedup").map_err(error)?;
    for _ in 0..2 {
        elapsed(&ccm, &corpus, true)?;
        elapsed(Path::new("scc"), &corpus, false)?;
    }
    let mut ratios = Vec::new();
    let mut ccm_times = Vec::new();
    let mut scc_times = Vec::new();
    for pair in 0..runs {
        let (ours, other) = if pair % 2 == 0 {
            let ours = elapsed(&ccm, &corpus, true)?;
            (ours, elapsed(Path::new("scc"), &corpus, false)?)
        } else {
            let other = elapsed(Path::new("scc"), &corpus, false)?;
            (elapsed(&ccm, &corpus, true)?, other)
        };
        writeln!(csv, "{pair},{ours:.9},{other:.9},{:.6}", other / ours).map_err(error)?;
        ccm_times.push(ours);
        scc_times.push(other);
        ratios.push(other / ours);
    }
    println!("Corpus: {}", corpus.display());
    println!("Input totals verified before timing; line classifications can differ by policy.");
    println!("{runs} alternating warm pairs; end-to-end terminal rendering to a null sink.");
    println!(
        "Median CCM: {:.6}s; scc: {:.6}s; paired speedup: {:.3}x.",
        median(&mut ccm_times),
        median(&mut scc_times),
        median(&mut ratios.clone())
    );
    let (low, high) = bootstrap(&ratios);
    println!("Paired median bootstrap 95% interval: [{low:.3}, {high:.3}]x.");
    println!("Raw samples: {}", path.display());
    memory(&root, &ccm, &corpus)?;
    println!("This workload does not establish an all-project speed or memory claim.");
    Ok(())
}

fn create_corpus(path: &Path) -> Result<(), String> {
    std::fs::create_dir_all(path).map_err(error)?;
    for index in 0..2000 {
        write_fixture(
            &path.join(format!("tiny-{index:04}.rs")),
            format!("// module {index}\npub fn value_{index}() -> usize {{\n    {index}\n}}\n\n")
                .as_bytes(),
        )?;
    }
    let mut large = String::from("// large module\nfn large() {\n");
    for _ in 0..50_000 {
        large.push_str("    let value = 1; // explanation\n");
    }
    large.push_str("}\n\n");
    for index in 0..4 {
        write_fixture(&path.join(format!("large-{index}.rs")), large.as_bytes())?;
    }
    Ok(())
}

fn write_fixture(path: &Path, source: &[u8]) -> Result<(), String> {
    match std::fs::read(path) {
        Ok(existing) if existing == source => Ok(()),
        Ok(_) => Err(format!(
            "Refusing to change an unexpected benchmark fixture: {}",
            path.display()
        )),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            std::fs::write(path, source).map_err(error)
        }
        Err(e) => Err(error(e)),
    }
}

fn command(program: &Path, path: &Path, ours: bool) -> Command {
    let mut command = Command::new(program);
    if ours {
        command.args(["--no-cost", "--sort", "name"]);
    } else {
        command.args([
            "--count-ignore",
            "--no-config",
            "--no-cocomo",
            "--no-complexity",
            "--no-size",
            "--sort",
            "name",
        ]);
    }
    command.arg(path).env("NO_COLOR", "1");
    command
}

fn verify(ccm: &Path, corpus: &Path) -> Result<(), String> {
    for (program, ours) in [(ccm, true), (Path::new("scc"), false)] {
        let output = command(program, corpus, ours)
            .args(["--format", "csv"])
            .output()
            .map_err(error)?;
        if !output.status.success() {
            return Err(String::from_utf8_lossy(&output.stderr).into_owned());
        }
        let csv = String::from_utf8(output.stdout).map_err(error)?;
        let mut totals = [0u64; 4];
        let mut files = 0;
        for line in csv.lines().skip(1) {
            let fields: Vec<_> = line.split(',').collect();
            let start = if ours { 5 } else { 2 };
            for (column, total) in totals.iter_mut().enumerate().take(3) {
                *total += fields
                    .get(start + column)
                    .ok_or("Missing CSV column")?
                    .parse::<u64>()
                    .map_err(error)?;
            }
            files += if ours {
                1
            } else {
                fields
                    .get(7)
                    .ok_or("Missing SCC file count")?
                    .parse::<usize>()
                    .map_err(error)?
            };
        }
        if totals[..3] != [206_008, 2004, 2004] || files != 2004 {
            return Err(format!(
                "Counting conformance failed for {}: {files} files, {totals:?}",
                program.display()
            ));
        }
    }
    Ok(())
}

fn elapsed(program: &Path, corpus: &Path, ours: bool) -> Result<f64, String> {
    let start = Instant::now();
    let output = command(program, corpus, ours)
        .stdout(Stdio::null())
        .output()
        .map_err(error)?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).into_owned());
    }
    Ok(start.elapsed().as_secs_f64())
}

fn median(values: &mut [f64]) -> f64 {
    values.sort_by(f64::total_cmp);
    let middle = values.len() / 2;
    if values.len() % 2 == 0 {
        (values[middle - 1] + values[middle]) / 2.0
    } else {
        values[middle]
    }
}

fn bootstrap(ratios: &[f64]) -> (f64, f64) {
    let mut state = 0x43434d_u64;
    let mut sample = Vec::with_capacity(ratios.len());
    let mut estimates = Vec::with_capacity(5000);
    for _ in 0..5000 {
        sample.clear();
        for _ in ratios {
            state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
            sample.push(ratios[(state >> 32) as usize % ratios.len()]);
        }
        estimates.push(median(&mut sample));
    }
    estimates.sort_by(f64::total_cmp);
    (estimates[125], estimates[4874])
}

fn error(error: impl std::fmt::Display) -> String {
    error.to_string()
}

fn memory(root: &Path, ccm: &Path, corpus: &Path) -> Result<(), String> {
    let time = Path::new("/usr/bin/time");
    if !time.exists() {
        return Ok(());
    }
    let mut csv = std::fs::File::create(root.join("target/bench-rss.csv")).map_err(error)?;
    writeln!(csv, "pair,ccm_peak_rss_kib,scc_peak_rss_kib").map_err(error)?;
    let mut ours = Vec::new();
    let mut other = Vec::new();
    for pair in 0..30 {
        let left = rss(time, ccm, corpus, true)?;
        let right = rss(time, Path::new("scc"), corpus, false)?;
        writeln!(csv, "{pair},{left},{right}").map_err(error)?;
        ours.push(left as f64);
        other.push(right as f64);
    }
    println!(
        "Median peak RSS (30 additional runs/tool): CCM {:.0} KiB; scc {:.0} KiB.",
        median(&mut ours),
        median(&mut other)
    );
    Ok(())
}

fn rss(time: &Path, program: &Path, corpus: &Path, ours: bool) -> Result<u64, String> {
    let measured = command(program, corpus, ours);
    let output = Command::new(time)
        .args(["-f", "%M", "--"])
        .arg(measured.get_program())
        .args(measured.get_args())
        .env("NO_COLOR", "1")
        .stdout(Stdio::null())
        .output()
        .map_err(error)?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).into_owned());
    }
    String::from_utf8(output.stderr)
        .map_err(error)?
        .trim()
        .parse()
        .map_err(error)
}

fn verify_project(ccm: &Path, corpus: &Path) -> Result<(), String> {
    let mut values = Vec::new();
    for (program, ours) in [(ccm, true), (Path::new("scc"), false)] {
        let output = command(program, corpus, ours)
            .args(["--format", "csv"])
            .output()
            .map_err(error)?;
        if !output.status.success() {
            return Err(String::from_utf8_lossy(&output.stderr).into());
        }
        let text = String::from_utf8(output.stdout).map_err(error)?;
        let mut total = (0u64, 0u64, 0u64);
        for row in text.lines().skip(1) {
            let fields = csv_fields(row)?;
            let number = |index: usize| -> Result<u64, String> {
                fields
                    .get(index)
                    .ok_or_else(|| "Missing CSV column".to_owned())?
                    .parse()
                    .map_err(error)
            };
            total.0 += if ours { 1 } else { number(7)? };
            total.1 += number(if ours { 4 } else { 1 })?;
            total.2 += number(if ours { 3 } else { 6 })?;
        }
        values.push(total);
    }
    if values[0] != values[1] {
        return Err(format!("Input coverage differs: {values:?}"));
    }
    println!(
        "Equivalent input: {} files, {} physical lines, {} bytes.",
        values[0].0, values[0].1, values[0].2
    );
    Ok(())
}

/// Splits one CSV row, honoring double-quoted fields with doubled quotes inside.
pub(super) fn csv_fields(row: &str) -> Result<Vec<String>, String> {
    let mut result = Vec::new();
    let mut field = String::new();
    let mut quoted = false;
    let mut chars = row.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '"' {
            if quoted && chars.peek() == Some(&'"') {
                field.push('"');
                chars.next();
            } else {
                quoted = !quoted;
            }
        } else if c == ',' && !quoted {
            result.push(std::mem::take(&mut field));
        } else {
            field.push(c);
        }
    }
    if quoted {
        return Err("Unclosed benchmark CSV field".into());
    }
    result.push(field);
    Ok(result)
}
