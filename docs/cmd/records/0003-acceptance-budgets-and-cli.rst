:cmd-number: 0003
:cmd-title: Acceptance budgets and the first working CLI
:cmd-state: committed
:cmd-type: Standards Track
:cmd-authors: ["Vikrant Rathore"]
:cmd-created: 2026-10-23
:cmd-updated: 2026-10-23
:cmd-discussion: Local implementation review at the maintainer's request
:cmd-labels: ["architecture", "economics", "cli"]

Acceptance budgets and the first working CLI
============================================

Abstract
--------

We adopt a bounded first implementation of the count and finite-model kernels in CMD 0002.
We provide terminal, technical, JSON, and standalone HTML reports.
We add an acceptance budget that prices modeled activities with explicit assumptions.
We do not use COCOMO, a source-size effort regression, or an assumed AI productivity multiplier.
The dollar figure is a planning scenario, not an empirical prediction or a valuation of software.
We adopt this implementation boundary at the maintainer's explicit request to proceed while the wider research program remains open.

Review of CMD 0002
------------------

The reachability, sparse trigger, and evidence-transport constructions have finite executable definitions.
They can support a useful first engine without settling every research candidate.
A dependency closure is exact within an imported graph, but it does not prove that the graph covers a language's semantics.
An accepted evidence declaration is an input to the policy model, not a signature or an executed proof checker.
A user-friendly report must retain these boundaries.

We therefore implement physical counts, exact byte comparison of selected files, finite propagation witnesses, triggered checks, shared and joint check terms, and transported residual checks.
We defer tree-sitter extraction, resolved dependencies, behavioral ambiguity, certificate-cut search, compression, topology, separator inference, task-ledger imports, caches, parallel scanning, and performance claims.
A mature parser runtime may later justify a dependency, but the finite kernel and first counter require none.
We leave CMD 0002 in discussion because its broader mathematical, extraction, empirical, and benchmark gates remain open.

Economics and observable demand
-------------------------------

Economic cost concerns resources consumed by activities under a production process.
Software also has cheap replication and potentially costly verification; size alone does not identify either development cost or economic value [digital]_.
Time-driven activity-based costing prices resource capacity and explicit activity times [tdabc]_.
Recent task-chain models of human and AI production emphasize that assignments and interdependent steps matter [chains]_.
We apply these ideas to the change and acceptance activities defined by CCC.
We do not claim a new general economic theory or calibrated productivity law.

For a declared change, let :math:`n=|D(S)|` be affected artifacts and :math:`u=U(S,F_S)` be residual check policy units after evidence transport.
Without a task model, :math:`n` is the number of changed selected files, or all selected files when no comparison is requested.
In that case :math:`u` is unknown, not zero.
We import or explicitly assume the following parameters:

.. list-table:: Scenario inputs
   :header-rows: 1

   * - Input
     - Meaning and unit
     - Demonstration default
   * - :math:`t_r`
     - Review hours per affected artifact.
     - 0.25 hours
   * - :math:`t_q`
     - Verification hours per residual policy unit.
     - 1 hour
   * - :math:`h_i`
     - Implementation hours per attempt, supplied directly.
     - 0 hours
   * - :math:`p_h`
     - Loaded human resource cost per hour, in USD.
     - 100 USD/hour
   * - :math:`a`
     - Agent and tool spending per attempt, in USD.
     - 0 USD
   * - :math:`r`
     - Stationary probability that an attempt requires another attempt.
     - 0

These defaults are illustrative assumptions, not measured population averages.
We display them in every report and allow them to be changed independently.
Policy weights acquire an assumed time price only through :math:`t_q`; the calculus itself does not assign minutes to policy units.
Different artifact granularity or policy weights requires a new calibration.
A zero implementation budget means no implementation activity was budgeted, not that implementation is free.

The activity time and expected resource cost are

.. math::

   h=n t_r+u t_q+h_i,\qquad
   c=p_h h+a,\qquad
   \widehat C=\frac{c}{1-r}.

