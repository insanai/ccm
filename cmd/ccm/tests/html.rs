//! Contracts for the standalone HTML report: one offline file, an embedded data island,
//! safe escaping of untrusted names, the report byte budget, and the change drill-down.
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};

struct Fixture(PathBuf);

impl Fixture {
    fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let path = std::env::temp_dir().join(format!(
            "ccm-html-test-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir_all(&path).unwrap();
        Self(path)
    }

    fn write(&self, path: &str, source: &str) {
        let path = self.0.join(path);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, source).unwrap();
    }

    fn run(&self, arguments: &[&str]) -> Output {
        ccm(arguments, Some(&self.0))
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn ccm(arguments: &[&str], path: Option<&Path>) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_ccm"));
    command.args(arguments).env("NO_COLOR", "1");
    if let Some(path) = path {
        command.arg(path);
    }
    command.output().unwrap()
}

fn text(output: &Output) -> String {
    String::from_utf8(output.stdout.clone()).unwrap()
}

fn examples() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples")
}

/// Returns the text of the JSON data island.
fn island(html: &str) -> &str {
    let start = "<script type='application/json' id=ccm-data>";
    let begin = html.find(start).expect("data island") + start.len();
    let end = html[begin..].find("</script>").unwrap() + begin;
    &html[begin..end]
}

#[test]
fn report_is_one_file_with_embedded_data_and_a_static_summary() {
    let fixture = Fixture::new();
    fixture.write("src/main.rs", "// entry\nfn main() {}\n\n");
    fixture.write("README.md", "Prose.\n");
    let output = fixture.run(&["--html"]);
    assert!(output.status.success());
    let html = text(&output);
    assert!(html.starts_with("<!doctype html>"));
    assert_eq!(html.matches("id=ccm-data").count(), 1);
    let data = island(&html);
    assert!(data.starts_with("{\"schema\":\"ccm.html-data.v1\""));
    assert!(data.contains("\"dirs\":[\"\",\"src\"]"), "{data}");
    assert!(data.contains("[1,\"main.rs\",1,0,1,1,1,"), "{data}");
    // The static layer works without scripts.
    assert!(html.contains("<span>Program lines</span><strong>1</strong>"));
    assert!(html.contains("<section id=languages-static>"));
    assert!(html.contains("src/main.rs"));
    // The logo is compiled in, and the report is deterministic.
    assert!(html.contains("<title id=\"ccm-logo-title\">CCM mark</title>"));
    assert_eq!(html, text(&fixture.run(&["--html"])));
}

#[test]
fn report_references_no_external_or_companion_files() {
    let fixture = Fixture::new();
    fixture.write("lib.rs", "fn f() {}\n");
    let html = text(&fixture.run(&["--html"]));
    let lower = html.to_ascii_lowercase();
    for forbidden in [
        " src=",
        " src =",
        "href=",
        "<link",
        "@import",
        "url(http",
        "url(//",
        "<iframe",
        "<img",
        ".js\"",
        ".css\"",
        ".js'",
        ".css'",
        "fetch(",
        "xmlhttprequest",
        "import(",
    ] {
        assert!(
            !lower.contains(forbidden),
            "external reference: {forbidden}"
        );
    }
    assert!(html.contains("default-src 'none'"));
}

#[test]
fn untrusted_names_cannot_close_the_script_or_inject_markup() {
    let fixture = Fixture::new();
    fixture.write("a</script><b>.rs", "fn f() {}\n");
    fixture.write("c<!--d.rs", "fn g() {}\n");
    fixture.write("e&\"'f.rs", "fn h() {}\n");
    let output = fixture.run(&["--html"]);
    assert!(output.status.success());
    let html = text(&output);
    // Three script elements: the theme, the data island, and the page script.
    assert_eq!(html.matches("</script>").count(), 3);
    assert_eq!(html.matches("<script").count(), 3);
    assert!(!html.contains("<b>"));
    assert!(!html.contains("<!--"));
    assert!(html.contains("a&lt;/script&gt;&lt;b&gt;.rs"));
    assert!(html.contains("e&amp;&quot;&#39;f.rs"));
    let data = island(&html);
    // The name `a</script><b>.rs` is a file `script><b>.rs` in a directory `a<`.
    assert!(data.contains("\"dirs\":[\"a\\u003c\","), "{data}");
    assert!(
        data.contains("\"script\\u003e\\u003cb\\u003e.rs\""),
        "{data}"
    );
    assert!(data.contains("\"c\\u003c!--d.rs\""), "{data}");
    assert!(data.contains("\"e\\u0026\\\"'f.rs\""), "{data}");
}

