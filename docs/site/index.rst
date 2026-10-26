.. meta::
   :description: CCM counts source, follows declared changes, and makes evidence and development estimates inspectable.

CCM
===

.. container:: hero

   .. container:: hero-copy

      .. rst-class:: eyebrow

      CODE, CHANGE, AND CONFIDENCE

      .. raw:: html

         <h1>Measure the code.<br>Understand <em>the change.</em></h1>

      .. rst-class:: lead

      A local source counter with a deeper question: what does a change affect, what still needs checking, and what resources should we budget?

      .. container:: actions

         `Get CCM 1.0.0-beta.3 <https://github.com/insanai/ccm/releases/tag/v1.0.0-beta.3>`_
         `Read the book <mount://book/index.html>`_
         `Download the PDF <mount://pdf/ccm-book-en.pdf>`_

      .. rst-class:: hero-note

      Rust. No external runtime crates. Terminal, HTML, and JSON.

      Linux, Windows, macOS, and FreeBSD. `Source on GitHub <https://github.com/insanai/ccm>`_.

   .. container:: hero-art

      .. image:: ../shared/_static/ccm-logo.svg
         :alt: The CCM mark, a cut isometric cube that holds a small pink cube.

      .. rst-class:: art-caption

      COUNT / TRACE / EXPLAIN

.. container:: pillars

   .. container:: pillar

      .. rubric:: 01 / A familiar starting point

      Count the source.
      A clear table of languages, files, lines, comments, and code, with explicit selection and language policies.

   .. container:: pillar

      .. rubric:: 02 / A declared task

      Follow the change.
      Trace affected parts, required checks, and the evidence that survives across a stated dependency model.

   .. container:: pillar

      .. rubric:: 03 / Visible assumptions

      Explain the estimate.
      Separate AI, tool, and human activities with an editable 2026 research profile and inspectable units.

.. container:: section-heading

   .. rst-class:: eyebrow

   A REPORT WE CAN READ

   .. rubric:: Clear enough for the first look. Detailed enough for the next question.

The normal CLI keeps the essentials close together.
The technical view exposes the calculation.
Standalone HTML carries the report into a meeting, while JSON preserves the structured result for an agent.

.. figure:: ../shared/_static/cli-overview.png
   :alt: The actual CCM CLI report for a four-file Rust example, with grouped code counts and a short development estimate.
   :width: 100%

   A live report from the book's checked-in example.

.. container:: section-heading

   .. rst-class:: eyebrow

   THE LINUX KERNEL IN TWO SECONDS

   .. rubric:: Twenty-eight million program lines, each one in exactly one class.

We ran CCM on the Linux 6.12 source release.
It counts 82,405 files and 1.46 GB in about two seconds with 125 MB of memory, and every file counts.
The image below is that run's terminal output, captured by ``cargo xtask showcase`` and rendered without edits.
The same command writes the full HTML reports for Linux 6.12, FreeBSD 15.1, and Chromium 156 to the repository's ``examples`` folder.

.. figure:: ../shared/_static/linux-terminal.png
   :alt: The CCM terminal report for Linux 6.12: 27,828,995 program lines in 82,405 files, 87 percent of them C, with a priced development estimate.
   :width: 820px

   The terminal report for Linux 6.12, from the release build.

.. container:: split-section

   .. container:: split-copy

      .. rst-class:: eyebrow

      MORE THAN A BIG NUMBER

      .. rubric:: The reasons travel with the result.

      A small patch can invalidate several assumptions.
      Two edits can share a check and create another check that neither needed alone.
      CCM's finite model keeps those relationships visible instead of hiding them in a single score.

      The model is explicit.
      Counting a language does not imply that semantic dependencies have been extracted automatically.
      The book walks through the definitions, proofs, limits, and executable checkout example.

      `Follow the worked change <mount://book/chapters/worked-model.html>`_

   .. image:: ../shared/_static/change.svg
      :alt: Two changed parts reach checkout while an unchanged payment boundary retains its evidence.

.. container:: book-section

   .. image:: ../shared/_static/book-cover.png
      :alt: The CCM book cover, with the logo, the title, and an isometric lattice in which one change rises and spreads through a bounded closure.

   .. container:: book-copy

      .. rst-class:: eyebrow

      THE CCM BOOK

      .. rubric:: Code, change, and confidence.

      A book about the things we can count, the changes we must understand, and the evidence we need before accepting them.
      We follow one example from source bytes to a review plan and a priced resource scenario.

      Inside are algorithms, architecture layers, sequence diagrams, mathematical derivations, actual report screenshots, research questions, and a calibration plan.
      The web and PDF editions come from the same prose through Guidedog.

      .. container:: actions

         `Start reading <mount://book/index.html>`_
         `Open the PDF <mount://pdf/ccm-book-en.pdf>`_

.. container:: section-heading

   .. rst-class:: eyebrow

   MEASUREMENTS WITH A METHOD

   .. rubric:: Counts we can check without trusting the counter.

We do not grade CCM against another counter.
Each line class has a definition, and each definition implies a property that a short program checks on the input itself.
Every physical line is in exactly one class, so code, comment, and blank lines add up to the line count.
The blank lines are exactly the whitespace-only lines.
A new policy version may move a line between code and comment, but it never changes a file's line count.
And CCM selects no file that Git ignores.
These hold for every file of Linux 6.12, FreeBSD 15.1, and Chromium 156, which together hold more than 120 million program lines.

The registry contains 658 language and format names and aliases, with native and declarative policy tiers.
The source definitions, raw measurements, and reproducible method are in the book.

.. container:: text-links

   `Language policies <mount://book/chapters/inventory.html>`_
   `Algorithms <mount://book/chapters/lexical.html>`_
   `The experiment <mount://book/chapters/performance.html>`_
   `2026 estimation research <mount://book/chapters/profile.html>`_

.. _start:

Build it. Try a real tree.
--------------------------

`Download a native executable <https://github.com/insanai/ccm/releases/tag/v1.0.0-beta.3>`_ or build from source.
The beta release includes six platform archives, checksums, source provenance, and the PDF book.

On macOS and Linux we can install the matching archive through the `CCM Homebrew tap <https://github.com/insanai/homebrew-ccm>`_.

.. code-block:: sh

   brew tap insanai/ccm
   brew install insanai/ccm/ccm
   ccm .

For a source build:

.. code-block:: sh

   git clone https://github.com/insanai/ccm.git
   cd ccm
   cargo build --release
   cargo install --path cmd/ccm --offline
   ccm .
   ccm . --html -o report.html
   ccm . --json -o report.json

The installed executable measures locally and does not execute source or contact an agent service.
We can add an explicit task model when we need a change and acceptance report.
The estimates remain resource scenarios with stated assumptions, and the full source inventory remains available without them.
