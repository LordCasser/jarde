# Private CF07 lastIndexOf Region diagnostic

This is a private test-only patch proposal for `crates/jarde-java/src/region.rs`. It does not modify the repository and has not been compiled or run. No production code or OpenSpec text changes are included.

The test reuses the exact real-class analysis setup from the earlier `diagnose_cf07_counted_if_join_from_real_class_ir` diagnostic: frozen javac23 `LoopCases.class`, `AnalysisStage::ALL`, Java 23 runtime profile, `JAVA_8` recovery profile, and one `analyze_method_ir` result feeding canonical CFG, SSA, decoded Operations, normal-flow view, and `recover`.

It prints the recovered Region tree and narrows the actual loop header to javap BCI 5 and nested If to branch BCI 16. It locates physical goto BCI 25 from the same decoded method, maps it by the published canonical block span (rather than guessing block identity), and prints that block's SSA instructions, full canonical outgoing edges, and all canonical method edges. It also prints the normal-flow natural-loop block/latch identities, the loop's existing `gateway_origins`, and counts of that physical goto's canonical block across the recovered then arm, else arm, If, loop, and method, plus any `LoopContinue { source_bci: 25 }` leaves.

The sole source facts pinned before the run are from the frozen javap listing: loop header 5, conditional branch 16, `iinc` 22, and `goto` 25. The recovered tree shape, canonical leader/edge, natural-latch result, gateway list, and owner counts are diagnostic outputs, not claimed results. Root must apply it only under its guarded temporary-test workflow, inspect the raw output, and restore the product pins afterwards.

Proposed focused test: `jarde_java::region::tests::diagnose_cf07_last_index_latch_region_ownership_from_real_class_ir`.
