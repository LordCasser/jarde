# PriorAssert positive array-name fixture correction (private v4)

Root's scoped Rust v5 run established that the PriorAssert method is a valid positive case: its rendered method keeps `if (!ArrayPriorProjection.$assertionsDisabled)`, the nested `if (!ok)`, and `throw new java.lang.AssertionError()`, while the returned initializer is `new int[]{VALUE}`. The method has one `IntegerConstantName` projection with a `Field(VALUE)` and `MethodPoint(value(Z)[I, bci 23)` anchor. The physical recovery report still contains `return new int[]{7};`. The recorded failure was the old mistaken `contains("new int[]{7}")` expectation; the diagnostic output lives in `results/scoped-rust-root-v5/0.stderr.raw` and its command record is `results/scoped-rust-root-v5/execution.json`.

The delta makes the PriorAssert test a positive same-AST case, checking those actual control-flow tokens, exactly one method projection, the `VALUE` source span, both physical identities, and the unchanged numeric physical recovery text. It renames the old combined test to remove `staged_target`; the independent staged-target adversarial test remains separate.

Patch SHA-256: `b30ad0c09b63e4376f4a0fa3b52137b7aa270cd9b19e49093ce5830bbdcff962`. The private diff was generated against the current applied v3 test source. No toolchain or Git command was run; no product source was edited and the patch is not applied.
