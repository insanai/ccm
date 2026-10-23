:cmd-number: 0004
:cmd-title: Competitive counting and evidence-conditioned reports
:cmd-state: committed
:cmd-type: Standards Track
:cmd-authors: ["Vikrant Rathore"]
:cmd-created: 2026-10-23
:cmd-updated: 2026-10-23
:cmd-discussion: Local implementation review at the maintainer's request
:cmd-labels: ["architecture", "cli", "algorithms", "validation"]

Competitive counting and evidence-conditioned reports
======================================================

Abstract
--------

We extend CCM to a practical, bounded, parallel source inventory and an executable finite assurance model.
We compare observable results with the installed scc 4.1.0, without copying its implementation or treating its output as a language specification.
We retain zero external crate dependencies.
We report evidence of particular improvements rather than asserting superiority on every workload, language, or feature.
This record supersedes CMD 0003's initial counting and report boundary while retaining its explicit activity-pricing scenario.

Review findings
----------------

The reported Guidedog run completed its selected source scan.
Its long list of license symlinks was informational, not an error.
The first interface made normal selection behavior look like a failure and required a populated TERM variable before enabling color.
It also lacked ignore rules, grouped all documentation as Text, supported too few lexical policies, and scanned sequentially.
The finite kernel implemented propagation and transported residual work, but not the declared behavioral case universe.
A failing observation also needed a representation distinct from a favorable certificate and from an unchecked assertion.

Counting and selection
-----------------------

We adopt lexical-v2 with 55 immutable language entries, exact filename rules, case-insensitive extension recognition, extension overrides, and extensionless shebang detection.
We distinguish program source, documentation, and data without changing the disjoint physical line classes.
The terminal's Content column means code for source files and nonblank content for prose and data.
Program-line totals exclude documentation and data.
Language names are policies, not promises of complete parsing or semantic extraction.
``docs/counting.rst`` records the supported constructs and deliberate omissions.

We handle Rust raw strings and nested comments, C++ delimited raw strings, Go backtick strings, Python triple strings, JavaScript regular-expression character classes, shell heredocs and arithmetic shifts, Lua long delimiters, markup comments, Markdown fences, and the registered common comment profiles.
We refuse detected unterminated constructs instead of silently assigning apparently complete counts.
Known lexical subsets remain explicit; a passing fixture is not a proof of complete language coverage.

Selection reads scoped .gitignore, .ignore, and .ccmignore rules.
Ordinary repositories also read .git/info/exclude.
We support anchoring, directory pruning, ordered negation, escaped literals, star, double-star, question mark, bracket ranges, and ASCII POSIX character classes.
A negation cannot reinclude a child of an excluded directory unless the directory itself is reenabled.
We stop ancestor discovery at a repository boundary.
We do not read global Git configuration or resolve external Git metadata referenced by a worktree's .git file.
Explicit exclusions remain literal names or path prefixes; extension filters and generated-header exclusion are separate options.

Symlinks and binary files are normal skips with aggregate counters.
Unreadable inputs, malformed models, changing source, exhausted limits, and output failures remain errors.
Lexical coverage failures produce a report with complete=false and status 2.
Default text reports show at most five coverage warnings; verbose output and JSON retain the details.

Bounded parallel execution
---------------------------

A deterministic walker feeds a bounded synchronous queue with four slots per worker.
Scoped workers reuse byte buffers and classify files independently.
The walker checks selection and byte budgets before enqueueing a file.
Workers read at most the selected length plus one byte and compare the open file's length and modification time with the selection stamp.
This detects ordinary mutation, not adversarial races or atomic snapshot identity.

One short receiver lock assigns a whole file; no lock is held during file reads or lexical scanning.
Every worker owns a receiver reference.
If all workers fail, their receiver references disappear and a blocked sender disconnects.
Dropping the sender after selection also releases workers waiting after a traversal error.
We merge reports in stable path order, so worker scheduling does not change complete JSON results.
The default is the available CPU count capped at eight, with an explicit range of one through 64 workers.