Without a task model we omit the unknown :math:`u t_q` term and mark the scenario partial.
We never describe that partial number as a total project development estimate.
A user can supply implementation hours from a local planning process without pretending that source alone identified them.
All reports distinguish assumed cost, measured source quantities, and unavailable semantic quantities.

**Expected-cost proposition.** Suppose every attempt consumes the same budget :math:`c`, independently repeats with probability :math:`r<1`, and eventually terminates on a nonrepeat outcome.
Then expected total cost is :math:`c/(1-r)`.

**Proof.** The probability of requiring attempt :math:`k\ge1` is :math:`r^{k-1}`.
Linearity of expectation gives :math:`c\sum_{k\ge1}r^{k-1}=c/(1-r)`.
The result also follows from :math:`C=c+rC`.
The stationary repeat assumption is part of the scenario, not an inference from source, unresolved checks, or an agent's confidence.
Real retries can change work demand and evidence; a future ledger model must account for those changes rather than reusing this assumption silently.
We cap the CLI's repeat probability at 0.95 to avoid near-singular scenarios.

**Evidence-value proposition.** Hold the task, review demand, prices, repeat assumption, and certificate validity fixed.
The scenario's avoided verification spending from adding accepted evidence is :math:`p_h t_q (W-U)/(1-r)`.
It is monotone and submodular by CMD 0002's fixed-validity evidence law and multiplication by a nonnegative constant.
This is a resource-cost saving under stated prices, not software asset value or expected avoided business loss.
A change can invalidate that evidence, and the fixed-validity conclusion then no longer applies.
The first CLI reports residual demand and its cost; it does not optimize evidence procurement or assign a market value to proofs.

For the checkout fixture, changing pricing and inventory reaches three artifacts.
Five checks activate, totaling seven policy units.
An unrelated payment review survives and discharges one unit; two older evidence routes need rechecking.
The pending six units and default review assumption give :math:`h=3(0.25)+6(1)=6.75` hours and an illustrative 675 USD budget.
The shared compatibility check contributes minus one policy unit; the combined integration check contributes plus three.
We retain both terms and report their net plus two rather than claiming independence.

CLI, library, and report contract
---------------------------------

We run ``ccm [PATH ...]`` for an inventory.
``--compare OLD CURRENT`` compares selected source files by bytes, including comment-only edits and added or deleted files.
``--model FILE`` imports a versioned explicit finite model.
Repeated ``--changed NAME`` options select seeds; without them, comparison paths supply seeds and must all exist in the model.
Without either, the model is evaluated for standing checks only.
With at least two seeds we compare the first artifact in model order against the remaining seeds, retaining shared and joint checks.
This is a specified batch split, not an exhaustive interaction expansion.

``--technical`` adds paths, per-file details, and policy units.
``--json`` emits schema ``ccm.report.v1`` without color or progress.
Unavailable coordinates are JSON nulls; unverified evidence is not accepted evidence.
JSON is an output format, not a finite-model input format in this release.
``--html`` emits a self-contained, responsive, printable HTML report with no scripts, network assets, or source snippets.
``--output FILE`` creates a new file and refuses to overwrite existing content.
Default terminal color requires a color-capable terminal and respects ``NO_COLOR``; ``--color`` overrides it explicitly.
Text remains readable without color or Unicode.

The crate boundary is core mathematics, lexical scanning, filesystem selection, and CLI presentation.
Reusable libraries own their returned reports, borrow their inputs, never print, and return structured errors.
All dependencies are workspace-local.
We use standard-library collections, I/O, and argument handling.
The detailed language policy, finite input mapping, examples, and schema guarantees are in ``docs/counting.rst``, ``docs/model.rst``, and ``docs/reports.rst``.

Bounds, security, and compatibility
-----------------------------------

