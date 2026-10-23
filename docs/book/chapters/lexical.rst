Counting languages and mixed documents
========================================

A lexical counter needs enough structure to distinguish a comment marker from the same bytes inside a literal.
It does not need to answer every question a compiler answers.
The useful boundary is a documented policy whose results we can reproduce, including the cases it cannot safely classify.

The native source lexer keeps explicit states for normal text, nested block comments, quotes, and delimited literals.
Language policy selects line-comment markers and enables specialized constructs.
Ordinary spans are scanned together until a relevant special byte appears.
The lexer preserves a disjoint physical-line classification as it crosses those states.

Rust provides a good example of why delimiter tables alone are insufficient.
A raw string can contain quote characters and apparent comment markers until its particular closing quote-and-hash sequence appears.
A lifetime apostrophe is not an open character literal.
Nested block comments need a depth counter and a bound.
CCM handles these cases natively rather than pretending that every apostrophe begins the same sort of string.

Shell syntax poses another problem.
A heredoc delimiter governs later physical lines, while a here-string does not open the same deferred region.
The characters used for a bit shift inside arithmetic must not become a heredoc opener.
The shell policy therefore tracks heredoc queues and bounded arithmetic nesting in addition to ordinary quotes.
JavaScript regex character classes, Python triple strings, Go backticks, Lua long delimiters, and markup comments require their own documented distinctions.

A declarative catalog
---------------------

The wider catalog separates literal syntax definitions from the algorithm that applies them.
A definition can supply several block-comment pairs, several quote pairs, line-comment markers, case-insensitive matching, escape behavior, and declared nesting.
The bounded machine can then count many language families without importing a large parser dependency for each one.

Catalog presence does not establish compiler-level fidelity.
Some languages require fixed-column rules, preprocessing, interpolation grammars, or content-based extension disambiguation that a generic delimiter machine does not implement.
CLOC's executable regular-expression filters are not imported into CCM.
The registry's name and extension checks establish compatibility coverage, while focused fixtures establish particular lexical behavior.
The JSON policy tier keeps those claims distinguishable.

This is also why a count difference is not resolved by copying another counter's total.
We first align selected files and physical lines.
Then we locate the policy choice or unsupported construct responsible for the classification difference.
Only a language specification and a stated counting convention can determine which change to make.

One physical line, several languages
------------------------------------

A Vue component can contain a template, TypeScript, and CSS in one file.
A closing script tag and an opening style tag can even share one physical line.
Counting each extracted region independently and adding its line totals would count that line twice.

CCM's mixed-document adapter records a small classification mask for each physical line.
It locates supported script and style regions outside markup comments and quoted attribute examples.
The appropriate lexer classifies the embedded source, while markup classification handles the surrounding document.
The adapter combines their code and comment flags at the original line position.
A code flag wins over a comment-only flag, and each physical line contributes once.

Vue and Svelte use this mechanism for supported JavaScript, TypeScript, CSS, and SCSS blocks.
An unsupported declared embedded language produces a coverage diagnostic.
Arbitrary preprocessors and template-expression grammars remain outside this native subset.
The adapter's precision comes from preserving the shared physical coordinate system, not from claiming to compile the component.

Typst is a document language
----------------------------

A prose quote in Typst is not necessarily a code string.
A fenced raw example can contain an unmatched quote or an unfinished comment because those bytes are examples, not active Typst syntax.
Applying a conventional source-string lexer to the entire document caused valid repository documents to be rejected.

Typst nests three modes: markup, code, and math.
A ``"`` opens a string only in code and math, so the lexer must know which mode it is in at every byte.
It keeps a stack of open frames: content blocks, code blocks and groups, equations, and expressions embedded with ``#``.

An embedded expression is the delicate part, because nothing marks where it ends.
Typst ends it by rule, and the lexer follows the same rules.
A statement such as ``#let`` or ``#set`` runs to a newline or ``;`` outside any delimiter, so a header that spans several lines inside parentheses stays code.
A control expression such as ``#if`` ends after a body block that no ``else`` follows.
Any other expression is a chain such as ``#text(red)[body].len()``, which ends at the first space.

An earlier scanner forgot an open statement at each line end.
In this book's own template, the header of ``#let book(...) = {`` spans three lines, so the scanner returned to markup too early, and the next ``rgb("#...")`` opened a string that never closed.
The mode stack removes that failure, and a test keeps it removed.
A raw example still protects its contents, and a link such as ``https://a//b`` in markup is text, not a comment.
The cover drawing of this book is also written in Typst; CCM counts it like any other Typst file.

Work tokens are not API tokens
-------------------------------

An optional source-work counter collapses recognized literals and counts lexical words and punctuation outside comments.
Its policy identity is source-work-v1.
It is collected for technical inspection and is not the billing-token quantity used by the AI estimate.
No tokenizer library can turn this lexical measure into observed historical model usage without a separate empirical mapping.

Native scanning is linear in ordinary input traversal with bounded language-state overhead, while delimiter and catalog dispatch add costs determined by the selected policy.
Mixed-file mapping adds storage proportional to the physical line count.
The implementation bounds file size, delimiters, and nesting rather than relying on a universal grammar-complexity claim.
The resulting measurement is a physical one, useful because its algorithm and limits are stated.
