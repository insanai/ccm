Finite task models
===================

We supply a model with ``--model FILE`` and optional repeated ``--changed ARTIFACT`` seeds.
The preferred header is ``ccm-model<TAB>2`` with a literal tab.
Version 1 input remains supported with its original failed-evidence interpretation.
Records contain tab-separated fields, one physical line each.
Blank lines and lines beginning with ``#`` are ignored.
Inputs are UTF-8, bounded to 4 MiB and 100,000 physical record lines.
References must name earlier declarations.

A context identity is required.
It should bind a source snapshot, task, environment, tool policy, and acceptance rules.
The model author supplies that binding; CCM does not authenticate it, execute checkers, or cryptographically bind source bytes.
Tests, reviews, proofs, and unchecked assertions remain different evidence classes.

.. list-table:: Input records
   :header-rows: 1

   * - Record
     - Fields after the record name
     - Meaning
   * - context
     - identity
     - One declared measurement context.
   * - artifact
     - name
     - A modeled file, interface, environment, or assumption.
   * - edge
     - assumption, dependent
     - Invalidation travels from the first artifact to the second.
   * - check
     - name, nonnegative integer weight, triggers
     - A required check and its activation policy.
   * - evidence
     - name, class, status, support, addressed checks
     - An evidence item, its assumptions, and its claims.
   * - case
     - name, observation label
     - One finite hypothesis and the outcome visible to the task observer.
   * - allows
     - evidence name, compatible case names
     - A compatibility predicate used when that evidence survives.

Commas join members of a set and semicolons separate alternative trigger sets.
``pricing,inventory`` requires both artifacts in the invalidation closure.
``pricing;inventory`` requires either.
A trigger ``-`` is an empty trigger and activates a standing check.
A trigger ``!`` contains no triggers and never activates.
A support, discharge, or compatibility field ``-`` is the empty set.
Each sentinel must occupy its entire set field or trigger alternative.
Names are nonempty, unique within their declaration kind, and cannot contain controls, commas, semicolons, or exclamation marks; ``-`` is reserved.

Evidence and outcomes
---------------------

Classes are test, review, proof, and assertion.
Statuses are accepted, failed, and unverified.
In version 2 accepted means a declared admissible favorable result, while failed means a declared admissible checked counterexample.
Both may refine the case universe when their support survives.
Only favorable results discharge checks.
A surviving failed result leaves its addressed active checks pending and marks them failed, even when another route passes.
Assertions must be unverified in version 2.
Unchecked declarations cannot discharge checks or narrow cases as checked observations.

In version 1 failed records remain unaccepted and neither discharge checks nor constrain cases.
This preserves the original input contract.
The schema version is included in the JSON model result.
Rust callers can construct typed conclusions and explicit admissibility without importing a text format.

Accepted evidence survives only when its support avoids the invalidation closure.
An empty support always survives under this declared policy.
Alternative evidence routes can address the same check, and overlapping favorable coverage never subtracts a check twice.
A route invalidated by changed assumptions loses both discharge power and its behavioral filtering effect.

Behavioral ambiguity
---------------------

Cases are optional.
Without them, behavioral ambiguity is unavailable.
With them, we intersect the compatible cases of surviving admissible evidence and count distinct observation labels.
Different hypotheses with the same observation contribute one task outcome.
Positive outcome counts have a rounded log2 display in distinction bits.
No probability distribution is assumed.
An empty compatible universe is inconsistent and its distinction bits are null, not zero.
The finite kernel bounds cases and filters at 100,000 each and case-filter work at 20 million steps per query.

The checkout fixture supplies a complete version 2 model with graph, checks, evidence, cases, and compatibility predicates.

.. code-block:: sh

   ccm --model examples/checkout.ccm --changed pricing --changed inventory --technical
   ccm --model examples/checkout.ccm --changed pricing --json
   ccm --model examples/checkout.ccm --changed pricing --fail-on-pending

Without explicit seeds, comparison paths become model artifact names and all must resolve.
Without comparison or seeds, we evaluate standing checks.
We deduplicate and order seeds by model declaration index.
For at least two seeds we also compare the first against the remaining seeds, preserving shared and joint checks instead of showing only their net charge.

JSON retains affected artifacts, predecessor witnesses, active checks, negative results, transport status, and exact case cardinalities.
It is not a lossless model serialization: original edge lists, predicates, supports, comments, and formatting are not exported for reimport.
This version makes no output-to-input round-trip guarantee.
