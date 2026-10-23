AI development budgets: 2026 research profile
=============================================

We estimate the resource cost of rebuilding the selected program source with an AI coding agent.
We also price human activities and remaining declared acceptance work.
We report a planning scenario, not a historical bill, business value, or a fixed-price quote.
The mathematics specifies how quantities and prices combine.
It cannot establish the empirical accuracy of a workload assumption.

Evidence and provenance
-----------------------

We froze the first profile on 2026-10-23.
We use 2026 development evidence rather than older agent benchmarks or foundation-model training expenditure.

`Span's July 2026 development study <https://www.span.app/research/agent-effectiveness-july2026>`_ describes 103 teams over 12 weeks from May through July 2026.
Its prompt-clarity examples price 50 merged AI lines at 6.90 USD and 4.45 USD.
These imply 0.138 USD and 0.089 USD per merged line in those examples.
We choose their midpoint, 0.1135 USD, as an R&D prior.
The examples are observational associations, not population bounds or a measured greenfield development rate.
The public summary does not identify a universal input/output token count per line.

`Wu, Anderson, and Guha, July 2026 <https://arxiv.org/html/2607.22807v1>`_ report language effects on output-token consumption for matched programming problems.
Table 7 gives Java, Rust, and OCaml ratios relative to Python for two proprietary models.
We average each pair to obtain provisional output factors 1.24, 1.26, and 1.29.
These are task-level effects, not measured per-line price differences.
We transfer them to the output workload as an explicit modeling assumption.
We treat other languages as neutral until we have evidence or local calibration.
Applying a mixed-cohort dollar prior to a Python reference and transferring language factors are both unvalidated choices.

`OpenAI's official pricing <https://developers.openai.com/api/docs/pricing>`_ lists GPT-6 Astra standard input at 10 USD and output at 50 USD per million tokens on the snapshot date.
It also lists a 1 GB hosted shell or code-interpreter container at 0.03 USD per 20-minute session.
We use uncached standard token rates and include reasoning in output.
We make no API calls and do not download prices at runtime.
A provider may change prices after the snapshot date.

Inferred workload
------------------

We need a declared conversion to reprice the dollar prior with another model.
We assume a 90 percent input share and a 10 percent output share.
This mix is an R&D guess, not a result reported by Span.
At the reference prices, we infer the following token-equivalent workload per program line:

.. math::

   t = \frac{0.1135 \times 10^6}{0.9(10) + 0.1(50)} = 8107.142857\ldots

   t_I = 0.9t, \qquad t_O = 0.1t.

Thus the default profile uses about 7296.43 input tokens and 810.71 output tokens per line before language adjustment.
These are inferred billing equivalents, not measured agent traces or a tokenizer's count of the source.
The source-work-v1 lexical token count is a separate descriptive quantity and does not enter this estimate.

Let :math:`L_l` be the selected program lines in language :math:`l`, :math:`f_l` its output multiplier, and :math:`m` a workload multiplier.
The implementation scenario is:

.. math::

   I = m t_I \sum_l L_l, \qquad O = m t_O \sum_l f_l L_l,

   C_{\mathrm{implementation}} = (p_I I + p_O O) / 10^6.

The prior already includes generic agent exploration, iteration, and verification.
We do not add another default probability for those same iterations.
The displayed low and high implementation scenarios scale the midpoint result by :math:`0.089/0.1135` and :math:`0.138/0.1135`.
They are sensitivity anchors, not a confidence interval, a prediction interval, or guaranteed bounds.
Physical line counts remain sensitive to formatting and verbosity.
A larger estimate does not imply a more valuable or more complex program.

Connection to the change and certificate calculus
-------------------------------------------------

CMD 0002 distinguishes change seeds, their dependency closure, required obligations, and evidence-conditioned residual obligations.
We retain that distinction when attaching resource prices.
Implementation quantities attach to source files matching the change seeds.
Human review attaches to the affected closure.
Extra acceptance work attaches to the residual weighted checks, :math:`U`.
Surviving accepted evidence can remove that work; a failed check cannot discharge it.
Shared and joint obligations are computed before pricing.
No information-bit quantity is converted directly into money.

