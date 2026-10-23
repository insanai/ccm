:cmd-number: 0001
:cmd-title: The CCM Discussion Process
:cmd-state: discussion
:cmd-type: Process
:cmd-authors: ["Vikrant Rathore"]
:cmd-created: 2026-10-23
:cmd-updated: 2026-10-23
:cmd-discussion: Local repository review; no external discussion URL assigned
:cmd-labels: ["process"]

The CCM Discussion Process
==========================

Abstract
--------

Code Counter Machine Discussions (CMD) record proposals, reasoning, and decisions for CCM.
CCM is a source measurement CLI and a family of reusable Rust libraries.
We aim to succeed cloc, SLOCCount, and scc through accurate counting, useful measurements of change and verification, and measured performance.
We require evidence for mathematical, empirical, and performance claims separately.
This record proposes our process; its number does not imply acceptance.

Scope and principles
--------------------

We write a CMD for changes to counting semantics, language support contracts, measurement definitions, algorithms, public Rust APIs, CLI behavior, output schemas, effort estimation, caches, resource limits, and benchmark policy.
Routine refactors and fixes that preserve a specified contract need no new record.
We amend the relevant record when its explanation needs correction.
The manual will explain use; CMD explains decisions and their grounds.

We keep a simple common CLI path and a small raylib-inspired Rust API.
We use plain option structs, explicit contexts, verb-noun functions, structured errors, and Rust ownership.
Libraries do not depend on the application or print diagnostics.
Fast counting does not silently perform parsing, history analysis, compression, symbolic execution, or network access.
Optional analyses state their additional inputs and costs.

We measure source artifacts and specified tasks.
We do not infer authorship, human ability, correctness, or business value from source size or a complexity score.
AI-assisted development motivates change and verification measurements; it does not excuse weaker evidence.

Lifecycle and numbering
-----------------------

We start a draft as ``records/XXXXX-<slug>.rst`` in ``prediscussion``.
Promotion assigns the next unreserved number with at least four digits and moves the draft to ``discussion``.
Numbers identify records and are never reused.
The filename, metadata, registry, and reading order change together through GuideDog.

.. list-table:: Review states
   :header-rows: 1
   :widths: 22 43 35

   * - State
     - Meaning
     - Allowed next states
   * - ``prediscussion``
     - A local placeholder draft.
     - discussion by promotion; abandoned
   * - ``discussion``
     - Numbered and open for review.
     - accepted; published; abandoned
   * - ``accepted``
     - Maintainers agree on direction.
     - published; committed; abandoned
   * - ``published``
     - Maintainers adopt the design text.
     - committed; abandoned
   * - ``committed``
     - The record's acceptance criteria have implementation evidence.
     - none
   * - ``abandoned``
     - Closed with a reason; history remains.
     - none

.. graphviz::
   :caption: The usual path; the table also permits shortcuts and abandonment.
   :alt: A draft is numbered for discussion, accepted, published, and implemented.

   digraph lifecycle {
     graph [rankdir=LR, bgcolor="transparent"];
     node [shape=box, fontname="Helvetica"];
     "prediscussion" -> "discussion" [label="promote"];
     "discussion" -> "accepted";
     "accepted" -> "published";
     "published" -> "committed" [label="evidence"];
   }

``discussion.toml`` defines the executable state machine.
A state change records a reason; entry to ``committed`` also records evidence.
Only maintainers adopt or close proposals.
We do not invent review, acceptance, or implementation evidence when drafting a record.
A jj change, a Git commit, a successful documentation build, and a mathematical proof each establish different facts.
None alone establishes that a proposed tool works.
We preserve closed records and link a successor when a committed design changes.

Record structure and review
---------------------------

We use ``template/rfc-template.rst`` and begin with the catalog metadata.
Authors and labels are JSON arrays; dates use ISO format.
Every record has an Abstract and References.
We normally include motivation, scope, definitions, specification, worked examples, alternatives, compatibility, operational considerations, acceptance criteria, implementation status, open questions, and review history.
We keep normative definitions in the record rather than scattering them through implementation comments.

