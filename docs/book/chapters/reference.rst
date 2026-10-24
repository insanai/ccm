Contracts and a reading desk
=============================

The earlier chapters explain the design in sequence.
This chapter gathers the contracts we most often need while using or extending the program.
The executable's ``--help`` remains the authority for the complete current option list.
The source records retain the rationale and unresolved questions behind an interface.

The supported surfaces
-----------------------

.. list-table:: Runtime interfaces
   :header-rows: 1
   :widths: 31 69

   * - Surface
     - Contract in this edition
   * - Human reports
     - Terminal, plain, standalone HTML, verbose detail, and technical detail.
   * - Agent and table reports
     - ccm.report.v3 JSON, file CSV, and record-oriented NDJSON.
   * - Language registry
     - ccm.languages.v2 JSON with native-lexical-v5 and declarative-v1 policy tiers.
   * - Task model
     - Preferred tab-separated ccm-model version 2; version 1 keeps its legacy failed-evidence interpretation.
   * - Cost profile
     - Bounded UTF-8 flat key=value entries, applied in argument order.
   * - Development assets
     - Rust SVG generator, custom Typst cover, Guidedog publication, and actual-report screenshots.

The report identifies lexical-v5 as its aggregate count policy.
A file also identifies its own language-policy tier.
The current finite model is author-declared; ``semantic_guarantee`` remains unavailable.
Machine consumers should preserve null and unknown values rather than replace them with zeros.

Useful invocations
-------------------

.. code-block:: sh

   ccm .
   ccm --no-cost .
   ccm --color=always .
   ccm --by-file --sort code .
   ccm --technical --json .
   ccm --format csv --include-ext rs,py .
   ccm --no-generated --exclude vendor .
   ccm --cost-profile examples/ai-costs-2026.conf --verbose .

The ``--model`` and ``--changed`` options connect a declared task to explicit seeds.
A comparison can supply seed paths when names match the model.
``--fail-on-pending`` makes open active checks or inconsistent modeled observations an exit-status condition.
It does not execute those checks.

.. list-table:: Exit status
   :header-rows: 1
   :widths: 15 85

   * - Status
     - Meaning
   * - 0
     - Selected supported text completed; pending checks remain report data unless the explicit failure mode is enabled.
   * - 1
     - Arguments, I/O, model validation, resource admission, or output failed.
   * - 2
     - A report was produced with a detected lexical coverage failure.
   * - 3
     - Explicit fail-on-pending mode found an open active check or inconsistent modeled observations.

Limits and evidence
--------------------

The task file is bounded to 4 MiB and 100,000 physical record lines.
The finite evaluator admits at most 100,000 artifacts, checks, certificates, or seeds of the corresponding kind, with one million edges and one million relevant memberships.
The behavior model admits 100,000 cases and filters and bounds case-filter work to twenty million steps.
Physical selection and output limits have their own options.
The ordinary renderer budget is 256 MiB.

These are admission contracts, not targets every ordinary run must approach.
A library error carries a category, plain explanation, hint, and optional source location.
Libraries do not print it themselves.
The application renders a diagnostic appropriate to the output surface and preserves failure status for automation.

A small glossary
-----------------

An artifact is a modeled part, not necessarily a source file.
A seed is a part declared changed in the current task.
The affected closure contains every part reached through declared invalidation edges.
An obligation is a check activated by positive trigger terms.
A policy unit is its declared nonnegative weight.
Evidence support names the assumptions on which an admitted record depends.
A surviving record has support disjoint from the affected closure under the finite rule.
Pending demand is the weighted sum of active checks left open by the conclusion policy.
A behavioral hypothesis is one enumerated candidate, while an observation label identifies the task distinction that candidate exposes.

Research and source references
-------------------------------

The founding design is CMD 0002, a change and certificate calculus for code measurement.
CMD 0003 adopts the finite runtime and activity budgets.
CMD 0004 develops competitive counting and evidence-conditioned reports.
CMD 0005 records the 2026 AI development profile, and CMD 0006 records the broad catalog and compact report.
CMD 0007 records the Guidedog publication pipeline and shared visual identity.
Their source files live under ``docs/cmd/records`` and their web catalog is mounted beside the book.
The founding research record remains a discussion; implemented follow-up records do not automatically close its empirical gates.

The 2026 spending source is `Span's Beyond the Model study <https://www.span.app/research/agent-effectiveness-july2026>`_.
The language source is `The Best Programming Language for Tokenmaxxing <https://arxiv.org/html/2607.22807v1>`_ by Wu, Anderson, and Guha.
The price reference is the `official provider pricing page <https://developers.openai.com/api/docs/pricing>`_, with the profile frozen on 2026-10-23.
The chapter on the research profile explains the transfer assumptions before those sources are used for estimation.

Language-catalog provenance and metadata hashes are retained in ``docs/benchmarks/language-catalog-2026.json``.
The `CLOC catalog <https://github.com/AlDanial/cloc/blob/master/Unix/cloc>`_, `scc syntax metadata <https://github.com/boyter/scc/blob/master/languages.json>`_, and `SLOC definitions <https://github.com/flosse/sloc/blob/master/src/sloc.coffee>`_ have different roles and fidelity limits in CCM.
The bundled MIT notices are available through ``ccm --licenses``.
The counting algorithm and finite economic model are separate implementations.

The cover uses the `CeTZ drawing package <https://typst.app/universe/package/cetz/>`_ and `Fletcher <https://typst.app/universe/package/fletcher/>`_ with explicit pinned versions.
`Typst <https://typst.app/docs/>`_ typesets the PDF, while Guidedog builds the web and book editions from the same prose.
All of these publication sources remain inspectable in the repository.
