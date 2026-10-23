:cmd-number: 0008
:cmd-title: Lexical policy v4 and measured coverage
:cmd-state: committed
:cmd-type: Standards Track
:cmd-authors: ["Vikrant Rathore", "Ronak Rathore", "Kanak Rathore"]
:cmd-created: 2026-10-23
:cmd-updated: 2026-10-23
:cmd-discussion: No external discussion URL assigned
:cmd-labels: ["counting","performance","licensing"]

Lexical policy v4 and measured coverage
=======================================

Abstract
--------

We change the counting policy from lexical-v3 to lexical-v4.
A whitespace-only line is now blank under every lexer, and a docstring that begins a line counts as a comment.
We replace the Typst scanner with a mode-stack lexer and add the literal rules that real projects need: Perl quote-like operators, POD, Odin raw strings, CMake brackets, Lisp quotes, TeX escapes, and single-line literal recovery.
A file that still cannot be counted is reported with the line where its unterminated construct began.
CCM now counts its own repository, the Zig source tree, the Cargo registry, and the Python standard library without a coverage warning.
We release CCM under the GNU Lesser General Public License, version 3 or later.

Motivation and scope
--------------------

CCM 0.5.0 reported its own repository as incomplete.
The Typst scanner reset its statement state at every line end, so the multi-line header of ``#let book(...) = {`` dropped back into markup, and the next ``rgb("#...")`` opened a string that never closed.
On another project an apostrophe in an Odin backtick string, ``message's``, opened a character literal and excluded the file.
A one-line fix for each report would not tell us how many such failures remain.
We therefore measured coverage on large, varied trees, fixed each class of failure at its cause, and recorded the result.

Comparisons with scc raised a second question.
When two counters disagree, either one may be wrong.
We checked the disagreements file by file instead of treating either tool as the reference.

Specification
-------------

Blank lines.
A line that holds only whitespace is blank, even inside a multiline string, heredoc, CDATA section, or block comment.
The Typst and declarative lexers already followed this rule; the native and markup lexers now follow it too.
The invariant is checkable without CCM: the blank count of a file equals its number of whitespace-only lines.

Docstrings.
A triple-quoted Python or Julia string that begins a line, after indentation only, is a docstring, and its lines count as comments.
A triple-quoted string inside an expression stays code.

Typst.
The lexer keeps a stack of frames: content blocks in markup mode, code blocks and groups, equations, and expressions embedded with ``#``.
An embedded statement (``let``, ``set``, ``show``, ``import``, ``include``) ends at a newline or ``;`` outside any delimiter.
A control expression (``if``, ``for``, ``while``) ends after a body block that no ``else`` follows.
Any other embedded expression is a chain of identifiers, fields, calls, and content arguments with no space between them.
Strings exist only in code and math; links such as ``https://a//b`` in markup are not comments.

Other languages.
Perl and Ruby documentation blocks (``=pod`` to ``=cut``, ``=begin`` to ``=end``) count as comments, and lines after ``__END__`` count as data.
Perl quote-like operators accept any delimiter, including bracket pairs, ``#``, ``,``, and ``=``, and a bare ``/`` opens a pattern only in operator position.
Odin backtick strings may span lines.
CMake bracket arguments ``[=[ ]=]`` and bracket comments ``#[[ ]]`` are recognized, and ``'`` is an apostrophe.
In Scheme and Clojure ``'`` quotes a form, and character literals such as ``#\"`` open no string.
TeX has no string literals, and ``\%`` is text.
A YAML ``#`` starts a comment only after whitespace.
A C++ raw string starts only at a token boundary or after ``L``, ``u``, ``U``, or ``u8``.
Shell and CMake escapes outside quotes, such as ``\"``, open no string.

Recovery.
In languages whose ordinary literals cannot cross a line end, such as C, Java, JavaScript, Python, and Go, an unclosed ``'`` or ``"`` closes at the line end unless a backslash continues the line.
Declarative catalog profiles apply the same rule.
An unterminated construct that remains at the end of a file is reported as, for example, ``Unterminated string that begins on line 233``.