We write short, complete sentences in precise English and use first person plural.
We do not hard-wrap prose or split an inline expression across lines.
ASCII suffices except for names, other languages, and explicit Unicode examples.
We use RST mathematics for equations and Graphviz when a diagram explains a relation.

Review proceeds in four parts.

1. We check that the problem, observable behavior, scope, and alternatives are clear.
2. We check definitions, assumptions, proofs, counterexamples, and computational bounds.
3. We check the evidence plan, reproducibility, threat model, and resource limits.
4. We check implementation results against the adopted criteria before claiming completion.

Mathematical and empirical evidence
-----------------------------------

A **definition** fixes a term or construction.
An **axiom** states a modeling choice or assumption; we do not present it as an observed law of software development.
A **lemma** or **theorem** states a proposition with explicit hypotheses and a proof.
A **conjecture** is an unproved mathematical proposition.
An **empirical hypothesis** predicts measured behavior and names an experiment that could refute it.
A **heuristic** computes a useful approximation without the stronger guarantee.

For each mathematical result we state its domain, units, boundary cases, and dependence on extraction assumptions.
We distinguish exact quantities, certified lower or upper bounds, estimates, and unavailable results.
Unknown is never encoded as zero.
We distinguish soundness of an abstraction from usefulness of a score.
We do not infer semantic equivalence from token similarity, proof from test coverage, or correctness from a low score.

A proposed novel theory identifies its relation to prior work and the specific contribution we seek to test.
We check small finite cases exhaustively and publish counterexamples before implementation depends on a theorem.
We seek independent proof review.
Machine-checked proofs, when available, name their checker, version, assumptions, and trusted boundary.
A proof about a model does not prove that a language adapter implements the model faithfully.

Predictive claims require held-out projects or time periods, named outcomes, baselines, uncertainty, and an ablation of the proposed features.
We record failures and avoid tuning the model on its final evaluation data.
We version any fitted model separately from the measurement definitions.
An effort estimate requires task and process observations; source text alone does not determine time or price.

Performance and implementation evidence
---------------------------------------

We compare equivalent work against a pinned scc release or revision and retain correctness checks alongside timings.
We record corpus revisions, file selection, language policies, toolchain, build flags, machine, thread count, cache conditions, output cost, repetitions, elapsed time, and peak memory.
We publish per-corpus results and uncertainty as well as any aggregate.
We report regressions rather than hiding them behind an average.
An optional semantic analysis cannot establish a speed claim against a counter doing less work.

Rust implementation follows the project rules: small modules and functions, a workspace with shared package metadata and dependencies, explicit ownership, bounded input-driven growth, and structured errors.
We prefer the standard library and justify substantial dependencies beside their declarations.
Before an implementation record is marked ``committed``, we run the build, tests, Clippy for all targets, and the repository limit checker.
Before a documentation change is ready for review, we run ``guidedog dsc check`` and build the catalog with warnings treated as errors.
We record unavailable checks and failures explicitly.

Rust coding and tool rules
--------------------------

We adopt the engineering and diagnostic rules of JLD 0001 in the sibling ``../jlavi/docs/jld`` catalog, adapted to CCM.
We write the durable rules here so that a checkout does not depend on a home directory or sibling repository.
``rustfmt.toml`` sets ``max_width = 99`` and ``clippy.toml`` sets ``too-many-lines-threshold = 70`` when the workspace is created.

* A function has at most 70 logical lines, excluding its signature, blank lines, comments, and ornamental delimiters.
* An implementation file has at most 1,408 physical lines.
* Code has a soft width of 99 characters and a hard width of 108 characters.
* These limits apply to Rust and other implementation languages, including CSS, JavaScript, HTML, TOML, shell, and executable Typst templates.
* Documentation prose, diagrams, and markup structure are exempt from code width, file length, and function length limits.
* Generated implementation code has no blanket exemption; pinned third-party assets retain their upstream form and provenance.