#[test]
fn report_respects_the_byte_budget() {
    let fixture = Fixture::new();
    for index in 0..50 {
        fixture.write(&format!("dir{}/file{index}.rs", index % 7), "fn f() {}\n");
    }
    let html = text(&fixture.run(&["--html"]));
    let size = html.len().to_string();
    let exact = fixture.run(&["--html", "--max-report-bytes", &size]);
    assert!(exact.status.success());
    // The budget is part of the embedded options, so its digits change the size slightly.
    let smaller = (html.len() - 100).to_string();
    let output = fixture.run(&["--html", "--max-report-bytes", &smaller]);
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    let output = fixture.run(&["--html", "--max-report-bytes", "4096"]);
    assert_eq!(output.status.code(), Some(1));
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("exceeds its byte or allocation budget")
    );
    // A file row costs tens of bytes, not a rendered table row.
    let per_file = island(&html).len() / 50;
    assert!(per_file < 120, "{per_file} bytes per file");
}

#[test]
fn model_and_comparison_add_drill_down_data() {
    let book = examples().join("book");
    let after = book.join("after");
    let output = ccm(
        &[
            "--html",
            "--compare",
            book.join("before").to_str().unwrap(),
            "--model",
            book.join("checkout.ccm").to_str().unwrap(),
        ],
        Some(&after),
    );
    assert!(output.status.success());
    let html = text(&output);
    let data = island(&html);
    for contract in [
        "\"artifacts\":[\"pricing.rs\",\"inventory.rs\",\"checkout.rs\",\"payment.rs\"]",
        "\"edges\":[[0,2],[1,2]]",
        "\"seeds\":[0,1]",
        "\"affected\":[[0,null],[1,null],[2,0]]",
        "\"required_units\":7,\"pending_units\":6",
        "[\"Payment review\",\"review\",true,\"surviving\",\"passed\",[3],[4]]",
        "\"behavior\":{\"fidelity\":\"exact-declared-universe\",\"consistent\":true",
        "\"interaction\":{\"split\":\"first-seed-vs-remaining\",\"shared\":[2],\"joint\":[3]",
        "\"comparison\":{\"kind\":\"exact-file-bytes\"",
        "[\"inventory.rs\",\"modified\",{\"bytes\":37,",
        "\"expected_cost\":675.8582357142857",
        "\"implementation_sensitivity_usd\":[",
    ] {
        assert!(data.contains(contract), "missing {contract}");
    }
    for view in [
        "renderChange",
        "renderComparison",
        "closureGraph",
        "renderEstimate",
    ] {
        assert!(html.contains(view), "missing view {view}");
    }
    let plain = text(&ccm(&["--html", "--no-cost"], Some(&after)));
    let data = island(&plain);
    assert!(data.contains("\"model\":null"));
    assert!(data.contains("\"comparison\":null"));
    assert!(data.contains("\"budget\":null"));
}

#[test]
fn report_written_to_a_file_matches_standard_output() {
    let fixture = Fixture::new();
    fixture.write("x.rs", "fn x() {}\n");
    let target = fixture.0.join("out").join("report.html");
    std::fs::create_dir_all(target.parent().unwrap()).unwrap();
    let path = target.to_str().unwrap();
    let output = ccm(&["--html", "-o", path], Some(&fixture.0.join("x.rs")));
    assert!(output.status.success());
    let written = std::fs::read_to_string(&target).unwrap();
    let printed = text(&ccm(&["--html"], Some(&fixture.0.join("x.rs"))));
    assert_eq!(written, printed);
    let entries = std::fs::read_dir(target.parent().unwrap()).unwrap().count();
    assert_eq!(entries, 1, "no companion files");
}
