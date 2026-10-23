:cmd-number: 0005
:cmd-title: Research-priced AI development budgets
:cmd-state: committed
:cmd-type: Standards Track
:cmd-authors: ["Vikrant Rathore"]
:cmd-created: 2026-10-23
:cmd-updated: 2026-10-23
:cmd-discussion: No external discussion URL assigned
:cmd-labels: ["economics","cli","validation"]

Research-priced AI development budgets
======================================

Abstract
--------

We add an automatic AI development scenario to the existing source inventory and finite change calculus.
We display 2026 research provenance, inferred token quantities, language adjustments, hosted-tool assumptions, and an overall budget.
We retain zero external crate dependencies.
This record extends CMD 0003 and CMD 0004's economic reporting without changing CMD 0002's finite mathematical definitions.

Motivation and decision
------------------------

The earlier default priced only review and modeled verification.
Its zero explicit implementation input made the report unsuitable for estimating a rebuild with AI.
We now price an explicit resource workload, separate from semantic measurements and human activities.
We use published 2026 development observations and language research, with prices frozen on 2026-10-23.
We do not use COCOMO or foundation-model training costs.

Evidence and mathematics
-------------------------

``docs/economics.rst`` records the sources, equations, assumptions, and transfer limits.
Span's May-July 2026 examples imply 0.089 and 0.138 USD per merged AI line.
We choose their midpoint as an unvalidated planning prior.
We infer token-equivalent quantities under an assumed 90:10 input/output mix and reference input/output prices of 10/50 USD per million tokens.
The research does not report those inferred quantities as measured tokens per line.
July 2026 language research supplies provisional output-token multipliers.
Other languages use a neutral multiplier unless overridden.

For the finite CCC model, implementation scope attaches to source paths matching change seeds.
Review attaches to the affected closure.
Additional acceptance work attaches to residual weighted obligations after evidence transport.
We price resources linearly with nonnegative unit prices and apply an optional stationary repeat multiplier.
This is a resource-cost scenario, not a theorem relating semantic information to economic value.
Its quantities require empirical calibration before use as a validated forecast.

Interfaces and limits
----------------------

The terminal displays the overall development budget and its AI, tool, and human components.
Verbose and technical reports include the profile year, price card, inferred tokens, language factors, and source URLs.
HTML provides the same explanation in a collapsible assumptions panel.
JSON uses ccm.report.v3 and retains the complete numeric scenario.
Money includes cents so small hosted-tool charges remain visible.

``--cost-profile`` reads at most 65536 bytes of UTF-8 flat key=value entries.
``examples/ai-costs-2026.conf`` lists the supported fields.
We reject unknown keys, nonfinite or negative values, excessive coefficients, invalid years, and degenerate session throughput.
``--ai-input-price``, ``--ai-output-price``, and ``--ai-workload`` override individual assumptions.
Arguments apply in order.
``--agent-cost`` replaces automatic AI/tool spending.
``--implementation-hours`` remains additional human work.

The default implementation scope excludes documentation, data, and recognized generated files.
Comparisons price whole changed current source files, not patch effort.
Unmatched task seeds remain visibly unpriced.
Source-work-v1 lexical tokens are reported separately and never represented as API tokens.
No source or external tool is executed during measurement.
No price lookup or agent API call occurs at runtime.

Compatibility
--------------

The public activity-cost API remains available.
Default CLI economic totals change because automatic AI and tool components are now present.
``--agent-cost 0`` retains a human-activity-only scenario.
``--no-cost`` omits all economic output.
Count policy lexical-v2 and finite model versions 1 and 2 remain unchanged.
Earlier performance measurements describe their recorded binary, not the new default cost calculation.

Validation and remaining research
----------------------------------

We test reference calibration, independent repricing, language adjustment, finite assumptions, residual-check charging, source selection, manual replacement, and all report formats.
We check source limits, formatting, compilation, tests, and Clippy before marking this record implemented.
We still need local trajectories with billed token categories, tool invoices, accepted changes, and disjoint implementation/verification activities.
The current model is explicitly an R&D starting point.

References
----------

* `Span 2026 development observations <https://www.span.app/research/agent-effectiveness-july2026>`_
* `Wu, Anderson, and Guha, July 2026 <https://arxiv.org/html/2607.22807v1>`_
* `Official price snapshot source <https://developers.openai.com/api/docs/pricing>`_
* `Research profile and equations (source) <../_static/references/economics.rst>`_
* `Editable profile <../_static/references/ai-costs-2026.conf>`_

.. rubric:: Lifecycle event

::

   2026-10-23: prediscussion -> discussion. Implement the requested 2026 AI
   development scenario

.. rubric:: Lifecycle event

::

   2026-10-23: discussion -> accepted. Use explicit research-derived priors and
   preserve finite CCC residual work

.. rubric:: Lifecycle event

::

   2026-10-23: accepted -> committed. Implement research-priced AI budgets and
   reports in CCM 0.3.0 44 tests pass; workspace builds, Clippy with warnings
   denied, formatting, source limits, and HTML/JSON reconciliation pass
