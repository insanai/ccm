//! Native release archives and a complete, verified download manifest.
use std::path::{Path, PathBuf};
use std::process::Command;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const RELEASE_FILES: [&str; 7] = [
    "ccm-linux-amd64.tar.gz",
    "ccm-linux-arm64.tar.gz",
    "ccm-macos-amd64.tar.gz",
    "ccm-macos-arm64.tar.gz",
    "ccm-windows-amd64.zip",
    "ccm-freebsd-amd64.tar.gz",
    "ccm-book-en.pdf",
];

pub(super) fn run(arguments: &[String]) -> Result<(), String> {
    let target = arguments
        .first()
        .ok_or("Usage: cargo xtask dist TARGET [DIRECTORY]")?;
    if arguments.len() > 2 {
        return Err("Usage: cargo xtask dist TARGET [DIRECTORY]".into());
    }
    let package = package(target)?;
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .map_err(error)?;
    if let Ok(tag) = std::env::var("CCM_RELEASE_TAG") {
        if tag != format!("v{VERSION}") {
            return Err(format!(
                "Release tag {tag} does not match Cargo version {VERSION}"
            ));
        }
    }
    let output = output_dir(&root, arguments.get(1))?;
    let windows = target.contains("windows");
    let executable = if windows { "ccm.exe" } else { "ccm" };
    let binary = root
        .join("target")
        .join(target)
        .join("release")
        .join(executable);
    smoke(&root, &binary)?;
    let stage = output.join("staging").join(package);
    std::fs::create_dir_all(&stage).map_err(error)?;
    std::fs::copy(&binary, stage.join(executable)).map_err(error)?;
    for file in ["README.rst", "COPYING", "COPYING.LESSER", "NOTICE"] {
        std::fs::copy(root.join(file), stage.join(file)).map_err(error)?;
    }
    std::fs::copy(
        root.join("crates/ccm-scan/LICENSE-language-data"),
        stage.join("LICENSE-language-data"),
    )
    .map_err(error)?;
    provenance(&root, &stage, target)?;
    let extension = if windows { "zip" } else { "tar.gz" };
    let name = format!("{package}.{extension}");
    let archive = output.join(&name);
    let status = Command::new("tar")
        .arg(if windows { "-a" } else { "-z" })
        .arg("-cf")
        .arg(&archive)
        .arg("-C")
        .arg(output.join("staging"))
        .arg(package)
        .status()
        .map_err(error)?;
    if !status.success() {
        return Err("Release archiving failed; tar is required (bsdtar on Windows).".into());
    }
    let digest = crate::sha256::hex_digest(&std::fs::read(&archive).map_err(error)?);
    std::fs::write(
        output.join(format!("{name}.sha256")),
        format!("{digest}  {name}\n"),
    )
    .map_err(error)?;
    println!("Verified and packaged {}", archive.display());
    Ok(())
}

fn package(target: &str) -> Result<&'static str, String> {
    match target {
        "x86_64-unknown-linux-musl" => Ok("ccm-linux-amd64"),
        "aarch64-unknown-linux-musl" => Ok("ccm-linux-arm64"),
        "x86_64-apple-darwin" => Ok("ccm-macos-amd64"),
        "aarch64-apple-darwin" => Ok("ccm-macos-arm64"),
        "x86_64-pc-windows-msvc" => Ok("ccm-windows-amd64"),
        "x86_64-unknown-freebsd" => Ok("ccm-freebsd-amd64"),
        _ => Err(format!("Unsupported release target: {target}")),
    }
}

fn output_dir(root: &Path, selected: Option<&String>) -> Result<PathBuf, String> {
    let path = selected.map_or_else(|| root.join("dist"), PathBuf::from);
    std::fs::create_dir_all(&path).map_err(error)?;
    path.canonicalize().map_err(error)
}

fn smoke(root: &Path, binary: &Path) -> Result<(), String> {
    let version = capture(root, binary, &["--version"])?;
    if version.trim() != format!("ccm {VERSION}") {
        return Err(format!(
            "The release executable has the wrong version: {version}"
        ));
    }
    let report = capture(
        root,
        binary,
        &[
            "examples/book/after",
            "--compare",
            "examples/book/before",
            "--model",
            "examples/book/checkout.ccm",
            "--json",
        ],
    )?;
    for contract in [
        "\"complete\":true",
        "\"required_units\":7,\"pending_units\":6",
        "\"expected_cost\":675.8582357142857",
    ] {
        if !report.contains(contract) {
            return Err(format!("Release smoke test failed: {contract}"));
        }
    }
    let html = capture(
        root,
        binary,
        &["examples/book/after", "--html", "--no-cost"],
    )?;
    if !html.contains("<svg") || !html.contains("</html>") {
        return Err(
            "The release executable did not produce a complete branded HTML report".into(),
        );
    }
    Ok(())
}

fn provenance(root: &Path, stage: &Path, target: &str) -> Result<(), String> {
    let revision = capture(root, Path::new("git"), &["rev-parse", "HEAD"])?;
    let rust = capture(root, Path::new("rustc"), &["--version"])?;
    let source = format!("https://github.com/insanai/ccm/tree/{}", revision.trim());
    std::fs::write(
        stage.join("BUILD-INFO.txt"),
        format!(
            "CCM {VERSION}\nTarget: {target}\nCompiler: {}\nSource: {source}\n\
            License: LGPL-3.0-or-later\n\n\
            Build from the corresponding source with:\n\
              cargo build --release --locked -p ccm --target {target}\n\n\
            Linux uses musl; Windows uses the static MSVC runtime.\n\
            macOS deployment target: 12.0. FreeBSD build host: 15.1.\n\
            The executable was run against the published checkout example.\n",
            rust.trim()
        ),
    )
    .map_err(error)
}

fn capture(root: &Path, program: &Path, arguments: &[&str]) -> Result<String, String> {
    let output = Command::new(program)
        .current_dir(root)
        .args(arguments)
        .output()
        .map_err(|e| format!("Cannot run {}: {e}", program.display()))?;
    if !output.status.success() {
        return Err(format!(
            "{} failed: {}",
            program.display(),
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    String::from_utf8(output.stdout).map_err(error)
}

pub(super) fn checksums(arguments: &[String]) -> Result<(), String> {
    if arguments.len() != 1 {
        return Err("Usage: cargo xtask checksums DIRECTORY".into());
    }
    let root = Path::new(&arguments[0]);
    let mut manifest = String::new();
    for name in RELEASE_FILES {
        let bytes = std::fs::read(root.join(name))
            .map_err(|e| format!("Incomplete release: {name}: {e}"))?;
        if bytes.is_empty() {
            return Err(format!("Empty release artifact: {name}"));
        }
        let line = format!("{}  {name}\n", crate::sha256::hex_digest(&bytes));
        if !name.ends_with(".pdf") {
            let expected =
                std::fs::read_to_string(root.join(format!("{name}.sha256"))).map_err(error)?;
            if expected != line {
                return Err(format!("Release artifact checksum mismatch: {name}"));
            }
        }
        manifest.push_str(&line);
    }
    std::fs::write(root.join("SHA256SUMS"), manifest).map_err(error)?;
    println!("Verified all six platform archives and the PDF book.");
    Ok(())
}

fn error(value: impl std::fmt::Display) -> String {
    value.to_string()
}
