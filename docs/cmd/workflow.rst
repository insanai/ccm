Work on a discussion
====================

We run these commands in this folder.
Its ``discussion.toml`` gives the prefix, ``cmd``, the lifecycle, and the numbering, so the commands need no ``--prefix`` or ``--dir``.
``guidedog dsc config`` prints the settings in effect.

.. code-block:: sh

   guidedog dsc new "Cache policy" --labels=architecture
   guidedog dsc list
   guidedog dsc check
   guidedog dsc promote cache-policy --dry-run
   guidedog dsc promote cache-policy
   guidedog dsc state 1 --to=accepted --reason="Review converged"
   guidedog dsc build

A new draft starts from ``template/rfc-template.rst``.
Its metadata is the first field list: one ``:cmd-name:`` field per line, with JSON arrays for authors and labels.

Promotion gives a draft its number and opens it for discussion.
A state change needs ``--reason``; ``committed`` also needs ``--evidence``.

``guidedog dsc build`` writes the website to ``_build/html`` and a PDF per record to ``_build/pdf``.
``guidedog dsc build cache-policy --target=pdf`` previews one record's PDF in ``_build/selected``.

If an update is interrupted, we run ``guidedog dsc recover --rollback`` and review recovery changes.

The :doc:`registry` lists every record.
:doc:`records/0001-cmd-process` defines the proposed CCM review and engineering rules.
