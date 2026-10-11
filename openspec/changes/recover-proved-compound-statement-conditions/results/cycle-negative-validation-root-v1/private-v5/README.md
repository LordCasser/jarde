# Private cycle-negative test correction (v5)

This candidate changes only the compound-cycle negative test in `region.rs`. It is based on the root-formatted source pin below and has not been compiled or applied.

The root's Java library run reached 348 passing tests and failed this test while looking up physical BCI 83 as a canonical block start in the irreducible mutation. The actual canonical IDs were `[(0, [], [0]), (29, [], [29]), (40, [], [40]), (62, [], [62]), (51, [], [51]), (74, [], [74])]`, with no unreachable blocks. In this mutation, redirecting the only goto at BCI 71 to BCI 40 removes BCI 83 as a raw basic-block leader. The physical suffix at 83 and the return at 97 are represented within the raw block starting at 74. Thus 83 is not a shared-join canonical identity in this variant.

The test now resolves the irreducible variant's candidate exit by finding the pathless canonical interval containing physical BCI 83, pins its start to 74, and verifies the interval's SSA still contains the physical return at 97. The cycle and no-repeat variants retain an exact pathless block-start pin at 83. The test name and local variable no longer assign a false “shared join 83” identity to the irreducible variant. Existing SCC edge/root/predecessor assertions remain unchanged; the refusal call uses the accurately resolved candidate-exit node.

This correction does not establish which proof guard refuses the irreducible candidate. Before this lookup failure, the test had not reached the `compound_statement_condition` proof call for that variant. Root should rerun the targeted Rust test after applying the patch and record the observed refusal path. No Rust/Cargo/JDK/CLI or repository/Git command was run for this candidate.

Files:

- `region.rs.base`: source at SHA-256 `e143ca7549f85ed6ee5d9e8ddcc08c319224c284ac5cdfb2805421e25d7e8db7`.
- `region.rs.candidate`: complete candidate file.
- `region.rs.patch`: unified diff from the pinned base.
- `pins.json`: computed SHA-256 values for the base, candidate, and patch.
