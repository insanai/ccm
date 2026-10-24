Experiments we can reproduce
==============================

A performance claim begins with a workload, an executable, and a measurement procedure.
The name of a competitor is not enough.
If one tool reads lockfiles and the other excludes them, their runtimes describe different work even when their tables look similar.
We align input before interpreting the timing.

We measure on three large source releases: Linux 6.12, FreeBSD 15.1, and Chromium 156.
For each tree, ``cargo xtask corpus`` keeps the files that CCM, scc, and cloc all select and copies them into a fresh directory with an empty .git boundary, so no enclosing ignore policy applies.
A manifest records relative paths, byte counts, and SHA-256 hashes.
The copies and manifests stay on the measuring machine; we do not commit other projects' source.

The comparison uses the binary built from the CCM 1.0.0-beta.1 tag, scc 4.1.0, and cloc 2.04.
Each tool writes its terminal report into a null sink, with cost models and scc's optional complexity calculation disabled.
The driver checks equal files and physical lines for all three tools, and equal bytes for CCM and scc, before admitting a corpus.
It does not force their code and comment classifications to agree.

.. code-block:: sh

   cargo xtask corpus --ccm CCM SOURCE target/bench-NAME
   cargo xtask bench --ccm CCM --cloc-runs 3 20 target/bench-NAME

Startup is part of the question
-------------------------------

We measure a monotonic wall clock around the child process, and GNU time reports CPU time and peak memory for the same run.
The observation includes startup, traversal, reading, counting, aggregation, and terminal rendering.
That is the practical operation a developer invokes.
A microbenchmark of a single scanner loop would answer a narrower question.

The driver performs two warm-ups per tool and then twenty CCM and scc pairs with alternating order.
For each pair it divides scc time by CCM time.
The summary uses the median of those paired ratios, which need not equal the ratio of the separate medians.
It bootstraps the paired ratios five thousand times with a fixed deterministic generator and reports the percentile interval of the median.
cloc takes tens of seconds per run, so we time it three times.
The procedure does not establish independence between successive warm samples.

Three source releases with CCM 1.0.0-beta.1
--------------------------------------------

.. list-table:: Median wall time on 16 threads (speedup above one: CCM faster than scc)
   :header-rows: 1

   * - Tree
     - Equal input
     - CCM / scc / cloc
     - Speedup over scc, 95% interval
   * - Linux 6.12
     - 76,410 files, 1.42 GB
     - 0.81 / 0.81 / 39.1 s
     - 0.976x [0.910, 1.019]
   * - FreeBSD 15.1
     - 75,386 files, 1.12 GB
     - 0.76 / 0.80 / 26.4 s
     - 1.056x [1.040, 1.059]
   * - Chromium 156
     - 215,906 files, 1.90 GB
     - 2.35 / 2.34 s / not yet run
     - 0.995x [0.993, 0.997]

CCM and scc take about the same wall time on all three trees, and cloc takes 30 to 50 times as long.
CCM uses about half of scc's peak memory or less: 117 against 325 MiB on Linux, and 114 against 249 MiB on Chromium.
On Chromium it also needs 44 percent less CPU time for the same wall time.
We have not yet timed cloc on Chromium; with 16 processes it may need more memory than our 12 GiB machine has.

Pinned to one core with one worker, CCM and scc spend the same user time on Linux, and scc is about five percent faster in wall time.
The difference is system time: after counting a file, CCM reads its size and time again to detect a change during the read.
We keep that check, because an accurate count comes before a faster one.

Equal input does not mean equal classification, and we take no tool as the reference.
A blank line has a definition that needs no counter, so the driver counts whitespace-only lines itself.
CCM's blank count equals that number exactly on all three trees; scc's is lower by 15,788 to 45,412 lines and cloc's by 570 to 745.
``docs/performance.rst`` records the executable hash, the environment, and every table.

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
