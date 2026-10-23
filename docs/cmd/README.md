# Code Counter Machine Discussions

Proposals and decisions for CCM, written in reStructuredText. The prefix of this catalog is `cmd`.


```sh
guidedog dsc new "A proposal"
guidedog dsc list
guidedog dsc check
guidedog dsc build
```

We use [workflow.rst](workflow.rst) for the commands. `guidedog dsc build` writes the website to `_build/html` and a PDF per record to `_build/pdf`.

Each PDF uses an RFC-style discussion layout, with a metadata panel, state badges, a status note and compact contents. The layout is embedded in the binary and copied here with `_templates/theme.typ` by `guidedog dsc init`. We can customize it in `_templates/discussion.typ`; ordinary books use `_templates/book.typ`.
