# CompoundIf shared void-return continuation candidate v6

This revision is based byte-for-byte on the frozen v5 private candidate:

- Base: `/private/tmp/jarde-compound-condition-focused-fix-luna-v5/region.rs.candidate`
- Base SHA-256: `4348b48b566d8d28e85f2421e868a171ec984a309c4dfc264d4ee1fbb8418189`
- Candidate: `region.rs.candidate`
- Candidate SHA-256: `e1b43a707abf73cb330a2a0bdaf28b43d6d77c217c65b236ebf7b9c9f2954e6a`
- Patch: `region.rs.patch`
- Patch SHA-256: `7f5c5066dccbb7ddfd2f55f1e8186e9b21b0ade2eac4e8a20566a2d76e833a75`

v6 makes only the requested tightenings to the narrow CompoundIf continuation: the return block must contain exactly one SSA instruction, decoded as opcode `0xb1` / `Operation::Return`; both the arm frame and parent frame scope must contain the return node. The Switch2 test adds two real-canonical refusal cases: an arm scope excluding the return node and a parent-frame case-entry set containing it. Both assert that `visited`, `unclosed_tail_at`, `catch_joins`, `fragmented`, and the run remain unchanged.

No workspace files were changed and no compiler, tests, Git, or other toolchain was run. Root should apply v6 instead of v5 and perform serial validation.
