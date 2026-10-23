Building the CCM publications
==============================

The publication project lives under ``docs/conf.toml``.
We use the installed Guidedog CLI to assemble four mounts: the landing page, the HTML book, the PDF book, and the design catalog.
Guidedog owns reading, references, page generation, PDF source generation, mounted outputs, and final assembly.
The project contains no alternative website or book generator.

.. code-block:: sh

   guidedog ssg build docs --out docs/_build/site
   guidedog build html docs/book -W -n
   guidedog build pdf docs/book -W

Guidedog generates the PDF through its normal built-in Typst support.
The book configuration does not override that compiler with an external executable.
The separately installed Typst CLI is used only to regenerate SVG artwork and PNG previews.
The asset task checks for it before making changes and reports an Elm-style diagnostic with a concrete installation hint when it is unavailable.
A publication build from the prepared assets does not require that CLI.
The artwork imports CeTZ 0.3.4 and Fletcher 0.5.8, pinned together for the cover.
Their Typst package cache is the standard Typst cache.

The complete site starts at ``docs/_build/site/index.html``.
The HTML book starts at ``docs/_build/site/book/index.html``.
The assembled PDF is ``docs/_build/site/books/ccm-book-en.pdf``.
Generated build trees are ignored by version control and source-limit checks.

Assets and screenshots
-----------------------

``assets/ccm-logo.svg`` is the canonical colored mark, with a monochrome variant beside it.
``tools/ccm-art`` generates their vector contours, projected extrusion layers, and gradients with the Rust standard library.
The same tool writes architecture, sequence, dependency, and resource-flow diagrams.
It adds no external crate dependency to CCM.

``docs/book/_templates/cover-art.typ`` draws the black-and-white notebook and pencil illustration with CeTZ and Fletcher.
``book.typ`` supplies the actual cover, type hierarchy, page furniture, captions, tables, and code treatment.
``render-cover.typ`` exports the vector illustration, and ``cover-preview.typ`` renders the first page for the landing page.
The illustration uses deterministic offset strokes and restrained hatching; it contains no raster texture or random seed.

The CLI screenshots capture actual ANSI output from the executable against ``examples/book``.
A small Rust helper displays that output with the book's terminal palette, and the installed Chromium CLI captures it.
The HTML screenshot is generated from CCM's own report for the same source and model.
The corresponding structured result is staged as ``worked-example.json``.
The preparation task also copies the catalog's supporting source references into the shared static tree so their links work in the standalone catalog and assembled site.

After changing any of these inputs we run:

.. code-block:: sh

   cargo xtask assets
   cargo xtask docs

This optional preparation task regenerates the assets, invokes the installed Typst CLI for SVG and PNG cover assets, builds CCM, captures screenshots, and invokes the installed Guidedog CLI for publication.
``--guidedog PATH`` chooses another installed Guidedog executable, and ``--no-pdf`` requests an HTML preview through Guidedog's mount builder.
The ordinary publication command remains ``guidedog ssg build`` and works from the checked-in publication inputs.
CCM measurement itself invokes none of these documentation tools.

The shared static tree includes a local MathJax SVG bundle so mathematical pages do not depend on a remote script.
Its upstream license and provenance are retained alongside the bundle.
No screenshot or illustration requires an external hosted image.
Building the site does not publish it to an external host.