We split modules by responsibility and do not pack statements onto a line to evade a limit.
The repository limit checker counts logical function lines, including closures and multiline expressions, with a lexer that excludes strings and comments.
Rustfmt and Clippy alone do not establish compliance.
Hard limits fail the check; the checker also provides a mode that fails on soft-width findings.

Libraries live under ``crates/``, binaries under ``cmd/`` or ``src/bin``, and repository tools under ``tools/``.
Tools are Rust binaries, including ``cargo xtask`` and the limit checker; we do not add Python or shell maintenance scripts.
Core measurement types and deterministic mathematics have no I/O or async dependencies.
Adapters depend on core; the application composes adapters.
Every crate inherits one version, edition, license, minimum Rust version, and lint policy from the root workspace.
Every dependency is declared once in ``[workspace.dependencies]`` and inherited by members.
We update any ``VERSION`` file with the root version.

We prefer descriptive names, small functions, exhaustive enums, newtype identifiers, checked arithmetic, and narrow exports.
Comments explain why a choice exists.
Ownership and borrowing replace manual allocator schemes and runtime handle-validity checks.
We borrow slices and strings, use ``Cow`` for usually unchanged text, reuse buffers, stream large inputs, and prefer flat vectors with newtype indices for graphs.
We use ``Rc`` or ``Arc`` only for genuinely shared immutable ownership.
We use scoped standard-library threads when parallelism is justified.
We measure before optimizing and keep bounds checks enabled.

The standard library is our first choice.
A mature external dependency must solve a substantial problem and have its reason beside its manifest entry.
We invoke an installed tool with ``std::process::Command`` and an argument vector, without a shell, when that boundary is reliable.
We avoid needless traits, generics, macros, and builder hierarchies.
Every public item documents behavior, borrowed data, preconditions, ownership, failure modes, and any justified complexity bound.

Untrusted input has explicit byte, object, depth, and work limits.
Input-driven large allocations use ``try_reserve`` and return a capacity, memory, or limit error.
We avoid unbounded input-driven recursion.
Library code does not call ``unwrap`` or ``expect`` on input-dependent values.
Panics state internal invariants established by validation.
``unsafe`` is confined to foreign-function modules and every block has a ``// SAFETY:`` explanation.
Scoped resources have explicit lifetimes and cleanup.
Rust's global allocator can still abort for allocations outside the recoverable boundary; we do not claim otherwise.

Raylib-style API, idiomatic Rust
--------------------------------

Raylib supplies the design style: few concepts, plain data, and a small verb-noun vocabulary.
We adapt that style to Rust rather than copying C's ownership, sentinels, or globals.

* Public option structs have public fields and ``Default``; callers use struct-update syntax.
* The common case has one function call beside a staged API for buffer reuse and streaming.
* Names use ``load_*``, ``begin_*`` and ``end_*`` where a real staged operation needs them, ``is_*``, and ``*_count``; accessors avoid ``get_``.
* ``Drop`` releases resources instead of paired ``unload_*`` functions or manual cleanup.
* An owned result is easy to retain; borrowed views state their lifetime and cannot outlive or alias an invalidating context mutation.
* ``Result`` and ``Option`` replace error sentinels and null pointers; enums use exhaustive matching.
* Registries are immutable, mutable workspaces belong to a worker, and there is no hidden global state.
* Reusable libraries do not depend on the CLI, print diagnostics, or choose terminal presentation.

An ``end_*`` operation may finalize a fallible calculation, but ``Drop`` still cleans up if it is skipped.
We add concepts only when a concrete use requires them.
CMD 0002 proposes the initial measurement API; later implementation records must demonstrate its ownership and error behavior.

Elm-style diagnostics
---------------------

