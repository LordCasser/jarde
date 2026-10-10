# Ordinary while latch-origin private patch (Luna v2)

Status: **private candidate only**. This complete patch supersedes v1 for review; v1 remains unchanged. It is not applied or tested, and no OpenSpec tasks are marked complete.

Patch: `loop-latch-origins-luna-v2.patch`. It still changes only `crates/jarde-java/src/region.rs` and `crates/jarde-java/tests/p3_loop_exit_gateways.rs`. The production proof is unchanged from v1.

## Test-only corrections from v1

The no-prefix positive now asserts `default.source_map == all.source_map`, names the frozen method’s exact 11-instruction BCI inventory once, and checks every primary and derived source-map origin belongs to that inventory. It reconstructs the expected standalone `PhysicalDefinitionId` from the frozen class bytes and compares each method origin’s complete owner against it, in addition to checking the method name and descriptor.

The budget case now measures a complete `RecoveryEvidenceRequest::essential()` run and retries that same primary request with one fewer `AnalysisSteps` unit. This makes the no-partial-artifact stop assertion independent of optional evidence work. Cancellation likewise uses the essential request. The all-evidence positive remains responsible for the full BCI/source-map assertions.

## Fixture and validation boundary

The input remains frozen javac 23 `PlainOneArmLoops.class`, SHA-256 `a94a7af3f6258765adac1161d02d76b77f0de3ba69ab2fa3b74327b9819d0989`; `noPrefix(ZI)I` has BCIs `0,1,2,3,6,7,8,11,14,17,18`, with `goto@14` returning to header `6`. The assertions are proposed checks, not test results. Unified hunk counts and offsets were checked structurally; no `git apply --check`, Cargo, formatter, JDK, CLI, or OpenSpec validation was run.

After root reviews/applies the candidate, run the focused integration target and adjacent origin/continue regressions:

```sh
cargo test -p jarde-java --test p3_loop_exit_gateways
cargo test -p jarde-java --test p3_loop_body_double_jumps
cargo test -p jarde-java --test p3_loop_terminal_return
cargo test -p jarde-java --test p3_effectful_exits
cargo test -p jarde-java --test p3_loop_arm_join
```

The current checkout has `p3_effectful_exits.rs` but no `p3_loop_arm_join.rs` test target. The last command is included because the requested adjacent loop-arm-join gate needs explicit coverage; root should resolve its actual target before running that command rather than treating the missing target as a passing check.