We assume an additional :math:`b = 1000` agent tokens per pending policy unit.
This is a configurable R&D budget, not an empirical duration or a theorem about policy weights.
With input share :math:`s = t_I/(t_I+t_O)`, the additional acceptance scenario is:

.. math::

   C_{\mathrm{checks}} = m b U (s p_I + (1-s)p_O)/10^6.

We use :math:`s=0.9` if both token coefficients are zero.
These checks mean project-specific acceptance beyond the generic iteration included in the implementation prior.
We must calibrate the boundary to avoid charging twice for the same activity.
We can set ``check_tokens=0`` if the implementation prior already covers those checks.
Human check hours represent separate human work; we can set them to zero for entirely automated checks.
Unmodeled acceptance demand stays unknown and is excluded from the additional check budget.

We assume one hosted session per 50 delivered program lines, rounding up over the complete selected scope.
The 50-line session throughput is a guess; Span's illustrative 50-line batch is not evidence for that throughput.
A zero-line scope incurs no default session charge, even if declared check work remains.
We exclude additional sandbox usage for checks, local hardware, CI, search services, storage, deployment, and subscription allocations unless we budget them explicitly.
Tool-related model tokens are already part of the inferred token workload.
There is no invented fee for each local tool call.

Let :math:`H` be human hours per attempt, :math:`p_H` the loaded hourly resource price, :math:`A` explicit AI/tool spending, :math:`S` hosted sessions, :math:`p_S` their price, and :math:`r` an additional stationary repeat probability.
The budgeted total is:

.. math::

   C = \frac{p_H H + A + C_{\mathrm{implementation}} + C_{\mathrm{checks}} + p_S S}{1-r}.

We use resource and opportunity costs rather than deriving asset value from source size.
We price marginal remaining assurance demand within the declared task model.
Nonnegative prices preserve monotonicity in a fixed workload's residual demand.
The CCC calculations are exact within the finite input model; the workload coefficients and the mapping from repositories to tasks are not empirically validated.

Scope and overrides
--------------------

Without a model or comparison, we estimate a counterfactual rebuild of selected non-generated program files.
Documentation, data, and recognized generated files do not contribute implementation lines.
They can still contribute to the separately declared human per-file review budget.
Unknown extensions and unsupported constructs can hide additional work.
We do not infer historical authorship, actual tokens consumed, requirements quality, or missing product development activities.

With a comparison, we use whole changed source files in the current snapshot.
This is a rebuild scenario for those files, not a patch development estimate.
Removed files contribute no current implementation lines but still contribute review work.
With a task model, we use source paths matching its seeds.
Unmatched seed names leave that implementation work unpriced and visibly partial.
A dependency path is a reason to review a dependent artifact, not proof that we must rewrite it.

.. code-block:: sh

   ccm --verbose .
   ccm --cost-profile examples/ai-costs-2026.conf --verbose .
   ccm --ai-input-price 4 --ai-output-price 20 --verbose .
   ccm --ai-workload 2 --html -o development.html .
   ccm --agent-cost 75 --implementation-hours 4 .

``--cost-profile`` reads a bounded, UTF-8 flat ``key=value`` file.
It is deliberately not a full TOML parser.
We apply profile entries and command-line overrides in argument order.
The example lists every numeric coefficient and the presentation model label.
``language.NAME`` changes an output factor for a registered language.
``--ai-input-price`` and ``--ai-output-price`` reprice the same inferred workload; the model label remains the profile's reference label.
``--agent-cost`` replaces all automatic AI and hosted-tool spending for one attempt.
``--implementation-hours`` adds explicitly budgeted human implementation hours.
``--no-cost`` omits the economic scenario.
Optional lexical work-token collection requires ``--technical`` and remains separate from inferred AI billing tokens.
Default and verbose terminal reports retain a short estimate and an assumptions reference.
Technical output and HTML details retain the full calculation.
JSON preserves assumptions, sources, scope, components, and the overall expected total in ``ccm.report.v3``.

Next calibration
----------------

We can replace these priors with local trajectories linking billed input, cached input, output, reasoning, tool invoices, accepted changes, and declared acceptance work.
A useful calibration separates implementation from verification and identifies language, task type, acceptance criteria, caching, and human intervention.
We should validate on held-out tasks and report prediction error before presenting a statistical interval.
The current research profile is a reproducible starting point for that work.
