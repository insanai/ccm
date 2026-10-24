Measured counting performance
==============================

We measure CCM 1.0.0-beta.1 against scc 4.1.0 and cloc 2.04 on three large source releases: Linux 6.12, FreeBSD 15.1, and Chromium 156.
A benchmark describes one executable, one input, and one machine.
It is evidence about those, not a guarantee for every repository.
A faster counter also does not validate economic assumptions or semantic extraction.

Method
------

Equal input
~~~~~~~~~~~

The three tools choose files by different rules, so timing them on the same directory would compare different work.
``cargo xtask corpus`` therefore runs each tool on the source tree, keeps the files that all of them select, and copies those files into a fresh directory under ``target/``.
The copy has an empty ``.git`` directory as a repository boundary, so it inherits no ignore rules from the enclosing repository.
A manifest of each file's path, size, and SHA-256 sits next to the copy.
Neither the copy nor the manifest is committed; they hold other projects' files, and we delete each copy once its tree is measured.

Before timing, ``cargo xtask bench`` requires that the tools count the same files and the same physical lines, and that CCM and scc count the same bytes.
cloc reports no bytes.
The classification of a line into code, comment, or blank may differ, and we report those differences separately below.

Commands
~~~~~~~~

We measured the binary built from the ``v1.0.0-beta.1`` tag, with SHA-256 ``1a99fbd960c01a29ec6ba84607b526c4727542779628307660914335cdf8e4d6``.
For each tree, one at a time:

.. code-block:: sh

   cargo xtask corpus --ccm CCM SOURCE target/bench-NAME
   cargo xtask bench --ccm CCM --cloc-runs 3 20 target/bench-NAME

Each tool runs with its defaults, its terminal report goes to a null sink, and extras that the others do not compute are off:

.. code-block:: text

   ccm  --no-cost --sort name PATH
   scc  --count-ignore --no-config --no-cocomo --no-complexity --no-size --sort name PATH
   cloc --quiet --skip-uniqueness --processes=16 PATH

CCM and scc use one worker per hardware thread by default.
cloc uses one process unless told otherwise, so we give it 16, as many as the others use.
``--skip-uniqueness`` turns off cloc's search for duplicate files, which the other tools do not make, so that all three count the same files.

Procedure
~~~~~~~~~

After two warm-up runs of each, the driver times 20 pairs of CCM and scc runs in alternating order.
Wall time is a monotonic clock around the child process; GNU time reports its CPU time and peak resident memory in the same run.
For each pair the speedup is the scc time divided by the CCM time, so a speedup above one means CCM was faster.
We report the median paired speedup and a percentile interval from 5,000 bootstrap resamples with a fixed generator.
cloc takes tens of seconds per run, so we time it three times and report the median.
The caches were warm; we did not clear them or pin the CPU frequency.

Environment
~~~~~~~~~~~

We ran on 2026-10-24 on an AMD Ryzen 7 5800H with 16 logical processors and 12 GiB of memory, under Linux 7.0.0-38-generic, with the files on ZFS.
CCM was built with Rust 1.98.1 in the workspace release profile: optimization level 3, thin LTO, one codegen unit, and stripped debug information.

Results
-------

.. list-table:: The equal-input corpora
   :header-rows: 1

   * - Tree
     - Selected by CCM / scc / cloc
     - Common files
     - Physical lines
     - Bytes
   * - Linux 6.12
     - 82,405 / 82,175 / 76,645
     - 76,410
     - 37,448,550
     - 1.42 GB
   * - FreeBSD 15.1
     - 93,582 / 82,877 / 78,148
     - 75,386
     - 30,371,498
     - 1.12 GB
   * - Chromium 156
     - 429,002 / 215,949 / not yet run
     - 215,906
     - 48,029,383
     - 1.90 GB

.. list-table:: Wall time, median, all cores
   :header-rows: 1

   * - Tree
     - CCM
     - scc
     - cloc
     - Paired speedup over scc, 95% interval
   * - Linux 6.12
     - 0.808 s
     - 0.809 s
     - 39.1 s
     - 0.976x [0.910, 1.019]
   * - FreeBSD 15.1
     - 0.759 s
     - 0.802 s
     - 26.4 s
     - 1.056x [1.040, 1.059]
   * - Chromium 156
     - 2.348 s
     - 2.337 s
     - not yet run
     - 0.995x [0.993, 0.997]

On all three trees CCM and scc take about the same wall time: the largest difference is about five percent.
cloc takes 30 to 50 times as long.

