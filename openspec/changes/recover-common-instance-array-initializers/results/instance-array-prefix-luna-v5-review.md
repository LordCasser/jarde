# Instance-array initializer projection: private v5 review

This unapplied patch is based on the current workspace source and retains the v4 changes. It does not alter the frozen v4 patch/review or the frozen `no-clinit-super-args-v1` sources.

Patch SHA-256: `6d21b5dc235c843d11d521a5b70437c9f688bea50194cc2f23ce28c61b4630aa`.

## v5 delta from v4

- Removed the duplicate `compare_literal` helper. `compare_array_value` already compares exact primitive values, presented types, casts, and call arguments through its charged work stack; the removed helper had no callers and could trigger dead-code/Clippy diagnostics.
- Building the removed-BCI list now polls and charges analysis work for each constructor prefix write. Its vector capacity remains covered by the aggregate `IrItems` charge immediately before allocation.
- After every field/method placement is staged and output-charged, the group polls cancellation once immediately before the no-fail mutation loop. No field declaration or member text is committed if that poll returns Stop.

The implementation still publishes only text-only member placements, using `assert_projection_text`; it does not invent a re-spellable emission or source-map anchors. It keeps the exact field/RHS/call-target proof and one atomic field/member-text commit.

## Stop-test scope

The existing façade budget test uses a zero analysis-step limit and proves an early public-call stop leaves the candidate array field unprojected; the pre-cancel test proves pre-cancelled requests remain cancelled. Neither test injects Stop after all candidate text has been staged. The new final `budget.poll()` explicitly covers cancellation at that boundary in the implementation, but v5 does not claim a dynamic late-boundary rollback test or add a cancellation hook.

## No-`<clinit>` control

The frozen source and Runner in `results/no-clinit-super-args-v1/` are input for root's separate original-class oracle. They remain uncompiled and are not yet represented by a classfile-backed Rust test. The child has final arrays, two direct-super constructors with distinct `super(...)` arguments and suffixes, and same-class `mark`/`run` helpers; the static trace has no explicit initializer.

## Validation boundary

The v4 patch was applied only to a temporary copy of the current workspace files, and `patch --dry-run -p1` succeeded there. The v5 patch itself has not been applied to the product, compiled, or tested. No Git, Cargo, rustfmt, JDK, JADX, or CLI command was run.
