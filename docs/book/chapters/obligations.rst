When checks become necessary
=============================

The affected closure describes propagation.
Acceptance work needs another declaration: which conditions make a check necessary?
The same affected set can require different checks under different task policies.
A library update, a security repair, and a behavior-preserving cleanup need not share one acceptance contract.

Each check q has a nonnegative weight and a collection of trigger terms.
A term is a set of artifacts that must all be affected.
Several terms provide alternatives.
This is a positive disjunctive normal form: an OR of ANDs, without negated affectedness tests.

Let the trigger family of q be T(q).
The check is active when at least one term is contained in the affected closure.

.. math::

   q\in O(S)\quad\Longleftrightarrow\quad
   \exists t\in T(q):t\subseteq D(S).

An empty term is always contained in the closure.
It therefore represents standing work.
A check with no trigger terms would have no way to activate, which is different from a check with one empty term.
The input parser's dash notation supplies the latter.

Why positive activation matters
--------------------------------

Suppose we enlarge S to T.
The closure law tells us that D(S) is contained in D(T).
Any trigger term already contained in the smaller closure remains contained in the larger one.
Active checks are therefore monotone under added seeds for a fixed graph and trigger model.

Define required demand as the sum of active check weights.
The runtime uses checked nonnegative integer policy units; the algebra also makes sense for arbitrary nonnegative real weights.
Within the supported integer range, its reported weight sum is exact.

.. math::

   R(S)=\sum_{q\in O(S)}w_q.

Nonnegative weights preserve monotonicity of this demand.
The statement would fail for an activation language with arbitrary negation or for a sum containing negative weights.
Those alternatives can be useful in another model, but they need different laws.
The implementation does not quietly adopt them.

Alternatives and conjunctions
-----------------------------

Checkout compatibility has two singleton terms: pricing and inventory.
Either edit activates it.
The combined checkout test has one two-member term containing both parts.
It activates only after the combined closure contains both.
These declarations distinguish shared work from newly joint work without guessing from source size.

A larger trigger is not necessarily a larger check weight.
The author can give a small joint check a weight of one and a broad standing review a weight of ten.
The units encode a declared policy scale.
They remain separate from counts of source files, minutes, tokens, or dollars.

Evaluation visits trigger terms and their memberships against the closure's seen vector.
It accumulates active indices and checked weight sums.
The model bounds total memberships so a compact-looking input cannot conceal unbounded conjunction work.
An overflow is a diagnostic rather than a wrapped total.

From necessity to completion
----------------------------

An active check is necessary under the model.
It is not necessarily unfinished.
Evidence can already support it, and that evidence can either survive or be invalidated by the current change.
The distinction between active and pending is the reason we need a certificate calculus rather than a list of tests.

The checkout example activates seven units of checks.
The surviving payment review supports one of them.
We therefore see seven required units and six pending units.
The difference is meaningful only because the report names the evidence and its support.
A bare total could not explain whether the reduction came from reuse, omission, or an arbitrary discount.
