:cmd-number: 0010
:cmd-title: Accuracy on Chromium
:cmd-state: committed
:cmd-type: Standards Track
:cmd-authors: ["Vikrant Rathore", "Ronak Rathore", "Kanak Rathore"]
:cmd-created: 2026-10-24
:cmd-updated: 2026-10-24
:cmd-discussion: No external discussion URL assigned
:cmd-labels: ["counting","coverage","languages"]

Accuracy on Chromium
====================

Abstract
--------

We counted the Chromium 156 source release and fixed what it exposed.
Two kinds of error appeared.
Some files could not be counted: 230 of 393,498.
Worse, some counted files had wrong counts without any warning: Objective-C was read as MATLAB, and assembly, Bazel, and Jinja comments were counted as code.
We add native scanners for assembly and Starlark, decide two shared extensions by content, and correct the JavaScript, Ruby, Shell, PowerShell, and Dockerfile rules.
Eight files remain uncounted, and none of them is valid source in the language its name suggests.

Motivation and scope
--------------------

Chromium is a monorepo of about 63 million program lines in more than thirty languages, much of it vendored from other projects.
It mixes languages within files: assembly passed through the C preprocessor, HTML in JavaScript template literals, Jinja templates that generate C++, and shell scripts in Dockerfiles.
A counter that is right on one-language projects can be wrong here, so Chromium tests the rules where they meet.

A failure to count is visible: the report says it is incomplete.
A wrong count is not.
We therefore compared the classification of every file with the previous build and checked each class of difference by reading the files.

Specification
-------------

Assembly.
A native scanner replaces the declarative profile, which knew only ``;`` comments.
A file does not say which assembler it targets, so we accept each dialect's comment where it cannot be code.
``//``, ``;``, and ``/* */`` are comments everywhere.
A ``#`` or ``@`` at the start of a line begins a comment, except in a preprocessor directive such as ``#define`` and in a MASM ``@@`` label.
After code, ``#`` and ``@`` begin a comment only between spaces, so ``#1``, ``foo@PLT``, and ``.type f, @function`` stay code.
Strings in ``"``, ``'``, and backticks end at their quote or the line end; ``$'a`` is a GNU as character constant.

Starlark.
Bazel files are Starlark, a Python dialect: ``BUILD``, ``BUILD.bazel``, ``WORKSPACE``, ``WORKSPACE.bzlmod``, ``MODULE.bazel``, ``.bzl``, ``.bazel``, ``.star``, and ``.sky``.
They follow the Python rules, so ``'`` strings and docstrings are recognized.
The catalog names ``Bazel`` and ``Starlark`` both select this policy.

