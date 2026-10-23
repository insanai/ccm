Code Counter Machine
====================

CCM measures source, explains a declared change's acceptance work, and produces readable reports for developers, planning teams, and agents.
CCM 0.5 uses Rust and has no external crate dependencies.
We keep physical counts, modeled impact, evidence, and economic assumptions separate.

.. code-block:: sh

   cargo build --release
   ./target/release/ccm .
   ./target/release/ccm --color=always .
   ./target/release/ccm --technical crates
   ./target/release/ccm --format=csv --include-ext=rs,py .
   ./target/release/ccm --json . > inventory.json
   ./target/release/ccm --html -o inventory.html .
   ./target/release/ccm --compare ../before . --technical

We can install the CLI with ``cargo install --path cmd/ccm``.
Rust 1.85 or later is required.
``ccm --help`` lists selection, resource, report, and budget options.
We use ``target/release/ccm`` or the installed ``ccm`` for speed comparisons; Cargo debug builds retain development overhead.

The CCM book and website
-------------------------

``docs/book`` contains *CCM: Code, Change, and Confidence*, a prose account of the algorithms, calculations, estimation, research, and system design.
The twenty chapters follow one executable checkout example and include mathematical derivations, architecture and sequence diagrams, illustrations, and actual report screenshots.
The PDF cover draws an isometric lattice of modules in which one change spreads through its bounded closure, made with CeTZ.

The installed Guidedog CLI drives publication, and Guidedog generates the PDF with its own Typst support.
The source project assembles a landing page, an HTML book, a PDF book, and the design catalog.

.. code-block:: sh

   guidedog ssg build docs --out docs/_build/site
   guidedog build pdf docs/book
   cargo xtask serve

The site starts at ``docs/_build/site/index.html`` and the assembled PDF is ``docs/_build/site/books/ccm-book-en.pdf``.
``cargo xtask serve`` previews the assembled site at ``http://127.0.0.1:8000/`` and rebuilds it when a source changes.
``guidedog serve`` previews one project, such as ``docs/book``; it cannot resolve the home page's links to the other mounts.
The separate Typst CLI is needed only when regenerating SVG and PNG artwork.
``cargo xtask assets`` checks for it before generating files and gives an explained installation hint if it is missing.

After editing the logo or screenshots, we can refresh the publication inputs and run the same Guidedog build with ``cargo xtask docs``.
The Rust artwork tool under ``tools/ccm-art`` writes the reusable 3D SVG identity without external Rust crates.
``docs/publication.rst`` records the toolchain, asset sources, and reproduction commands.

A change and its acceptance budget
----------------------------------

.. code-block:: sh

   ccm --model examples/checkout.ccm --changed pricing --changed inventory --technical
   ccm --model examples/checkout.ccm --changed pricing --changed inventory \
       --hourly-cost 125 --implementation-hours 8 --agent-cost 10 \
       --rework-probability 0.2 --html -o checkout.html

The example reaches three artifacts and activates five checks.
Six policy units still need evidence after older evidence is invalidated.
The example has a 675 USD human review and check component at the default assumed activity prices.
We can inspect every input and change each price independently.
This is activity-based resource costing, not COCOMO, measured effort, a quote, or software value.
The default report also estimates AI implementation, remaining declared checks, and hosted tools using an editable 2026 research profile.
We label the token conversion and workload assumptions as inferred and unverified.
Without an explicit model, project-specific acceptance demand stays unknown while the source-based AI rebuild scenario remains available.
The normal report shows a short cost breakdown and a link to the assumptions.
``ccm --verbose`` adds a compact price summary and coverage details; ``--technical`` exposes the full calculation.
``--cost-profile examples/ai-costs-2026.conf`` supplies editable assumptions.
``--agent-cost`` replaces automatic AI/tool spending; ``--implementation-hours`` adds explicit human implementation work.

Current scope
--------------

We provide 64 native policies and a wider declarative registry covering the recorded CLOC and scc names and SLOC extensions.
The registry contains 658 names, formats, and aliases; their syntax fidelity varies.
We describe the tiers, provenance, ambiguities, and deliberate limits in ``docs/language-catalog.rst``.
Unknown file types are disclosed and excluded.
Detected unsupported constructs produce coverage notices; counts are never semantic proofs.
We skip symlinks and common build/vendor directories.
We apply scoped .gitignore, .ignore, and .ccmignore rules, with explicit extension and generated-file filters.
We summarize normal symlink and binary skips without warnings.

Change analysis uses an explicit finite dependency and check model.
We compute dependency paths, shared and joint checks, surviving evidence, and remaining verification demand.
Explicit finite cases add evidence-conditioned outcome counts and distinguish inconsistent evidence from zero ambiguity.
Admissible failing evidence leaves checks open; it does not count as successful verification.
Tree-sitter adapters, automatic task extraction, authenticated certificates, imported work ledgers, and empirical labor calibration remain future work.
We require immutable input checkouts for reproducible comparisons.

* `Language catalog and policy tiers <docs/language-catalog.rst>`_
* `2026 AI cost research and equations <docs/economics.rst>`_
* `Competitive CLI decision <docs/cmd/records/0004-competitive-counting-and-evidence-reports.rst>`_
* `Measured performance and raw samples <docs/performance.rst>`_
* `Count and selection policy <docs/counting.rst>`_
* `Finite task models <docs/model.rst>`_
* `Reports, budgets, and exit statuses <docs/reports.rst>`_
* `Implementation decision and economic equations <docs/cmd/records/0003-acceptance-budgets-and-cli.rst>`_
* `Change and certificate calculus <docs/cmd/records/0002-change-certificate-calculus.rst>`_

Development
------------

.. code-block:: sh

   cargo fmt --all --check
   cargo build --workspace
   cargo test --workspace
   cargo clippy --workspace --all-targets -- -D warnings
   cargo xtask check --soft-width
   cargo xtask bench 30

Reusable libraries live in ``crates/``; the application lives in ``cmd/ccm``.
``ccm-core`` contains I/O-free mathematics and structured errors.
``ccm-scan`` contains the lexical policies.
``ccm-project`` handles bounded filesystem selection and comparison.
``tools/xtask`` checks repository limits without external tools.

Automatic color works without TERM and honors a nonempty NO_COLOR.
``--color=always`` overrides automatic detection.
``--fail-on-pending`` enables an explicit acceptance gate for CI.

Authors and license
-------------------

Vikrant Rathore wrote CCM, with assistance from Ronak Rathore and Kanak Rathore.

CCM is free software: we can redistribute it and modify it under the terms of the GNU Lesser General Public License as published by the Free Software Foundation, either version 3 of the License, or (at our option) any later version.
The license text is in ``COPYING.LESSER``, and the GNU General Public License it extends is in ``COPYING``.
The SPDX identifier is ``LGPL-3.0-or-later``.
The bundled language metadata keeps its MIT notices in ``crates/ccm-scan/LICENSE-language-data``, shown by ``ccm --licenses``.
