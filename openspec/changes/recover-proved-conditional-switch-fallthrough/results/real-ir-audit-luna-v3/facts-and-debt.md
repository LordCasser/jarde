# Facts and deferred debt

## Verified by the completed temporary diagnostic

- The two focused test commands in `execution.json` exited 0; the target diagnostic test reports 1 passed, 0 failed, 0 ignored in stdout.
- Input was the complete frozen original class (1563 bytes); method: `test(IZZ)Ljava/lang/String;`.
- Switch BCI 7 dispatches to 32, 117, 146, and 149; common join is 171.
- All 17 canonical rows are Normal, with no canonical Return rows and no Exception/Call rows; all node paths are empty.
- Case 1 from 32 has paths to case-2 entry 117 and join 171. Case-2 entry 117 and entries 146/149 only reach join 171.
- Exact incoming sets and current fallback regions are recorded in README.md.

## Not established

- No product change was built or accepted by this diagnostic.
- The fixture does not exercise non-Normal edge rejection, terminal source return/throw leaves, clone paths, cycles, or external predecessors.
- The raw dump does not establish source/runtime/map equality after a product change.
- The existing `Loop` fallback label at block 117 does not mean the actual canonical graph contains a cycle; its exact internal reason beyond the printed recovery result has not been diagnosed here.
