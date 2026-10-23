Reports, color, budgets, and exit status
=======================================

We use the default terminal report for an inventory, an optional acceptance plan, and an inspectable dollar scenario.
``--sort`` orders the language summary by code, name, files, lines, comments, or blanks.
``--by-file`` adds file details, and ``--technical`` includes dependency witnesses and explicit finite-model units.
We keep one predecessor witness per affected artifact so a long dependency chain does not produce quadratic path output.

Automatic color requires terminal stdout.
TERM may be absent; TERM=dumb, a nonempty NO_COLOR, or CLICOLOR=0 disables automatic color.
An empty NO_COLOR does not disable color.
``--color=always`` explicitly enables ANSI colors, and ``--color=never`` disables them.
The Code column uses cyan, comments use green, pending checks use amber, and failed checks use red.
Pipes and machine formats remain plain.
COLUMNS controls wrapping between 40 and 160 columns, with 80 as the fallback.

Symlinks, binary files, and intentionally excluded generated files have concise aggregate counters.
They are normal selection decisions and do not make a scan incomplete.
Coverage failures are distinct warnings; the default report shows at most five and ``--verbose`` shows all.
Verbose output also groups unrecognized file types by extension.
JSON retains coverage details and skip counters.

Formats and files
------------------

``--html`` creates a responsive standalone report with dark-mode and print styles.
It escapes imported text and uses no scripts, remote assets, or source snippets.
``--html -o report.html`` protects an existing file by default.
``--force`` explicitly replaces a report through a temporary file and rename.
The output byte budget is 64 MiB by default, adjustable through ``--max-report-bytes``.
Exceeding it returns an error before a report is written.
Output files inside a future scan are ordinary input unless excluded.

``--json`` emits one ccm.report.v3 object.
It includes the count policy, selection settings, all retained file details, file purposes, generated flags, skip counters, changes, finite-model results, and economic assumptions.
Absent semantic coordinates are null.
Exact case and observation cardinalities remain integers; distinction bits and scenario money are floating-point approximations.
Consumers that need exact 64-bit policy arithmetic must preserve integer precision.
There are no terminal controls or progress messages in JSON stdout.
Errors go to stderr and do not produce a partial report object.

``--format=csv`` emits RFC-style quoted file rows with CRLF endings.
Formula-leading path cells receive a leading apostrophe for spreadsheet safety; JSON preserves exact paths.
``--format=ndjson`` emits a file object per line and a final report object.
NDJSON is a record layout; this implementation still retains the measured inventory and renders within the output budget before writing.
``--languages`` supports these formats for registry discovery as well.

.. list-table:: Exit status
   :header-rows: 1

   * - Status
     - Meaning
   * - 0
     - The selected supported text files were counted. Open checks remain report data by default.
   * - 1
     - Arguments, I/O, model validation, resource limits, or output failed.
   * - 2
     - A report was produced with at least one lexical coverage failure.
   * - 3
     - The explicit fail-on-pending mode found open active checks or inconsistent modeled observations.

Budget scenarios
-----------------

We report an overall development scenario with AI implementation, extra declared checks, hosted tools, human review, and explicit spending.
The default AI profile uses 2026 research and prices frozen on 2026-10-23.
The default table uses Language, Files, Lines, Blank, Comments, and Code, with grouped numbers.
The default budget shows nonzero component costs and the total, with a short assumptions reference.
``--verbose`` adds prices and coverage details.
``--technical`` explains inferred input and output tokens, source scope, language factors, human activity assumptions, and research sources.
We describe the calibration, equations, and transfer limits in `AI development budgets <economics.rst>`_.

.. code-block:: sh

   ccm --verbose .
   ccm --cost-profile examples/ai-costs-2026.conf --html -o development.html .
   ccm --model examples/checkout.ccm --changed pricing \
       --hourly-cost 125 --review-hours 0.5 --check-hours 2 \
       --implementation-hours 8 --agent-cost 12 --rework-probability 0.2

We multiply review artifacts by assumed review hours and residual check policy units by assumed human verification hours.
Explicit implementation hours add human work.
``--agent-cost`` replaces all automatic AI and hosted-tool spending.
A stationary repeat assumption multiplies the complete per-attempt budget by ``1 / (1 - probability)``.
Without a task model, project-specific verification demand remains unknown and excluded.
Generic agent iteration is already included in the implementation prior.
``--no-cost`` omits the scenario.

The per-file human review assumption is sensitive to file granularity.
The AI estimate is sensitive to source formatting and transfers observational and benchmark findings to a new workload.
Neither component is a fitted forecast for the selected project.
There is no statistical confidence interval for these unvalidated assumptions.
The low and high implementation scenarios are sensitivity anchors.
The performance benchmark confidence interval describes measured tool runtime, not economic prediction accuracy.
