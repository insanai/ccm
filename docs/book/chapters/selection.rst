Selecting work without losing control
=====================================

Before a byte can be classified, the project adapter must decide whether to read it.
This stage often determines both the meaning and the runtime of a report.
A very fast lexer does not help if every ignored path triggers unnecessary work.
A complete-looking table is not useful if the selection rules cannot be explained.

CCM begins with explicit roots and a bounded walk.
It normalizes retained file names relative to each root and avoids duplicate paths across overlapping roots.
Common build and vendor directories are excluded by default.
Explicit literal exclusions, extension filters, generated-file selection, and scoped ignore files refine that initial policy.
Symlinks are ordinary skips and their targets are not intentionally followed.
Measurements assume stable input checkouts; mutation detection is best effort rather than an atomic filesystem snapshot.

Ignore rules have scope
-----------------------

A root or ancestor can contribute rules, while a nested directory can add later local rules.
CCM reads .gitignore, .ignore, and .ccmignore.
An ordinary repository can also contribute .git/info/exclude.
The current implementation does not interpret global Git configuration or resolve an external Git directory referenced by a worktree's .git file.

Rule order matters because a later inclusion can reverse an earlier exclusion within the supported scope.
Anchored rules apply relative to their base.
Basename rules compare a candidate's name; path rules compare its relative path.
A negation cannot restore a child of an excluded directory unless the parent directory is itself reenabled and traversed.
This is why selection cannot be implemented as an unordered set of filename suffixes.

The rule engine supports escaped literals, stars, double stars, question marks, bracket classes, and ASCII POSIX classes.
Its general matcher uses a bounded dynamic program rather than recursive backtracking.
For a pattern with p tokens and a path with n bytes, that state machine uses work proportional to p times n and linear row storage in n.
A separate work budget prevents an adversarial collection of patterns from demanding unbounded matching.

The common case can be cheaper without changing the rule.
Literal prefixes and suffixes reject impossible matches early.
A single star surrounded by literal text has a direct matcher, with the same slash-crossing rule as the full state machine.
Tests compare these paths with the full matcher over generated path strings.
Scope-relative paths are resolved once for a group of local rules instead of once per rule.

The walk and the queue
----------------------

Directory enumeration discovers which ignore filenames are present.
The loader can then avoid negative metadata lookups for every absent file in every directory.
Candidates rejected by ignore rules do not need the same retained-file metadata as candidates sent to a worker.
These changes matter on repositories with many small files, where filesystem and selection overhead can dominate lexical counting.

The walker takes each entry's type from the directory listing, so it does not call ``stat`` for every file.
It sends selected files through a synchronous queue with 64 slots per worker.
A full queue applies backpressure to selection.
Workers reuse byte buffers rather than allocate a new buffer for each file.
The default uses one worker per hardware thread, at most sixteen, with an explicit CLI range from one to sixty-four.
Increasing that number is not automatically an improvement: scheduling, storage, file size, and the serial selection stage all matter.

The worker opens each file and reads its size and modification time from the open file.
It applies the per-file and snapshot byte budgets, performs a bounded read, and compares the open file's metadata before and after the read.
A detected change becomes an error rather than a silently inconsistent snapshot.
This is a useful guard under the stable-checkout assumption, not a proof against every filesystem race.

Results have no scheduling order
--------------------------------

Workers can finish in any order.
The project adapter sorts retained files and notices before aggregation and presentation.
This makes repeated JSON reports deterministic for stable inputs and options, even when worker counts differ.
Determinism belongs to the result, not to a claim that threads executed the same interleaving.

Limits constrain the walk, selected files, per-file bytes, total snapshot bytes, nesting, ignore records, and rendering.
The ordinary defaults allow 1,000,000 selected files, 64 MiB per file, 64 GiB per snapshot, and traversal depth 128, enough for a Linux kernel checkout.
A limit failure is explicit.
The CLI does not publish a normal success table after silently discarding work needed to remain within a budget.

The design lesson is broader than a counter.
A bounded pipeline makes the amount of admitted work part of the contract.
That contract gives later claims about speed, completeness, and memory a stable object to refer to.
