# Replay script review

Reviewed `/private/tmp/jarde-arithmetic-stack-replay-root-v1.py` without running it. No repository files were changed and no Jarde CLI, Rust, or JDK command was run.

The comparison itself is appropriately strict: `differences()` recursively compares the complete JSON shape and values, and `non_time_differences` permits only numeric leaves whose path ends in `.elapsed_millis`. The 2 MiB invocation uses the same `LambdaDeep.class` path and all-evidence JSON options as the default-stack baseline. The five historical manifests are checked against their recorded statuses, but those status strings are not treated as product completion.

Before relying on the replay result, make these small provenance fixes:

1. Bind every old report to the bytes being replayed. For the LambdaDeep pair, read the baseline manifest and verify that the selected old CLI row used the exact `argv`, had `main_stack_limit_bytes: null`, exited zero, and recorded the same class SHA as `pin(BASE / 'LambdaDeep.class')`; also verify `cli-default-stdout.raw` against a pin recorded for that row (the manifest records `stdout_bytes`, so add or derive its SHA from the raw file at review time and freeze it). For each historical class-source row, require `pin(row['input']['path'])` to match its recorded input path, byte count, and SHA before replay, and require the pinned old stdout bytes/SHA to match the row's stream record. This is the missing link that proves each pair used the same original class input and source/maps report.
2. Preserve a standalone diff artifact for each pair. Write the complete recursive diff to `N.diff.raw` (including `[]` when identical) before asserting. `execution.json` currently retains the diff in structured form, but does not create the requested raw diff files.
3. Exclude booleans from the elapsed-time exception (`type(value) in (int, float)` or an equivalent explicit `not isinstance(value, bool)` check); Python considers `bool` a subclass of `int`.
4. Label the final status as replay/report-equivalence only and record that it makes no product-completion claim. The present `passed` status describes this runner, but could be mistaken for acceptance of the historical records, some of which are explicitly incomplete or pending.

No additional report-field exclusions are justified. Keep the complete JSON comparison; in particular, source maps and all other fields remain exact. Do not promote the historical manifest statuses: they are provenance labels only, and this replay can establish old-behavior equivalence for the pinned inputs and outputs, not overall product completion.
