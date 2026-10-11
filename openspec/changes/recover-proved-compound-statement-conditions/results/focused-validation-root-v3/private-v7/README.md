# CompoundIf shared void-return continuation candidate v7

This revision starts from the frozen v6 private candidate:

- Base: `/private/tmp/jarde-compound-condition-focused-fix-luna-v6/region.rs.candidate`
- Base SHA-256: `e1b43a707abf73cb330a2a0bdaf28b43d6d77c217c65b236ebf7b9c9f2954e6a`
- Candidate: `region.rs.candidate`
- Candidate SHA-256: `40d3338e4d9a9bf437229d47f7990ccc5a2fde2513bcb76d50fa42a8dc89ec40`
- Patch: `region.rs.patch`
- Patch SHA-256: `55cf002162885a0ed5e46c7c3fdf43a5492620df994ceb7ff61510d16b01a583`

v7 corrects the real caller's frame contract. The continuation target must differ from the arm frame boundary; `region_at` walks it in that original frame, without substituting or widening a frame. The Switch2 direct test now gives both frames the actual enclosing boundary at BCI 164 while the shared return is at BCI 136. This addresses the earlier synthetic test's mistaken arm boundary.

All v6 constraints remain: the return node must be in both frame scopes when scopes exist, must not be a frame/shared/switch boundary or case entry, and must contain exactly one SSA instruction decoded as opcode `0xb1` / `Operation::Return`. The existing same-canonical scope and case-entry negatives still check full Walker/run rollback.

Root reported its v6 run as 16 passed / 1 failed, with Switch2 still `ArmsDoNotMeet@71`; no v7 build or test has been run here. Candidate is based on the v6 private source (`e1b43...`), while root's formatted v6 source pin was `b9ae117f6b0465966f105c941fc37789cde116a5a7ecde813b3fefb79433c95e`; root should compare this narrow v6-to-v7 delta and apply it to the formatted source as needed.
