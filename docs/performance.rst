Measured counting performance
==============================

We compare a named workload with the installed scc 4.1.0 rather than claiming superiority for every language or repository.
The counter and the finite assurance engine answer different questions.
A faster counter does not validate economic assumptions or semantic extraction.

CCM 0.6.0 on three workloads
-----------------------------

We measured the final 0.6.0 release executable on three inputs of increasing size.
Its SHA-256 is ``96ad1f202fa86bb883b7394dc91d571f9b23280644b1e8d24b7668b1fe86bdbf``.
The environment is the one described below, with the default of 16 workers, and the installed scc 4.1.0.

.. code-block:: sh

   cargo build --release
   cargo xtask bench 30
   cargo xtask corpus ~/projects/ensodoc target/bench-ensodoc-v1
   cargo xtask bench 30 target/bench-ensodoc-v1
   cargo xtask corpus ~/projects/zig target/bench-zig-v1
   cargo xtask bench 20 target/bench-zig-v1

``cargo xtask corpus SOURCE DESTINATION`` copies the files that both tools select into a fresh directory with an empty .git boundary.
It writes a manifest of paths, byte counts, and SHA-256 hashes next to the copy.
The driver checks equal file, physical-line, and byte totals before timing.

.. list-table:: CCM 0.6.0 and scc 4.1.0, warm, median of alternating pairs
   :header-rows: 1

   * - Workload
     - Files
     - Bytes
     - CCM
     - scc
     - Paired speedup, 95% interval
     - Peak RSS, CCM / scc
   * - Synthetic fixture, 30 pairs
     - 2,004
     - 6.9 MB
     - 14.420 ms
     - 23.543 ms
     - 1.607x [1.576, 1.649]
     - 10,000 / 20,792 KiB
   * - Ensodoc snapshot, 30 pairs
     - 821
     - 7.8 MB
     - 11.389 ms
     - 13.547 ms
     - 1.195x [1.152, 1.231]
     - 4,470 / 20,864 KiB
   * - Zig source tree, 20 pairs
     - 40,821
     - 411 MB
     - 423.597 ms
     - 330.208 ms
     - 0.780x [0.750, 0.806]
     - 70,618 / 107,916 KiB

A speedup above one means CCM was faster.
CCM is faster on the two small workloads, where startup and traversal dominate.
On the 411 MB Zig tree scc is about 22 percent faster, and CCM uses about 35 percent less memory.

The Zig tree is the source distribution of the Zig compiler and standard library, without a Git checkout.
Its manifest has SHA-256 ``bf1c53322fa6500e3e74cfc2eeb52dfed166a8f8204df37fb2356df623f0c7c3``; at 5 MB we do not commit it.
The Ensodoc manifest is `2026-10-23-ensodoc-input.csv <benchmarks/2026-10-23-ensodoc-input.csv>`_.
The raw samples are in ``docs/benchmarks``:

* fixture: `timing <benchmarks/2026-10-23-ccm-0.6-fixture-counting.csv>`_, `memory <benchmarks/2026-10-23-ccm-0.6-fixture-memory.csv>`_;
* Ensodoc: `timing <benchmarks/2026-10-23-ccm-0.6-ensodoc-counting.csv>`_, `memory <benchmarks/2026-10-23-ccm-0.6-ensodoc-memory.csv>`_;
* Zig: `timing <benchmarks/2026-10-23-ccm-0.6-zig-counting.csv>`_, `memory <benchmarks/2026-10-23-ccm-0.6-zig-memory.csv>`_.

Where the time goes
~~~~~~~~~~~~~~~~~~~

On the Zig tree, traversal alone takes 0.11 s; it took 0.29 to 0.42 s before we stopped calling ``stat`` on each directory entry.
We then varied the worker count of both tools.

