# Constructor-conditional own-CI verifier candidate v2

This private v2 changes only workspace batch observation cross-checking in the v1 candidate. The failed preflight was caused by a representation difference: the workspace runner stores convenience status `ignored`, while the pinned CI parser preserves the full status `ignored, <reason>`. The verifier now normalizes only that convenience status for comparison and requires every recorded `raw_line` to match the corresponding cleaned line from the SHA-verified captured stdout, in order. Thus the ignored reason and every other stdout difference remain visible and checked.

The comparator also rechecks the outcome count, exact named outcome order, summaries and their raw lines, per-target counts, and aggregate totals. Existing metadata-derived target/batch coverage, command receipts, cleanup, source pins, focused receipt/CLI binding, and CI job/step/seed checks are unchanged. This is a narrow comparator fix; it does not alter product code or infer a passing CI result.

The only validation performed for this v2 is Python AST parsing and diff/schema review. The candidate verifier was not executed. No Cargo, Git, network, JDK, JADX, or CLI commands were run.

The parent can review `candidate.patch`, then run the actual preflight against the real receipts.
