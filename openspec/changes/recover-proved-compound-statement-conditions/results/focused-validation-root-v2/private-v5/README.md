# CompoundIf shared void-return continuation candidate v5

This private candidate is based byte-for-byte on the clean source supplied by root:

- Base: `/private/tmp/jarde-compound-region-diagnostic-root-v5.base.rs`
- Base SHA-256: `761582f7ea807969083ecdb2ff16efbbe276e93f39a5996c202398fc68e8ca87`
- Candidate: `region.rs.candidate`
- Candidate SHA-256: `4348b48b566d8d28e85f2421e868a171ec984a309c4dfc264d4ee1fbb8418189`
- Patch: `region.rs.patch`
- Patch SHA-256: `c522de76032fc80198b10655e7403ff2b4996c1182c181cf67df559a5b159da8`

The production change adds one narrowly scoped path before the existing boolean-return continuation. It accepts only one outer `If` whose then arm is one already-proved `CompoundIf`, whose else arm is empty, and where both joins equal the current arm's next block. The next block must be fresh, outside the parent frame boundary and case entries, in the same canonical path, and end in a physical `return` opcode `0xb1` with no canonical outgoing edge. Every canonical incoming edge must be a unique Normal edge from a physical block owned by the outer `If` run; at least one must originate in a nonempty CompoundIf arm. The existing walker then claims exactly that one return block in the parent frame. Refusal and Stop after the walk begins restore `visited`, `unclosed_tail_at`, `catch_joins`, and `fragmented`; analysis charges and cancellation polling remain active.

The pre-existing boolean-return path keeps its old checks and behavior. No production code was built or tested in this private task. The existing real Switch2 focused test now also exercises the narrow continuation directly, including exact one-block ownership, late analysis-budget Stop rollback, and cancellation rollback. Root must apply this candidate serially and run the focused suite and broader accepted checks. Edge-injection/exception/case variants are not represented by this test; they remain for root's whole-reader controlled-class validation.
