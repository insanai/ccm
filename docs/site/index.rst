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

         `Read the book <mount://book/index.html>`_
         `Download the PDF <mount://pdf/ccm-book-en.pdf>`_

      .. rst-class:: hero-note

      Rust. No external runtime crates. Terminal, HTML, and JSON.

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

   .. rubric:: Breadth, speed, and the limits behind both.

The registry contains 658 language and format names and aliases, with native and declarative policy tiers.
The recorded CCM 0.4 release benchmark measured a 1.077x median paired speedup over scc on one equal-input 821-file snapshot.
Those are scoped results, not an all-language correctness or performance guarantee.
The source definitions, raw measurements, and reproducible method are in the book.

.. container:: text-links

   `Language policies <mount://book/chapters/inventory.html>`_
   `Algorithms <mount://book/chapters/lexical.html>`_
   `The experiment <mount://book/chapters/performance.html>`_
   `2026 estimation research <mount://book/chapters/profile.html>`_

.. _start:

Build it. Try a real tree.
--------------------------

.. code-block:: sh

   cargo build --release
   cargo install --path cmd/ccm --offline
   ccm .
   ccm . --html -o report.html
   ccm . --json -o report.json

The installed executable measures locally and does not execute source or contact an agent service.
We can add an explicit task model when we need a change and acceptance report.
The estimates remain resource scenarios with stated assumptions, and the full source inventory remains available without them.
