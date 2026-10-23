use std::path::Path;
use std::process::{Command, Output};
mod terminal;

pub(super) fn run(arguments: &[String], assets_only: bool) -> Result<(), String> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .map_err(error)?;
    let tools = preflight(arguments, assets_only)?;
    execute(
        Command::new("cargo")
            .current_dir(&root)
            .args(["run", "-p", "ccm-art"]),
    )?;
    execute(Command::new(&tools.typst).current_dir(&root).args([
        "compile",
        "--root",
        ".",
        "docs/book/_templates/render-cover.typ",
        "assets/cover-art.svg",
    ]))?;
    execute(
        Command::new(&tools.typst)
            .current_dir(&root)
            .args(["compile", "--root", ".", "--pages", "1", "--input"])
            .arg(format!("version={}", env!("CARGO_PKG_VERSION")))
            .args([
                "docs/book/_templates/cover-preview.typ",
                "docs/shared/_static/book-cover.png",
            ]),
    )?;
    stage(&root)?;
    if assets_only {
        println!("SVG artwork and PNG cover preview are ready.");
        return Ok(());
    }
    execute(
        Command::new("cargo")
            .current_dir(&root)
            .args(["build", "-p", "ccm"]),
    )?;
    examples(&root, tools.browser.as_deref())?;
    let mut build = Command::new(
        tools
            .guidedog
            .as_deref()
            .ok_or("Guidedog preflight was not completed")?,
    );
    build
        .current_dir(&root)
        .args(["ssg", "build", "docs", "--out", "docs/_build/site"]);
    if arguments.iter().any(|arg| arg == "--no-pdf") {
        build.arg("--no-pdf");
    }
    execute(&mut build)?;
    println!("Website: docs/_build/site/index.html");
    if !arguments.iter().any(|arg| arg == "--no-pdf") {
        println!("Book: docs/_build/site/books/ccm-book-en.pdf");
    }
    Ok(())
}

struct Tools {
    typst: std::path::PathBuf,
    guidedog: Option<std::path::PathBuf>,
    /// The browser that captures screenshots; `None` keeps the committed images.
    browser: Option<std::path::PathBuf>,
}

fn preflight(arguments: &[String], assets_only: bool) -> Result<Tools, String> {
    let typst = crate::toolchain::require(
        "typst",
        "The Typst CLI renders SVG artwork and PNG previews.
Guidedog can still build the book from prepared assets.",
        concat!(
            "Install Typst from https://typst.app/open-source/ and put it on PATH. ",
            "Run typst --version, then cargo xtask assets."
        ),
    )?;
    let selected = arguments
        .windows(2)
        .find(|w| w[0] == "--guidedog")
        .map(|w| w[1].as_str())
        .unwrap_or("guidedog");
    let guidedog = if assets_only {
        None
    } else {
        Some(crate::toolchain::require(
            selected,
            "Guidedog drives book and site publication.",
            "Install Guidedog, put it on PATH, and run guidedog --version.",
        )?)
    };
    let browser = if assets_only || arguments.iter().any(|arg| arg == "--no-screenshots") {
        None
    } else {
        let browser = crate::toolchain::browser()?;
        if browser.is_none() {
            println!(concat!(
                "No Chrome, Chromium, or Edge found; keeping the committed screenshots. ",
                "Set CCM_BROWSER to a browser executable to capture new ones."
            ));
        }
        browser
    };
    Ok(Tools {
        typst,
        guidedog,
        browser,
    })
}

fn stage(root: &Path) -> Result<(), String> {
    let destination = root.join("docs/shared/_static");
    std::fs::create_dir_all(&destination).map_err(error)?;
    for entry in std::fs::read_dir(root.join("assets")).map_err(error)? {
        let entry = entry.map_err(error)?;
        if entry.path().extension().is_some_and(|s| s == "svg") {
            std::fs::copy(entry.path(), destination.join(entry.file_name())).map_err(error)?;
        }
    }
    let references = destination.join("references");
    std::fs::create_dir_all(&references).map_err(error)?;
    for source in [
        "docs/economics.rst",
        "docs/language-catalog.rst",
        "docs/performance.rst",
        "examples/ai-costs-2026.conf",
    ] {
        let source = root.join(source);
        let name = source.file_name().ok_or("Reference has no filename")?;
        std::fs::copy(&source, references.join(name)).map_err(error)?;
    }
    Ok(())
}

/// Writes the worked example's terminal transcripts, HTML report, and JSON record, and
/// captures their screenshots when a browser is available. Without one, the committed
/// images stay; they change only when the report output does.
fn examples(root: &Path, browser: Option<&Path>) -> Result<(), String> {
    let executable = root.join("target/debug/ccm");
    let output_dir = root.join("docs/shared/_static");
    let mut captures = Vec::new();
    for (name, changed) in [("cli-overview", false), ("cli-change", true)] {
        let mut command = Command::new(&executable);
        // A fixed width keeps the report inside the rendered terminal window.
        command
            .current_dir(root)
            .env("COLUMNS", "72")
            .args(["--color=always", "examples/book/after"]);
        if changed {
            change_options(&mut command);
        }
        let output = capture(&mut command)?;
        let source = String::from_utf8(output.stdout).map_err(error)?;
        let html = output_dir.join(format!("{name}.html"));
        std::fs::write(&html, terminal::render(&source)).map_err(error)?;
        let image = output_dir.join(format!("{name}.png"));
        let height = 130 + 23 * source.lines().count();
        captures.push((html, image, 820, height));
    }
    let html = output_dir.join("report.html");
    let mut report = Command::new(&executable);
    report
        .current_dir(root)
        .args(["examples/book/after", "--html", "--force", "--output"])
        .arg(&html);
    change_options(&mut report);
    execute(&mut report)?;
    captures.push((html, output_dir.join("report.png"), 1000, 1350));
    let mut json = Command::new(&executable);
    json.current_dir(root)
        .args(["examples/book/after", "--json", "--force", "--output"])
        .arg(output_dir.join("worked-example.json"));
    change_options(&mut json);
    execute(&mut json)?;
    if let Some(browser) = browser {
        for (html, image, width, height) in &captures {
            screenshot(browser, html, image, *width, *height)?;
        }
    }
    Ok(())
}

fn change_options(command: &mut Command) {
    command.args([
        "--compare",
        "examples/book/before",
        "--model",
        "examples/book/checkout.ccm",
    ]);
}

fn screenshot(
    browser: &Path,
    html: &Path,
    image: &Path,
    width: usize,
    height: usize,
) -> Result<(), String> {
    execute(
        Command::new(browser)
            .args([
                "--headless",
                "--no-sandbox",
                "--disable-gpu",
                "--hide-scrollbars",
            ])
            .arg(format!("--window-size={width},{height}"))
            .arg(format!("--screenshot={}", image.display()))
            .arg(format!("file://{}", html.display())),
    )
}

fn execute(command: &mut Command) -> Result<(), String> {
    let output = capture(command)?;
    if !output.stdout.is_empty() {
        print!("{}", String::from_utf8_lossy(&output.stdout));
    }
    Ok(())
}

fn capture(command: &mut Command) -> Result<Output, String> {
    let output = command
        .output()
        .map_err(|e| format!("Cannot run {command:?}: {e}"))?;
    if !output.status.success() {
        return Err(format!(
            "{command:?}:\n{}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    Ok(output)
}

fn error(value: impl std::fmt::Display) -> String {
    value.to_string()
}
