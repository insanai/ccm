.. image:: assets/ccm-logo.svg
   :width: 112px
   :align: center
   :alt: CCM's cut purple cube, with a pink change at its centre.
   :target: https://insanai.github.io/ccm/

CCM
===

**Measure the code. Understand the change.**

`Download <https://github.com/insanai/ccm/releases/tag/v1.0.0-beta.1>`_ | `Website <https://insanai.github.io/ccm/>`_ | `The book <https://insanai.github.io/ccm/book/>`_ | `PDF <https://insanai.github.io/ccm/books/ccm-book-en.pdf>`_ | `Design <https://insanai.github.io/ccm/decisions/>`_

.. image:: https://img.shields.io/badge/release-1.0.0--beta.1-7341aa
   :alt: Release 1.0.0-beta.1
   :target: https://github.com/insanai/ccm/releases/tag/v1.0.0-beta.1

.. image:: https://github.com/insanai/ccm/actions/workflows/ci.yml/badge.svg
   :alt: Build and test status
   :target: https://github.com/insanai/ccm/actions/workflows/ci.yml

.. image:: https://img.shields.io/badge/license-LGPL--3.0--or--later-ee77ad
   :alt: LGPL 3.0 or later
   :target: COPYING.LESSER

CCM, the Code Counter Machine, is a fast source counter and a planning tool for software changes.
We can start with a familiar table of languages, files, lines, comments, and code.
We can then ask what a declared change affects, which checks remain necessary, and what AI, tool, and human work might cost.
The terminal, HTML report, and JSON record explain the same result at different levels of detail.

CCM is written in Rust and has **no external crate dependencies**.
It runs locally, does not execute the source it measures, and does not contact an agent service.

Features
--------

* **Readable reports:** aligned tables, grouped numbers, useful colors, language shares, and a compact development estimate.
* **Reports we can share:** standalone HTML with the CCM identity, plus JSON, NDJSON, and CSV for agents and other tools.
* **Broad counting support:** 64 native lexical policies and a registry of 658 language and format names and aliases, covering the recorded CLOC and scc names and SLOC extensions.
* **Mixed documents:** explicit handling for components, markup, code fences, strings, comments, and common embedded languages.
* **Controlled selection:** scoped ignore files, generated-file filters, extension overrides, bounded workers, and explicit resource limits.
* **Change explanations:** exact file-byte comparisons and an optional declared dependency model with paths to affected parts.
* **Reusable evidence:** checks, surviving results, invalidated results, and shared or additional work caused by combined changes.
* **Editable cost scenarios:** separate AI implementation, hosted tools, human review, and remaining acceptance work, with a frozen 2026 research profile.
* **Portable executables:** Linux, Windows, macOS, and FreeBSD, with licenses, source provenance, and checksums in each release.

The language registry includes aliases and formats; it is not a claim of 658 complete parsers.
`The counting policies <docs/language-catalog.rst>`_ explain the native and declarative tiers.

A real source tree
------------------

.. image:: docs/shared/_static/linux-terminal.png
   :width: 820px
   :alt: CCM's recorded Linux 6.12 report, with language shares, line counts, and the development scenario.

This is the same Linux 6.12 terminal capture shown on the website.
The recorded run selected 82,405 files and 27,828,995 program lines, reading 1.46 GB in about two seconds on the documented machine.
These are measurements of that snapshot and machine, not a speed guarantee for every repository.
The displayed dollar figure is a planning scenario, not the Linux project's development bill or value.
The `measurement record <docs/performance.rst>`_ gives the method and limits.

Download and run
----------------

On macOS and Linux we can use the `CCM Homebrew tap <https://github.com/insanai/homebrew-ccm>`_:

.. code-block:: sh

   brew tap insanai/ccm
   brew install insanai/ccm/ccm
   ccm .

The tap follows published releases, including betas, and selects the matching x86-64 or ARM64 archive with its SHA-256 checksum.
Its hourly synchronization workflow checks for new releases and can also be run on demand.

`Release 1.0.0-beta.1 <https://github.com/insanai/ccm/releases/tag/v1.0.0-beta.1>`_ contains these archives and the PDF book.

.. list-table:: Release executables
   :header-rows: 1
   :widths: 25 42 33

   * - Platform
     - Archive
     - Build target
   * - Linux x86-64
     - ``ccm-linux-amd64.tar.gz``
     - Static musl
   * - Linux ARM64
     - ``ccm-linux-arm64.tar.gz``
     - Static musl
   * - macOS Apple Silicon
     - ``ccm-macos-arm64.tar.gz``
     - macOS 12 or later
   * - macOS Intel
     - ``ccm-macos-amd64.tar.gz``
     - macOS 12 or later
   * - Windows x86-64
     - ``ccm-windows-amd64.zip``
     - Windows 10 or later; static MSVC runtime
   * - FreeBSD x86-64
     - ``ccm-freebsd-amd64.tar.gz``
     - FreeBSD 15.1

This is a beta release.
The macOS executables are unsigned.
Each archive contains the executable, license texts, third-party notices, and a build record that names the corresponding source commit.
``SHA256SUMS`` covers all six archives and the PDF book.

For Linux x86-64:

.. code-block:: sh

   curl -fLO https://github.com/insanai/ccm/releases/download/v1.0.0-beta.1/ccm-linux-amd64.tar.gz
   curl -fLO https://github.com/insanai/ccm/releases/download/v1.0.0-beta.1/SHA256SUMS
   sha256sum --ignore-missing -c SHA256SUMS
   tar -xzf ccm-linux-amd64.tar.gz
   ./ccm-linux-amd64/ccm .

