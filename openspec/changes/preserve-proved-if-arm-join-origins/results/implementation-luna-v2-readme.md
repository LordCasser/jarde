# Private proposal v2: proved nonempty If join transfer origin

`implementation.patch` is a private, un-applied proposal. It updates the production hunk and tests in v1. No repository files were modified, and no formatter, compiler, test, CLI, or JVM execution was run. The proposed code and tests have not been established to compile or pass.

The production helper checks the last direct `Straight` block of a nonempty arm, or the last direct `Straight` child of a `Sequence`. It requires the final SSA instruction to decode as `Operation::Transfer`, then charges and polls before filtering the complete canonical edge slice. It uses `first()`/`next()` iterator checks instead of allocating a `Vec`; it accepts exactly one outgoing edge, and that edge must be `Normal` and target this If's exact join. Exceptional or additional edges therefore reject the origin too. `Operation::Transfer` is decoded only for `goto`/`goto_w`. The existing empty-arm rule remains unchanged. The implementation relies on `region::recover`'s completed-tree `overlapping_owner` invariant and does not build another owner index or recurse into nested final children.

The positive test pins the frozen CF07 class. It checks all 23 counted method BCIs, default/all text and map equality, condition@14 and outer while latch@30, and goto@20's exact source-map interval: from the unique `if (` position through the else block's closing brace and following newline, wholly inside the outer while body. It also checks BCI 20's physical owner, method name, and descriptor. The budget/cancellation test requires an empty artifact after Stop.

The new counterexample test locates BCI 20 with `inspect_method_bytecode` and uses `InstructionFact.span.start` to modify a copy of the frozen class bytes. It never modifies the fixture or executes either variant in a JVM:

- The first variant changes `goto@20` to target the outer loop header at BCI 6 (`a7 ff f2`, offset -14). It requires the transfer to retain a source while gaining no derived If source.
- The second replaces the three-byte goto with `iinc slot 2, 2` (`84 02 02`). This leaves the terminal block to fall through toward the join; it requires BCI 20 to remain the primary source of its assignment and gain no derived If source.

These are candidate analysis-only boundary tests, not executed evidence. A changed target can affect recovered structure, so the first test asserts only that the If origin is absent and the BCI remains sourced. Extra normal or exceptional outgoing-edge cases still require root to construct a real byte-level counterexample. This patch does not claim those are covered.

Root still needs to read/apply the exact diff, resolve any compile or formatting issues, run focused and production checks under the stated guards, and complete full CF07/JDK/CLI/exact-product CI acceptance. `lastIndexOf@25` and nested else latch remain separate gaps.