Defaults cap selected files at 100,000, each file at 16 MiB, each snapshot at 1 GiB, traversal at depth 128, and visited entries at ten times the selected-file budget.
Model input is UTF-8, capped at 4 MiB and 100,000 physical record lines.
Lexical comment nesting is capped at 256.
Limit and traversal failures stop with a diagnostic; supported files with lexical coverage failures produce an incomplete report and exit status 2.
Normal completion returns 0, including checks that still need evidence.
Other errors return 1.
This version does not silently skip unsupported constructs inside a counted file when it detects them.
It discloses unsupported file extensions separately.

We do not execute source, checkers, Git, shells, or external tools during measurement.
Symlinks are skipped and build/vendor exclusions are explicit.
We do not implement gitignore semantics yet.
Input length and modification-time checks detect ordinary concurrent writes, including changes between counting and comparison.
They are not content identities, an atomic filesystem snapshot, or protection against adversarial races.
We use immutable checkouts for reproducible comparisons.
Model context and evidence binding are author declarations; semantic authentication remains unavailable.
Terminal control characters and HTML markup are escaped; JSON strings are encoded separately.
We retain selected paths and reports, so memory includes file records, the finite model, two bounded comparison buffers, and rendered output.
This is a sequential reference implementation, not the streaming or parallel target of CMD 0002.

This is the first runtime API and report schema.
The count, finite-input, output, and economic policies have separate version identities.
Changing the economic assumptions does not change source counts or model reachability.
No prior runtime compatibility or predecessor counting equivalence is claimed.

Acceptance and evidence
-----------------------

We require formatting, workspace build, workspace tests, Clippy for all targets, and the repository limit checker.
Behavioral tests cover disjoint line classes, nested comments and strings, empty input, byte comparison, evidence invalidation and survival, joint activation, economic reconciliation, resource failures, deterministic output, symlinks, and escaping.
We also rerun the independent finite research harness from CMD 0002.
We publish no speed comparison or predictive accuracy claim.
Research and extraction gates in CMD 0002 remain open.

Review history
--------------

2026-10-23: The maintainer requested a working CLI, HTML and machine reports, conservative dependencies, and a dollar estimate grounded in CCC and economic theory rather than COCOMO.
We adopted this narrow executable boundary and explicit activity-pricing scenario in response.
Implementation evidence is recorded in the lifecycle entry after checks pass.

References
----------

* :doc:`0002-change-certificate-calculus` supplies the definitions of closure, check activation, evidence transport, and residual work.
* :doc:`0001-cmd-process` supplies implementation and lifecycle rules.

.. [tdabc] Robert S. Kaplan and Steven R. Anderson. `Time-Driven Activity-Based Costing <https://www.hbs.edu/faculty/Pages/item.aspx?num=15805>`_, Harvard Business School working paper, 2003.
.. [digital] Avi Goldfarb and Catherine Tucker. `Digital Economics <https://doi.org/10.1257/jel.20171452>`_, Journal of Economic Literature, 2019.
.. [chains] Mert Demirer, John J. Horton, Nicole Immorlica, Brendan Lucier, and Peyman Shahidi. `Chaining Tasks, Redefining Work: A Theory of AI Automation <https://www.nber.org/papers/w34859>`_, NBER Working Paper 34859, 2026. This is a research reference, not validation of CCM's scenario parameters.

.. rubric:: Lifecycle event

::

   2026-10-23: prediscussion -> discussion. Assigned a permanent number and
   opened for discussion.

.. rubric:: Lifecycle event

::

   2026-10-23: discussion -> accepted. Implement the bounded count and
   finite-model kernels with explicit activity-based acceptance budgets at the
   maintainer's request; retain broader research gates in CMD 0002.

.. rubric:: Lifecycle event

::

   2026-10-23: accepted -> committed. Implemented the bounded CLI and
   acceptance-activity economic scenario with no external crate dependencies.
   cargo fmt --all --check; cargo build --workspace; cargo build --release;
   cargo test --workspace (15 tests, including exhaustive closure over 4096
   graph/seed cases); cargo clippy --workspace --all-targets -- -D warnings;
   cargo xtask check --soft-width; standalone CCC finite research harness;
   independent JSON parsing; Chromium HTML inspection; guidedog dsc check and
   HTML build. All passed.
