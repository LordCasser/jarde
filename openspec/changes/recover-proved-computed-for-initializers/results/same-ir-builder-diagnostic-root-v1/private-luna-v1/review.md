# Computed-for task 1.1 Builder diagnostic candidate

This is a private, unexecuted, test-only diagnostic patch. The repository was not edited. It adds no permanent assertions and changes no production decisions.

The existing frozen reader/API test `region::tests::cf07_return_arm_latch_proof_stops_at_shape_and_edges_from_real_ir` already opens the pinned CF07 `LoopCases.class`, analyzes the real `lastIndexOf([IIII)I` method through `analyze_method_ir`, constructs its canonical CFG, SSA, decoded operations, frame facts and recovered regions. The candidate reuses those exact objects to call the existing `build` API once, with the same recovered region tree and method signature. This is the smallest route from the existing frozen test to the real Builder statement; no second reader, IR, fixture, or production mechanism is introduced.

The temporary output identifies the exact preheader `Store(Local(4))` among BCIs 0–3, finds the resulting Builder `Stmt` by its direct or derived origin anchor, and prints all statements for context. It also prints each BCI 0–3 operation/instruction and its SSA values' frame type, definition and uses. A `#[cfg(test)]` probe immediately before `Program` publication prints Builder statements, deferred values, binding plans, committed bindings, binding rejections and refused bindings for only this physical method. These are the actual Builder materialization fields, not a second inferred statement model.

The diagnostic invokes existing `reuse::plan` and `build` with independent bounded test budget. All other build evidence uses the same decoded method and existing real-region output. Review the printed `CF07_BUILDER_*` lines together; this candidate deliberately reports facts without asserting the desired computed-for result.

Inputs: `region.rs` SHA-256 `acdb5d22195ce0b5d283fe1b7a3003f35830c85811b18e2e713b7f4728e93168` (the requested exact base); `build.rs` SHA-256 `f4660755cf2ebc699dccf8ee7f7c984a94f6695a22a62cf649d4b728c76a1ade`. The current OpenSpec task/design file hashes are in `pins.json`.

No Cargo/Rust/JDK/Git/CLI command was run. Compilation, test execution and emitted diagnostic values remain unverified; root must run the exact test and restore both source files to the pinned base bytes afterward.
