# CF11 cross-path row review

Static-only review. No source was changed in the shared checkout; no Cargo/JDK/JADX command was run.

Current `nested_side_exit_edge_rows_proved` uses full `CanonicalBlockId` equality in `owners`, `internal`, `entry`, and `gateway` sets. It refuses an owner-incident edge unless it is an exact internal row, the one exact entry, or the one exact gateway. Thus an owner-to-foreign-path row is refused by the existing policy. The current CF11 real-Reader test exercises real pathless edges and numerous local malformed-row cases, but has no non-empty-path identity in that same test. The existing `real_legacy_clone_target_with_different_path_refuses_switch_certificate` test proves a separate switch-certificate path boundary; `real_legacy_clone_compound_root_refuses_uncloned_join_path_without_state_leaks` also proves clone-path refusal for compound roots. Neither calls `nested_side_exit_edge_rows_proved` with a real CF11 owner and a Reader-produced clone ID.

The private patch adds that one direct policy case to the existing CF11 real-Reader test. It analyzes the already-existing `legacy-clone/Test.class` (`method()V`) using the same Reader API and captures its real BCI 7 path `[3]` ID. It then substitutes that ID as the target of the CF11 branch-to-prefix row and expects the exact row policy to reject it. `CanonicalBlockId` has only BCI and call path fields; it carries no owner/class identity. The patch does not claim it does. The test deliberately documents this as a local row metamorphism and explicitly disclaims a valid mixed class, reconstructed `CanonicalCfg`, or changed `NormalFlowView`.

The canonical graph builder's call/return rows can legitimately cross path boundaries, so the patch uses a `Normal` row incident to a proved owner. The side-exit proof's allowed rows are exact path-bound identities; the injected row is not a permitted internal edge, entry, or gateway. It is appropriate to say the policy rejects this incident cross-path row, not that all canonical edges must have equal paths.

## Pins

- Base `crates/jarde-java/src/region.rs`: SHA-256 `9d0f8d057a9c47da7c85c9acde525a0b3e14dc3a653c0f737fd3c3a14b6dc0be`.
- Real legacy clone `tests/fixtures/p3-conditional-switch-boundaries/legacy-clone/Test.class`: SHA-256 `5ef72ab6afa95cd945cb53123155ca99f86f4c99c9ad7fa78c2d58e044e04fa5`.
- CF11 `tests/fixtures/java8/cf11/UpstreamNestedLoops4.class`: SHA-256 `8152aa6a52958eff721ff237f8783dca4572764ac750cc2de909b31ef6c036b5`.
- The patch is a proposed diff only and has not been applied or compiled.