Encodings and selection.
Text in a legacy encoding such as Latin-1 is counted by its ASCII structure; only NUL bytes make a file binary.
``LICENSE-MIT`` and similar notice variants count as text.
A ``.ccm`` file that begins with ``ccm-model`` is a CCM model; other ``.ccm`` files are C++ modules.
A file above ``--max-file-bytes`` is skipped with a notice and makes the report incomplete instead of stopping the scan.

Throughput.
Traversal takes file types from directory listings and no longer calls ``stat`` for each entry.
Workers read size and time from the open file, apply the byte budgets, and compare the metadata before and after the read.
The default worker count is the number of hardware threads, capped at 16.

Rationale and alternatives
--------------------------

We considered keeping the lexical-v3 blank rule, which matches scc for strings.
But scc counts a blank line inside a block comment as a comment, and inside a backtick string of a language it does not know as blank.
cloc counts every whitespace-only line as blank.
One rule for every lexer is easier to state, to test, and to check against the files themselves.

For docstrings every common counter we compared, cloc, tokei, and scc, counts them as comments.
A docstring is documentation by convention, so we follow that convention.

We considered skipping a file silently when its lexer fails.
That would make totals look complete when they are not.
We keep the incomplete report and exit status 2, and we make each failure rarer and easier to locate.

We considered a work-stealing thread pool such as rayon.
With equal thread counts CCM scales like scc, so the remaining gap is the cost per byte, which a scheduler does not change.
We keep standard-library threads.

Security and operational considerations
---------------------------------------

All new lexer state is bounded: delimiter and comment nesting stop at 256, and heredoc delimiters at 256 bytes and 16 pending.
Decoding a legacy encoding never panics; invalid sequences become replacement characters that no rule matches.
The serve command added to ``xtask`` binds to 127.0.0.1 only and refuses request paths that leave the site directory.

Backwards compatibility
-----------------------

Counts change for files with blank lines inside strings or comments, for Python and Julia docstrings, and for files that 0.5.0 excluded with a coverage warning.
Physical line totals do not change for files that 0.5.0 counted.
Reports state ``lexical-v4``, so a reader can tell which rules produced a number.
Without a model, human review now counts program files only; the project's own estimate fell from 4,450 to 1,475 USD.
The license changes from ``MIT OR Apache-2.0`` to ``LGPL-3.0-or-later``.

Acceptance criteria and implementation status
---------------------------------------------

Implemented in CCM 0.6.0.
Each rule above has a unit test in ``crates/ccm-scan``.

Coverage.
CCM counts its own repository, the Zig source tree (40,821 files), the local Cargo registry (29,740 files), ``/usr/include``, and the Python 3.13 standard library with exit status zero.
On ``/usr/share`` six of 9,612 files remain excluded: three prose tutorials with a source extension and three rare Perl forms.

Accuracy.
On every file of the Zig tree the blank count equals the number of whitespace-only lines.
Where CCM and scc disagreed, we checked the files directly, and CCM matched in each case we examined.

Throughput against scc 4.1.0, as the median paired speedup with its 95 percent interval:
1.607x [1.576, 1.649] on the 2,004-file fixture, 1.195x [1.152, 1.231] on the 821-file Ensodoc snapshot, and 0.780x [0.750, 0.806] on the 411 MB Zig tree.
CCM's median peak memory was lower on all three, for example 70,618 KiB against 107,916 KiB on the Zig tree.
``docs/performance.rst`` records the method, the executable hash, and the raw samples.

Open questions
--------------

Perl decides between a pattern and a division from the previous token, which may sit on an earlier line.
Our line-local rule fails on rare code, such as a substitution delimited by a control character.
Text files with a source extension, such as tutorials named ``tutor.pl``, still produce warnings, because their extension promises a language they are not written in.

Review history
--------------

2026-10-23: Proposed, implemented, and measured together; committed with 0.6.0.

References
----------

- ``docs/counting.rst``: the counting and selection policy.
- ``docs/performance.rst``: coverage and timing measurements.
- CMD 0004 and CMD 0006, which defined lexical-v2 and lexical-v3.
