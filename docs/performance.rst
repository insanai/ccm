Measured counting performance
==============================

We compare a named workload with the installed scc 4.1.0 rather than claiming superiority for every language or repository.
The counter and the finite assurance engine answer different questions.
A faster counter does not validate economic assumptions or semantic extraction.

Ensodoc snapshot with CCM 0.4.0
--------------------------------

We repeated the comparison on an Ensodoc-derived snapshot after fixing its Typst failures and adding Vue counting.
The source checkout was at revision ``a8efa4d66686b4bae52e26b533b5b6afd7e81db3``.
The release executable SHA-256 was ``7f05b7d12d4982d52b0b568d6301bd36446245d150e30bb000090c60dc3238f6``.
We used the environment described below and the installed scc 4.1.0.

The snapshot contains the same 821 regular files selected by scc with ``--count-ignore`` that CCM also recognizes.
It has 187,402 physical lines and 7,754,367 bytes.
The copy retains relative paths and an empty .git directory as a repository boundary, so it does not inherit the enclosing CCM repository's ignore rules.
The input manifest records each path, byte count, and SHA-256 in `2026-10-23-ensodoc-input.csv <benchmarks/2026-10-23-ensodoc-input.csv>`_.
We do not copy another repository's source into committed benchmark data.
The local corpus is under ``target/bench-ensodoc-v1``.
The driver checks equal file, physical-line, and byte totals before timing.
Comment/code classifications differ by the documented language policies and are not asserted equal.

.. code-block:: sh

   cargo build --release
   cargo xtask bench 30 target/bench-ensodoc-v1

.. code-block:: text

   ccm --no-cost --sort name PATH
   scc --count-ignore --no-config --no-cocomo --no-complexity --no-size --sort name PATH

Thirty alternating warm pairs measured median CCM runtime at 12.621 ms and scc runtime at 13.629 ms.
The median paired speedup was 1.077x, with a paired bootstrap 95 percent interval of 1.052x to 1.095x.
Thirty additional runs per tool measured median peak RSS of 3,886 KiB for CCM and 20,858 KiB for scc.
The timing CSV is `2026-10-23-ensodoc-counting.csv <benchmarks/2026-10-23-ensodoc-counting.csv>`_.
The memory CSV is `2026-10-23-ensodoc-memory.csv <benchmarks/2026-10-23-ensodoc-memory.csv>`_.
These are release-build, equal-input measurements for one snapshot, not an all-language speed guarantee.
The first implementation was slower on this workload; the final measurements follow optimization of ignore matching, directory metadata lookups, optional work-token collection, and generated-header scans.

A normal scan of the complete selected Ensodoc tree counted 828 files, including additional lockfiles and configuration data, with no coverage warnings and exit 0.
It counted all 56 Typst files and all 123 Vue files.
That broader inventory is not the timed equal-input corpus.
The tool's default file-selection policies differ, so we do not compare those unequal inventories as though they were identical work.

Earlier 0.2 fixture reproduction
--------------------------------

The following sections retain the recorded 0.2 fixture experiment and its original commands.
They are historical measurements of that binary, not measurements of the current default AI estimate.
The current driver additionally passes ``--count-ignore`` to scc; the synthetic fixture has no ignore files, so this does not alter its declared counts.

Reproduction
-------------

.. code-block:: sh

   cargo build --release
   cargo xtask bench 30

The Rust driver is ``tools/xtask/src/bench.rs``.
It requires an installed scc executable; runtime CCM counting does not invoke it.
The driver creates and checks the bytes of a deterministic fixture under ``target/bench-fixture-v1``.
Unexpected fixture content is an error.

The fixture contains 2,000 small Rust files and four large Rust files.
Its hand-declared totals are 2,004 files, 206,008 code lines, 2,004 comment-only lines, and 2,004 blank lines.
Before timing, the driver parses both tools' CSV reports and requires those exact totals.
This is an equivalent-count fixture for these declared constructs, not a claim of general Rust or language equivalence.

We use these timed arguments, with the same fixture path:

