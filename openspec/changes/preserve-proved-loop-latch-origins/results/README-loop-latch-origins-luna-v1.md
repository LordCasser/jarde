# Ordinary while latch-origin private patch (Luna v1)

Status: **private candidate only**. The unified patch is not applied, and no Rust/JDK/JADX/CLI or regression command was run for it. It does not mark OpenSpec tasks complete.

## Files and evidence

- `loop-latch-origins-luna-v1.patch` changes only `crates/jarde-java/src/region.rs` and `crates/jarde-java/tests/p3_loop_exit_gateways.rs`.
- The positive fixture is the frozen javac 23 class `openspec/evidence/java-syntax-2026-10-10/one-arm-loop-controls/baseline-root-v1/cases/javac23-original/classes/PlainOneArmLoops.class`, SHA-256 `a94a7af3f6258765adac1161d02d76b77f0de3ba69ab2fa3b74327b9819d0989`. Its `noPrefix(ZI)I` instruction BCIs are `0,1,2,3,6,7,8,11,14,17,18`; `goto@14` is the natural latch back to header `6`.
- The source audit explaining the existing origin fold is `openspec/evidence/java-syntax-2026-10-10/one-arm-loop-controls/implicit-loop-transfer-source-audit-luna-v1.md` (SHA-256 `8ef113a774fe7a97975debf26e84d572177a29bd61d67c930eac74f1a3354114`).

## Candidate delta

A single private `Walker::implicit_tail_latch_origin` proof is called only after either existing header-tested loop construction path has passed its current coverage check. It accepts only a final `Straight` run whose last block is the unique latch of the current, reducible natural loop; `for_header` must be absent; the physical terminal instruction must be `goto`/`goto_w` decoded as `Operation::Transfer`; normal-flow successors and the complete canonical outgoing-edge scan must both identify exactly the current header; and no exception/call edge may leave that block. The candidate and bounded graph scans are polled/charged before the scans; `StopReason` propagates unchanged.

When `loop_arm_join_source` sees origins, it admits only a `While` with no `for_header`, exactly one origin, and that same proof yielding exactly that BCI. Existing exit, closed-body, complete natural-loop ownership, single entry/exit, path, visited, scope, and edge checks stay in place. `two_level_loop_join_sources` and all other loop constructors are untouched. The patch adds no region field, public API, IR type, frame rule, pass, or accepted body shape.

The permanent positive test is in the existing loop-gateway integration test. It checks default/all text equality; all 11 physical instruction BCIs in the all-evidence source map; BCI 14 derived from a nonempty `while` span; exact physical method name/descriptor/class-byte digest and length for every origin; and one structured region owner for each canonical block start `0,6,11,17`. A second test checks budget and cancellation both stop without publishing text or source-map segments. These are assertions to run, not evidence of passing tests.

## Root validation commands

After reviewing/applying the private patch, run the focused new test first, then adjacent regressions:

```sh
cargo test -p jarde-java --test p3_loop_exit_gateways
cargo test -p jarde-java --test p3_loop_body_double_jumps
cargo test -p jarde-java --test p3_loop_terminal_return
```

The first command covers the new positive and budget/cancel checks alongside existing gateway negatives. The next two retain cross-layer continue/jump and terminal-loop regressions. This preparation did not run these commands, `git apply --check`, formatting, or OpenSpec validation.

## Review boundaries

This is a narrow source-provenance proposal, not a general loop recovery claim. Multiple latches, `for` updates, do-while/endless forms, nested/non-Straight final regions, explicit continue/break regions, mismatched targets, multiple normal successors, and any extra/exit gateway origin remain outside the candidate. Root should independently inspect the patch, apply-check it, run the focused and adjacent tests, and then record actual results before treating the slice as accepted.
