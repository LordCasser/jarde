# Read-only billing pin audit for bulk corpus

## Evidence in this run

`/private/tmp/jarde-compound-tail-regressions-root-v2/9.stdout.raw` records the `flat-mixed` observed `Billing` as `analysis_steps: 1540`; the old constant is 1517. Its other eight `Billing` dimensions match. The old-arm test records both direct and shared `analysis_steps: 19685`; each old constant is 19570. Their other billing dimensions match. The test log reports that old-arm text and shape assertions ran before the billing assertions; preserve the failure evidence as-is.

This supports a deterministic count change from newly executed analysis work. `AnalysisSteps` is the operation's counted work dimension; the fixed-shape test pins it as a regression ledger, not a target or budget. Reducing or hiding the charge would make the account disagree with the execution and erase the cost of the new scan. The correct next step, if the code change intentionally adds this traversal, is to regenerate the complete constants from the ignored recorder and then verify the normal assertions.

## Recorder contract and minimum capture

`tests/p5_bulk_corpus.rs:1792-1823` documents and implements `record_the_billing_table`. It runs every corpus case at the canonical shape, prints the complete `Billing { … }` literal plus class/method/outcome counts, and exists specifically to regenerate measured pins rather than infer deltas. It does not print the aggregate direct/shared arm literals; the regular `the_old_per_method_arms_keep_their_ledger_and_the_same_text` test does, immediately before asserting them.

After root chooses to capture the post-change numbers, minimum commands from repository root are:

```sh
cargo test --test p5_bulk_corpus --locked -- --ignored --nocapture record_the_billing_table
cargo test --test p5_bulk_corpus --locked -- --nocapture
```

The first emits all six case literals (do not fill unobserved case counts by extrapolation). The second rechecks the ledger and prints the exact direct/shared totals, including whether the observed +115 persists. Preserve the current failure raw as historical evidence; the recorder is additive evidence, not a replacement for it.

## Exact values currently evidenced

| Pin | prior `analysis_steps` | observed `analysis_steps` | evidence |
| --- | ---: | ---: | --- |
| `Billing::FLAT_MIXED` | 1517 | 1540 | `9.stdout.raw`, full nine-dimension literal |
| `Billing::DIRECT_ARM` | 19570 | 19685 | `9.stdout.raw`, full direct row |
| `Billing::SHARED_ARM` | 19570 | 19685 | `9.stdout.raw`, full shared row |

No other corpus case delta is asserted here. The recorder must supply those literals. The corresponding expected source edits, after capture, are only the measured `analysis_steps` fields and comments documenting the newly billed scan; do not alter other dimensions or weaken equality checks.
