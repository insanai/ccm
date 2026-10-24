:cmd-number: 0009
:cmd-title: Limits for large monorepos
:cmd-state: committed
:cmd-type: Standards Track
:cmd-authors: ["Vikrant Rathore", "Ronak Rathore", "Kanak Rathore"]
:cmd-created: 2026-10-24
:cmd-updated: 2026-10-24
:cmd-discussion: No external discussion URL assigned
:cmd-labels: ["counting","limits","coverage"]

Limits for large monorepos
==========================

Abstract
--------

We raise the default limits so that CCM counts a Linux kernel checkout without options.
We make the ignore-matching budget charge the work a match actually does, and grow it with the file limit.
We fix the Make, Shell, and YAML rules that excluded 27 of the kernel's files.
Every file that CCM counted before is counted exactly as before.

Motivation and scope
--------------------

CCM 0.6.0 stopped on Linux 6.12 before counting a file.
The kernel has 86,618 files and 1.48 GB in directories at most 13 levels deep, and its largest file, a generated register header, has 24 MB.
Three limits failed in turn.
The ignore matcher exhausted its 200 million steps, the 1.48 GB exceeded the 1 GiB snapshot budget, and the 24 MB header exceeded the 16 MiB file limit.
With the limits raised by hand, 28 files still failed to count.

The first failure was an accounting error, not a real cost.
A rule such as ``*.o`` is decided by comparing a two-byte suffix, but the matcher charged it the cost of its general table: the pattern length times the path length.
Across about 92,000 paths and the root ignore file's rules, these charges reached the budget while the actual work was small.

Specification
-------------

Defaults.
A scan may select 1,000,000 files, read 64 MiB per file and 64 GiB per snapshot, and descend 128 levels.
A report may take 256 MiB.

Memory.
Peak memory grows by about 700 bytes per counted file, for its record and path, so a million files need about 700 MB.
Each worker reads one file at a time into a reused buffer and releases a buffer above 4 MiB after use.
The snapshot byte budget therefore bounds work, not memory.

Ignore matching.
A literal rule costs one step when the lengths differ and the compared length otherwise.
A rule with a literal prefix and suffix costs their length, plus the path length when it has one wildcard.
Only a general pattern costs the pattern length times the path length.
The budget is 2,000 steps per selectable file, and at least 200 million.

Make.
Make has no string literals: it passes quotes to the shell as plain characters.
A ``#`` starts a comment.

Shell.
Arithmetic, ``((...))`` and ``$((...))``, is tracked by parenthesis depth, so ``$(( (a & (b << c))) ))`` and nested subshells such as ``((( cmd ) ))`` balance as the shell balances them.
Inside arithmetic ``<<`` is a shift.
Inside a double-quoted string, ``$(`` opens a command substitution in which quoting starts afresh, as in ``"$(echo "$i" | wc -c)"``; its closing parenthesis returns to the string.

YAML.
A quote opens a string only where a scalar begins: at the start of a value, after ``key:``, ``-``, or ``?`` and whitespace, or in a flow collection after ``[``, ``{``, ``,``, or ``:``.
Elsewhere it is text in a plain scalar, as in ``events':' Page received``.
A line that ends with a block scalar indicator, such as ``key: |`` or ``- >-``, opens a block.
Following lines that are indented deeper are its text, in which quotes open nothing.
A text line that starts with ``#`` counts as a comment, because such blocks usually embed a script, as in a CI ``run: |`` step.

Rationale and alternatives
--------------------------

We considered keeping small defaults and asking users of large trees to raise them.
But the snapshot byte budget does not protect memory, and the old file limit stopped short of common monorepos.
A default that fails on the Linux kernel is a defect, not a safeguard.
The limits remain, so a mistaken root such as ``/`` still stops with an explained error.

For text in YAML block scalars we considered counting every line as content, as the YAML specification reads it.
That would move comment lines of embedded CI scripts to content and change counts that scc, cloc, and CCM 0.6.0 agree on.
We keep them as comments; the block rule only stops quotes in the text from opening strings.

Backwards compatibility
-----------------------

On CCM's own repository, the Zig source tree, the Ensodoc snapshot, the Cargo registry, ``/usr/include``, ``/usr/share``, and the Python 3.13 standard library, the output is byte-identical to CCM 0.6.0.
Only files that 0.6.0 could not count now have counts.
The policy identity stays ``lexical-v4``.

Acceptance criteria and implementation status
---------------------------------------------

Implemented after CCM 0.6.0, with unit tests for each rule and for the matching budget.
On Linux 6.12 CCM counts 82,115 files in about 1.1 seconds with 16 workers and 135 MB of peak memory, and with one worker in about 5 seconds and 57 MB.
The blank count of every counted file equals its number of whitespace-only lines.
The totals agree with scc 4.1.0 to within 0.3 percent: 29,655,627 code, 4,454,557 comment, and 4,911,587 blank lines against 29,727,380, 4,465,455, and 4,906,620.

Open questions
--------------

One kernel file, ``arch/m68k/ifpsp060/src/isp.S``, is still excluded.
Its m68k assembly uses ``#`` comments that contain ``/*``.
The catalog's assembly profile knows only ``;`` comments, and in a ``.S`` file a ``#`` line can also be a preprocessor directive such as ``#define``.
A rule that tells them apart needs more evidence than one file.

Review history
--------------

2026-10-24: Proposed, implemented, and measured together.

2026-10-24: Corrected the memory per file from an estimate of 330 bytes to the 700 bytes measured on Chromium; CMD 0010 counts ``isp.S``.

References
----------

- ``docs/counting.rst``: the counting policy and its bounds.
- CMD 0008, which defined lexical-v4.