Libraries return structured errors implementing ``std::error::Error`` and ``Display``.
An error carries a stable namespaced code, severity, title, message, consequence, a location when known, and repair directions.
Terminal and JSON renderings use the same facts.
A terminal report contains a title rule, a real location, a bounded excerpt with a caret when source exists, a plain explanation, and an actionable ``Hint:``.

.. code-block:: text

   -- UNKNOWN ANALYSIS MODE -------------------------------- ccm.toml:2:12

   2 | profile = "semnatic"
                  ^^^^^^^^

   The analysis mode "semnatic" is not recognized.
   No source files were analyzed.

   Hint: Set profile = "semantic". Did we mean "semantic"?

We do not invent a source location when an operation has no source span.
We use the command, path, or resource identity as context instead.
A repair direction identifies a concrete edit, argv vector, or API field and value.
JSON retains codes, spans, consequences, and directions without terminal decoration.
We respect ``NO_COLOR`` and explicit color choices, escape terminal control sequences, and redact secrets.
Reports are bounded; exhaustion preserves the first failure and states that further reports were truncated.

Required implementation checks
-------------------------------

Before implementation work is complete, ``cargo fmt --check``, ``cargo build``, ``cargo test``, ``cargo clippy --all-targets``, and the repository limit checker pass.
Workspace changes use the corresponding workspace-wide checks.
We test behavioral boundaries and failure paths rather than mirroring implementation details.
These are obligations for the future workspace; no Cargo workspace or limit checker exists yet.
Documentation builds cannot substitute for these checks.

Operations, compatibility, and recovery
----------------------------------------

We use ``jj`` in this repository, which has a Git backend.
We preserve existing work and keep related changes small.
We do not add vendor attribution to commits or pull requests.
Agent instruction files remain ignored and outside versioned deliverables.

We run the discussion commands from ``docs/cmd``; :doc:`../workflow` gives examples.
GuideDog locks mutations, replaces whole files, and journals updates involving several files.
``--dry-run`` previews a mutation.
After interruption, ``guidedog dsc recover --rollback`` refuses to overwrite a later edit.
We review recovery changes before continuing.

New measurement definitions receive a versioned identity.
Changes to normalization, language extraction, evidence rules, weights, or file selection can break comparability even when the CLI syntax stays the same.
We preserve old definitions or provide explicit migration notes.
We treat source, paths, manifests, caches, and imported evidence as untrusted input.
Default measurement does not execute repository code or contact a remote service.

Acceptance criteria and implementation status
---------------------------------------------

This process is ready for adoption when maintainers review its CCM scope, evidence rules, and lifecycle against ``discussion.toml`` and the catalog builds cleanly.
GuideDog supplies the catalog machinery; CCM has no implementation in this repository yet.
No algorithm, predictive improvement, or speed advantage is claimed as shipped behavior.

Review history
--------------

2026-10-23: We adapted the initialized process to CCM and prepared CMD 0002 for mathematical and algorithmic review.
We adopted JLD 0001's engineering standards and made the raylib-inspired, idiomatic Rust API contract explicit at the maintainer's request.
Both records remain proposals pending maintainer review.

References
----------

* :doc:`../workflow`, the local GuideDog commands.
* GuideDog's local ``crates/gd-discussion`` implementation and ``docs/gds/records/0001-gds-process.rst`` in the sibling ``../guidedog`` repository.
* JLD 0001, ``docs/jld/records/0001-jld-process.rst`` in the sibling ``../jlavi`` repository, especially its Rust coding, diagnostic, and validation rules.
* `raylib API reference <https://www.raylib.com/cheatsheet/cheatsheet.html>`_, the source of the small-function API inspiration.
* `scc <https://github.com/boyter/scc>`_, a comparison baseline whose exact revision we must pin for experiments.

.. rubric:: Lifecycle event

::

   2026-10-23: prediscussion -> discussion. Prepare the CCM-specific process for
   review
