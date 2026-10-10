# P5 billing audit: per-local-write AnalysisSteps charge

## Finding

The new charge is a real unit of analysis work, not a change to an archive, a wall-clock reading, or a higher budget. In `crates/jarde-java/src/build.rs:2444-2462`, `decide_types` iterates each local variable's `SlotUse` records. For every record with `written: Some(_)`, it now charges one `AnalysisSteps` at `use_.bci` before polling and inspecting the write's path and stored value. The following char-producer scan may do further bounded evidence work, but the explicit new unit is one per physical write examined. The caller's existing `Budget` is used; no limit is changed.

That matches the evidence model in the surrounding function: char refinement requires checking all physical writes, not just the first write that supplies the declaration type. Charging only the first write would leave the newly required examination of later writes invisible to the budget. If the unchanged budget cannot afford a write scan, the operation should stop at that write's BCI rather than complete without accounting for the work.

## Pinned values affected

The saved raw run `/private/tmp/jarde-typed-workspace-batches-root-v3/62.stdout.raw`, starting at its `running 6 tests` marker after the preceding workspace-batch binary, shows the exact current deltas:

| Ledger | Old pin | Measured | Delta | Evidence |
| --- | ---: | ---: | ---: | --- |
| `Billing::FLAT_MIXED.analysis_steps` | 1501 | 1517 | +16 | `flat-mixed` fails its fixed-shape assertion; every other printed dimension matches. |
| `Billing::DIRECT_ARM.analysis_steps` | 19458 | 19570 | +112 | Direct-arm ledger prints 19570; stale assertion reports old 19458. |
| `Billing::SHARED_ARM.analysis_steps` | 19458 | 19570 | +112 | Shared-arm ledger also prints 19570, matching direct work. |

The raw output prints both arm ledgers before the direct-arm assertion fails. All their non-analysis counters match their pins. It also shows the direct and shared paths still have matching work dimensions, as the retention invariant requires.

The per-case table contains six constants: `FLAT_MIXED`, `NESTED_MIXED`, `TWO_ORIGINS`, `MANY_METHOD_CLASS`, `DAMAGED_TAIL`, and `DEEP_EXPRESSION` (`tests/p5_bulk_corpus.rs:637-837`). The failed normal test exits on the first case, so this particular raw record does not establish the other five case rows. The exact update supported by this raw is `FLAT_MIXED.analysis_steps = 1517`, plus both arm totals at 19570. Re-run the ignored recorder to obtain all six case rows, then verify the five not printed by the failed fixed-shape assertion before leaving their pins unchanged. Do not change any other field based on this evidence.

## What the recorder covers

`record_the_billing_table` is an ignored, explicit regeneration entry point (`tests/p5_bulk_corpus.rs:1786-1815`). It reconstructs each of the six archives with `run_case(..., 1, Duration::ZERO)`, reads `Billing::of(&run.report.usage)`, and prints a literal plus summary for every case. It is sufficient to measure the six bulk-operation rows, including rows the ordinary assertion stops before reaching. It does not run the two per-method control arms and cannot regenerate `DIRECT_ARM` or `SHARED_ARM`.

The arm measurements come from `the_old_per_method_arms_keep_their_ledger_and_the_same_text` (`tests/p5_bulk_corpus.rs:1580-1770`). That test independently runs all 187 method requests through the no-store and shared-store paths, compares text, outcome, content, and execution state, then prints both aggregate ledgers before asserting the pins. Its current failure occurs at the stale direct-arm billing comparison after the shared result has already been measured and printed. After pin updates, rerun this ordinary test to confirm both count assertions and the semantic equality checks.

## Archive inputs

The P5 archives are built in-process from committed class bytes and deterministic builders; there is no generated archive file to refresh (`tests/p5_bulk_corpus.rs:15-36`). `flat-mixed` is exactly `Scope`, `Shape`, `LambdaSample`, and `Holder` in one archive (`tests/p5_bulk_corpus.rs:346-353, 424-439`). The corpus shape test rebuilds the inputs and checks byte equality (`the_corpus_is_rebuilt_from_the_committed_bytes_every_time`). The observed change is therefore in the reader/builder's charged analysis work over the same inputs, not in the archive fixture bytes.

## Budget and follow-up

Keep `bulk_support::limits()` and the configured analysis budget unchanged. The P5 ledger records usage; it is not a performance target or a budget limit. Raising the limit would conceal real work and could turn a legitimate budget stop into an apparent success. The correct update is to regenerate and review the changed count pins, document the new per-write scan charge next to the existing billing history, and rerun the regular shape and arm tests after the source constants are updated.

This is a read-only audit. No repository files, archives, budgets, or pins were modified, and no Cargo, Git, JDK, or CLI command was run.