.. list-table:: Zig tree, median elapsed time by thread count
   :header-rows: 1

   * - Threads
     - 4
     - 8
     - 16
   * - CCM
     - 0.63 s
     - 0.44 s
     - 0.37 s
   * - scc
     - 0.56 s
     - 0.38 s
     - 0.31 s

Both tools gain about the same factor from more threads.
The remaining gap is the cost per byte of the lexer, which a different scheduler such as rayon would not change.
We therefore keep standard-library threads.

One core at a time
~~~~~~~~~~~~~~~~~~

More threads hide cost; they do not remove it.
After 0.6.0 we therefore also compared the tools on one worker and pinned to one core, and separated the time spent in our code (user) from the time spent in the kernel (system).
Both tools read the same files from the same ZFS file system, so they meet the same file system costs.

.. list-table:: Zig tree, one worker, seconds
   :header-rows: 1

   * - Build
     - Wall, one core
     - User
     - System
   * - CCM 0.6.0
     - 2.44
     - 1.01
     - 1.41
   * - CCM after the changes below
     - 1.90
     - 0.90
     - 0.98
   * - scc 4.1.0
     - 1.90
     - 1.09
     - 0.80

A harness that first loads every file into memory isolates the lexer: it counted the 411 MB in 1.01 s with 0.6.0 and in 0.63 s after the changes, 650 MB per second on one core.
Three changes made the difference, and none changes a count.

* Inside a comment or string the lexer jumps to the next byte that can end it, instead of testing every byte.
  The search tests eight bytes at a time in a ``u64`` (SIMD within a register) and skips 32-byte blocks with a test that the compiler turns into vector instructions; both are safe, portable Rust.
* A file is checked for NUL bytes once instead of twice, and the generated-header test looks up candidate bytes in a table.
* The walker hands files to workers in batches of 16.
  One handoff per file cost a thread wake-up per file; Go's scheduler switches goroutines without the kernel, so scc never paid it.

The remaining difference in system time is mostly deliberate.
CCM reads the size and time of each file again after reading it, to detect a file that changed while we counted it.
Without that check CCM took 1.75 s on one core, but we keep it, because an accurate count comes first.
The output of every tree we measured, about 85,000 files, is byte-identical before and after these changes.

Classification agreement
~~~~~~~~~~~~~~~~~~~~~~~~

Equal inputs do not imply equal classifications.

.. list-table:: Line classes on the equal-input corpora
   :header-rows: 1

   * - Corpus
     - Tool
     - Code
     - Comments
     - Blank
   * - Ensodoc
     - CCM
     - 155,962
     - 16,437
     - 15,003
   * - Ensodoc
     - scc
     - 155,175
     - 17,920
     - 14,307
   * - Zig
     - CCM
     - 7,734,377
     - 1,396,038
     - 1,074,261
   * - Zig
     - scc
     - 7,768,611
     - 1,395,001
     - 1,041,064

We did not assume either tool is right.
We checked the disagreements file by file against the files themselves.
In every case we examined, CCM matched a direct count.
For example, CCM and ``grep`` both find 755 Zig ``//`` comment lines in one sample where scc reports 279, and scc reads ``/*`` inside Typst raw text and backticks inside TypeScript regular expressions as delimiters.
CCM's blank count of each file equals its number of whitespace-only lines, with no mismatch in a 5,389-file sample and in all 40,821 Zig files.

Coverage
~~~~~~~~

CCM 0.6.0 counts these trees completely, with exit status zero: its own repository, the Zig tree, the local Cargo registry (29,740 files), ``/usr/include``, and the Python 3.13 standard library.
On ``/usr/share`` six files remain excluded with a coverage warning.
Three are prose tutorials with a source extension, such as ``tutor1.pl``, and three are rare Perl forms, such as a substitution delimited by a control character.
CMD 0008 records these as open questions.

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
CCM 0.6.0 uses one worker per hardware thread, capped at 16, so 16 on this host; earlier versions used eight. scc uses its installed defaults.
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
