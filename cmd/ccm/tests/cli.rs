//! End-to-end contracts for selection, reports, finite models, and cost scenarios.
use std::path::PathBuf;
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};

struct Fixture(PathBuf);

impl Fixture {
    fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let path = std::env::temp_dir().join(format!(
            "ccm-test-{}-{}",
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
        Command::new(env!("CARGO_BIN_EXE_ccm"))
            .args(arguments)
            .arg(&self.0)
            .env("NO_COLOR", "1")
            .output()
            .unwrap()
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[test]
fn deterministic_json_partition_limits_and_partial_exit_status() {
    let fixture = Fixture::new();
    fixture.write("lib.rs", "// note\nfn main() {}\n\n");
    fixture.write("target/generated.rs", "this must not count\n");
    fixture.write("ignore.xyz", "unsupported\n");
    let first = fixture.run(&["--json"]);
    assert!(first.status.success());
    assert_eq!(first.stdout, fixture.run(&["--json"]).stdout);
    let json = String::from_utf8(first.stdout).unwrap();
    assert!(json.contains("\"lines\":3,\"code\":1,\"comments\":1,\"blank\":1"));
    assert!(json.contains("\"unsupported_files\":1"));
    assert!(!json.contains("\u{1b}"));
    assert_eq!(
        fixture.run(&["--max-file-bytes", "1"]).status.code(),
        Some(2)
    );
    fixture.write("broken.rs", "/* unfinished");
    let partial = fixture.run(&["--json"]);
    assert_eq!(partial.status.code(), Some(2));
    assert!(
        String::from_utf8(partial.stdout)
            .unwrap()
            .contains("\"complete\":false")
    );
}

#[test]
fn byte_comparison_tracks_added_removed_and_comment_only_changes() {
    let old = Fixture::new();
    let new = Fixture::new();
    old.write("same.rs", "fn unchanged() {}\n");
    new.write("same.rs", "fn unchanged() {}\n");
    old.write("edit.rs", "// before\nfn f() {}\n");
    new.write("edit.rs", "// after!\nfn f() {}\n");
    old.write("removed.py", "pass\n");
    new.write("added.py", "pass\n");
    let output = new.run(&["--json", "--compare", old.0.to_str().unwrap()]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let json = String::from_utf8(output.stdout).unwrap();
    assert!(json.contains("\"kind\":\"modified\""));
    assert!(json.contains("\"kind\":\"added\""));
    assert!(json.contains("\"kind\":\"removed\""));
    assert!(json.contains("\"review_artifacts\":3"));
}

#[test]
fn finite_model_transport_interactions_and_economic_cost_are_connected() {
    let fixture = Fixture::new();
    fixture.write("lib.rs", "fn f() {}\n");
    let model = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/checkout.ccm");
    let output = fixture.run(&[
        "--json",
        "--model",
        model.to_str().unwrap(),
        "--changed",
        "pricing",
        "--changed",
        "inventory",
        "--agent-cost",
        "0",
    ]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let json = String::from_utf8(output.stdout).unwrap();
    assert!(json.contains("\"pending_units\":6"));
    assert!(json.contains("\"required_units\":7"));
    assert!(json.contains("\"status\":\"invalidated\""));
    assert!(json.contains("\"status\":\"surviving\""));
    assert!(json.contains("\"shared_units\":1"));
    assert!(json.contains("\"joint_units\":3"));
    assert!(json.contains("\"net_units\":2"));
    assert!(json.contains("\"expected_cost\":675"));
}

#[test]
fn published_checkout_example_reproduces_the_book_calculation() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let output = Command::new(env!("CARGO_BIN_EXE_ccm"))
        .current_dir(root)
        .args([
            "examples/book/after",
            "--compare",
            "examples/book/before",
            "--model",
            "examples/book/checkout.ccm",
            "--json",
        ])
        .output()
        .unwrap();
    assert!(output.status.success(), "{:?}", output.stderr);
    let json = String::from_utf8(output.stdout).unwrap();
    for contract in [
        "\"complete\":true",
        "\"bytes\":198,\"lines\":12,\"code\":12",
        "\"seeds\":[\"pricing.rs\",\"inventory.rs\"]",
        "\"required_units\":7,\"pending_units\":6",
        "\"review_artifacts\":3",
        "\"expected_cost\":675.8582357142857",
    ] {
        assert!(
            json.contains(contract),
            "Published example changed: {contract}"
        );
    }
}

#[test]
fn reports_escape_untrusted_names_and_output_refuses_overwrite() {
    let fixture = Fixture::new();
    fixture.write("ampersand&name.rs", "fn f() {}\n");
    #[cfg(unix)]
    fixture.write("<script>alert(1)</script>.rs", "fn f() {}\n");
    let html = fixture.run(&["--html"]);
    assert!(html.status.success());
    let html = String::from_utf8(html.stdout).unwrap();
    assert!(html.contains("ampersand&amp;name.rs"));
    #[cfg(unix)]
    assert!(html.contains("&lt;script&gt;"));
    assert!(!html.contains("<script>alert"));
    let report = fixture.0.join("report.html");
    let path = report.to_str().unwrap();
    assert!(fixture.run(&["--html", "-o", path]).status.success());
    assert_eq!(fixture.run(&["--html", "-o", path]).status.code(), Some(1));
    assert_eq!(
        fixture.run(&["--hourly-cost", "NaN"]).status.code(),
        Some(1)
    );
    assert_eq!(
        fixture.run(&["--rework-probability", "1"]).status.code(),
        Some(1)
    );
}

#[cfg(unix)]
#[test]
fn symlinks_are_skipped_and_terminal_controls_are_escaped() {
    let fixture = Fixture::new();
    fixture.write("bad\u{1b}[31m.rs", "fn f() {}\n");
    std::os::unix::fs::symlink(&fixture.0, fixture.0.join("loop")).unwrap();
    let output = fixture.run(&["--technical"]);
    assert!(output.status.success());
    let terminal = String::from_utf8(output.stdout).unwrap();
    assert!(!terminal.contains('\u{1b}'));
    assert!(terminal.contains("\\u{1b}[31m.rs"));
    assert!(terminal.contains("Skipped: 1 symlink."), "{terminal}");
}

#[test]
fn nested_ignore_rules_negation_anchoring_and_worker_counts_are_reproducible() {
    let fixture = Fixture::new();
    fixture.write(".gitignore", "*.tmp\n/root.rs\nignored/\n!keep.tmp\n");
    fixture.write("root.rs", "invalid ignored source\n");
    fixture.write("nested/root.rs", "fn keep() {}\n");
    fixture.write("nested/.gitignore", "local.rs\n");
    fixture.write("nested/local.rs", "fn ignored() {}\n");
    fixture.write("nested/note.tmp", "ignored\n");
    fixture.write("ignored/child.rs", "fn ignored() {}\n");
    fixture.write("keep.tmp", "fn keep() {}\n");
    let first = fixture.run(&["--json", "--jobs", "1", "--count-as", "tmp:Rust"]);
    let parallel = fixture.run(&["--json", "--jobs=4", "--count-as=tmp:Rust"]);
    assert!(first.status.success());
    assert_eq!(first.stdout, parallel.stdout);
    let json = String::from_utf8(first.stdout).unwrap();
    assert!(json.contains("nested/root.rs"));
    assert!(json.contains("keep.tmp"));
    assert!(!json.contains("nested/local.rs"));
    assert!(!json.contains("ignored/child.rs"));
    let all = fixture.run(&["--json", "--no-ignore"]);
    assert!(
        String::from_utf8(all.stdout)
            .unwrap()
            .contains("ignored/child.rs")
    );
}

#[test]
fn registry_shebangs_generated_files_and_csv_are_usable() {
    let fixture = Fixture::new();
    fixture.write("run", "#!/usr/bin/env python3\nprint('hello')\n");
    fixture.write("app.ts", "// note\nconst n: number = 1;\n");
    fixture.write(
        "generated.rs",
        "// @generated DO NOT EDIT\nfn generated() {}\n",
    );
    let json = String::from_utf8(fixture.run(&["--json", "--no-generated"]).stdout).unwrap();
    assert!(json.contains("\"language\":\"Python\""));
    assert!(json.contains("\"language\":\"TypeScript\""));
    assert!(json.contains("\"generated\":1"));
    let csv = fixture.run(&["--format=csv", "--include-ext=ts"]);
    assert!(csv.status.success());
    let csv = String::from_utf8(csv.stdout).unwrap();
    assert_eq!(csv.lines().count(), 2);
    assert!(csv.contains("\"app.ts\",\"TypeScript\",source"));
    let languages = fixture.run(&["--languages"]);
    assert!(
        String::from_utf8(languages.stdout)
            .unwrap()
            .contains("TypeScript")
    );
}

#[test]
fn finite_observation_counts_follow_surviving_evidence() {
    let fixture = Fixture::new();
    fixture.write(
        "model.ccm",
        concat!(
            "ccm-model\t1\ncontext\texample\nartifact\ta\n",
            "evidence\tcheck\ttest\taccepted\ta\t-\n",
            "case\tok\taccepted\ncase\tbad\trejected\nallows\tcheck\tok\n"
        ),
    );
    let model = fixture.0.join("model.ccm");
    let stable = fixture.run(&["--json", "--model", model.to_str().unwrap()]);
    let changed = fixture.run(&[
        "--json",
        "--model",
        model.to_str().unwrap(),
        "--changed",
        "a",
    ]);
    assert!(stable.status.success() && changed.status.success());
    assert!(
        String::from_utf8(stable.stdout)
            .unwrap()
            .contains("\"observations\":1")
    );
    assert!(
        String::from_utf8(changed.stdout)
            .unwrap()
            .contains("\"observations\":2")
    );
}

#[cfg(unix)]
#[test]
fn piped_output_is_plain_and_forced_color_is_explicit() {
    let fixture = Fixture::new();
    fixture.write("file.rs", "fn f() {}\n");
    let plain = fixture.run(&[]);
    assert!(!plain.stdout.contains(&27));
    let forced = fixture.run(&["--color=always"]);
    assert!(forced.stdout.contains(&27));
    let json = fixture.run(&["--json", "--color=always"]);
    assert!(!json.stdout.contains(&27));
}

#[test]
fn admissible_failures_refine_cases_but_never_discharge_acceptance_checks() {
    let fixture = Fixture::new();
    fixture.write(
        "model.ccm",
        concat!(
            "ccm-model\t2\ncontext\tnegative-evidence\nartifact\ta\n",
            "check\tclaim\t1\t-\nevidence\tfailed-run\ttest\tfailed\ta\tclaim\n",
            "case\tok\taccepted\ncase\tbad\trejected\nallows\tfailed-run\tbad\n"
        ),
    );
    let model = fixture.0.join("model.ccm");
    let output = fixture.run(&[
        "--json",
        "--model",
        model.to_str().unwrap(),
        "--fail-on-pending",
    ]);
    assert_eq!(output.status.code(), Some(3));
    let json = String::from_utf8(output.stdout).unwrap();
    assert!(json.contains("\"pending_units\":1"));
    assert!(json.contains("\"failed\":true"));
    assert!(json.contains("\"observations\":1"));
    let changed = fixture.run(&[
        "--json",
        "--model",
        model.to_str().unwrap(),
        "--changed",
        "a",
    ]);
    let json = String::from_utf8(changed.stdout).unwrap();
    assert!(json.contains("\"failed\":false"));
    assert!(json.contains("\"observations\":2"));
}

#[test]
fn help_force_and_literal_option_names_work_without_scanning_unrelated_paths() {
    let fixture = Fixture::new();
    fixture.write("file.rs", "fn f() {}\n");
    assert!(fixture.run(&["--json", "--help"]).status.success());
    let output = fixture.0.join("report.json");
    std::fs::write(&output, "old content").unwrap();
    let result = fixture.run(&["--json", "--output", output.to_str().unwrap(), "--force"]);
    assert!(result.status.success());
    assert!(
        std::fs::read_to_string(&output)
            .unwrap()
            .contains("ccm.report.v3")
    );
    let result = fixture.run(&["--format=ndjson", "--include-ext=rs"]);
    assert_eq!(String::from_utf8(result.stdout).unwrap().lines().count(), 2);
}

#[test]
fn report_limits_fail_before_creating_an_output_file() {
    let fixture = Fixture::new();
    fixture.write("a.rs", "fn f() {}\n");
    let path = fixture.0.join("limited.json");
    let result = fixture.run(&[
        "--json",
        "--max-report-bytes",
        "32",
        "-o",
        path.to_str().unwrap(),
    ]);
    assert_eq!(result.status.code(), Some(1));
    assert!(!path.exists());
    assert!(result.stdout.is_empty());
    assert!(
        String::from_utf8(result.stderr)
            .unwrap()
            .contains("REPORT-LIMIT")
    );
}

#[test]
fn worktree_metadata_is_not_treated_as_a_directory_and_v1_failures_keep_legacy_semantics() {
    let fixture = Fixture::new();
    fixture.write(".git", "gitdir: /not-used-by-ccm\n");
    fixture.write("lib.rs", "fn f() {}\n");
    assert!(fixture.run(&["--json"]).status.success());
    fixture.write(
        "model.ccm",
        concat!(
            "ccm-model\t1\ncontext\tlegacy\ncheck\tclaim\t1\t-\n",
            "evidence\told-failure\ttest\tfailed\t-\tclaim\n"
        ),
    );
    let model = fixture.0.join("model.ccm");
    let output = fixture.run(&["--json", "--model", model.to_str().unwrap()]);
    let json = String::from_utf8(output.stdout).unwrap();
    assert!(json.contains("\"status\":\"not-accepted\""));
    assert!(json.contains("\"pending_units\":1"));
}

#[cfg(unix)]
#[test]
fn invalid_utf8_arguments_and_selection_failures_are_errors_without_panics() {
    use std::os::unix::ffi::OsStringExt;
    let invalid = std::ffi::OsString::from_vec(vec![b'x', 255]);
    let output = Command::new(env!("CARGO_BIN_EXE_ccm"))
        .arg(invalid)
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    assert!(
        String::from_utf8(output.stderr)
            .unwrap()
            .contains("ARGUMENT-ENCODING")
    );
    let fixture = Fixture::new();
    for index in 0..64 {
        fixture.write(&format!("file{index:02}.rs"), "fn f() {}\n");
    }
    let invalid = std::ffi::OsString::from_vec(vec![b'z', 255, b'.', b'r', b's']);
    std::fs::write(fixture.0.join(invalid), "fn f() {}\n").unwrap();
    let output = fixture.run(&["--json"]);
    assert_eq!(output.status.code(), Some(1));
    assert!(
        String::from_utf8(output.stderr)
            .unwrap()
            .contains("PATH-ENCODING")
    );
}

#[test]
fn double_dash_preserves_filenames_that_look_like_options() {
    let fixture = Fixture::new();
    fixture.write("--help.rs", "fn f() {}\n");
    let output = Command::new(env!("CARGO_BIN_EXE_ccm"))
        .current_dir(&fixture.0)
        .args(["--", "--help.rs"])
        .output()
        .unwrap();
    assert!(output.status.success());
    assert!(
        !String::from_utf8(output.stdout)
            .unwrap()
            .contains("Usage: ccm")
    );
}

#[test]
fn ignore_selection_agrees_with_git_on_scoped_rules_and_posix_classes() {
    if Command::new("git")
        .arg("--version")
        .stdout(std::process::Stdio::null())
        .status()
        .is_err()
    {
        return;
    }
    let fixture = Fixture::new();
    assert!(
        Command::new("git")
            .args(["init", "-q"])
            .arg(&fixture.0)
            .status()
            .unwrap()
            .success()
    );
    fixture.write(
        ".gitignore",
        "[[:digit:]].rs\n*.tmp\n!keep.tmp\n/root.rs\n/deep**/z.rs\n/y/**q.rs\n[]q].rs\n",
    );
    let names = [
        "3.rs",
        "x.rs",
        "root.rs",
        "nested/root.rs",
        "a.tmp",
        "keep.tmp",
        "deepx/z.rs",
        "deep/a/z.rs",
        "y/r/q.rs",
        "y/rq.rs",
        "q.rs",
        "].rs",
    ];
    for name in names {
        fixture.write(name, "fn f() {}\n");
    }
    let scan = fixture.run(&["--json", "--count-as=tmp:Rust"]);
    assert!(scan.status.success());
    let json = String::from_utf8(scan.stdout).unwrap();
    for name in names {
        let ignored = Command::new("git")
            .arg("-C")
            .arg(&fixture.0)
            .args(["check-ignore", "--no-index", name])
            .env("LC_ALL", "C")
            .stdout(std::process::Stdio::null())
            .status()
            .unwrap()
            .success();
        let included = json.contains(&format!("\"path\":\"{name}\""));
        assert_eq!(included, !ignored, "{name}");
    }
}

#[test]
fn ai_development_defaults_use_2026_priors_and_exclude_prose_and_generated_work() {
    let fixture = Fixture::new();
    fixture.write("main.py", "print('hello')\n");
    fixture.write("README.md", "A hundred words are not a program.\n");
    fixture.write("generated.rs", "// @generated\nfn f() {}\n");
    let output = fixture.run(&["--json"]);
    assert!(output.status.success());
    let json = String::from_utf8(output.stdout).unwrap();
    assert!(json.contains("\"year\":2026"));
    assert!(json.contains("\"program_lines\":1,"));
    assert!(json.contains("\"implementation_usd_per_attempt\":0.1135"));
    assert!(json.contains("\"tools_usd_per_attempt\":0.03"));
    let verbose = String::from_utf8(fixture.run(&["--verbose"]).stdout).unwrap();
    assert!(verbose.contains("DEVELOPMENT ESTIMATE"));
    assert!(!verbose.contains("Inferred implementation tokens:"));
    assert!(verbose.contains("docs/economics.rst (2026)"));
    assert!(verbose.contains("$0.03"));
    let omitted = String::from_utf8(fixture.run(&["--json", "--no-cost"]).stdout).unwrap();
    assert!(omitted.contains("\"ai_development\":null"));
}

#[test]
fn editable_cost_profiles_reprice_the_same_workload_and_manual_overrides_are_explicit() {
    let fixture = Fixture::new();
    fixture.write("main.py", "pass\n");
    fixture.write(
        "cost.conf",
        "input_price=0\noutput_price=0\nsession_price=0\n",
    );
    let profile = fixture.0.join("cost.conf");
    let output = fixture.run(&[
        "--json",
        "--include-ext=py",
        "--cost-profile",
        profile.to_str().unwrap(),
    ]);
    assert!(output.status.success());
    let json = String::from_utf8(output.stdout).unwrap();
    assert!(json.contains("\"implementation_usd_per_attempt\":0,"));
    assert!(json.contains("\"overall_expected_usd\":25,"));
    let manual = String::from_utf8(
        fixture
            .run(&["--json", "--include-ext=py", "--agent-cost=10"])
            .stdout,
    )
    .unwrap();
    assert!(manual.contains("\"manual_override\":true"));
    assert!(manual.contains("\"overall_expected_usd\":35,"));
    fixture.write("cost.conf", "input_price=NaN\n");
    assert_eq!(
        fixture
            .run(&["--cost-profile", profile.to_str().unwrap()])
            .status
            .code(),
        Some(1)
    );
}

#[test]
fn ai_residual_check_cost_follows_ccc_evidence_without_charging_unrelated_source() {
    let fixture = Fixture::new();
    fixture.write("main.py", "pass\n");
    fixture.write("unrelated.rs", "fn f() {}\n");
    fixture.write(
        "model.ccm",
        concat!(
            "ccm-model\t2\ncontext\tbudget\nartifact\tmain.py\n",
            "check\taccept\t2\tmain.py\n",
            "evidence\tpassed\ttest\taccepted\tmain.py\taccept\n"
        ),
    );
    let model = fixture.0.join("model.ccm");
    let unchanged = fixture.run(&["--json", "--model", model.to_str().unwrap()]);
    assert!(unchanged.status.success());
    assert!(
        String::from_utf8(unchanged.stdout)
            .unwrap()
            .contains("\"checks_usd_per_attempt\":0,")
    );
    let changed = fixture.run(&[
        "--json",
        "--model",
        model.to_str().unwrap(),
        "--changed",
        "main.py",
    ]);
    assert!(changed.status.success());
    let json = String::from_utf8(changed.stdout).unwrap();
    assert!(json.contains("\"program_lines\":1,"));
    assert!(json.contains("\"check_tokens_inferred\":2000,"));
    assert!(json.contains("\"language\":\"Python\",\"lines\":1,"));
    assert!(!json.contains("\"language\":\"Rust\",\"lines\":"));
}

#[test]
fn catalog_covers_recorded_cloc_scc_names_and_sloc_extensions() {
    let fixture = Fixture::new();
    let result = fixture.run(&["--languages", "--json"]);
    assert!(result.status.success());
    let json = String::from_utf8(result.stdout).unwrap();
    let coverage = include_str!("../../../docs/benchmarks/language-coverage-2026.tsv");
    for row in coverage.lines().skip(1) {
        let (tool, name) = row.split_once('\t').unwrap();
        if tool == "sloc-extension" {
            assert!(
                ccm_scan::Language::from_path(std::path::Path::new(&format!("file.{name}")))
                    .is_some(),
                "SLOC extension: {name}"
            );
        } else {
            assert!(
                json.contains(&format!("\"name\":\"{name}\"")),
                "{tool}: {name}"
            );
        }
    }
}

#[test]
fn mixed_sources_and_typst_raw_examples_produce_a_complete_compact_report() {
    let fixture = Fixture::new();
    fixture.write(
        "main.typ",
        "A lone \" prose quote.\n`views/*.vue`\n```rs\n/* raw\n```\n",
    );
    fixture.write(
        "App.vue",
        concat!(
            "<template>\n<!-- comment -->\n<p>Hello</p>\n</template>\n",
            "<script>\n// note\nconst x = 1;\n</script>\n"
        ),
    );
    fixture.write("Cargo.lock", "# generated\nversion = 4\n");
    fixture.write("some.png", "not read as source\n");
    let result = fixture.run(&[]);
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let report = String::from_utf8(result.stdout).unwrap();
    assert!(report.contains("Comments") && report.contains("Code") && report.contains("Lines"));
    assert!(!report.contains("Content"));
    assert!(report.contains("Cargo lock") && report.contains("Vue") && report.contains("Typst"));
    assert!(!report.contains("AI required checks: $0.00"));
    assert!(!report.contains("Inferred implementation tokens"));
    assert!(!report.contains("Warning:"));
    assert!(report.contains("Skipped: 1 binary asset."), "{report}");
}

fn text(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

#[test]
fn header_program_lines_exclude_generated_source_like_the_estimate() {
    let fixture = Fixture::new();
    fixture.write("main.rs", "fn main() {}\nfn other() {}\n");
    fixture.write(
        "table.rs",
        "// @generated\nconst A: u8 = 1;\nconst B: u8 = 2;\n",
    );
    fixture.write("README.md", "Prose.\n");
    let report = text(&fixture.run(&["--verbose"]));
    assert!(
        report.contains("2 program lines (+2 generated)"),
        "{report}"
    );
    assert!(
        report.contains("Priced scope: 2 program lines;"),
        "{report}"
    );
    let json = text(&fixture.run(&["--json"]));
    assert!(json.contains("\"program_lines\":2,\"generated_source_lines\":2,\"source_lines\":4,"));
    let html = text(&fixture.run(&["--html"]));
    assert!(html.contains("<span>Program lines</span><strong>2</strong>"));
    assert!(html.contains("2 generated source lines"));
}

#[test]
fn oversized_files_are_skipped_as_coverage_gaps() {
    let fixture = Fixture::new();
    fixture.write("small.rs", "fn f() {}\n");
    fixture.write("large.rs", &"fn f() {}\n".repeat(20));
    let output = fixture.run(&["--max-file-bytes", "100"]);
    assert_eq!(output.status.code(), Some(2));
    let report = text(&output);
    assert!(
        report.contains("1 file over the per-file limit (--max-file-bytes)"),
        "{report}"
    );
    assert!(report.contains("large.rs: 200 bytes exceed the per-file limit of 100"));
    assert!(
        report.contains("Incomplete: 1 file not counted (exit 2)."),
        "{report}"
    );
    let json = text(&fixture.run(&["--json", "--max-file-bytes", "100"]));
    assert!(json.contains("\"oversized\":1"));
    assert!(json.contains("\"path\":\"small.rs\""));
}

#[test]
fn ccm_extension_counts_models_by_header_and_cpp_modules_otherwise() {
    let fixture = Fixture::new();
    fixture.write("plan.ccm", "ccm-model\t1\ncontext\tx\n");
    fixture.write("module.ccm", "export module m;\nint f() { return 1; }\n");
    let json = text(&fixture.run(&["--json"]));
    assert!(
        json.contains("\"path\":\"plan.ccm\",\"language\":\"CCM model\""),
        "{json}"
    );
    assert!(
        json.contains("\"path\":\"module.ccm\",\"language\":\"C++\""),
        "{json}"
    );
}

#[test]
fn default_build_names_exclude_directories_only_and_exclude_accepts_dot_slash() {
    let fixture = Fixture::new();
    fixture.write("dist", "#!/bin/sh\necho dist\n");
    fixture.write("build/out.rs", "fn generated() {}\n");
    fixture.write("sub/a.rs", "fn a() {}\n");
    fixture.write("keep.rs", "fn keep() {}\n");
    // Without a repository or an ignore file, as in an unpacked archive, names decide.
    let json = text(&fixture.run(&["--json"]));
    assert!(json.contains("\"path\":\"dist\""), "{json}");
    assert!(!json.contains("build/out.rs"));
    assert!(json.contains("\"unsupported_files\":0"));
    // A repository governs its own tree: `build/` counts unless an ignore rule says not.
    fixture.write(".git", "gitdir: elsewhere\n");
    let json = text(&fixture.run(&["--json"]));
    assert!(json.contains("build/out.rs") && !json.contains("\"path\":\".git\""));
    fixture.write(".gitignore", "/build/\n");
    assert!(!text(&fixture.run(&["--json"])).contains("build/out.rs"));
    // `--no-ignore` sets the ignore files aside, so the names apply again.
    std::fs::remove_file(fixture.0.join(".gitignore")).unwrap();
    std::fs::remove_file(fixture.0.join(".git")).unwrap();
    fixture.write("sub/.gitignore", "\n");
    fixture.write("sub/build/b.rs", "fn b() {}\n");
    assert!(text(&fixture.run(&["--json"])).contains("sub/build/b.rs"));
    assert!(!text(&fixture.run(&["--json", "--no-ignore"])).contains("sub/build/b.rs"));
    let json = text(&fixture.run(&["--json", "--exclude", "./sub/"]));
    assert!(!json.contains("sub/a.rs") && json.contains("keep.rs"));
    let json = text(&fixture.run(&["--json", "--exclude", "."]));
    assert!(json.contains("keep.rs"));
}

#[test]
fn parent_ignore_files_apply_only_inside_an_enclosing_repository() {
    let fixture = Fixture::new();
    fixture.write(".gitignore", "*.rs\n");
    fixture.write("child/a.rs", "fn a() {}\n");
    let child = fixture.0.join("child");
    let run = || {
        text(
            &Command::new(env!("CARGO_BIN_EXE_ccm"))
                .args(["--json"])
                .arg(&child)
                .output()
                .unwrap(),
        )
    };
    assert!(run().contains("\"path\":\"a.rs\""));
    std::fs::create_dir(fixture.0.join(".git")).unwrap();
    assert!(!run().contains("\"path\":\"a.rs\""));
}

#[test]
fn terminal_reports_never_exceed_any_supported_width() {
    let fixture = Fixture::new();
    fixture.write("main.rs", "fn main() {}\n");
    fixture.write("docs/a-rather-long-documentation-file-name.md", "Prose.\n");
    fixture.write("data.json", "{}\n");
    let model = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/checkout.ccm");
    let model = model.to_str().unwrap();
    for columns in 44..=130 {
        for color in ["never", "always"] {
            let output = Command::new(env!("CARGO_BIN_EXE_ccm"))
                .args(["--technical", "--model", model, "--changed", "pricing"])
                .arg(format!("--color={color}"))
                .arg(&fixture.0)
                .env("COLUMNS", columns.to_string())
                .output()
                .unwrap();
            let report = text(&output);
            assert!(report.contains("LANGUAGES"), "{report}");
            for line in report.lines() {
                let visible = visible_width(line);
                assert!(visible <= columns, "{columns} {color}: {line:?}");
            }
        }
    }
}

/// Counts visible characters of a line, skipping ANSI SGR sequences.
fn visible_width(line: &str) -> usize {
    let mut width = 0;
    let mut escape = false;
    for c in line.chars() {
        if c == '\u{1b}' {
            escape = true;
        } else if escape {
            escape = c != 'm';
        } else {
            width += 1;
        }
    }
    width
}

#[test]
fn plain_format_is_ascii_and_terminal_format_groups_rows_by_purpose() {
    let fixture = Fixture::new();
    fixture.write("main.rs", "fn main() {}\nfn other() {}\n");
    fixture.write("table.rs", "// @generated\nconst A: u8 = 1;\n");
    fixture.write("README.md", "Prose.\n");
    let plain = text(&fixture.run(&["--format", "plain", "--color=always", "--technical"]));
    assert!(plain.is_ascii(), "{plain}");
    assert!(!plain.contains('\u{1b}'));
    for expected in [
        "Program source",
        "Generated",
        "Documentation",
        "2 program lines (+1 generated)",
        "DEVELOPMENT ESTIMATE",
        "Range  $",
        "Total",
    ] {
        assert!(plain.contains(expected), "{expected}: {plain}");
    }
    let unicode = Command::new(env!("CARGO_BIN_EXE_ccm"))
        .arg(&fixture.0)
        .env("NO_COLOR", "1")
        .env("LC_ALL", "C.UTF-8")
        .output()
        .unwrap();
    let unicode = text(&unicode);
    assert!(
        unicode.contains('\u{2588}') && !unicode.contains('\u{1b}'),
        "{unicode}"
    );
    let ascii = Command::new(env!("CARGO_BIN_EXE_ccm"))
        .arg(&fixture.0)
        .env("LC_ALL", "C")
        .output()
        .unwrap();
    assert!(text(&ascii).is_ascii());
}

#[test]
fn model_reports_sketch_the_change_closure_and_explain_pending_exits() {
    let fixture = Fixture::new();
    fixture.write("lib.rs", "fn f() {}\n");
    let model = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/checkout.ccm");
    let model = model.to_str().unwrap();
    let output = fixture.run(&[
        "--format",
        "plain",
        "--model",
        model,
        "--changed",
        "pricing",
        "--fail-on-pending",
    ]);
    assert_eq!(output.status.code(), Some(3));
    let report = text(&output);
    assert!(
        report.contains("1 changed -> 2 affected -> 3 checks (2 pending)"),
        "{report}"
    );
    assert!(
        report.contains("[ ] pending    Pricing contract"),
        "{report}"
    );
    assert!(
        report.contains("[x] supported  Payment boundary review"),
        "{report}"
    );
    assert!(report.contains("Human acceptance checks"), "{report}");
    assert!(
        report.contains("Pending: 2 checks still need evidence (exit 3, --fail-on-pending)."),
        "{report}"
    );
}

#[test]
fn model_estimates_never_print_negative_zero_and_custom_profiles_drop_sensitivity() {
    let fixture = Fixture::new();
    fixture.write("lib.rs", "fn f() {}\n");
    let model = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/checkout.ccm");
    let model = model.to_str().unwrap();
    let base = ["--model", model, "--changed", "pricing"];
    let terminal = text(&fixture.run(&[&base[..], &["--technical"]].concat()));
    let json = text(&fixture.run(&[&base[..], &["--json"]].concat()));
    assert!(
        !terminal.contains("-0") && !json.contains(":-0"),
        "{terminal}\n{json}"
    );
    assert!(json.contains("\"implementation_sensitivity_usd\":["));
    fixture.write("cost.conf", "input_price=1\n");
    let profile = fixture.0.join("cost.conf");
    let json = text(&fixture.run(&["--json", "--cost-profile", profile.to_str().unwrap()]));
    assert!(json.contains("\"implementation_sensitivity_usd\":null"));
}

#[test]
fn review_without_a_model_counts_program_files_only() {
    let fixture = Fixture::new();
    fixture.write("main.py", "pass\n");
    fixture.write("README.md", "Prose.\n");
    fixture.write("data.json", "{}\n");
    fixture.write("gen.rs", "// @generated\nfn f() {}\n");
    let json = text(&fixture.run(&["--json"]));
    assert!(json.contains("\"review_artifacts\":1,"), "{json}");
    let old = Fixture::new();
    old.write("main.py", "x = 1\n");
    old.write("README.md", "Old prose.\n");
    old.write("gone.py", "pass\n");
    let json = text(&fixture.run(&["--json", "--compare", old.0.to_str().unwrap()]));
    assert!(json.contains("\"review_artifacts\":2,"), "{json}");
}

#[test]
fn model_errors_show_the_source_line_with_an_aligned_caret() {
    let fixture = Fixture::new();
    let mut source = String::from("ccm-model\t1\ncontext\tx\n");
    source.push_str(&"artifact\ta\n".repeat(0));
    source.push_str(&"# filler\n".repeat(8));
    source.push_str("edge\ta\tb\n");
    fixture.write("model.ccm", &source);
    let model = fixture.0.join("model.ccm");
    let output = fixture.run(&["--model", model.to_str().unwrap()]);
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8(output.stderr).unwrap();
    let lines: Vec<_> = stderr.lines().collect();
    assert!(lines[0].starts_with("-- MODEL-INPUT --") && lines[0].ends_with(":11:1"));
    let excerpt = lines
        .iter()
        .position(|l| l.starts_with("11| edge"))
        .unwrap();
    assert_eq!(lines[excerpt + 1], "    ^", "{stderr}");
    assert!(stderr.contains("Hint: "));
}