.. code-block:: text

   ccm --no-cost --sort name PATH
   scc --no-config --no-cocomo --no-complexity --no-size --sort name PATH

Both tools render their terminal reports to a null stdout sink.
We include process startup, traversal, reading, counting, aggregation, and rendering.
We disable cost models and optional traditional complexity so neither is charged for an analysis the other does not perform in this track.
The human report layouts differ and remain part of the observed end-to-end runtime.
Machine-readable count conformance is checked separately before timing.

We run two warm-up invocations per tool, followed by 30 paired measurements with alternating tool order.
Elapsed time uses a monotonic wall clock around the child process.
For each pair the speedup is the scc time divided by the CCM time.
The reported estimate is the median paired ratio, which need not equal the ratio of separate medians.
We resample 30 paired ratios with replacement 5,000 times using a fixed deterministic generator and report the percentile interval of their medians.
The generator seed and percentile indices are in the driver.
These measurements followed implementation tuning and are engineering evidence, not an independent preregistered confirmation.
We have not established temporal independence of successive samples.

Thirty additional runs per tool measure peak resident memory with GNU time's percent-M output.
Those invocations are not included in the elapsed-time samples.
Their order is CCM then scc in each memory pair.
We retain raw runtime and memory samples below.

Environment
------------

We ran on 2026-10-23 with an AMD Ryzen 7 5800H, 16 logical processors, x86_64 Linux 7.0.0-38-generic, and Rust 1.98.1.
CCM uses the workspace release profile: optimization level 3, thin LTO, one codegen unit, and stripped debug information.
The default scanner uses eight workers on this host; scc uses its installed defaults.
Inputs and page caches were warm after the declared warm-up.
We did not clear caches, pin CPU frequency, or claim cold-storage behavior.
The binary identities and final measurement values follow in the recorded results section.

Results
--------

.. list-table:: Final warm fixture results
   :header-rows: 1

   * - Measurement
     - CCM 0.2.0
     - scc 4.1.0
   * - Median elapsed time
     - 20.392 ms
     - 24.082 ms
   * - Median peak RSS
     - 9,596 KiB
     - 20,562 KiB

The median paired speedup is 1.174, with a descriptive 95 percent bootstrap interval of [1.131, 1.228].
This is about 17 percent faster and 53 percent lower median peak RSS on this fixture.
The binary SHA-256 identities are ``ba38d81514bdb7c6edca314630686629748705e9b86893451859a6943b3cf0e7`` for CCM and ``4763d7437218d2d642e2f2a4f7f078e044aa67e0e15b90bab6c70d5fc0ab4022`` for scc.

* `Raw elapsed-time pairs <benchmarks/2026-10-23-counting.csv>`_.
* `Raw peak-RSS pairs <benchmarks/2026-10-23-memory.csv>`_.

Limits and review findings
---------------------------

The first implementation was materially slower on this fixture.
Literal ignore-rule fast paths, ordinary-byte scanning, fair file scheduling, overlapped traversal, and release optimization improved its measured runtime.
The final reported run is retained rather than selecting the best individual sample.

We also reproduced the maintainer's Guidedog scan at source revision c20d9b30d81fbbf53331e45d7759d3d9ec66d8b0.
The initial CCM counted 1,653 files and emitted 52 normal symlink notices.
The expanded policy counts 1,911 files in the inspected working tree, with no lexical warnings, and summarizes those symlinks without treating them as errors.
The file-count increase reflects broader registered policies and selection changes, not proof of greater semantic correctness.
Working-tree fixtures and generated inputs remain part of that inspection; the source revision alone does not identify every byte.
This inspection is not the equivalent-count timed track.

We do not infer a universal speedup, complete feature parity, or complete language conformance from these measurements.
In particular, COCOMO/LOCOMO, Git history views, MCP serving, full embedded-language parsing, and universal parser extraction are not the CCM implementation boundary.
The meaningful additional results are finite change propagation, check activation, transported evidence, declared behavioral outcomes, and explicit acceptance-cost scenarios.
Their predictive usefulness and source extraction soundness require separate evidence.
CMD 0002's primary GuideDog performance gate and broader corpus gate remain open.