Shared extensions.
The first 4 KiB of a ``.m`` or ``.tmpl`` file decide its language.
A ``.m`` file whose first nonblank line starts with ``//``, ``/*``, ``#``, or ``@`` is Objective-C; MATLAB files start with ``%`` or code.
A ``.tmpl`` file with a Jinja ``{%`` or ``{#`` tag is Jinja; other ``.tmpl`` files stay Go templates.

JavaScript and TypeScript.
A backslash escapes a character in a template literal, so an escaped backtick does not close it.
A ``${`` substitution holds JavaScript until its matching ``}``, nested templates included.
A ``/`` in regex position that does not close on its line is a division or JSX markup, such as ``<Item />``.
A ``!`` after an operand is a non-null assertion, so ``ratio! / 2`` divides.

Ruby, Shell, PowerShell, and Dockerfile.
Ruby heredocs ``<<ID``, ``<<-ID``, and ``<<~ID`` are recognized; an unquoted delimiter ends at the identifier, as in ``<<~EOS.strip``.
A Shell ``$'...'`` string takes backslash escapes.
PowerShell escapes with a backtick, so ``"C:\"`` is a whole string.
A Dockerfile has no string literals, as Make has none: its quotes pass to the shell, and its comments are lines that start with ``#``.
A BuildKit heredoc, ``RUN <<'EOF'``, holds literal text.

Rationale and alternatives
--------------------------

We considered importing an assembly lexer.
Assemblers do not expose one, and syntax-highlighting grammars are large and cover one dialect each.
The comment rules fit in a 174-line scanner whose every rule names the dialect it serves.

We considered keeping ``#`` lines in ``.S`` files as code, because ``#define`` is code.
But 683 of Chromium's 1,393 ``.S`` files use ``#`` comments, and the C preprocessor passes them through for exactly this reason.
A short list of directive names tells the two apart.

We do not handle MSVC ``__asm { }`` blocks inside C.
Chromium's 28 files with such blocks have one ``;`` comment line in all.
GCC inline assembly sits in strings and needs nothing.

Backwards compatibility
-----------------------

Physical line totals do not change for any file that was counted before.
Lines move between code and comments, and 1,876 files change language name:

.. list-table:: Changes on Chromium for files counted before
   :header-rows: 1

   * - Class
     - Files
     - Code lines moved to comments
   * - Assembly comments
     - 1,305
     - 52,564
   * - MATLAB to Objective-C
     - 345
     - 11,242
   * - Bazel to Starlark, with docstrings
     - 1,374
     - 9,133
   * - Go Template to Jinja
     - 156
     - 366

JavaScript, TypeScript, Ruby, and Dockerfile counts change in 28 files by at most 136 lines each.
A comment inside a template substitution becomes a comment, and text inside a template or heredoc becomes code.
The Cargo registry moves 44,063 assembly lines and the Zig tree 2,029, all comments in ``.S`` files.
On our other corpora nothing changes.
One ``/usr/share`` prose file named ``TUTORIAL.ru`` now fails as Ruby on ``<<Blank lines ...>>``, as ``TUTORIAL.pl`` already did as Perl.

Every one of these changes corrects a count, but a report under ``lexical-v4`` can now differ from one made before.
Whether this calls for ``lexical-v5`` is an open question below.

Acceptance criteria and implementation status
---------------------------------------------

Implemented with unit tests for each rule.
On Chromium 156.0.8078.47, CCM counts 393,498 files, 63,425,990 program lines and 4.65 GB, in 10.4 seconds with 16 workers and 399 MB of peak memory, and in 44 seconds and 263 MB with one worker.
The blank count of every file equals its number of whitespace-only lines, once a leading byte-order mark is ignored as the policy says.
Git ignores none of the selected files.
On the 202,961 files that scc 4.1.0 also counts, the totals agree to within 0.7 percent: 33,038,552 code, 5,328,563 comment, and 6,052,365 blank lines against 33,059,980, 5,346,394, and 6,013,106.
scc selects 190,537 fewer files, none of which Git ignores, and 12,723 that CCM's default exclusions remove.
The m68k ``isp.S`` that CMD 0009 left uncounted now counts.

Open questions
--------------

Eight files are still not counted: a CSS minifier test with a deliberately open comment, a Jinja template named ``.kt``, four cram test transcripts named ``.t``, Perl with embedded Rez resource data, and the Tcl ``autosetup`` script.

The policy identity.
Counts of files that were counted before changed, so ``lexical-v5`` may be the honest name.
CMD 0011 adopts ``lexical-v5``.

Default exclusions.
CCM excludes every directory named ``build``, ``vendor``, ``dist``, or ``target``.
In Chromium, ``build/`` is hand-written source, and the exclusion removes 12,723 files that Git tracks.
With ``--no-default-ignore`` CCM counts 428,561 files and 70.6 million program lines.
CMD 0011 lets ignore files decide in a governed tree, which counts these files by default.

Review history
--------------

2026-10-24: Proposed, implemented, and measured together.

References
----------

- ``docs/counting.rst``: the counting policy.
- ``docs/language-catalog.rst``: names, precedence, and shared extensions.
- CMD 0009, which raised the limits that Chromium needs.