The byte scan is linear in selected bytes under bounded delimiter state.
Sorting retained results adds file-count sorting cost.
Ignore matching uses literal comparison where possible and a bounded dynamic program for globs, never recursive backtracking.
The aggregate glob work budget is 200 million steps; ignore files total at most 4 MiB and 10,000 rules, with at most 256 KiB per file and 1,024 bytes per pattern.
Existing source, traversal, and finite-model budgets remain in force.

We cap rendered reports at 64 MiB by default.
A bounded formatting buffer stops allocation and subsequent entry rendering on exhaustion and returns a report-limit diagnostic before any output is written.
The output budget has an explicit override.
Technical graph reports retain one predecessor witness per reached artifact instead of materializing every full path, avoiding quadratic path presentation.

Finite behavioral ambiguity
----------------------------

We add optional finite case and evidence-compatibility records to the model input.
A case has an identity and a task observation label.
Different cases can share the same observation.
For surviving admissible evidence :math:`L`, the engine intersects compatible case sets and counts the unique observation image.
It reports exact case and observation cardinalities, plus the rounded display :math:`\log_2|g(\Omega_L)|` when the image is nonempty.
No probability distribution is inferred, and this quantity is not Shannon entropy.

An empty compatible universe is inconsistent, with unavailable distinction bits.
It is never presented as zero uncertainty, successful verification, or evidence that a source program failed.
A missing universe remains unavailable rather than being synthesized from syntax.
The finite core also implements the signed evidence interaction

.. math::

   \chi(e,f\mid L)=H_A(L\cup\{e\})+H_A(L\cup\{f\})-H_A(L)-H_A(L\cup\{e,f\}).

A positive value means the joint distinction reduction exceeds the separate reductions; negative values express overlap.
The computation returns unavailable if one of the required intersections is inconsistent.
Tests include both signs, observation quotienting, monotonicity over every pair of four-case filters, and restoration of uncertainty after evidence invalidation.
This executes CMD 0002's declared finite construction without claiming source extraction soundness.

Admissibility and conclusions
-----------------------------

Version 2 model input distinguishes admissibility from the evidence conclusion.
An accepted favorable result can discharge its declared active checks.
A declared checked failure remains admissible evidence, may eliminate incompatible hypotheses, and leaves the associated active checks pending and marked failed.
A favorable route does not hide a surviving negative result for the same active check.
Unverified evidence and assertions cannot discharge checks or narrow cases as checked observations.
All evidence still requires author-declared context and support; the scanner executes no checker and authenticates no certificate.

We retain version 1 input compatibility.
In version 1 a failed record remains unaccepted, as specified in CMD 0003.
Version 2 deliberately changes that interpretation and is therefore explicitly versioned.
The updated checkout fixture uses version 2.

CLI and output contracts
------------------------

We add sorted language summaries, per-file output, language discovery, extension filters and remapping, generated-file filtering, worker selection, CSV, and NDJSON.
The existing terminal, technical, JSON, HTML, snapshot comparison, and activity-budget reports remain available.
The report schema is ccm.report.v2 and exposes file purpose, generated status, skip counters, exact finite outcomes, and checker conclusions.
CSV protects spreadsheet formula-leading paths; JSON preserves exact path strings.
NDJSON emits one file object per line and a final report object; it is a record format, not a promise of constant-memory input processing.

Automatic color requires a terminal but does not require TERM to exist.
A nonempty NO_COLOR, TERM=dumb, or CLICOLOR=0 disables automatic color.
Explicit color=always overrides automatic detection, while JSON and other machine formats never contain terminal decoration.
The palette uses portable ANSI colors, including cyan source counts, green comments, amber pending checks, and red failed checks.
Output respects the configured terminal column count with readable ASCII fallback.

Existing output files are protected by default.
Explicit force writes a temporary file in the destination directory and renames it into place.
The temporary file is cleaned up on failure.
Broken pipes are normal termination.
Non-UTF-8 arguments return diagnostics instead of panicking.
A double dash preserves literal filenames that resemble flags.

