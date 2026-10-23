Experiments we can reproduce
==============================

A performance claim begins with a workload, an executable, and a measurement procedure.
The name of a competitor is not enough.
If one tool reads lockfiles and the other excludes them, their runtimes describe different work even when their tables look similar.
We align input before interpreting the timing.

The recorded Ensodoc-derived experiment uses 821 regular files, 187,402 physical lines, and 7,754,367 bytes.
A manifest records relative paths, byte counts, and SHA-256 hashes.
The source checkout's revision and the measured CCM 0.4.0 release executable hash are retained in ``docs/performance.rst``.
The snapshot contains an empty .git directory as a repository boundary so it does not inherit the enclosing project's ignore policy.
It is a local benchmark fixture, not source copied into published benchmark data.

The comparison uses installed scc 4.1.0.
Both tools produce terminal output into a null sink, with cost models disabled and scc's optional traditional complexity calculation disabled.
The driver checks equal file, byte, and physical-line totals before admitting a named corpus.
It does not force their code/comment classifications to agree.
Those classifications retain the language-policy differences explained earlier.

.. code-block:: sh

   cargo build --release
   cargo xtask bench 30 target/bench-ensodoc-v1

Startup is part of the question
-------------------------------

We measure a monotonic wall clock around the child process.
The observation includes startup, traversal, reading, counting, aggregation, and terminal rendering.
That is the practical operation a developer invokes.
A microbenchmark of a single scanner loop would answer a narrower question.

The driver performs two warm-ups per tool and then thirty paired measurements with alternating order.
For each pair it divides scc time by CCM time.
The summary uses the median of those paired ratios, which need not equal the ratio of the separate medians.
It bootstraps the paired ratios five thousand times with a fixed deterministic generator and reports the percentile interval of the median.
The procedure does not establish independence between successive warm samples.

The recorded median runtimes are 12.621 ms for CCM and 13.629 ms for scc.
The median paired speedup is 1.077x, with a 95 percent interval from 1.052x to 1.095x under that resampling procedure.
Thirty additional runs per tool record median peak resident memory of 3,886 KiB and 20,858 KiB respectively.
The raw time and memory CSV files are retained with the input manifest.

These are historical measurements of the identified 0.4.0 executable.
The present book and branding belong to 0.5.0.
We do not relabel the older experiment as a measurement of every later binary or of every language.
A new runtime claim needs its own executable and inputs.

Where the improvement came from
--------------------------------

The first implementation was slower on this workload.
Measurements pointed toward repeated ignore matching and filesystem work, not merely the arithmetic in the lexer.
Resolving a rule scope once, adding semantics-preserving glob fast paths, discovering available ignore files during enumeration, avoiding unused work-token collection, and normalizing generated headers once all reduced admitted work.
The final reported measurements follow that engineering process rather than an independent preregistered confirmation.

The complete selected Ensodoc inventory contains 828 files, including additional lockfiles and configuration data.
It counts all 56 Typst files and 123 Vue components with no coverage warnings and exit status zero.
That useful complete scan is distinct from the equal-input timed corpus.
A wider inventory and a faster aligned count are two different results, and both deserve their own evidence.

Correctness is another experiment
----------------------------------

Small exhaustive graph checks compare closure against a fixed-point implementation.
Lexical fixtures exercise named language constructs.
Catalog tests check recorded names and extensions.
Glob tests compare optimized paths against the full matcher and use Git-backed selection checks.
Report tests cover escaping, output limits, replacement behavior, and machine-readable components.

A passing test is evidence about the property it exercises.
It is not an empirical validation of the cost prior or a proof of semantic extraction completeness.
The book keeps mathematical proofs, finite test coverage, runtime experiments, and economic calibration in different paragraphs because they establish different things.
