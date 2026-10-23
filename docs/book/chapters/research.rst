Where the calculus can grow
============================

A research program needs more than a list of attractive mathematical subjects.
Each candidate must define an observable object, a computable quantity, a computational cost, and a claim that an experiment can reject.
The design record applies that rule to description length, topology, observation abstraction, path entropy, and typed dependency graphs.
The present executable implements bounded finite instances of the calculus, not every proposed extraction or analysis.

The distinction protects the program from its own ambition.
A novel-looking scalar can be easy to calculate and difficult to interpret.
We should first ask which decision the scalar would improve and which cheaper baseline it must outperform.
The following constructions are research candidates under that discipline.

Description length and change description
-----------------------------------------

A repeated source file is easy to compress.
That observation does not establish that the file is easy to change safely.
A small edit to a repeated schema can invalidate many consumers, while a larger isolated edit can require little cross-boundary review.
Description and propagation therefore remain separate coordinates.

A reproducible compression feature must name its canonicalization, codec, framing, dictionary, and resource limits.
If a dictionary is trained, its cost and identity must remain visible.
A two-part description charges for the dictionary as well as the encoded units.

.. math::

   L_{\mathrm{two}}(X,D)=L(D)+\sum_{x\in X}L_c(x\mid D).

A conditional patch codec can similarly describe a new source snapshot given an earlier one.
The reconstruction rule must be explicit: the patch and prior snapshot must recover the claimed new bytes or canonical representation.
A short description is then a precise coding result under that scheme.
It is not exact Kolmogorov complexity, intrinsic semantic difficulty, or proof that review will be short.

A useful experiment would compare these lengths with token counts, clone features, and source changes at fixed time and memory budgets.
Training a dictionary on the evaluated unit would contaminate held-out novelty.
Random identifiers, generated boilerplate, minification, small-file overhead, and compact effectful algorithms are necessary controls.
The current CLI does not run these codecs as a hidden complexity engine.

A geometry before a topological interpretation
-----------------------------------------------

For a topological experiment we first need a finite state sample and an encoder into a specified metric space.
The chosen metric determines which states are near one another.
A Vietoris-Rips construction can connect points whose distances are within a scale and add simplices for compatible cliques.
Persistent homology can then ask which features remain across scales.

A persistent loop in this chosen space is not automatically a bug, race, or architectural cycle.
The interpretation requires a justified relationship between the state encoder, the task observation, and the feature being measured.
Changing the encoder can change the geometry while leaving the program's externally relevant behavior unchanged.
A research adapter must expose that dependence rather than presenting an unlabeled topology score.

The computational cost is also part of the candidate.
A full clique complex can grow exponentially with the number of sampled states.
A bounded implementation needs declared dimension, sample, simplex, and memory budgets, together with an explicit approximation policy.
A stability theorem applies under its own metric and filtration assumptions; it does not validate an arbitrary source-derived geometry by association.

Evidence persistence is a different construction.
We can examine the unresolved task distinctions after successive evidence acquisitions at an increasing verification budget.
With a fixed universe and retained compatible evidence, the surviving observation set can only shrink.
An area under its log-cardinality curve would have units such as bits-times-hours when the budget is measured in hours.
It would describe the persistence of unresolved distinctions, not their business value or probability of failure.
Changing context, invalidating evidence, or admitting an inconsistent intersection requires separate treatment.

Observers and composition
--------------------------

An observer maps programs or hypotheses to the distinctions a task cares about.
Two objects are equivalent for that observer when they receive the same observation.
The quotient groups those indistinguishable objects.
Our finite behavioral model already uses an explicit version of this idea through repeated observation labels.

Category-theoretic language can help state how observation abstractions should respect composition.
Its useful question is whether an intended mapping preserves a specified operation or diagram.
The existence of a mapping alone establishes no program-correctness result.
A certificate transported across an abstraction needs the corresponding preservation assumptions and proof obligations.
The present runtime does not infer those assumptions from an arbitrary dependency edge.

This research direction is especially relevant at service boundaries.
A boundary contract can hide internal implementation details while exposing the outcomes a dependent task must distinguish.
A successful abstraction could reduce the model without losing those distinctions.
The validation obligation is to show what the abstraction preserves for the declared task, not to claim universal semantic compression.

Path observations and distributions
------------------------------------

A bounded trace experiment can record execution observations up to a declared horizon.
Counting possible observed words gives a capacity measure.
Assigning empirically justified probabilities enables Shannon entropy or entropy-rate questions.
The horizon, state abstraction, input process, and observation policy must all remain explicit.

We cannot infer uniform input probabilities from source syntax.
Nor can an observed low-entropy trace establish worst-case safety.
A useful comparison would examine path capacity, empirical entropy, branch counts, and transition coverage under the same task and horizon.
The current CLI reports no execution-derived distribution and performs no such execution by default.

Typed graphs before graph spectra
----------------------------------

A code property graph can combine syntax, control-flow, value-dependence, call, effect, and certificate-support relations.
These relation types must remain distinguishable.
An AST edge is not a data-flow proof, and a syntactic call is not necessarily a resolved runtime dependency.
A future adapter should retain source provenance and a policy for unresolved boundaries.

A spectral experiment can choose a nonnegative symmetric adjacency matrix from selected layers and declared weights.
With degree matrix D and adjacency W, the ordinary Laplacian is D minus W.
Its quadratic form measures differences across the chosen weighted edges.

.. math::

   L=D-W,\qquad
   f^{T}Lf=\frac12\sum_{i,j}W_{ij}(f_i-f_j)^2.

This identity follows by expanding the square and using symmetry.
It says something exact about that matrix model.
It does not make a weak graph cut intrinsically good or bad software design.
Symmetrizing also discards direction, so the original directed relations must remain available for invalidation and effects.

Degree summaries, cut conductance, and direct closure are cheaper baselines than a full eigendecomposition.
A solver adds tolerances, convergence, residual checks, and input-dependent iteration costs.
Nonisomorphic graphs can share spectra, so a spectral feature is not a unique program identity.
These costs and ambiguities belong in the research gate rather than outside the headline result.

The gate back into the runtime
-------------------------------

A candidate earns an implementation slot by answering a specified question with a bounded method and improving a held-out decision beyond simpler measurements.
Sound abstract interpretation, slicing, compositional logic, separators, and factorized finite inference can all contribute under their stated contracts.
None automatically supplies every guarantee to every language in the lexical registry.

The combined calculus is the research proposal: task-relative description, propagated impact, obligations, evidence transport, behavioral distinctions, and resource accounting with explicit units.
Reachability, partition lattices, coding theory, graph spectra, and queueing theory are established foundations.
The book does not claim their invention.
The experiment must determine whether the construction built from them is useful for software development.
