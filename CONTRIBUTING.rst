Contributing to CCM
====================

We use small changes with a clear reason and a result we can reproduce.
A report that depends on a language rule should include a minimal source example, the expected line classes, the count-policy version, and the relevant selection options.
We do not need an entire private repository to investigate a counting defect.

Build and check
----------------

The workspace has no external crate dependencies.
Rust 1.85 is the minimum supported version; release builds use the toolchain named in ``.github/workflows/release.yml``.

.. code-block:: sh

   cargo fmt --all --check
   cargo build --workspace --locked
   cargo test --workspace --locked
   cargo clippy --workspace --all-targets --locked -- -D warnings
   cargo xtask check --soft-width

Public library items have documentation.
Functions have at most 70 logical lines, files at most 1,408 lines, and Rust lines at most 99 columns in the strict check.
Errors explain the cause and a concrete correction.
Libraries return structured errors and do not print them.
We prefer the standard library and record a specific reason before adding an external crate.

Version control
----------------

We use Jujutsu with a Git backend locally; ordinary Git checkouts also work.
We use short imperative commit descriptions, ASCII text, and no generated attribution footer.
We keep build outputs and local agent instruction files out of version control.

.. code-block:: sh

   jj status
   jj diff
   jj commit -m "scan: handle a nested comment boundary"

GitHub pull requests should explain the concrete behavior, the reason, and the checks run.
The pull request template provides those fields.
A new mathematical claim needs its hypotheses; an empirical claim needs its workload, method, and source data.
We keep model inference separate from measured facts and economic assumptions.

Documentation
--------------

We write precise prose in the first person plural and keep each sentence on one source line.
We do not hard-wrap prose to a code width.
The installed Guidedog CLI builds the complete publication, including its PDF:

.. code-block:: sh

   guidedog ssg build docs --out docs/_build/site
   guidedog build html docs/book -W -n
   guidedog build pdf docs/book -W

``cargo xtask docs`` refreshes the artwork and worked-example captures before invoking Guidedog.
Its Typst CLI requirement belongs to SVG and PNG generation.
A build from the committed publication inputs needs only Guidedog.
The showcase reports for large trees change only through an explicit ``cargo xtask showcase`` invocation on already available source trees.

Security reports
-----------------

We report a suspected security defect through the repository's private vulnerability reporting page linked from ``SECURITY.md``.
Ordinary count-policy and usability defects belong in the public bug form.
