# Facts: successful calls whose return is popped

## Reproduced evidence already on disk

`recover-proved-local-source-types/results/focused-root-v2/1.stderr.raw` is an actual failed focused run (not a new run). The Rust compilation succeeded; the failing exact-source assertion names physical BCIs `[82, 92, 105]`. Its produced Java body is complete and uses `char c = str.charAt(i);`, `switch (c)`, and three calls `sb.append('_')`, `sb.append('A')`, and `sb.append(c)`. `1.stdout.raw` records the char test failed, the null/String test passed, and the budget/cancellation test passed. Parent's typed diagnosis identifies 82/92/105 as the `pop` following each non-void `StringBuilder.append` invocation. No switch-transfer BCI is named by the failing assertion.

## Where each bytecode pop goes

`Builder::discarded_evaluations` scans each instruction stream once and recognizes two distinct, narrow shapes. For `invoke; pop`, it records `discards[call_bci] = pop_bci` only when the pop reads exactly the value the immediately preceding invocation produced, that invocation writes no local, and the SSA value has exactly one use at that pop. It marks the pop accounted. `instruction()` then returns `Ok(())` for that accounted `Operation::Other`, so the pop does not emit a fallback statement or its own anchor.

For `produce; pop; invokestatic`, the map instead stores `qualifiers[static_call_bci] = (pop_bci, value)`. The static call expression uses the discarded value as its source qualifier. If a call refuses, `quoted_bcis` and `quoted_qualifier_producer` include the relevant pop(s), so failure output does not lose them.

On success, ordinary invocation paths construct an expression and a statement with only the call BCI as the statement's direct origin: the inline `write.is_none()` path in `Builder::instruction`, and `Builder::call_statement` (also used by accessor fallback). `call_expr` makes the call expression's primary origin the invocation BCI. Neither reads `discards[call_bci]`; thus the accounted pop is neither a statement nor an origin on a successful call. This matches the failure: the text has all three append statements, but their following physical pop BCIs have no source-map record.

The source-map emitter wraps the complete `Stmt` in a node before replaying its nested `Expr` node. A derived origin on the statement therefore maps the pop to the full `append(...);` statement span; no emitter change or fake statement is needed. `discarded_construction` already demonstrates the distinct constructor case: the finished construction statement is directly anchored at its verified discard pop. That does not cover ordinary invocation result discards.

The deferred-call path is separate. A call whose result is read is postponed and later written at the consuming reader; it is not an `invoke; pop` shape because the SSA use list would contain the reader instead of a unique pop. If deferred presentation fails, the quote walk already appends `pops_of(deferred_call)`. Do not add an expression statement or discard-pop origin to a deferred call that is not emitted there.

There is a second recognized pop role, the static-call qualifier (`produce; pop; invokestatic`). The successful `call_expr` renders the value as a qualifier but its call `Expr` currently starts with only the invocation's direct origin; it does not add the qualifier pop. That is a related possible whole-method origin gap, but it is not the `[82, 92, 105]` failure: those are `discards` entries. Keep this proposed repair keyed to `discards` and record qualifier-pop mapping as a separate follow-up unless the new change explicitly takes the complete accounted-pop contract. If that follow-up is taken, the qualifier pop belongs on the call-expression node (which may be nested), not indiscriminately on the enclosing statement.

## Existing nearby permanent coverage

- `crates/jarde-java/tests/cf12_proved_local_source_types.rs::cf12_charat_stored_value_drives_char_source_type_and_full_origins` checks exact physical instruction coverage on the frozen upstream class. It is the direct permanent anchor; its current real failure is exactly the three missing pop sources.
- `crates/jarde-java/tests/p3_java_recovery.rs::a_discarded_construction_with_a_dynamic_argument_stays_refused` and the discarded/unconsumed dynamic-site cases exercise refusal and preservation of side effects, but do not assert a successful ordinary call-result pop's source origin.
- `p3_java_recovery.rs::an_unconsumed_array_length_keeps_the_loop_test_quoted` exercises a different `arraylength; pop` refusal. It is not a successful discarded-call shape.
- `p3_patterns.rs` has a classfile interpreter that models `append` effects, not an origin assertion for Jarde's statement.

I found no existing permanent test asserting `derived_of_bci(pop)` for a successfully emitted ordinary invocation. The failing CF12 exact-coverage test already supplies the missing regression case.
