Counting and selection policy v3
================================

We count physical LF or CRLF lines, including a nonempty final segment without a newline.
An empty file has zero lines.
We ignore a leading UTF-8 BOM for classification and include it in byte counts.
Lone CR is whitespace within a line.
Program source must be UTF-8; selected files containing NUL bytes are treated as binary skips.

Code, comment-only, and blank classes partition physical lines.
Mixed code/comment lines count as code.
The native source lexer counts whitespace inside a multiline string as code and whitespace inside a block comment as a comment.
Typst markup and declarative profiles classify physically empty lines as blank.
Python docstrings are strings and count as code.
These choices can differ from other counters and are part of the policy identity.
We do not parse program grammar or prove semantic equivalence.

The terminal Code column contains code lines for program sources and nonblank content for documentation and data.
Program-line totals include only languages classified as source.
Per-file JSON includes kind=source, documentation, or data, while the underlying disjoint count fields retain their names.

Registered policies
--------------------

``ccm --languages`` lists the 658 bundled names, formats, and aliases with their filenames and extensions.
``ccm --languages --json`` emits machine-readable registry metadata.
We recognize exact filenames such as Dockerfile, Makefile, CMakeLists.txt, Cargo.lock, and common license filenames before checking extensions.
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
     - Line and block comments, ordinary strings, C++ delimited raw strings, continued line comments.
     - No complete preprocessor translation, trigraph processing, or delimiter reconstruction across escaped newlines.
   * - JavaScript and TypeScript
     - Line/block comments, ordinary and backtick strings, common expression-position regex literals and character classes.
     - Template expression nesting, JSX/TSX embedded markup, and parser-level slash disambiguation remain lexical approximations.
   * - Python
     - Hash comments, ordinary strings and triple strings.
     - No complete expression grammar inside interpolated strings.
   * - Shell
     - Word-sensitive hash comments, quotes, queued quoted/unquoted heredocs, tab-only heredoc stripping, here-strings, arithmetic shifts.
     - No complete shell grammar or every form of mixed quoted heredoc delimiter.
   * - Go, Lua, Kotlin, Scala, Swift, Julia, Haskell, OCaml, and F#
     - Registered comment profiles plus backticks, long delimiters, triple strings, or nested comments where the policy enables them.
     - No embedded-language extraction, expansion, or parser validation.
   * - HTML, XML, and Markdown
     - Markup comments and CDATA; Markdown fences retain literal examples as content.
     - No full HTML/XML parser, Markdown inline grammar, or script/style sublanguage parsing.
   * - CSS, SCSS, SQL, PHP, Ruby, Perl, R, PowerShell, and remaining source policies
     - Their registered common comment and quote profiles; PowerShell here-strings.
     - Specialized interpolation and delimiter grammars can exceed the lexical subset.
   * - TOML, JSON, JSONC, YAML, HCL, INI, and gettext
     - Their registered comment and string profiles.
     - Counting is not document validation.
   * - reStructuredText, plain text, and patches
     - Nonblank versus blank content.
     - We do not interpret directives or embedded source.

Every registry entry has a line-partition fixture.
Each registered source comment profile also has fixtures for standalone comments, mixed lines, and quoted delimiters.
Additional fixtures cover the language-specific constructs above.
This does not establish complete conformance to every language grammar.
Detected unterminated strings, comments, or heredocs produce a coverage warning and exclude that file from totals.
Invalid UTF-8 produces an incomplete report; normal binary and symlink skips do not.
Unknown extensions are disclosed separately rather than silently treated as a supported language.

Selection
----------

We walk explicit paths, or the current directory, without following symlink entries.
Overlapping roots count a canonical regular file path once.
Explicit root files bypass directory pruning; each comparison requires one root per snapshot.
Root-relative labels can repeat across different roots, so we prefer one root when artifact names are used as model seeds.

Default traversal excludes .git, .jj, target, node_modules, .venv, vendor, dist, build, _build, and __pycache__.
``--exclude`` adds an exact entry name or root-relative path prefix.
``--include-ext`` and ``--exclude-ext`` accept comma-separated extensions without dots.
``--no-default-ignore`` disables the default name exclusions.
An explicitly selected root is honored even when its name normally denotes a build directory.

We apply nested .gitignore, .ignore, and .ccmignore rules in that order, with nearer and later rules taking precedence.
Ordinary .git directories also supply .git/info/exclude before their .gitignore.
We support slash anchoring, directory rules, ordered negation, escapes, wildcards, double-star directories, bracket ranges, and ASCII POSIX character classes.
An excluded parent must be reenabled before a child can be reenabled.
``--no-ignore`` disables these files without disabling the default directory exclusions.
We do not read global Git configuration or follow an external gitdir pointer in a worktree's .git file.

We mark generated files when their first 4,096 bytes contain a common generated-content declaration.
``--no-generated`` excludes those files.
This is a header heuristic, not inferred authorship.
All skip classes and unsupported types have separate counters.

Bounds and snapshots
---------------------

The defaults are 100,000 selected files, 16 MiB per file, 1 GiB selected bytes per snapshot, depth 128, and at most ten traversal entries per selected-file budget unit.
The CLI exposes the file and byte budget overrides.
Ignore files are capped at 256 KiB each, 4 MiB in total, 10,000 rules, 1,024 bytes per pattern, and 200 million aggregate matching steps.
Lexical comment and arithmetic nesting is capped at 256; Rust raw hashes and Lua delimiter equals signs are capped at 255.

A bounded queue overlaps traversal with scoped scanner workers.
The default worker count is available CPUs capped at eight; ``--jobs`` accepts one through 64.
Each worker reuses a buffer and reads at most the selected file length plus one byte.
We retain file records and deterministically merge results by path, making complete JSON independent of worker count.

We compare selected file size and modification time with the open file after reading.
Snapshot comparison reads matching paths again and compares exact bytes after stamp checks.
These are ordinary mutation checks, not atomic project snapshots, cryptographic identities, or protection against adversarial races.
We use immutable checkouts for reproducible work.
Limit and I/O failures return an error rather than an apparently complete total.


Broad catalog and mixed-language files
--------------------------------------

We extend the native policies with the immutable declarative catalog documented in `Language policies and catalog coverage <language-catalog.rst>`_.
The compatibility manifest covers the recorded CLOC and scc names and SLOC extensions.
Aliases and generic profiles do not imply compiler-level parsing or equal classification across counters.
Native Typst counting protects raw examples and distinguishes prose from code strings.
Vue and Svelte combine markup, JavaScript/TypeScript, and CSS/SCSS classifications without double counting physical lines.
Jinja recognizes both HTML and template comments.
We give SVG, DOT, CSV, TypeScript declarations, and dependency lockfiles explicit policies or categories.
Known binary asset extensions are normal skips; unsupported text constructs still produce coverage diagnostics.
