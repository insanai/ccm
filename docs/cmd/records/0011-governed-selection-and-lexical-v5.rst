:cmd-number: 0011
:cmd-title: Governed selection and lexical-v5
:cmd-state: committed
:cmd-type: Standards Track
:cmd-authors: ["Vikrant Rathore", "Ronak Rathore", "Kanak Rathore"]
:cmd-created: 2026-10-24
:cmd-updated: 2026-10-24
:cmd-discussion: No external discussion URL assigned
:cmd-labels: ["counting","selection","verification"]

Governed selection and lexical-v5
=================================

Abstract
--------

CMD 0010 left two questions open: whether its corrections need a new policy name, and whether directory names such as ``build`` should decide selection.
We answer both.
Directory names decide only where no Git repository or ignore file governs the tree.
The policy becomes ``lexical-v5``, which also adds Roff, file-name conventions for Make, Meson, and Bazel, and closing rules for Shell, YAML, and Markdown found on FreeBSD.
We state the properties that the counts guarantee and check them on Linux 6.12, FreeBSD 15.1, and Chromium 156, instead of grading CCM against another counter.

Motivation and scope
--------------------

CCM excluded every directory named ``target``, ``node_modules``, ``.venv``, ``vendor``, ``dist``, ``build``, ``_build``, or ``__pycache__``.
In Chromium, ``build/`` is hand-written source that Git tracks, and the rule removed 35,063 such files with 7.4 million program lines.
A project that keeps its output in ``build/`` already says so in its ``.gitignore``.
The name rule therefore only helps in a tree that says nothing, such as an unpacked archive.

Comparing with scc does not settle what a correct count is.
When two counters disagree, at least one is wrong; when they agree, both may be wrong in the same way.
A definition with a property we can check on the input settles it.

Specification
-------------

Governed selection.
A directory is governed when it or a directory above it, within the root or the enclosing repository, holds a ``.git``, ``.gitignore``, ``.ignore``, or ``.ccmignore``.
In a governed directory the ignore rules alone decide; the name exclusions apply only to directories that are not governed.
``.git`` and ``.jj`` stay excluded everywhere.
``--no-ignore`` sets ignore files aside, so nothing is governed and the names apply again; ``--no-default-ignore`` disables the names everywhere.

.. code-block:: text

   archive/            not governed: build/ and vendor/ are excluded
     build/
   repo/               governed by repo/.git: .gitignore decides
     .gitignore        /out/
     build/            counted
     out/              excluded by the rule
   plain/
     sub/              governed by sub/.gitignore
       .gitignore
       build/          counted

Roff.
Roff documents are documentation with a native policy: ``.\"`` and ``'\"`` begin a comment line, and ``\"`` and ``\#`` begin a comment after text.
``.roff``, ``.man``, ``.mdoc``, ``.ms``, ``.me``, and ``.tmac`` are Roff.
A manual page ``.1`` to ``.9`` is Roff only when its first line with content is a roff request such as ``.TH`` or ``.Dd``, because single-digit extensions also name rotated logs.

``.t`` files.
A ``.t`` file is a Perl test by convention, but FreeBSD also uses the extension for troff papers and pjdfstest shell scripts.
Its shebang decides first, then a roff opening, and Perl otherwise.

Names before suffixes.
A naming convention outranks the extension.
``Makefile.inc``, ``Makefile.am``, ``Makefile.in``, ``Makefile.asm``, and their kin are Make, because the suffix names what the makefile builds; only ``.PL``, ``.pl``, ``.py``, ``.m4``, the prose suffixes, and the data suffixes name the file's own language.
``meson.build``, ``meson_options.txt``, and ``meson.options`` are Meson.
A Bazel overlay such as ``zlib.BUILD`` or ``jpeg.BUILD.bazel`` is Starlark.
A ``.y`` file is a yacc grammar with C comments and character literals, not Happy, which has no comment rule in the catalog.

Shell.
A heredoc inside a command substitution may end on the line that closes the substitution: ``EOF)``, or ``EOF`` followed by the closing backtick.
A script may carry data after a top-level ``exit``, as FreeBSD's stress tests carry a C program that they extract with ``sed '1,/^EOF/d' < $0``.
The shell never reads that text.
When a script does not lex as a whole, we look for the last line that is ``exit`` or ``exit WORD`` in the first column and before which the script lexes cleanly, and count the nonblank lines after it as code.
A script that lexes as a whole keeps its own counts.

