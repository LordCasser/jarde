# Proposed minimal repair (not applied)

Keep the existing `DiscardedEvaluations` recognition and accounting rules. Add a narrow accessor such as `discarded_result_pop(call_bci) -> Option<u32>` that reads `discards`; do not infer a pop from adjacency a second time.

At the two successful call-statement construction seams, build the statement origin as:

```rust
let origin = OriginSet::new(Origin::direct(call_bci));
let origin = self.pops().discarded_result_pop(call_bci).map_or(origin.clone(), |pop_bci| {
    origin.plus_derived(Origin::derived(pop_bci))
});
Stmt::new(StmtKind::Expr(call), origin)
```

Factor that into one tiny `Builder` helper if it prevents the inline `instruction()` path and `call_statement()` path from drifting. The helper should return the statement origin, not create a broad call representation abstraction. The call expression keeps its own direct invocation origin; the enclosing statement gets one derived pop origin. This puts the pop on the complete statement span, once, while preserving the call's primary BCI and current expression-level mapping.

Do not add a derived pop to every `call_expr`: a call may be nested, deferred, assigned, returned, or used as a condition, and then the invocation is not the statement that owns a discarded result. `discarded_result_pop` is keyed only by the existing exact one-use `discards` proof. Do not switch the construction case to derived: its current direct anchor at the pop states its different source shape. `quoted_bcis` remains the refusal path.

Before applying, inspect all call-as-statement creation sites. The current direct inline invocation path (`instruction`) and `call_statement` cover ordinary calls plus accessor fallbacks. `ShortCircuitConsumer::Invoke` emits a call expression as a value-bearing conditional statement, not the uniquely discarded-call shape, so it must not receive this pop origin. Constructor sites that already have `Site::discarded` remain separately anchored at their pop.
