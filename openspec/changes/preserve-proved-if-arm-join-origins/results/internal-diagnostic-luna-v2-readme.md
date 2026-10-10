# CF07 internal Region diagnostic v2

`diagnostic.patch` is the v2 successor to `/private/tmp/jarde-cf07-if-join-internal-diagnostic-v1/diagnostic.patch`; v1 is left untouched. It is still a private, unexecuted test-only proposal for `crates/jarde-java/src/region.rs`'s internal test module.

The class input now points at the currently accepted baseline:

`openspec/changes/preserve-proved-for-latch-origins/results/cf07-candidate-root-v1/cases/javac23-original/classes/cf07/LoopCases.class`

The If diagnostic now distinguishes the canonical block leader from the physical branch instruction: the block is expected to start at BCI 11, while `branch_bci` is expected to be 14. The print includes both. No other diagnostic behavior changed.

The analysis follows the existing real fixture setup at `crates/jarde-java/src/build.rs:35354-35485`. In particular, the `recover` call arguments match that helper: canonical CFG, NormalFlowView, SSA, decoded Operations, the same pool, `plan_four_conditional_strings` result, empty init Sites, MethodCodeFacts, `Some(false)` synchronization, the descriptor-derived boolean-return fact (false for `(II)I`), `JAVA_8`, and the same mutable budget. The test requests `AnalysisStage::ALL` and derives every value from that one `analyze_method_ir` result. Its runtime profile is Java 23 because the frozen input is the javac23 class; the recovery rule profile remains the helper's `JAVA_8`.

The edge-print and assertions use the checked API types: `CanonicalEdge::kind()` returns `CanonicalEdgeKind` (`crates/jarde-jvm/src/canonical.rs:130-143`), so the source/target tuples infer as `(u32, CanonicalEdgeKind, u32)` and compare directly with the expected normal-edge tuples. The test prints all canonical edges as well as both arm tails' outgoing edges.

No repository file was changed. I did not apply the patch or run Cargo, rustfmt, Git, JDK, CLI, or candidate code. The patch must still be applied and compiled by root's guarded test workflow before use.