YAML and Markdown.
In YAML, a ``[`` or ``{`` opens a flow collection only where a value begins, and a quote after a ``,`` opens a string only inside a flow collection or on a line that continues one.
Braces and commas in a plain scalar, such as ``sed -e 's,a/${A}/b,c,'``, are text.
In Markdown, a code span is text, so `` `<!--` `` opens no comment.

What the counts guarantee.
``counting.rst`` states the properties and their checks.
For a file :math:`F` with :math:`n` physical lines, the counts satisfy :math:`C(F) + K(F) + B(F) = n`, and :math:`B(F)` is the number of whitespace-only lines.
For two policy versions and every file that both count, the blank count and the sum :math:`C + K` agree.
The selected files are a subset of the files Git would list.

Rationale and alternatives
--------------------------

We considered keeping the name list and removing ``build`` from it.
Any name can hold source in some project: ``vendor/`` holds Chromium's and FreeBSD's imported code, which Git tracks.
The project's own ignore files are the only rule that knows.

We considered treating every file after ``exit`` as data.
That would change counts that stand on their own, since ``exit`` often ends a branch in the first column of a function written without indentation.
Trying it only when the whole script fails keeps every count that already stood.

We considered counting ``.t`` files by their directory, as FreeBSD's ``share/doc``.
Their content says more, and it works in every tree.

Backwards compatibility
-----------------------

Reports now say ``lexical-v5``.
Physical line totals and blank counts do not change for any file that the previous build counted.
We checked this on ten corpora: our repository, the Zig and Ensodoc snapshots, the Cargo registry, ``/usr/include``, ``/usr/share``, the Python 3.13 library, and the three trees below.

.. list-table:: Changes from the build of CMD 0010
   :header-rows: 1

   * - Tree
     - Newly selected
     - Newly recognized
     - Relabeled
     - Code moved to comments
   * - Linux 6.12
     - 210
     - 79
     - 16
     - 449 lines in 10 files
   * - FreeBSD 15.1
     - 172
     - 8,186
     - 1,165
     - 6,400 lines in 109 files
   * - Chromium 156
     - 35,063
     - 441
     - 393
     - 683 lines in 24 files

Newly selected files are those that the name exclusions removed from a governed tree.
Newly recognized files were unsupported or could not be counted before: chiefly Roff manual pages and papers, Makefiles with a suffix, and FreeBSD shell scripts.
In FreeBSD, 406 ``Makefile.inc`` files were read with a mixed Pascal and Fortran profile and 299 pjdfstest scripts as Perl.
One file stops counting: byacc's ``err_syntax2.y`` opens a comment it never closes, which the Happy profile could not see.

Acceptance criteria and implementation status
---------------------------------------------

Implemented with unit and command-line tests for each rule.
The release build gives these results with 16 workers.

.. list-table:: The three trees under lexical-v5
   :header-rows: 1

   * - Tree
     - Files
     - Program lines
     - Bytes
     - Uncounted
     - Time and memory
   * - Linux 6.12
     - 82,405
     - 27,828,995
     - 1.46 GB
     - 0
     - 1.1 to 2.4 s, 125 MB
   * - FreeBSD 15.1
     - 93,582
     - 22,355,381
     - 1.25 GB
     - 18
     - 1.5 to 3.5 s, 145 MB
   * - Chromium 156
     - 429,002
     - 70,619,122
     - 5.21 GB
     - 8
     - 15 to 18 s, 410 MB

On every file of the three trees the blank count equals the number of whitespace-only lines, and ``git check-ignore`` rejects none of the selected files.
With one worker Chromium takes 71 seconds and 286 MB.
``cargo xtask showcase`` writes the three HTML reports to ``examples`` and captures the Linux terminal report for the website.

Open questions
--------------

Eighteen FreeBSD files are still not counted.
Four are not valid in their language: byacc's ``err_syntax2.y`` and a libyaml fuzzing input are deliberate syntax tests, a ZFS test script ends in a stray quote, and a DTrace test script never ends its heredoc, which the shell accepts with a warning.
The other fourteen need rules we do not have: two ed scripts named ``.t``, four DTrace ``.d`` programs, two bmake fragments, two test scripts that pass m4 quotes through the shell, a lex file, the Perl ``expn.pl``, the Tcl ``autosetup`` script, and the tzdata ``zone1970.tab`` table.
In Chromium, the Markdown rule now counts two READMEs, and the eight files that CMD 0010 lists remain.
