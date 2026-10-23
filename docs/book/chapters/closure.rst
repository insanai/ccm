The mathematics of affected parts
==================================

The finite calculus begins with a graph and a question.
Which declared parts can be reached by invalidation from the parts we changed?
We do not need a probabilistic model to answer that question once the graph is fixed.
We do need to state what its edges mean.

Let A be a finite set of artifacts and E a directed dependency relation.
An edge from a to b means that changing a may invalidate an assumption used by b.
It is a conservative propagation declaration, not a statement that b's source must be rewritten.
Let S be the set of changed seeds.

We define the affected closure by reachability, including paths of length zero.
Every seed therefore belongs to its own closure.

.. math::

   D(S)=\{b\in A\mid \text{some }a\in S\text{ reaches }b\}.

For the checkout example, the seeds are pricing and inventory.
Each reaches itself and checkout.
There is no declared path to payment, so payment is not affected by this calculation.
An omitted real-world dependency would make the model incomplete; it would not make the reachability algorithm nondeterministic.

Three closure laws
-------------------

The first law is extensivity: S is a subset of D(S).
The zero-length path proves it immediately.
The second is monotonicity: adding seeds cannot remove an already available path.
The third is idempotence: starting from every already reachable vertex cannot reach anything that was not reachable from an original seed.
Concatenating the two paths supplies the proof.

.. math::

   \begin{aligned}
   S &\subseteq D(S),\\
   S\subseteq T &\Rightarrow D(S)\subseteq D(T),\\
   D(D(S)) &=D(S).
   \end{aligned}

These laws explain the word closure.
They are properties of ordinary graph reachability, not a new discovery about software effort.
Their usefulness comes from giving later activation and evidence rules a precise set to inspect.

A fixed graph also gives a union law.
A path begins at a seed in the first batch or at a seed in the second batch.
Those two possibilities exhaust the union.

.. math::

   D(S\cup T)=D(S)\cup D(T).

This equality does not say that the required checks of a combined task are just the union of the separate checks.
A check can require affected parts from both sides before it activates.
The next chapter makes that distinction explicit.

Breadth-first evaluation
------------------------

The implementation validates every artifact index and builds adjacency lists.
It sorts and deduplicates neighbors, sorts and deduplicates seeds, marks each seed seen, and enters it into a queue.
When a vertex leaves the queue, unseen dependents are marked and appended.
The first discovery records a predecessor.

A predecessor chain is a witness that explains why a part was reached.
With unit graph edges and breadth-first traversal, first discovery gives a shortest edge-count path from the seed set.
Stable ordering makes the selected witness deterministic when several shortest paths exist.
The witness is not a causal proof that a dependent will fail; it is a reproducible path in the declared invalidation graph.

After adjacency construction, the traversal uses work proportional to the number of artifacts and edges.
Sorting contributes the corresponding neighbor-ordering cost before traversal.
Storage includes adjacency, a seen vector, a queue, and one optional predecessor per artifact.
The evaluator checks explicit object, edge, and membership budgets before admitting this work.

What the closure does not contain
---------------------------------

A closure is not a patch, a schedule, or a risk probability.
It can include an unchanged file whose assumptions deserve review.
It can omit an important component if the author omitted its edge.
It is exact as a finite inference and conditional as a model of a real system.

This separation gives us a useful design rule.
A source adapter may improve the graph, but it must not silently change the graph's meaning.
A lexical call-like token, a compiler-resolved dependency, and an explicitly reviewed assumption edge have different evidential strength.
A future typed graph should preserve those distinctions instead of merging them into one unexplained complexity score.
