# Private candidate: switch cross-case continuation test

This is a private, test-only revision of `tests/p3_switch_forward_join.rs`. The parent run at `/private/tmp/jarde-compound-tail-regressions-root-v1/2.stdout.raw` reports that BCI 37 maps to `"30"`, reflecting source-map token/expression granularity rather than a complete statement span. Existing tests use `text_of_bci` both for exact anchors and, more generally, to require nonempty mapped source; they do not require every bytecode instruction to map to an entire Java statement.

The candidate keeps the exact method text assertion, all-evidence completeness check, and all 13 physical BCI ownership checks. For BCI 37 and 40 it now asks whether at least one nonempty mapped piece lies within the already asserted exact source statement (`local1 = 30;` and `return local1;`). This accommodates expression-level mapping while still tying each physical BCI's map to the expected continuation/return span. It does not assume BCI 40 is the return opcode; the known opcode is BCI 41.

The common helper and cross-case/backedge split remain as in private v2. The original positive and budget tests retain their default evidence request and behavior. No tools or tests were run to prepare this patch; root should apply and verify it.
