Reading the source tree
========================

A source inventory is an observation under a selection policy.
We can change the policy and obtain another valid inventory of the same checkout.
The important question is whether the report makes that policy recoverable.

Consider a dependency lockfile.
It can contain many thousands of lines and still describe data rather than implementation work.
CCM gives Cargo and JSON lockfiles their own rows.
It counts their physical content by default, but those rows do not inflate Rust or JSON source groups and do not become program lines in the AI implementation workload.
A comparison with a tool that excludes lockfiles must first align the selected files.
Otherwise the apparent disagreement combines selection and classification.

The same reasoning applies to generated files.
A generated Rust file is Rust source for the physical inventory.
Its origin matters to a rebuild estimate, so recognized generated files are excluded from the default implementation driver.
The scanner looks for declared markers in a bounded header; it does not infer authorship from style.
``--no-generated`` changes selection and removes those files from the inventory itself.

Language names and policies
---------------------------

The registry contains 658 language and format names and aliases in this edition.
Sixty-four native policies cover common source forms and mixed documents.
The wider declarative catalog handles literal comment and quote definitions.
An alias can select the same native policy as another name, so the registry size is not a count of distinct parsers.

Native and declarative are implementation tiers.
Neither means that CCM compiles the language or proves semantic equivalence.
The distinction tells us where specialized handling exists and where a generic delimiter machine is being applied.
The machine-readable language list exposes that policy identity.

.. code-block:: sh

   ccm --languages
   ccm --languages --json
   ccm --count-as custom:Rust path/to/project

Exact native filenames and TypeScript declaration suffixes have priority.
Catalog filenames, compound suffixes, native extensions, and catalog extension aliases follow the documented selection order.
Extension matching is case-insensitive; exact filename matching retains case.
Where several languages share an extension, the default is deterministic rather than omniscient.
An explicit override is more honest than silently claiming to recognize every dialect from its contents.

The catalog's compatibility manifest records CLOC and scc names and SLOC extensions.
Those checks establish name and path coverage.
They do not establish that every language construct has been independently validated or that different tools assign every line to the same class.
The policy chapter will show why literal syntax definitions and specialized state machines both have a place.

What the totals preserve
------------------------

CCM recognizes LF and CRLF physical lines and counts a final nonempty segment without a newline.
An empty file has zero lines.
A leading UTF-8 byte-order mark is ignored for classification while its bytes remain in the byte total.
Selected text must be UTF-8.
NUL-containing inputs and known binary asset extensions are ordinary binary skips under the relevant selection rules.

The underlying classes retain their definitions through aggregation.
Adding language rows produces the same physical totals as adding file reports.
The native source lexer treats whitespace inside a multiline string as code and whitespace inside a block comment as a comment.
Typst markup and declarative profiles treat physically empty lines as blank.
These details are part of the versioned policy, not corrections made after looking at another tool's output.

Python docstrings are another useful example.
CCM's native policy counts a string as code even when an author uses it as documentation.
Another counter can reasonably choose a documentation convention.
We must record that difference before comparing Code or Comments, while the physical line total remains a direct cross-check.

Inventory as an audit trail
---------------------------

A JSON file record includes its path, language, purpose, generated marker result, counts, and language-policy identifier.
The enclosing report records selection limits, skip counters, notices, and whether the selected supported text was counted completely.
Unknown types remain visible in the coverage summary.
They can hide additional work, so they do not become zero-sized software components by implication.

The inventory cannot reveal missing requirements or code that has not yet been written.
It cannot establish the value of a feature or the skill needed to change it.
It supplies a physical coordinate system for later reasoning.
That coordinate system becomes useful when we keep it attached to its policy and resist asking it to answer a different question.
