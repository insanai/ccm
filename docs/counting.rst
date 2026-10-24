Counting and selection policy v4
================================

We count physical LF or CRLF lines, including a nonempty final segment without a newline.
An empty file has zero lines.
We ignore a leading UTF-8 BOM for classification and include it in byte counts.
Lone CR is whitespace within a line.
Selected files containing NUL bytes are treated as binary skips.
Text in a legacy encoding such as Latin-1 is counted by its ASCII structure, because every delimiter we recognize is ASCII.

Code, comment-only, and blank classes partition physical lines.
Mixed code/comment lines count as code.
A line that holds only whitespace is blank, even inside a multiline string, heredoc, or block comment.
Every lexer applies this rule, so the blank count of a file equals its number of whitespace-only lines.
A triple-quoted Python or Julia string that begins a line is a docstring, and its lines count as comments.
These choices are part of the policy identity.
They agree with cloc on blank lines and with cloc, tokei, and scc on docstrings.
We do not parse program grammar or prove semantic equivalence.

Version 4 changed three rules of version 3.
Version 3 counted whitespace-only lines inside strings as code and inside block comments as comments.
It counted Python docstrings as code.
It rejected text that was not UTF-8.

The terminal Code column contains code lines for program sources and nonblank content for documentation and data.
Program-line totals include only languages classified as source.
Per-file JSON includes kind=source, documentation, or data, while the underlying disjoint count fields retain their names.

Registered policies
--------------------

``ccm --languages`` lists the 658 bundled names, formats, and aliases with their filenames and extensions.
``ccm --languages --json`` emits machine-readable registry metadata.
We recognize exact filenames such as Dockerfile, Makefile, CMakeLists.txt, Cargo.lock, and common license filenames before checking extensions.
Notice variants such as ``LICENSE-MIT`` and ``COPYING_v2`` count as plain text.
When two formats share an extension, a signature at the start of the file decides.
A ``.ccm`` file that begins with ``ccm-model`` is a CCM model; any other ``.ccm`` file is a C++ module.
Extensions are case-insensitive.
Extensionless scripts can be recognized through a supported shebang interpreter without executing them.
``--count-as EXT:LANGUAGE`` explicitly selects a policy for an otherwise different extension.

.. list-table:: Important lexical constructs
   :header-rows: 1

   * - Policies
     - Implemented counting constructs
     - Deliberate boundary
   * - Rust
     - Line comments, nested block comments, character literals, lifetimes, ordinary and raw strings.
     - No macro expansion, type checking, or name resolution.
   * - C and C++
     - Line and block comments, ordinary strings, C++ delimited raw strings at a token start or after an encoding prefix, continued line comments.
     - No complete preprocessor translation, trigraph processing, or delimiter reconstruction across escaped newlines.
   * - JavaScript and TypeScript
     - Line/block comments, ordinary and backtick strings, common expression-position regex literals and character classes.
     - Template expression nesting, JSX/TSX embedded markup, and parser-level slash disambiguation remain lexical approximations.
   * - Python
     - Hash comments, ordinary strings and triple strings.
     - No complete expression grammar inside interpolated strings.
   * - Shell
     - Word-sensitive hash comments, quotes, escapes outside quotes, queued quoted/unquoted heredocs including ``<<\EOF``, tab-only heredoc stripping, here-strings, arithmetic ``((...))`` tracked by parenthesis depth so ``<<`` inside is a shift, and command substitutions inside double quotes, ``"$(echo "$i")"``, where quoting starts afresh.
     - No complete shell grammar or every form of mixed quoted heredoc delimiter.
   * - Perl and Ruby
     - POD and ``=begin`` blocks as comments, data after ``__END__``, Perl quote-like operators (``q qq qw qr m s tr y``) with any delimiter, patterns in operator position, punctuation variables such as ``$'``, and ``<<~`` heredocs.
     - Perl decides between a pattern and a division from the previous token; a bare ``/`` pattern that does not close on its line is read as a division.
   * - Typst
     - The markup, code, and math mode stack; strings only in code and math; raw text; nested comments; links that contain ``//``.
     - Embedded expressions are parsed only as far as needed to find where they end.
   * - Go, Odin, Lua, CMake, Kotlin, Scala, Swift, Julia, Haskell, OCaml, F#, Scheme, and Clojure
     - Registered comment profiles plus backtick strings, long brackets and CMake bracket arguments, triple strings, nested comments, or Lisp character literals where the policy enables them; ``'`` in a Lisp quotes a form.
     - No embedded-language extraction, expansion, or parser validation.
   * - HTML, XML, and Markdown
     - Markup comments and CDATA; Markdown fences retain literal examples as content.
     - No full HTML/XML parser, Markdown inline grammar, or script/style sublanguage parsing.
   * - CSS, SCSS, SQL, PHP, Ruby, Perl, R, PowerShell, and remaining source policies
     - Their registered common comment and quote profiles; PowerShell here-strings.
     - Specialized interpolation and delimiter grammars can exceed the lexical subset.
   * - TOML, JSON, JSONC, YAML, HCL, INI, and gettext
     - Their registered comment and string profiles; a YAML ``#`` starts a comment only after whitespace, a quote opens a string only where a scalar begins, and the text of a block scalar (``key: |``) is literal, with its ``#`` lines counted as comments as in an embedded script.
     - Counting is not document validation.
   * - Make
     - Hash comments; quotes are text that Make passes to the shell, so Make has no string literals.
     - No variable expansion or recipe parsing.
   * - TeX
     - Percent comments and control symbols such as ``\%``; TeX has no string literals.
     - No macro expansion.
   * - reStructuredText, plain text, and patches
     - Nonblank versus blank content.
     - We do not interpret directives or embedded source.

