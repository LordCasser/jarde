# Private proposal: proved nonempty If join transfer origin

`implementation.patch` contains a focused change to `Builder::region` and a frozen CF07 positive/budget regression. It is an un-applied patch proposal: no repository files were modified, and no formatter, compiler, test, CLI, or JVM execution was run. It must not be treated as implementation acceptance.

The production hunk carries `Region::If.join` into the existing origin construction. For a nonempty arm, it looks only at the arm's final direct `Straight` block, or the final direct child of a `Sequence` when that child is `Straight`. It requires the final SSA instruction to decode as `Operation::Transfer` and the complete canonical outgoing set of that block to contain exactly one `Normal` edge to this If's join. `Operation::Transfer` is produced by the decoder only for `goto` and `goto_w`. The BCI is then added with `OriginSet::plus_derived` to the existing If origin. A missing join, nested final child, missing SSA instruction, other terminal operation, extra edge, exceptional edge, or wrong target adds no origin. The existing empty-arm loop is preserved.

The completed Region tree already passes `region::recover`'s `overlapping_owner` check before Builder sees it. This proposal relies on that existing uniqueness invariant and does not build another owner index. It does not walk inside nested final children. Folded/refused conditional-value branches return before this source logic.

The new positive test includes the frozen javac23 CF07 class, checks all 23 instruction BCIs in `counted(II)I`, default/all text and source-map equality, BCI 20 derived from the complete If segment, the existing condition and outer while latch sources, and the physical owner/name/descriptor for BCI 20. Its budget and cancellation cases require no text or source map after Stop.

## Still required from root

- Read and apply the patch only after reviewing its exact diff; resolve any compile/fmt issues without broadening the change.
- Run the focused test and the production checks under the previously stated disk/time guards. This proposal has not established that the test compiles or passes.
- Build real analysis-only counterexamples from the same decoded/canonical/SSA class IR for a wrong join target, extra normal or exceptional outgoing edge, and a nonterminal transfer. Verify the new origin is absent while the original conservative recovery remains. Do not claim these are covered by the current positive test.
- Complete the full frozen CF07 source/JDK/CLI and exact-product CI acceptance required by OpenSpec. The implementation and tests here do not establish those results.
- Keep `lastIndexOf@25` and the nested else latch as separate known gaps.
