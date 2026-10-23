Learning from real development work
====================================

An estimate becomes more useful when it can learn from the work it is intended to describe.
The first economic profile is deliberately replaceable.
Replacing it responsibly needs a ledger that connects accepted changes, resource consumption, evidence, and the task context.
A new coefficient without that connection only changes the appearance of precision.

We begin by choosing the unit of observation.
A trajectory can contain several failed implementations, tool interactions, tests, and human interventions before one change is accepted.
A merged line is an output convention, not a complete account of those activities.
A repository snapshot is further removed: it does not identify which trajectory produced each line or what requirements were absent from the snapshot.

A useful resource ledger
------------------------

For a measured task we retain the task and source identities, acceptance policy, selected agent and model version, and the relevant environment.
Usage should distinguish uncached input, cached input, output, and billed reasoning where the provider exposes them.
Hosted sessions, search, CI, storage, and other tool charges need their own invoice-backed quantities.
Human implementation, review, intervention, and verification are separate activity observations.

The ledger should identify the accepted patch and the checks that justified acceptance.
It should also retain failed attempts rather than charge their resource use to an invisible denominator.
A model's narrative statement that it used a certain number of tokens is not a substitute for a usage record.
The current CCM CLI does not import or authenticate this ledger automatically.

.. list-table:: A proposed calibration record
   :header-rows: 1
   :widths: 30 70

   * - Coordinate
     - Observation to retain
   * - Task and context
     - Requirements, source identities, environment, policy version, and acceptance boundary.
   * - Source change
     - Accepted patch, selected lines, language, generated status, and formatting policy.
   * - Agent resources
     - Billed token categories, model identity, cache policy, prices, and invoices.
   * - Tool resources
     - Session counts and durations, CI use, search, storage, and explicit allocations.
   * - Human activities
     - Implementation, review, verification, intervention, and waiting time where measured.
   * - Assurance
     - Activated checks, evidence support, failures, reusable records, and acceptance outcome.

The schema is proposed research infrastructure.
It should not be confused with a current CLI input format merely because it appears in a table.
The runtime's supported task format and cost-profile format remain the smaller contracts described in the reference chapter.

Separate the activities before fitting
---------------------------------------

The implementation prior already includes generic agent iteration.
A task-specific verification coefficient must exclude those same activities or explicitly replace that part of the prior.
Human check hours must describe human work rather than duplicate automated runtime.
If a sandbox invoice covers a shared session, allocating the full session to each file would overcount the resource.
The accounting boundary must be settled before statistical fitting.

Language effects also need a task boundary.
Comparing tokens per physical line across two languages can reward verbosity or punish compact syntax.
Comparing matched accepted tasks answers another question.
A hierarchical calibration can retain language effects while controlling for task family, repository, acceptance criteria, and intervention, but its transfer claims still need held-out evaluation.
The first profile's neutral factors record missing calibration rather than asserting equal effort.

Prediction error before a confidence story
-------------------------------------------

We hold out tasks, and where possible repositories or later time periods, before estimating a forecast's error.
Training and test leakage can occur through dictionaries, near-duplicate tasks, generated code, shared trajectories, or repeated accepted patches.
A random split at the line level would rarely protect against those dependencies.

Absolute error and appropriately scoped relative error should accompany a point estimate.
Percentage error needs care when true spending is zero or very small.
An interval needs a stated procedure and evaluation of its coverage on held-out tasks.
The current low and high scenarios are sensitivity anchors, so the CLI must not relabel them as statistical prediction intervals.

Testing an AI effect
---------------------

A claim that an agent improves development is a causal claim, not a consequence of the source inventory.
A useful design can randomize access or pair comparable tasks, define accepted quality before timing, and record experience, environment readiness, and human intervention.
The outcome should include resource cost and quality, not only the quantity of accepted text.
A fast solution that changes the acceptance criteria is not the same task completed more efficiently.

CCC can help describe the acceptance boundary and reusable evidence in that experiment.
It does not remove selection bias merely by adding graph features.
A larger surviving hypothesis count can reflect a broader declared universe rather than a worse tool.
A smaller residual check sum can reflect changed policy weights rather than greater productivity.
Context identities make these alternatives inspectable.

Revision as a scientific operation
-----------------------------------

When new evidence supports a better workload or price profile, we version the inputs and retain the previous profile for reproducibility.
We change workload coefficients because of development observations and prices because of price observations.
We record the date, source, population, task boundary, and transfer limits of each revision.
The CLI's flat profile file makes that operation small enough to perform without changing the finite engine.

The aim is not to defend the first guessed coefficient indefinitely.
It is to make the next coefficient easier to justify, test, and replace.
That is how an R&D scenario can mature into a useful local forecasting tool.
