# Code Counter Machine Discussions

Proposals and decisions for CCM, written in reStructuredText. The prefix of this catalog is `cmd`.

We start with [CMD 0001](records/0001-cmd-process.rst), the process and Rust engineering rules, and [CMD 0002](records/0002-change-certificate-calculus.rst), the candidate measurement theory, engine design, and agent-impact model. Both are open for discussion.

[CMD 0003](records/0003-acceptance-budgets-and-cli.rst) records the first working CLI and its activity-based acceptance-budget scenario. Its narrow implementation leaves the broader research gates in CMD 0002 open.

```sh
guidedog dsc new "A proposal"
guidedog dsc list
guidedog dsc check
guidedog dsc build
```

We use [workflow.rst](workflow.rst) for the commands. `guidedog dsc build` writes the website to `_build/html` and a PDF per record to `_build/pdf`.

Each PDF uses an RFC-style discussion layout, with a metadata panel, state badges, a status note and compact contents. The layout is embedded in the binary and copied here with `_templates/theme.typ` by `guidedog dsc init`. We can customize it in `_templates/discussion.typ`; ordinary books use `_templates/book.typ`.

[CMD 0004](records/0004-competitive-counting-and-evidence-reports.rst) extends the CLI with parallel counting, scoped ignore rules, 55 lexical policies, reliable colors, and explicit finite behavioral outcomes. Runtime measurements and their limits are in [the performance report](../performance.rst).