.. list-table:: CPU time and peak memory, median, all cores
   :header-rows: 1

   * - Tree
     - CPU seconds, CCM / scc / cloc
     - Peak RSS, CCM / scc / cloc
   * - Linux 6.12
     - 10.1 / 10.2 / 242.8
     - 117 / 325 / 326 MiB
   * - FreeBSD 15.1
     - 8.9 / 10.7 / 218.2
     - 122 / 239 / 771 MiB
   * - Chromium 156
     - 18.8 / 33.5 / not yet run
     - 114 / 249 MiB / not yet run

CPU time is user plus system time over all threads.
On Chromium CCM needs 44 percent less CPU time than scc for the same wall time, and on every tree it needs about half of scc's memory or less.
For cloc, GNU time reports the largest single process, so its total memory over 16 processes is larger than the figure shown.

cloc on Chromium
~~~~~~~~~~~~~~~~

We have not yet measured cloc on Chromium.
On FreeBSD a single cloc process already reached 771 MiB, and Chromium's corpus is larger, so 16 processes may need more memory than this 12 GiB machine has.
Our first attempt with 16 processes did not finish.
We will measure cloc with fewer processes, or on a larger machine, and add the result here.
The CCM and scc figures above use the 215,906 files that both select; ``--no-cloc`` builds and times that corpus without cloc.

One core
~~~~~~~~

More threads hide cost; they do not remove it.
With ``--one-core`` the driver pins each tool to CPU 0 and gives it one worker: ``taskset -c 0``, CCM ``--jobs 1``, and cloc with one process.

.. list-table:: Linux 6.12 on one core, median seconds
   :header-rows: 1

   * - Tool
     - Wall
     - User
     - System
     - Peak RSS
   * - CCM
     - 4.66
     - 2.66
     - 1.98
     - 52 MiB
   * - scc
     - 4.42
     - 2.64
     - 1.73
     - 124 MiB
   * - cloc
     - 103.3
     - 97.0
     - 6.0
     - 329 MiB

On one core the two lexers cost the same user time, and scc is about five percent faster in wall time.
The difference is system time.
CCM reads the size and modification time of each file again after counting it, to detect a file that changed while we read it, and we keep that check because a correct count comes first.

What the counts say
-------------------

Equal input does not mean equal classification.
We do not take any of the three tools as the reference.
Instead we check each blank count against a definition that needs no counter: a line is blank when it holds only whitespace.
The driver counts such lines itself, in a few lines of Rust that read every file of the corpus; a byte-order mark at the start of a file is not content.

.. list-table:: Line classes on the equal-input corpora
   :header-rows: 1

   * - Tree
     - Tool
     - Code
     - Comments
     - Blank
     - Blank minus whitespace-only lines
   * - Linux 6.12
     - CCM
     - 28,340,894
     - 4,401,278
     - 4,706,378
     - 0
   * - Linux 6.12
     - scc
     - 28,373,030
     - 4,384,930
     - 4,690,590
     - -15,788
   * - Linux 6.12
     - cloc
     - 28,238,085
     - 4,504,657
     - 4,705,808
     - -570
   * - FreeBSD 15.1
     - CCM
     - 22,416,898
     - 4,548,146
     - 3,406,454
     - 0
   * - FreeBSD 15.1
     - scc
     - 22,430,505
     - 4,566,695
     - 3,374,298
     - -32,156
   * - FreeBSD 15.1
     - cloc
     - 22,381,269
     - 4,584,520
     - 3,405,709
     - -745
   * - Chromium 156
     - CCM
     - 35,744,193
     - 5,905,123
     - 6,380,067
     - 0
   * - Chromium 156
     - scc
     - 35,767,339
     - 5,927,389
     - 6,334,655
     - -45,412

The whitespace-only lines number 4,706,378 in Linux, 3,406,454 in FreeBSD, and 6,380,067 in Chromium.
CCM's blank count equals them exactly on every tree, as ``counting.rst`` proves it must.
scc and cloc count some whitespace-only lines, for example inside a string or a comment, as code or comment.
Because the physical lines agree, every whitespace-only line a tool leaves out of blank appears in its code or comment count instead.

Code and comment lines have no counter-free definition; they depend on each language's rules.
The code counts of the tools agree to within 0.5 percent on every tree, and their comment counts to within 2.7 percent.
``counting.rst`` states the rules CCM uses and the properties that its counts guarantee.

Coverage
--------

The corpora above hold only files that every tool can count, so they leave out what one tool counts and another does not.
On the whole trees CCM counts every Linux file, all but 18 FreeBSD files, and all but 8 Chromium files; CMD 0011 lists the exceptions.
The first table shows how many files each tool selects; ``git check-ignore`` rejects none of CCM's selection.

Limits
------

These are warm-cache measurements of one binary on one machine.
We have not established that successive samples are independent, and we did not measure cold storage.
The corpora are copies, so they leave out the cost of applying the trees' own ignore files.
A different machine, file system, or tree can change both the times and their ratio.