Every registry entry has a line-partition fixture.
Each registered source comment profile also has fixtures for standalone comments, mixed lines, and quoted delimiters.
Additional fixtures cover the language-specific constructs above.
This does not establish complete conformance to every language grammar.
In C, C++, Java, JavaScript, Python, Go, and the other languages whose ordinary literals cannot cross a line end, an unclosed ``'`` or ``"`` closes at the line end unless a backslash continues the line.
The same holds for declarative catalog profiles.
This recovers from stray quotes, such as the apostrophe in ``#error isn't supported``.
Detected unterminated strings, comments, or heredocs produce a coverage warning that names the line where the construct began, and exclude that file from totals.
Unknown extensions are disclosed separately rather than silently treated as a supported language.

Selection
----------

We walk explicit paths, or the current directory, without following symlink entries.
Overlapping roots count a canonical regular file path once.
Explicit root files bypass directory pruning; each comparison requires one root per snapshot.
Root-relative labels can repeat across different roots, so we prefer one root when artifact names are used as model seeds.

Default traversal excludes .git and .jj entries, and directories named target, node_modules, .venv, vendor, dist, build, _build, or __pycache__.
A file named ``build`` or ``dist`` is counted.
``--exclude`` adds an exact entry name or root-relative path prefix.
``--include-ext`` and ``--exclude-ext`` accept comma-separated extensions without dots.
``--no-default-ignore`` disables the default name exclusions.
An explicitly selected root is honored even when its name normally denotes a build directory.

We apply nested .gitignore, .ignore, and .ccmignore rules in that order, with nearer and later rules taking precedence.
Ordinary .git directories also supply .git/info/exclude before their .gitignore.
We support slash anchoring, directory rules, ordered negation, escapes, wildcards, double-star directories, bracket ranges, and ASCII POSIX character classes.
An excluded parent must be reenabled before a child can be reenabled.
``--no-ignore`` disables these files without disabling the default directory exclusions.
Ignore files in directories above a root apply only when the root lies inside a Git repository.
We do not read global Git configuration or follow an external gitdir pointer in a worktree's .git file.

We mark generated files when their first 4,096 bytes contain a common generated-content declaration.
``--no-generated`` excludes those files.
This is a header heuristic, not inferred authorship.
All skip classes and unsupported types have separate counters.

Bounds and snapshots
---------------------

The defaults are 1,000,000 selected files, 64 MiB per file, 64 GiB selected bytes per snapshot, depth 128, and at most ten traversal entries per selected-file budget unit.
They admit a large monorepo: a Linux 6.12 checkout has 86,618 files, 1.48 GB, a deepest directory of 13 levels, and a largest file of 24 MB.
Each counted file keeps a record of about 330 bytes, so a million files need about 330 MB.
Workers read one file at a time into a reused buffer and release a buffer above 4 MiB, so the snapshot byte budget bounds work, not memory.
The CLI exposes the file and byte budget overrides.
A file above the per-file limit is skipped with a notice and makes the report incomplete; the other limits stop the scan.
Ignore files are capped at 256 KiB each, 4 MiB in total, 10,000 rules, and 1,024 bytes per pattern.
Ignore matching may take 2,000 steps per selectable file, and at least 200 million steps.
A step is the work a match actually does: a rule such as ``*.o`` that its suffix decides costs a few steps, not a table over the whole path.
The Linux checkout, with 358 ignore files and 1,592 rules, takes about 800 steps per path.
Lexical comment and arithmetic nesting is capped at 256; Rust raw hashes and Lua delimiter equals signs are capped at 255.

A bounded queue overlaps traversal with scoped scanner workers.
Traversal takes file types from directory listings and does not call ``stat`` on each entry.
The default worker count is the number of hardware threads, capped at 16; ``--jobs`` accepts one through 64.
Each worker opens a file, reads its size and time from the open file, checks the byte budgets, and then reads at most that size plus one byte into a reused buffer.
We retain file records and deterministically merge results by path, making complete JSON independent of worker count.

We compare the open file's size and modification time before and after reading.
Snapshot comparison reads matching paths again and compares exact bytes after stamp checks.
These are ordinary mutation checks, not atomic project snapshots, cryptographic identities, or protection against adversarial races.
We use immutable checkouts for reproducible work.
Limit and I/O failures return an error rather than an apparently complete total.


Broad catalog and mixed-language files
--------------------------------------

We extend the native policies with the immutable declarative catalog documented in `Language policies and catalog coverage <language-catalog.rst>`_.
The compatibility manifest covers the recorded CLOC and scc names and SLOC extensions.
Aliases and generic profiles do not imply compiler-level parsing or equal classification across counters.
Native Typst counting tracks the markup, code, and math modes, so it protects raw examples and distinguishes prose quotes from code strings.
Vue and Svelte combine markup, JavaScript/TypeScript, and CSS/SCSS classifications without double counting physical lines.
Jinja recognizes both HTML and template comments.
We give SVG, DOT, CSV, TypeScript declarations, and dependency lockfiles explicit policies or categories.
Known binary asset extensions are normal skips; unsupported text constructs still produce coverage diagnostics.
