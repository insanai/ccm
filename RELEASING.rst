Releasing CCM
==============

We release one workspace version and identify it with a matching ``v`` tag.
The first public beta is ``1.0.0-beta.1``.
The package license remains LGPL-3.0-or-later.

Release inputs
---------------

We update ``Cargo.toml``, ``Cargo.lock``, the four publication configurations, the cover preview's fallback version, the README, and the changelog together.
We refresh the prepared book cover and worked-example reports with ``cargo xtask docs``.
Historical performance records and large-tree showcase reports retain their original measurement versions.

We run the checks in ``CONTRIBUTING.rst`` and build the publication with Guidedog before pushing.
We use ``jj`` for local commits and bookmarks; the Git backend supplies the SSH remote and annotated release tag.

.. code-block:: sh

   jj bookmark set main -r @-
   jj git push --remote origin --bookmark main
   git tag -a v1.0.0-beta.1 -m "Release CCM 1.0.0-beta.1" HEAD
   git push origin v1.0.0-beta.1

The main-branch CI must pass before we create the release tag.
We do not move a published release tag to another commit.

Native archives
----------------

``release.yml`` builds Linux x86-64 and ARM64 with musl, Windows x86-64 with a static MSVC runtime, macOS Intel and Apple Silicon with deployment target 12.0, and FreeBSD x86-64 in a FreeBSD 15.1 VM.
The Rust toolchain is pinned to 1.98.1 for the five rustup targets; FreeBSD uses its native Rust package and records that compiler version.
Each native job runs the test suite and executes the release binary against the book's checkout example before archiving it.

.. code-block:: sh

   rustup target add x86_64-unknown-linux-musl
   cargo build --release --locked -p ccm --target x86_64-unknown-linux-musl
   cargo xtask dist x86_64-unknown-linux-musl

``cargo xtask dist`` checks the version, finite-model result, economic total, and branded HTML output.
It writes an archive, a SHA-256 sidecar, the license texts, and a build manifest with the source commit and compiler.
It uses installed ``tar``; Windows supplies bsdtar, which also writes the ZIP.
The ``CCM_RELEASE_TAG`` environment variable, when present, must match the workspace version.

Publication
------------

The release workflow builds the PDF through the pinned Guidedog action.
The final job downloads the native artifacts, verifies their sidecars, and requires all six archives and the PDF before writing ``SHA256SUMS``.
Only a complete set can create the GitHub release.
The source archive linked by GitHub and the exact source commit in each binary supply the corresponding source.
The macOS binaries are unsigned; there is no installer or automatic service.

``workflow_dispatch`` builds and checks the same artifacts without publishing a release.
A tag push publishes with ``gh release create --latest --verify-tag`` only after every required build passes.
The publishing job receives repository write permission; the build jobs receive read permission.

The separate Pages workflow installs the same checksum-pinned Guidedog release and runs ``guidedog ssg build docs --out docs/_build/site``.
It publishes the landing page, HTML book, PDF book, and design catalog together.
No screenshot renderer, Typst CLI, or source-tree download is needed in that workflow.
