What the 2026 research profile contains
=======================================

The first profile is a reproducible starting point for research and development.
It uses development evidence published in 2026 and provider prices frozen on 2026-10-23.
It does not use foundation-model training expenditure or the historical COCOMO labor equation.
The purpose is to approximate development with an AI agent under visible assumptions.

The important distinction is between an observed spending example and a measured token workload.
The public evidence we used supplies the former more directly than the latter.
We therefore preserve the conversion as an inference.

Spending anchors
-----------------

`Span's July 2026 study <https://www.span.app/research/agent-effectiveness-july2026>`_ describes 103 engineering teams over twelve weeks of development activity from May through July.
Its prompt-clarity examples price fifty merged AI lines at 6.90 USD and 4.45 USD under different conditions.
These examples imply 0.138 USD and 0.089 USD per merged line.
We choose their midpoint, 0.1135 USD, as a provisional planning prior.
It is our chosen midpoint, not a reported population mean or a confidence interval.
The observational examples do not establish a causal effect or a universal greenfield rate.

The public summary does not provide a universal input/output token count per delivered line.
We assume a 90 percent input share and a 10 percent output share under reference prices of 10 and 50 USD per million tokens.
Those choices convert the dollar prior into token-equivalent quantities that can be repriced.

.. math::

   t=\frac{0.1135\times10^6}{0.9(10)+0.1(50)},\qquad
   t_I=0.9t,\quad t_O=0.1t.

The result is approximately 7,296.43 input tokens and 810.71 output tokens per reference program line.
These are inferred billing equivalents, including a broad agent trajectory under the prior.
They are not a tokenizer's measurement of a line of Rust or tokens observed in a particular repository run.
The source-work-v1 lexical counter is a different quantity and does not enter this equation.

Language effects
-----------------

`Wu, Anderson, and Guha's July 2026 investigation <https://arxiv.org/html/2607.22807v1>`_ compares coding-agent output-token use across matched programming problems.
Table 7 supplies Java, Rust, and OCaml ratios relative to Python for two proprietary models.
Averaging each pair gives provisional output factors of 1.24, 1.26, and 1.29.
The study's effects are task-level effects, not measured per-line price differences.
Applying them to a line-driven workload and treating the mixed-cohort dollar prior as a reference-language anchor are additional unvalidated transfers.

The profile keeps those choices editable.
Other languages receive factor one rather than an invented fitted coefficient.
A neutral prior does not claim equal difficulty; it records the absence of a more specific calibration in this first profile.

Prices and time
----------------

The `official OpenAI pricing page <https://developers.openai.com/api/docs/pricing>`_ supplied the snapshot card: GPT-6 Astra standard input at 10 USD and output at 50 USD per million tokens, with reasoning included in output.
The hosted shell/code-interpreter reference was 0.03 USD for a 1 GB twenty-minute session.
The profile uses uncached standard rates.
The CLI does not contact a provider or update these values at runtime.

A price change and a workload change are different revisions.
``--ai-input-price`` and ``--ai-output-price`` reprice the same inferred workload.
They do not establish that another model will use the same tokens or achieve the same accepted result.
A model label remains the reference label unless we change the profile explicitly.
This is useful for a controlled scenario and insufficient for a model-quality comparison.

Sensitivity and calibration
----------------------------

The technical report scales the implementation midpoint by the lower and upper spending-anchor ratios.
These are sensitivity scenarios, not statistical bounds.
Task clarity, repository readiness, accepted quality, cache use, language, and human intervention can all produce spending outside them.

A future profile should link accepted changes to billed input, cached input, visible output, reasoning, tool invoices, and human activity.
It should separate implementation from project-specific verification before estimating both coefficients.
Until such a calibration exists, an honest estimate is one whose assumptions can be located and replaced.
The versioned file in ``examples/ai-costs-2026.conf`` makes that replacement concrete.
