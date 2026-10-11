# CF-11 ledger update candidate

This private candidate updates only the existing CF-11 paragraphs in `control-flow.md` and `summary.md`. It records the accepted fixed `TestNestedLoops` whole-class slice: one inner iterator `foreach`, one outer `while`, preserved raw-`List` `Object` header and original `String` cast, and retained raw/generic refusal controls. The source of the accepted facts is `openspec/changes/recover-proved-hoisted-iterator-foreach/verification-root.md` (task 3.1 acceptance and its linked root acceptance artifacts).

The candidate keeps CF-11 marked partially tested, leaves the 71 source-level units and 612-file inventory unchanged, and makes no claim about future workspace or CI results. The `Refusal` and combined compile failures, absent runtime rows, and source gaps remain stated as refusals/limitations. `README.md` was read and intentionally has no proposed change because its file/unit counts are unchanged.

`ledger.patch` is a proposal only; it has not been applied to the shared checkout. The patch is based on the three source file pins below, although only the first two files are edited by the patch. No toolchain or Git command was run.
