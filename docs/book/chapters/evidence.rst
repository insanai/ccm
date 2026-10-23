Evidence that can survive a change
===================================

A previous result is useful only under assumptions that still hold.
We represent those assumptions as the support of an evidence record.
Support can name source parts, interfaces, environmental conditions, or other modeled artifacts.
The author is responsible for choosing a support set that is adequate for the claim.

Let sigma(c) be the support of evidence c.
In the finite runtime, an admitted record survives the change when its support is disjoint from the affected closure.

.. math::

   c\in C_S\quad\Longleftrightarrow\quad
   c\text{ is admitted and }\sigma(c)\cap D(S)=\varnothing.

This is a transport rule within a context.
It does not authenticate the evidence, execute a test, or prove that the declared support is sufficient.
The context identity and evidence policy say what an author claims to have admitted.
A future provenance adapter could strengthen that claim without changing the set calculation.

Admissibility and conclusion
----------------------------

Model version 2 distinguishes a record's admission from its conclusion.
A passed record can support named checks.
An admitted failed record can remain useful as an observation, but it cannot discharge acceptance.
An unverified assertion does neither.
The validator rejects an accepted assertion class rather than treating an unchecked statement as a passed check.

The evaluator first decides which admitted records survive.
It then records favorable coverage and surviving failures for each named check.
An active check is pending if favorable coverage is absent or if a surviving failure names it.
A surviving failure therefore wins over simultaneous favorable coverage in the acceptance result.
This conservative conflict rule is separate from the behavioral compatibility calculation.

In the running example, pricing evidence supports pricing and checkout evidence supports checkout.
Both supports intersect the affected closure and are invalidated.
Payment evidence supports only payment and survives.
The report can therefore explain the reusable result without pretending that all old tests remain valid after any edit.

Residual demand
----------------

Let P(S) be the active checks that remain pending after transport and the conclusion policy.
Residual demand is their weighted sum.

.. math::

   U(S)=\sum_{q\in P(S)}w_q.

For the pure favorable-evidence case, adding change seeds can only activate more checks and invalidate more favorable records.
A previously pending active check cannot become supported by losing favorable evidence.
Under these conditions, pending checks and residual demand are monotone.
This is a conditional theorem, not a general property of every evidence policy.

A failure counterexample
------------------------

The version 2 failure rule makes the condition important.
Consider one standing check q with weight one.
A passed record with empty support names q, and an admitted failed record supported by artifact b also names q.
With no changed seeds, both survive.
The failure keeps q pending, so residual demand is one.

Now change b.
The failed record is invalidated while the empty-support passed record survives.
The standing check is now supported with no surviving failure, so residual demand becomes zero.
The seed set grew, but pending demand decreased.

Nothing in this example changes graph closure or positive activation.
The change is in which signed observations survive.
A book about the calculus must preserve this distinction rather than carry a favorable-only monotonicity proof into a richer implementation without its hypothesis.
The evaluator implements the conclusion policy directly; an operator should inspect a conflict and its provenance before interpreting the smaller residual as a real-world improvement.

What reuse can justify
-----------------------

A reusable result saves only the activities it actually supports under the declared policy.
It does not grant a universal discount on implementation work.
It does not make a dependent source file unchanged in a byte comparison.
It does not convert a failed test into an acceptance certificate because the failure was informative.

This is why the economic adapter prices residual check demand separately from implementation scope.
A favorable record can remove modeled verification activity while leaving the source workload unchanged.
The effect has a named mechanism and explicit units.
That is a stronger account than multiplying an entire project price by an unexplained evidence score.