On macOS or FreeBSD, we extract the matching ``tar.gz`` archive and run its ``ccm`` executable.
On Windows, we extract the ZIP and run ``.\ccm-windows-amd64\ccm.exe .`` in PowerShell.
We can put the executable on PATH for everyday use.
No server, database, or language runtime needs to be installed.

A few useful commands
----------------------

.. code-block:: sh

   ccm .
   ccm . --no-cost
   ccm . --color=always
   ccm . --by-file --sort code
   ccm . --html -o report.html
   ccm . --json -o report.json
   ccm . --format=csv --include-ext=rs,py
   ccm . --compare ../before --technical

Color is automatic on supported terminals; a nonempty ``NO_COLOR`` disables automatic color.
``--color=always`` makes the choice explicit.
``ccm --help`` lists the selection, reporting, and resource options.
Unknown types and detected counting limitations are reported rather than silently assigned invented counts.

Why a different cost method?
-----------------------------

An uncalibrated COCOMO dollar total is a poor default for AI-assisted development.
The familiar source-counter estimate maps source size through a labor model; it does not observe the project's agent usage, tool spending, reusable evidence, or remaining acceptance work.
The same number of lines can be a fresh implementation, generated boilerplate, a reused component, or a small change to a well-tested system.
A fixed size-to-labor conversion cannot tell those situations apart.

`COCOMO II <https://boehmcsse.org/tools/cocomo-ii/>`_ remains a substantial estimation model when its cost drivers and calibration fit the project.
Our objection is to presenting a counter's default size-based figure as a useful present-day development budget without that information.
CCM therefore uses its own change and certificate calculus with explicit resource costing.

.. code-block:: text

   declared change -> affected parts -> required checks -> reusable evidence
   source workload + remaining checks -> AI, tool, and human resources -> cost

The finite model calculates affected parts and pending check units from declared dependencies and evidence.
The economic adapter prices separate activities, then applies an explicit repeat scenario.
For a source inventory without a task model, CCM can still price an AI rebuild scenario while leaving project-specific acceptance work unknown.

The default 2026 profile uses research-derived spending priors, inferred token equivalents, provisional language factors, provider prices, and explicit human and tool assumptions.
These are editable R&D assumptions, not measured token consumption or a validated prediction.
We can inspect them with ``--technical`` and replace them with ``--cost-profile``.
The `economic model <docs/economics.rst>`_ and `book <https://insanai.github.io/ccm/book/chapters/economics.html>`_ give the equations, sources, and calibration plan.

A change we can explain
------------------------

.. code-block:: sh

   ccm examples/book/after --compare examples/book/before \
       --model examples/book/checkout.ccm --technical

The checked-in example changes pricing and inventory, reaches three affected parts, activates seven check units, and leaves six units pending after evidence transport.
Its default scenario is 675.86 USD, including AI, tool, and human components.
We can inspect why each check is required and which older result still supports the unchanged payment boundary.
A passed result can discharge a check; an informative failure cannot count as acceptance.

The model is currently author-declared.
Automatic semantic dependency extraction, authenticated certificates, measured work ledgers, and empirical labor calibration remain research work.
CCM does not execute the checks in a task model.

Build and contribute
---------------------

We need Rust 1.85 or later to build the executable.
The release workflows pin Rust 1.98.1 and also check the minimum supported version.

.. code-block:: sh

   git clone https://github.com/insanai/ccm.git
   cd ccm
   cargo build --release --locked -p ccm
   ./target/release/ccm .
   cargo install --path cmd/ccm --locked --offline

Reusable libraries live in ``crates/``; the CLI lives in ``cmd/ccm``.
The mathematical kernel does not read files.
The scanner and project adapter provide language policies, selection, and bounded parallel work.
The repository tools are Rust programs under ``tools/``.

`CONTRIBUTING.rst <CONTRIBUTING.rst>`_ describes checks and the local ``jj`` workflow.
`RELEASING.rst <RELEASING.rst>`_ describes native builds, archive verification, and publication.
Bugs and proposals have dedicated GitHub issue forms, and pull requests include a review template.

Documentation
-------------

* `Website <https://insanai.github.io/ccm/>`_: overview, Linux capture, and downloads.
* `The CCM book <https://insanai.github.io/ccm/book/>`_: algorithms, architecture, mathematics, estimation, and research.
* `PDF book <https://insanai.github.io/ccm/books/ccm-book-en.pdf>`_: the same prose in a designed print edition.
* `Design records <https://insanai.github.io/ccm/decisions/>`_: the research program and implemented decisions.
* `Language policies <docs/language-catalog.rst>`_, `selection <docs/counting.rst>`_, and `machine reports <docs/reports.rst>`_.

Guidedog builds the complete website and PDF from the committed sources:

.. code-block:: sh

   guidedog ssg build docs --out docs/_build/site

The GitHub Pages workflow uses the same command and Guidedog's embedded PDF compiler.
Typst CLI is needed only to refresh SVG and PNG artwork with ``cargo xtask assets``.
The Linux, FreeBSD, and Chromium showcase reports are committed observations; publication never downloads or rescans those trees.

Authors and license
--------------------

Vikrant Rathore wrote CCM, with assistance from Ronak Rathore and Kanak Rathore.
CCM is free software under **LGPL 3.0 or later**.
``COPYING.LESSER`` and ``COPYING`` contain the terms; ``NOTICE`` identifies bundled materials.
The corresponding source and Rust build tools are available at every release tag.
Language metadata retains its MIT notice, also available through ``ccm --licenses``.
Organizations seeking other licensing terms can contact `Vikrant Rathore <mailto:admin@vyomtech.com>`_.
Third-party materials retain their own licenses.
