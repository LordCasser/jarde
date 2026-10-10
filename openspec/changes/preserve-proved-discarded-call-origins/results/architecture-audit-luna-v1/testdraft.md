# Test draft (not run)

Extend the existing exact physical coverage assertion for `cf12_charat_stored_value_drives_char_source_type_and_full_origins` rather than making a new fixture. For each known `append` result pop BCI 82, 92, and 105, assert:

- `direct_of_bci(pop)` is empty;
- `derived_of_bci(pop)` has exactly one segment;
- that segment's `text(&report.text).trim()` is the corresponding complete call statement (`sb.append('_');`, `sb.append('A');`, or `sb.append(c);`), including the semicolon;
- the call's own BCI remains the segment's direct primary origin, and the existing complete-method source-coverage/default-vs-all assertions continue to pass.

Add focused negatives around the existing `DiscardedEvaluations` proof seam (in a small private unit test or using the current `Fixture` assembler, not a general graph harness):

- `invoke; pop` with an extra SSA reader is not entered in `discards` and gets no discard-pop origin on that call statement;
- an intervening instruction between invocation and pop, a pop that reads another producer, a call result written to a local, and `pop2` do not qualify;
- `invoke; pop; invokestatic` keeps the existing qualifier behavior; distinguish qualifier pop from a result-discard pop, including a call that has both a qualifying pop before it and its own result pop after it;
- a deferred invocation consumed at a later reader remains anchored at that reader, while the existing failure quote still names any accounted pop;
- a direct ordinary invocation statement and an accessor-fallback `call_statement` both preserve their verified result pop exactly once;
- budget/cancellation behavior does not change: origin assembly adds no second graph scan or extra AST node, and the existing recovery-boundary test still publishes no partial report.

Do not assert that every arbitrary `pop` must become source. The test contract is the precise existing `invoke; pop` certificate; unrelated `pop`, `pop2`, and refused producers keep their current quotes.
