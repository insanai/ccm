:cmd-number: 0012
:cmd-title: Public beta releases and native distribution
:cmd-state: committed
:cmd-type: Standards Track
:cmd-authors: ["Vikrant Rathore"]
:cmd-created: 2026-10-24
:cmd-updated: 2026-10-24
:cmd-discussion: No external discussion URL assigned
:cmd-labels: ["release","distribution"]

Public beta releases and native distribution
============================================

Abstract
--------

We publish CCM 1.0.0-beta.1 in the insanai organization with native executables, a complete Guidedog publication, and a Homebrew tap.
The version contains the word beta, while the GitHub release is a regular release marked Latest with the prerelease flag disabled.
The release does not change the count policy, machine schemas, or economic coefficients.

Native distribution
--------------------

The release contains Linux x86-64 and ARM64, macOS Intel and Apple Silicon, Windows x86-64, and FreeBSD x86-64 archives.
Linux uses static musl, Windows uses the static MSVC runtime, macOS targets 12.0, and FreeBSD builds in a 15.1 VM.
The five rustup builds use Rust 1.98.1; the native FreeBSD compiler is recorded in its build manifest.

Each job tests its native code and runs the release binary against the published checkout example before packaging it.
The smoke check verifies the version, required and pending checks, expected cost, and complete branded HTML output.
Each archive includes LGPL terms, the language-data MIT notice, a README, and the exact source commit and compiler.
GitHub provides the corresponding source at the release tag.

The Rust xtask writes the archives with installed tar, using bsdtar for Windows ZIP output.
It writes a SHA-256 sidecar per archive.
The final job requires all six archives and the Guidedog PDF, verifies the archive sidecars, and writes one SHA256SUMS file.
A missing, empty, or mismatched artifact prevents publication.

Repository automation
----------------------

The main branch checks Linux, Windows, macOS, FreeBSD, and the minimum supported Rust version.
Release tags must match the workspace version.
Manual release workflow runs build the artifacts without publishing them.
GitHub publishing happens only after the native and PDF jobs complete.
The write permission belongs to the publishing job; build jobs have read permission.
Actions are pinned to reviewed commit identifiers.

The issue forms distinguish reproducible defects from feature or research proposals.
The pull request template asks for behavior, rationale, validation, and compatibility.
Contribution and release instructions explain the local Jujutsu workflow and Git backend.

Guidedog publication
--------------------

The checksum-pinned Guidedog action builds the landing page, HTML book, PDF book, and design catalog with the existing multi-mount configuration.
The PDF uses Guidedog's embedded compiler.
Typst CLI and a browser remain asset-preparation tools, and the release workflow uses committed screenshots and artwork.
The Linux capture is also prominent in the README.
Historical large-tree reports retain their measured versions.

Homebrew
---------

The insanai/homebrew-ccm repository supplies the insanai/ccm tap for macOS and Linux, on Intel and ARM64.
The formula selects a native release archive and verifies its SHA-256.
The generator lives in CCM's Rust xtask and reads the published release checksum manifest.
It rejects malformed, duplicate, and missing checksum entries.

The tap's hourly workflow finds the latest published release, checks out its tagged generator, and updates the formula when needed.
It can also be dispatched manually.
It tests installation before committing an update.
A repository-scoped workflow token writes only the tap; synchronization needs no copied personal credential from the CCM repository.

Licensing and interpretation
-----------------------------

CCM remains LGPL-3.0-or-later.
Organizations seeking other licensing terms can contact Vikrant Rathore through the GitHub repository.
Third-party materials retain their own terms.

The README explains why an uncalibrated source-size COCOMO figure is a poor default for modern agent-assisted work.
It does not claim that calibrated COCOMO II is universally useless or that CCM's R&D workload assumptions are validated predictions.
The finite calculus and explicit resource prices remain separate claims.

Validation and status
----------------------

We require local build, test, formatting, Clippy, source-limit, action-syntax, and strict publication checks.
The Linux musl archive must run the checkout smoke check, and an incomplete release must fail before producing a combined checksum manifest.
GitHub's native CI must pass before tagging.
The published artifacts, website, and Homebrew installation are checked before the release is reported complete.

References
----------

* ``RELEASING.rst`` defines the native build and publication commands.
* ``CONTRIBUTING.rst`` defines the pre-release checks.
* CMD 0007 defines the Guidedog book and website pipeline.
* ``docs/economics.rst`` records the economic assumptions and calibration boundary.

.. rubric:: Lifecycle event

::

   2026-10-24: prediscussion -> discussion. Assigned a permanent number and
   opened for discussion.

.. rubric:: Lifecycle event

::

   2026-10-24: discussion -> accepted. Publish the requested regular GitHub beta
   release, native archives, Guidedog site, and synchronized Homebrew tap.

.. rubric:: Lifecycle event

::

   2026-10-24: accepted -> committed. Publish CCM 1.0.0-beta.1 as the regular
   Latest GitHub release and synchronize its Homebrew tap. All native CI and six
   release archive jobs pass; Guidedog website and PDF are live; all seven
   public downloads match SHA256SUMS and the tagged source; the Homebrew sync
   workflow installed and tested the formula successfully. Release run
   40065655111; tap run 40065857061.
