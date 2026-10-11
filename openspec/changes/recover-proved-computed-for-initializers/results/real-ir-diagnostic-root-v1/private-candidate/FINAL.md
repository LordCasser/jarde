# Private real-IR diagnostic candidate

This directory contains an unexecuted, test-only diagnostic insertion in the existing frozen-class regression test. The production recovery path and existing assertions are unchanged. The diagnostic reads the same `lastIndexOf([IIII)I` class and the same IR artifacts already constructed by that test. It reports init BCI 0–3 SSA instructions and values, every local-4 access, relevant phis, natural-loop blocks, return leaves under the incoming `Frame::default()`, and canonical edges around the update/latch.

Apply `diagnostic.patch` only after review. The exact unit-test name is `region::tests::cf07_return_arm_latch_proof_stops_at_shape_and_edges_from_real_ir`; from the workspace root, the command is `cargo test -p jarde-java region::tests::cf07_return_arm_latch_proof_stops_at_shape_and_edges_from_real_ir -- --exact --nocapture`. No test or toolchain was run while preparing this candidate.

For byte-exact rollback, first retain the live `crates/jarde-java/src/region.rs` bytes and verify they match `base-region.rs` SHA-256 in `pins.json`; after the diagnostic run, restore those exact bytes (or reverse `diagnostic.patch`) and verify the same base SHA-256. This private copy is not a repository edit.

Files: `base-region.rs` is the complete source snapshot, `candidate-region.rs` the complete diagnostic candidate, and `diagnostic.patch` the three-context unified diff. `pins.json` records SHA-256 and byte lengths read from each actual input/artifact.
