Language policies and catalog coverage
=======================================

We use native lexical scanners for common languages and mixed documents, with declarative profiles for the wider catalog.
We keep names, extensions, literal comment markers, and literal quote delimiters separate from the counting engine.
We do not depend on an installed counter or execute another counter during normal measurement.
The workspace still has no external crate dependencies.

The 2026-10-23 compatibility snapshot records 374 scc names, 433 CLOC names, and 97 SLOC extensions.
The combined registry contains 658 names, formats, and aliases, including CCM's existing policies and model format.
These counts include alternate names and CLOC's ambiguous-language categories.
They are not 658 distinct programming languages or 658 complete parsers.
``ccm --languages`` lists the registry.
``ccm --languages --json`` includes each entry's policy tier.

Policy tiers
-------------

``native-lexical-v5`` selects one of the 67 native policies.
These include dedicated handling for assembly in the common dialects, Starlark, Roff, Typst markup/raw text, Vue and Svelte script/style regions, XML/HTML comments, Jinja comments, and the source constructs described in ``counting.rst``.
A native policy is still a lexical counter, not a complete compiler or a semantic proof.
``declarative-v1`` uses the bundled literal delimiter definitions and a bounded state machine.
It handles line comments, multiple block-comment and quote pairs, case-insensitive markers, escapes, and declared nested comments.
It does not implement arbitrary CLOC regular-expression filters, compiler preprocessing, filename-content heuristics, arbitrary interpolated-string grammars, fixed-column dialects, or every extension ambiguity.
A language's presence in the catalog is not evidence that every construct in that language has been independently validated.

We preserve a disjoint partition of physical lines into code, comment-only, and blank lines.
Mixed code/comment lines count as code.
A whitespace-only line is blank under every policy, even inside a literal or comment.
Python, Starlark, and Julia docstrings that begin a line count as comments.
Typst raw examples remain document code, even when an example contains another language's comment syntax.
These choices can differ from another counter's classification without changing the physical line total.
The report uses the familiar label Code for noncomment content, including data and documentation rows.
Program-line totals and the AI implementation workload exclude documentation and data.

Mixed documents
----------------

Vue and Svelte scans separate markup from script and style regions.
Script regions use JavaScript or TypeScript counting; style regions use CSS or SCSS counting.
We combine their classifications on a shared physical line map, so a line containing both a closing script tag and an opening style tag is counted once.
We skip comment and quoted-attribute examples when looking for embedded opening tags.
Unsupported declared script/style languages produce a coverage diagnostic rather than silently using the wrong policy.
Custom preprocessors, template-expression grammars, and language plugins remain outside this native subset.

Typst scans distinguish prose from code strings and preserve raw spans and fences.
Backticks protect sample source from being interpreted as Typst comments or strings.
Typst block comments can nest within a bounded depth.
Quotes in prose are not unterminated source strings.
The native scanner is bounded to 256 nested comments or delimiters.

Names and precedence
---------------------

Native exact filenames and TypeScript declaration suffixes take priority.
We then check bundled exact filenames, compound suffixes, native extensions, and catalog extension aliases.
Case-insensitive extension matching does not make exact filenames case-insensitive.
Ambiguous extension choices are deterministic and can be overridden with ``--count-as EXT:LANGUAGE``.
We retain aliases so that an existing tool's language name can be used for an explicit override.
Some aliases select the same native policy and therefore share its canonical report name.
We do not promise content-based disambiguation for every shared extension.
Three shared extensions are decided by the first 4 KiB of the file.
A ``.ccm`` file that begins with ``ccm-model`` is a CCM model, not C++ module source.
A ``.m`` file whose first nonblank line starts with ``//``, ``/*``, ``#``, or ``@`` is Objective-C; other ``.m`` files stay MATLAB, whose files start with ``%`` or code.
A ``.tmpl`` file that contains a Jinja ``{%`` or ``{#`` tag is Jinja; other ``.tmpl`` files stay Go templates.
``Bazel`` remains an alias of Starlark for ``--count-as``.

Cargo.lock and JSON dependency lockfiles have separate rows.
Their lines do not inflate the reported Rust, TOML, JSON, or program-size totals.
We count them by default because they are part of the selected repository inventory.
A comparison with a tool that excludes lockfiles must align the selected file set first.
Known binary extensions are ordinary skips unless an explicit counting override applies.
Unknown types remain visible in the coverage summary.

Provenance and licenses
-----------------------

The compatibility snapshot is stored in ``docs/benchmarks/language-catalog-2026.json`` with source URLs and SHA-256 hashes.
``docs/benchmarks/language-coverage-2026.tsv`` is the executable name/extension coverage manifest.
We use literal syntax metadata from `scc's MIT-licensed language definitions <https://github.com/boyter/scc/blob/master/languages.json>`_, not its counting algorithm, complexity calculation, CLI, or cost model.
We reviewed names and extension facts from `CLOC's language catalog <https://github.com/AlDanial/cloc/blob/master/Unix/cloc>`_.
We do not copy or execute CLOC's Perl filters in CCM.
Additional literal comment-family facts and the extension check refer to `SLOC's MIT-licensed definitions <https://github.com/flosse/sloc/blob/master/src/sloc.coffee>`_.
CCM's native scanners, mixed-line map, bounded declarative engine, presentation, and economic model remain separate implementations.

The bundled metadata attribution is in ``crates/ccm-scan/LICENSE-language-data`` and available in the executable with ``ccm --licenses``.
The generated Rust tables are immutable data, divided into bounded modules.
We mark those source files generated so that the default AI rebuild estimate excludes imported/generated registry content.
CLOC-only categories use explicit family profiles or appropriate native aliases rather than importing executable regular expressions.
Their dialect-specific fidelity requires further fixtures.

Validation
-----------

We check every recorded scc and CLOC name against the exposed registry and every recorded SLOC extension against path selection.
Every registered policy must produce deterministic disjoint counts for the common physical-line fixture.
Focused fixtures cover mixed Vue boundaries, embedded literals and comments, Typst raw examples and prose quotes, nested comments, Jinja comments, and source-language constructs.
The catalog checks establish registry coverage, not full cross-tool count equivalence.
We test optimized glob matching against the full state-machine matcher and keep Git-backed ignore-selection checks.
For performance comparisons, we separately require equal file, byte, and physical-line totals before timing.
