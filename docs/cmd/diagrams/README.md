# Selected Typst diagrams

We use CeTZ for the geometric filtration and Fletcher for the observation quotient and the separate CPG edge layers.
We retain Graphviz for the lifecycle, overview, evidence diamond, and engine graph because their layouts already explain the relations clearly.

The sources pin CeTZ 0.4.2 and Fletcher 0.5.8.
Fletcher 0.5.8 also depends on CeTZ 0.3.4.
Fletcher uses the MIT license; CeTZ uses LGPL-3.0-or-later.
They remain external Typst packages rather than vendored source.
We generated the SVG assets with Typst 0.15.1 and the Libertinus Sans and DejaVu Sans Mono fonts.
The SVG files contain their glyph outlines and need no browser font download.

We run these commands from the repository root to regenerate the assets:

```sh
typst compile --root docs/cmd docs/cmd/diagrams/rips-filtration.typ docs/cmd/_static/diagrams/rips-filtration.svg
typst compile --root docs/cmd docs/cmd/diagrams/observation-quotient.typ docs/cmd/_static/diagrams/observation-quotient.svg
typst compile --root docs/cmd docs/cmd/diagrams/cpg-layers.typ docs/cmd/_static/diagrams/cpg-layers.svg
guidedog dsc build --dir=docs/cmd --target=both
```

The current GuideDog conversion CLI accepts PDF, HTML, and RST outputs; it does not export Typst directly to SVG.
We therefore use the Typst compiler for the SVG assets and GuideDog for the document builds.
We retain generated SVGs so an ordinary catalog build does not need to download these packages or regenerate the diagrams.

We review a diagram directly with Typst's PNG output:

```sh
typst compile --format png --ppi 180 --root docs/cmd docs/cmd/diagrams/rips-filtration.typ docs/cmd/_build/research/rips-preview.png
```

GuideDog also retains each discussion's generated Typst source under `docs/cmd/_build/pdf/sources/`.
We can render that source directly to page PNGs with the same project root:

```sh
typst compile --format png --ppi 100 --root docs/cmd docs/cmd/_build/pdf/sources/cmd-0002-change-certificate-calculus-en.pdf.typ 'docs/cmd/_build/research/cmd-0002-{0p}.png'
```

The quoted page placeholder produces one PNG per page.
We inspect those pages for figure placement, captions, labels, and mathematical notation alongside the successful GuideDog PDF build.
