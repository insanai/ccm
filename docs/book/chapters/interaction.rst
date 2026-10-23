Shared work and additional joint work
======================================

Two changes can share a check and still create another check that neither needed alone.
A single net number can conceal both effects.
The calculus therefore keeps shared and additional joint work before taking their difference.

We compare two disjoint seed batches S and T in a fixed model.
The implementation rejects overlapping batches in this comparison API.
Standing checks are removed from the interaction terms so that work present before either batch is not mistaken for sharing created by the edits.

Let H contain nonstanding checks active for both batches.
Let J contain checks active for their union but for neither batch separately.
Their weights are the shared and additional joint demand.

.. math::

   H=(O(S)\cap O(T))\setminus O(\varnothing),
   \qquad J=O(S\cup T)\setminus(O(S)\cup O(T)).

The interaction report returns both sets and their sums.
It also returns the signed net quantity J minus H in policy units.

.. math::

   \begin{aligned}
   \Delta R&=R(S\cup T)-R(S)-R(T)+R(\varnothing)\\
   &=\sum_{q\in J}w_q-\sum_{q\in H}w_q.
   \end{aligned}

A proof by contribution
-----------------------

We can prove the identity one check at a time.
A standing check contributes once to every required-demand term and therefore contributes zero after the baseline correction.
A nonstanding check active in only one separate batch remains active for the union and contributes zero.
A check active in both separate batches contributes one copy for the union and loses two separate copies, leaving minus its weight.
A newly joint check contributes only for the union, leaving plus its weight.
Positive activation rules exclude a check that is active separately but absent from the union.
These cases exhaust the possibilities.

The proof describes required-demand activation.
It does not claim that evidence transport or human execution will have the same interaction.
A reused test, a conflict, a shared environment setup, and a repeated agent attempt can add further resource interactions that need their own model.

The checkout calculation
-------------------------

Pricing alone needs pricing, compatibility, and the standing payment check, for three units.
Inventory alone needs stock, compatibility, and payment, also three units.
Their union needs all five checks, for seven units.
The empty batch has the standing payment unit.

The baseline-corrected difference is therefore seven minus three minus three plus one, which is two.
We can also read it as three joint units minus one shared unit.
The combined checkout test supplies the joint work, while compatibility supplies the shared work.
Payment supplies neither term in the normalized interaction.

A zero net interaction would not establish independence.
A shared check of weight one and a newly joint check of weight one cancel numerically while both mechanisms remain present.
The report retains their names so an expert can inspect this case.

What the CLI compares
----------------------

For several CLI seeds, the current interaction report compares the first ordered seed with the remaining seeds.
It is not an exhaustive report of every possible partition or every pairwise interaction.
The public core method can compare other declared disjoint batches.
The technical report and JSON identify the split used by the CLI.

A scheduling decision needs more than the sign of this number.
Batching may share setup, delay feedback, concentrate review, or create integration work outside the declared checks.
The interaction calculation supplies an inspectable part of that decision.
It becomes valuable when we keep the named sets beside the signed sum.
