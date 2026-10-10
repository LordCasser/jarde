# Focused test draft — not executed

1. Real class: feed frozen `TestSwitchWithFallThroughCase$TestCls.class` to the existing full-class harness. Check dispatch and entries 32/117/146/149 remain separate; case 32 has one route to 117 and another to join 171; the earlier arm does not claim entry 117. Compile/run the complete generated class and compare raw runtime output to the oracle.
2. DAG acceptance: bounded conditional paths may terminate only at the proved switch join or one unique immediately adjacent case entry. Check canonical edge multiplicity and incoming ownership before accepting.
3. Rejection: multiple adjacent entries, cycle/backedge, outside predecessor, duplicate edge, unknown node, and canonical Return/Throw/Exception/Call edge must reject. Use full canonical edges, not a Normal-only projection.
4. Consumer regression: mixed join and adjacent-case exits must render separately. This is blocked until a path-specific switch-exit representation is chosen; group-level `fall_through: bool` is insufficient.
