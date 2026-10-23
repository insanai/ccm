:cmd-number: 0006
:cmd-title: Broad language policies and compact source reports
:cmd-state: committed
:cmd-type: Standards Track
:cmd-authors: ["Vikrant Rathore"]
:cmd-created: 2026-10-23
:cmd-updated: 2026-10-23
:cmd-discussion: No external discussion URL assigned
:cmd-labels: ["cli","languages","performance"]

Broad language policies and compact source reports
==================================================

Abstract
--------

We restore a conventional Code column, add physical Lines, and group large numbers.
We replace the verbose default estimate with a short, reconciled cost breakdown and an assumptions reference.
We fix valid Typst documents that were rejected as unterminated source strings.
We add mixed component counting and a broad, versioned declarative language catalog.
This record extends CMD 0004 and CMD 0005 without changing the finite calculus or the 2026 economic coefficients.

Language policies
------------------

We retain 64 native policies and add a bounded declarative scanner for the wider syntax catalog.
The registry exposes 658 names, formats, and aliases.
The compatibility manifest records 374 scc names, 433 CLOC names, and 97 SLOC extensions.
We check catalog presence and path recognition in the CLI integration suite.
Catalog coverage is not a claim of full grammar coverage or count equivalence with every named tool.
``docs/language-catalog.rst`` defines native and declarative tiers, precedence, deliberate omissions, source provenance, and license notices.

The independent engine uses literal line-comment, block-comment, and quote delimiters.
We attribute bundled MIT syntax metadata separately from CCM's implementation.
We use CLOC's catalog names and extension facts without importing its executable Perl filters.
The runtime does not invoke external counters or fetch definitions.
We retain zero external crate dependencies.

Native Typst counting separates prose, code strings, nested comments, and raw spans/fences.
Sample source inside raw text does not start a Typst source string or comment.
Vue and Svelte use a shared physical-line map for markup, JavaScript/TypeScript, and CSS/SCSS regions.
A physical line shared by several regions is counted once.
Unsupported declared embedded languages remain explicit coverage failures.
Jinja comments, SVG, DOT, CSV, TypeScript declarations, and lockfiles have explicit categories or policies.
Dependency lockfiles are data and have separate report rows.
Known binary asset extensions are normal skips.

Report behavior
----------------

The wide terminal table uses Language, Files, Lines, Blank, Comments, and Code.
The narrow table retains Language, Files, Lines, and Code.
Numbers use thousands separators, and colors distinguish code, comments, and subdued structure.
The report uses bounded rendering and preserves plain output for pipes and NO_COLOR.

Default and verbose reports show a compact expected-cost breakdown.
Every displayed component includes the same expected-attempt multiplier as the total.
We omit zero optional activities and do not represent unmodeled acceptance demand as a known zero cost.
Technical output, machine reports, and HTML details retain the full assumptions.
Default output refers to the 2026 economics documentation.
HTML uses the same cost components and hides an unused acceptance-plan section.
The count-policy version becomes lexical-v3; per-file JSON also identifies the language-policy tier.
The language catalog JSON schema becomes ccm.languages.v2.
The report JSON schema remains ccm.report.v3.

Performance changes and invariants
----------------------------------

We avoid collecting unused lexical work tokens in ordinary reports.
We resolve each ignore-rule scope once per candidate, rather than once per rule.
Literal glob prefixes and suffixes reject impossible matches early.
Single-star literal patterns have a direct bounded matcher; other patterns retain the full state machine.
We test those optimizations against the full matcher over generated path strings.
We discover available ignore filenames during directory enumeration instead of performing negative metadata lookups for every absent ignore file.
We preserve ignore precedence, negation, entry limits, nesting bounds, work budgets, and input mutation checks.
Generated-header checks normalize bounded input once instead of repeatedly folding case for each candidate substring.

The benchmark driver accepts a named corpus and requires equal file, byte, and physical-line totals before timing.
We retain per-language classification differences rather than adjusting counts to make another counter's output match.
The Ensodoc snapshot is described in ``docs/performance.rst`` with a file manifest and raw measurements.
Measurements apply to the named release binary and workload.
They do not establish that CCM is faster for every language or repository.
Debug builds are development artifacts and are not used for release-performance claims.

Compatibility and validation
-----------------------------

The CLI estimate changes presentation, not its resource-price mathematics.
JSON retains zero numeric values and unknown scope explicitly for agents.
Catalog and native-policy changes can change recognized files and line classifications.
We validate the reported Ensodoc failures, mixed-line boundaries, all recorded catalog names and extensions, deterministic partitions, report layouts, and ignore semantics.
We require builds, tests, Clippy, formatting, source limits, documentation checks, and a release report before marking this record implemented.

References
----------

* `Language policies and catalog coverage (source) <../_static/references/language-catalog.rst>`_
* `Measured performance (source) <../_static/references/performance.rst>`_
* `2026 economic profile (source) <../_static/references/economics.rst>`_

.. rubric:: Lifecycle event

::

   2026-10-23: prediscussion -> discussion. Fix document counting, broaden
   catalog coverage, and organize CLI reports

.. rubric:: Lifecycle event

::

   2026-10-23: discussion -> accepted. Keep familiar code metrics, short
   estimates, explicit policy tiers, and measured performance

.. rubric:: Lifecycle event

::

   2026-10-23: accepted -> committed. Implement the compact report, native
   mixed-document fixes, broad catalog, and measured counting improvements in
   CCM 0.4.0 50 tests pass; workspace and release builds, Clippy with warnings
   denied, formatting, source limits, catalog coverage, Ensodoc exit 0,
   HTML/JSON reports, and equal-input release benchmarks pass
