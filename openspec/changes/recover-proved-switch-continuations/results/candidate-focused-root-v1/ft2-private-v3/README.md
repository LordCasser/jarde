# FT2 private continuation candidate v3

This private candidate implements OpenSpec tasks 2.1 and 2.4 from the frozen `region.rs` SHA-256 `b34f8549ed8cd598a1b0f45b3ba5157df387882eb8955eaad1dc2ca351575f4c`. Its production hunks are byte-identical to the v2 candidate; v3 only reduces the FT2 real-class test's duplicated reader/IR setup.

The test now reuses `with_real_conditional_switch` with the frozen FT2 class, the 16 canonical block BCIs, all 25 Normal edge rows, switch terminal BCI 11, join 175, and case targets 36, 121, 150, and 153. The helper locates the dispatch canonical block from the SSA block whose last instruction has the switch terminal BCI, so it handles this switch at block start BCI 8. Inside the callback, the test takes the helper's canonical graph, SSA, code, pool, operations, and normal-flow view directly, then plans concatenation and runs recovery.

The consumer remains `continue_switch_arm`. The existing switch-arm caller passes `allow_straight_tail = false` and keeps its existing switch-to-switch path. The one-arm-if caller passes `true` only after the early-return and prefixed-loop consumers decline. The proof preserves the enclosing `Frame` and introduces no new `Region` or public API.

For the new path, the consumer first proves a finite physical route from the already-proved switch join to the exact enclosing frame boundary. A single charged `tail_positions` map detects repeated canonical block identities during the walk and is reused to validate every canonical row touching the route. It rejects path changes, branches, loop headers, nested switches, visited/case-entry/excluded nodes, non-dominating nodes, and conditional/switch terminal operations. All incoming edges to the first tail block must be Normal and come from switch owners; each later block has only its prior route block as predecessor; each tail block has exactly one Normal outgoing edge to its next block or the boundary. Acyclicity does not depend on increasing BCI values.

Only after the canonical proof succeeds does the normal region walker claim the route in the same enclosing frame. The returned shape, physical owner set, and newly visited delta must equal the proved route. A Stop or mismatch restores the pre-walk visited set.

The real-class test checks the outer-if terminal BCI 5 and boundary 197, switch terminal BCI 11 and join 175, case ownership excluding 175, one outer-arm owner of 175, and unique method-wide ownership. It maps each SwitchBreak source instruction BCI through the last instruction of its SSA block and verifies the corresponding Normal edge to 175. A negative consumer case removes owner 96 while keeping canonical rows unchanged, making its edge to 175 external to the offered switch-owner set. A zero-step consumer Stop checks location 175 and unchanged visited state.

The candidate has not been compiled or tested. It stayed outside the workspace; root owns integration, compilation, regression testing, and complete-class replay.
