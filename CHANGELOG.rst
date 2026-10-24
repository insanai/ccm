Changes in CCM
===============

1.0.0-beta.2 - 2026-10-24
-------------------------

The terminal report shows every count column whenever the terminal has room for it.
It stays 80 columns wide, and widens only as far as the language table needs and the terminal allows, so a wide terminal shows Lines, Blank, Comments, and Code for a tree such as Linux or lxc/incus.
On a narrow terminal the share bars narrow, down to five cells, before a count column is dropped, and only when that saves the column.

On the website, justified lines no longer run past the text column (issue #1).
The line breaker no longer counts a hyphen into the line after it, lines set tighter than their natural width are shrunk, and splitting at hyphens keeps every character.
Counts are unchanged; the count policy is still lexical-v5.

1.0.0-beta.1 - 2026-10-24
-------------------------

We publish the first public beta of the Code Counter Machine.
It includes a dependency-free Rust CLI, native and declarative language policies, bounded parallel selection, and terminal, HTML, JSON, CSV, and NDJSON reports.
The finite change model explains dependency closure, required checks, reusable evidence, and pending acceptance work.
The editable 2026 economic scenario prices AI, tool, and human resources separately.

The release provides six native archives across Linux, Windows, macOS, and FreeBSD, with source provenance, license texts, and checksums.
GitHub Actions checks the supported platforms, builds the release archives, and publishes the complete Guidedog website and PDF book.
The repository adds issue forms, a pull request template, contribution instructions, and security reporting guidance.

This is a beta interface and an R&D cost model.
Automatic semantic extraction and empirical labor calibration remain research work.
