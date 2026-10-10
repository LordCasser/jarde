CF16 public stop/publication patch (luna-v2)

Supersedes v1 for review; v1 is retained. The test uses one loop for zero-IR and zero-output budgets and asserts the exact StopReason::Budget dimension for each. It separately asserts StopReason::Cancelled for the pre-cancelled request. The no-publication contract checks only NotProduced, empty text/source map, empty rules/regions, and absent initializer/init. For the one-under successful all-evidence whole-request budget, a produced report must match the baseline body exactly with one finally and cleanup; a stopped report must satisfy the same empty-publication assertions. This still makes no claim about the internal Builder cutoff or rollback.

Patch only; not applied, compiled, or executed.
