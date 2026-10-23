The outcomes evidence leaves possible
======================================

The previous chapters ask which checks are required and supported.
A different question asks which task outcomes remain compatible with the evidence.
A failed observation can help answer that question while leaving acceptance unresolved.
The finite behavioral model exists to keep those two uses of evidence distinct.

The author enumerates a finite hypothesis universe Pi.
Each hypothesis has an observation label under a declared observer g.
Different hypotheses can share the same label when the task treats them as indistinguishable.
Each admitted evidence item can supply a compatibility subset of Pi.
The evaluator intersects the subsets belonging to surviving evidence.

.. math::

   K(S)=\bigcap_{c\in C_S}K_c,
   \qquad N(S)=|g(K(S))|.

Here the intersection is taken within the declared universe, with no selected filter leaving the full universe available.
The runtime reports both the surviving hypothesis count and the count of distinct observation labels.
The distinction matters: ten hypotheses can all produce one task observation.

The displayed distinction capacity is the base-two logarithm of the positive observation count.
The observation count is exact within the supported finite model; the logarithm is a rounded floating-point display.

.. math::

   B(S)=\log_2 N(S),\qquad N(S)>0.

This is log-cardinality, not Shannon entropy.
No probability distribution is assigned to the cases.
If no hypothesis survives, the result is inconsistent evidence within the declared universe and the logarithm is unavailable.
It is not zero uncertainty.
One surviving observation gives zero distinction bits, which is a different statement.

Complementary evidence
-----------------------

Suppose four hypotheses a, b, c, and d have two observations.
Hypotheses a and b produce the first observation; c and d produce the second.
One evidence filter retains a, b, and c.
Another retains a, b, and d.
Each filter alone leaves both observations possible, but their intersection leaves only the first.
The evidence is complementary for this observer even though neither item changes the observation count by itself.

The implementation can measure signed evidence interaction against a background set.
Writing B for background distinction bits, B1 and B2 for the separate additions, and B12 for their joint addition gives:

.. math::

   I=B_1+B_2-B-B_{12}.

In the example, the four values are one, one, one, and zero in the corresponding expression, producing positive interaction of one bit.
For redundant filters that each remove the same observation, the interaction can be negative.
If an intersection is inconsistent, the method returns no numeric interaction rather than applying a logarithm to zero.

An admitted failure can refine cases
------------------------------------

Model version 2 allows a failed test to be an admitted observation.
Its compatibility filter can exclude hypotheses that would not have produced the failure.
The same record does not discharge acceptance and can keep a named check pending.
A model can therefore become more informative while its acceptance report becomes less favorable.
That is a sensible description of learning from a failure.

The current CLI reports evidence-conditioned hypothesis and observation counts.
The public core also exposes the evidence-interaction method.
The CLI does not automatically search for an optimal experiment or claim that its declared cases cover real program behavior.

The relationship to entropy
----------------------------

If a future measurement supplies a probability distribution over the distinct outcomes, Shannon entropy becomes available.
It would sum the negative probability-weighted logarithms rather than merely count labels.
With N possible outcomes its maximum is log2 N, attained by a uniform distribution.
The current runtime has no basis for choosing those probabilities, so it reports capacity and keeps the interpretation narrower.

This restraint is useful for software evidence.
A finite universe often begins as an explicit set of requirements examples or mutation hypotheses, not a representative random sample of production behavior.
Treating it as a calibrated probability distribution would add a claim that the input does not support.
The exact finite calculation is already useful without that claim.
