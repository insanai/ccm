:cmd-number: 0007
:cmd-title: Guidedog publication and CCM visual identity
:cmd-state: committed
:cmd-type: Standards Track
:cmd-authors: ["Vikrant Rathore"]
:cmd-created: 2026-10-23
:cmd-updated: 2026-10-23
:cmd-discussion: No external discussion URL assigned
:cmd-labels: ["documentation","design"]

Guidedog publication and CCM visual identity
============================================

Abstract
--------

We publish one prose book in HTML and PDF beside a CCM landing page and the design catalog.
The installed Guidedog CLI owns all four publication mounts.
We give the book and reports a shared pink, purple, and orange identity, a projected 3D SVG mark, and a monochrome illustrated cover.
We preserve the distinction between exact finite calculations and unvalidated economic scenarios.

Motivation and scope
--------------------

The runtime needs an explanation that connects its selection rules, lexical policies, change model, evidence, and resource prices.
A collection of option descriptions cannot carry that argument.
We follow one checked-in checkout example through the algorithms and calculations, with actual terminal and HTML screenshots.
The twenty chapters also distinguish implemented behavior from the larger research program in CMD 0002.

Specification
-------------

``docs/conf.toml`` mounts the landing page, HTML book, PDF book, and design catalog.
The ordinary command is ``guidedog ssg build docs --out docs/_build/site``.
``guidedog build pdf docs/book`` uses Guidedog's built-in compiler support.
We do not configure an external Typst executable for PDF publishing.

The separate Typst CLI exports SVG artwork and PNG previews.
``cargo xtask assets`` checks that executable before regenerating the Rust SVG assets and the CeTZ/Fletcher cover.
A missing or nonworking executable produces an Elm-style diagnostic with an installation URL and verification command.
Prepared publication inputs remain usable by Guidedog without this separate CLI.

``cargo xtask docs`` additionally builds CCM, captures executable reports with installed Chromium, and invokes installed Guidedog.
The task probes all required executables before changing generated inputs.
``--no-pdf`` selects Guidedog's HTML preview mode, and ``--guidedog PATH`` selects a particular installed publisher.
The publication does not fetch remote images or mathematical JavaScript at reading time.
The bundled MathJax files retain their license and provenance.

``tools/ccm-art`` generates the canonical logo and explanatory SVG diagrams using the Rust standard library.
The application embeds the colored mark in standalone HTML reports.
Terminal output uses the same palette within its existing color and NO_COLOR rules.
The report schema and finite calculations remain unchanged.

Rationale and alternatives
--------------------------

Guidedog keeps references, source expansion, mounting, HTML, and PDF assembly in one publishing engine.
The custom Typst template controls the cover and print typography while the prose stays in reStructuredText.
The artwork uses pinned CeTZ 0.3.4 and Fletcher 0.5.8 packages with compatible drawing APIs.
The source-generated logo and diagrams stay editable, reproducible vector assets without adding Rust crate dependencies.
Screenshots come from actual output rather than hand-authored approximations.

Security and operational considerations
---------------------------------------

Repository tools invoke known executables through argument arrays without a shell.
CCM measurement itself invokes none of the publishing tools.
Generated build directories are ignored, and ordinary publication writes only local files.
The dependency and evidence diagrams describe an author-declared model, not automatically discovered semantic guarantees.
The cost profile remains an editable planning scenario rather than a validated prediction or an invoice.

Backwards compatibility
-----------------------

The release becomes 0.5.0 across the Rust workspace and publication editions.
Existing command options, machine schemas, count policy, and economic coefficients remain compatible.
HTML report styling and terminal accent colors change.
The frozen 0.4 performance experiment retains its measured version and binary hash; we do not relabel it as a 0.5 measurement.

Acceptance criteria and implementation status
---------------------------------------------

We require workspace builds, tests, Clippy, formatting, and source-limit checks.
The executable book fixture must reproduce twelve code lines, seven required check units, six pending units, three affected parts, and the 675.8582357142857 USD scenario before display rounding.
The missing-Typst path must produce its installation hint.
Guidedog must build the full publication and its HTML-only preview, and strict book HTML/PDF builds must complete.
We inspect the rendered cover, mathematical pages, report captures, and desktop and mobile landing layouts.

Open questions
--------------

The book describes additional parser adapters, empirical calibration, and richer semantic models as future work.
This publication does not close those research gates.

Review history
--------------

On 2026-10-23 we clarified the compiler boundary: Guidedog compiles the PDF, while the standalone Typst CLI generates SVG and PNG assets.

References
----------

* ``docs/publication.rst`` describes the reproduction commands and tool responsibilities.
* ``docs/book`` contains the book sources and custom cover template.
* ``examples/book`` contains the executable worked example.
* CMD 0002 states the founding research program; CMD 0003 through CMD 0006 record implemented runtime behavior.

.. rubric:: Lifecycle event

::

   2026-10-23: prediscussion -> discussion. Assigned a permanent number and
   opened for discussion.

.. rubric:: Lifecycle event

::

   2026-10-23: discussion -> accepted. Adopt the requested Guidedog publishing
   pipeline, book, reproducible artwork, and shared report identity.

.. rubric:: Lifecycle event

::

   2026-10-23: accepted -> committed. Implement the book, artwork, report
   identity, and Guidedog publication pipeline in CCM 0.5.0. 52 tests pass;
   workspace and release builds, Clippy, formatting, source limits, strict
   HTML/PDF builds, both site modes, missing-Typst diagnostics, visual
   inspection, and 2510 local reference checks pass.