Normal report completion returns 0, errors return 1, and incomplete counting returns 2.
An explicit fail-on-pending option requires a task model and returns 3 for open active checks or inconsistent modeled observations after writing the report.
This makes task acceptance usable in CI without conflating missing evidence with a failed scanner.

Economics
----------

We retain CMD 0003's activity-based expected resource-cost equation.
Source counts do not identify implementation time, business value, defect loss, or an AI productivity multiplier.
The default source-only number prices an assumed review activity and excludes unknown verification demand.
Explicit model checks supply residual policy demand, priced only through explicit hours-per-unit assumptions.
Implementation hours, labor price, agent spending, and stationary repeat probability remain separate inputs.
No semantic bit count is silently converted into money or hours.

Validation and performance
---------------------------

We require the workspace build, tests, formatting, Clippy for all targets, repository limit checks, independent JSON and NDJSON parsing, a pseudo-terminal color check, and HTML inspection.
Tests exercise every registered common comment profile, selected language-specific delimiters, deterministic worker counts, scoped ignore precedence, Git reference selections, failure conclusions, stale evidence, output limits, protected replacement, and command-line boundaries.
The standalone finite research harness remains separate evidence for the larger research construction.

We add a dependency-free Rust benchmark driver under cargo xtask bench.
It creates a deterministic corpus, checks both tools against hand-declared physical counts before timing, alternates paired tool order, retains raw samples, and reports a median paired bootstrap interval.
Peak RSS measurements are recorded separately.
``docs/performance.rst`` records the exact workload, toolchain, environment, flags, results, and limits.
A synthetic fixture result does not establish CMD 0002's primary GuideDog gate or superiority across every language and project.

Open boundaries
----------------

We have not implemented full semantic extraction, tree-sitter adapters, automatic certificate validation, arbitrary embedded-language parsing, Git history attribution, MCP serving, or calibrated labor prediction.
We do not claim complete feature parity with scc.
We provide useful additional assurance measurements where an explicit task model exists and keep absent semantic coordinates unavailable.
Wider language conformance, additional project benchmarks, and empirical economic calibration remain required before broader claims.

Review history
---------------

2026-10-23: The maintainer requested competitive results, a stronger mathematical model, quieter normal scans, and reliable colors in a color terminal.
We reproduced the reported source scan and implemented the reviewed changes without external crates.
Implementation and benchmark evidence follows in the lifecycle entry.

References
-----------

* :doc:`0002-change-certificate-calculus` defines the finite change and evidence mathematics and the broader validation gates.
* :doc:`0003-acceptance-budgets-and-cli` defines the economic scenario retained here.
* The installed scc 4.1.0 help, language registry, and executable supplied the local comparison surface.
* ``tools/xtask/src/bench.rs`` defines the reproducible fixture and measurement procedure.

.. rubric:: Lifecycle event

::

   2026-10-23: prediscussion -> discussion. Assigned a permanent number and
   opened for discussion.

.. rubric:: Lifecycle event

::

   2026-10-23: discussion -> accepted. Implement the requested competitive CLI
   review, correct color and skip handling, and extend the finite evidence model
   while keeping unsupported claims explicit.

.. rubric:: Lifecycle event

::

   2026-10-23: accepted -> committed. Implemented the competitive counting
   review, corrected automatic color and normal skip handling, and completed
   finite behavioral evidence reporting. 38 workspace tests; cargo fmt --all
   --check; cargo build --workspace; release build; cargo clippy --workspace
   --all-targets -- -D warnings; cargo xtask check --soft-width; standalone
   finite research harness; independent JSON, NDJSON, CSV, and registry parsing;
   real pseudo-terminal color matrix; Chromium HTML inspection; 30 paired
   equivalent-count fixture samples and 30 additional peak-RSS samples per tool;
   raw samples and binary identities in docs/performance.rst; offline local CLI
   installation.
