# Combined private computed-for task-1.1 diagnostic

This v2 combines the prior real-IR/SSA/CFG diagnostic and the Builder statement/materialization diagnostic in one temporary test-only candidate. Both source files are pinned to the requested region base and the same build base used for the Builder probe. The current workspace files still match those base pins.

The existing frozen reader/API test opens `LoopCases.class` and analyzes `lastIndexOf([IIII)I` once. The Builder probe consumes that test's exact IR, decoded operations and recovered regions through the existing `reuse::plan` and `build` API. It prints the exact `Store(Local(4))` BCI and Builder statement carrying that origin, the full statement list, BCI 0–3 instruction/value frame facts, and the Builder's deferred values, materialization plans, committed bindings and rejections. The 62-line real-IR probe then prints initialization SSA/uses, all slot-4 accesses, phis, natural-loop blocks, return leaves/frame and update/latch edges from those same artifacts. It uses the same test's existing `steps`/`real_walker` helpers and adds no recovery behavior.

The Builder probe in `build.rs` is guarded by `#[cfg(test)]` and the exact physical method name/descriptor. Neither insertion changes production decisions. The only new operation in the region test is an additional real Builder build with a separate bounded diagnostic budget; no new fixture or IR facts are synthesized.

The parent reports the standalone real-IR diagnostic candidate was already run once and passed. That prior run does not establish this combined candidate's compile or test result. Root should apply this combined patch only to the pinned base, run the guarded exact test once with captured output, and restore both source files byte-for-byte to their base pins. The combined candidate itself is unexecuted here.

Exact test: `region::tests::cf07_return_arm_latch_proof_stops_at_shape_and_edges_from_real_ir`.
