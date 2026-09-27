# Test13 verifier-valid neighboring shapes

These are small negative-boundary probes derived from the observable Test13 fixture. The `test(I)V` token stream is checked against the fixed JADX source. The helper-instrumented `template.class` has the same test-method BCI sequence, opcodes, and exception table as the frozen probe; only helper implementations and fields add controls to trigger cleanup and uncaught-error paths. Each byte mutation is local and recorded beside its class. Each retained variant passed `java -Xverify:all` while the runner checked its exact trace and escaping exception.

The samples exercise four adjacent differences:

- `cleanup-target.class` changes only the early-return cleanup call at BCI 11 from `doSomething4()V` to same-owner/same-signature `doSomething3()V`. The BCI sequence and exception table stay the same. On input `-12`, the trace becomes `do1,do3,` and no finally helper runs.
- `range-expanded.class` changes the catch-all row `[0,10) -> 56` to `[0,14) -> 56`, so the range now includes the early-return cleanup call at BCIs 10–13. `range-control.class` uses the original range with the same one-shot cleanup failure. In control, cleanup throws `AssertionError` after one finally event. With the expanded range, the handler runs cleanup a second time before rethrowing that same `AssertionError`; both classes verify. Pinned JADX compiles the expanded class but its runner misses the second finally event.
- `branch-bypass.class` changes only the `if_icmpne` edge at BCI 7 from target 15 to the existing return at BCI 63. For input `0`, it skips the branch body and its normal cleanup; the verified trace is `do1,`.
- `rethrow-changed.class` changes BCI 61 in the catch-all handler from loading the saved `Throwable` to `aconst_null`, leaving the following `athrow` in place. An injected `AssertionError` reaches the catch-all and runs cleanup, then `athrow null` completes with `NullPointerException`.

`*.test.javap.txt` records every instruction and the full exception table for each class. The `*.mutation.txt` files identify changed operands or ranges. `*.original.run.txt` records verifier-run traces; matching `*.jadx.*` and `*.jarde.*` files show decompiler behavior for the same class bytes. Pinned JADX preserves cleanup-target, branch-bypass, and rethrow behavior. The expanded-range case exposes a finally-count error after successful Java 8 compilation. The current Jarde source snapshot recorded here is `32155897af127badaa495979011d6c0838215400`, distinct from the frozen main baseline `dd35384c204f960e4be61b7d47adf3350f96fc71`: Jarde gives a whole-method explanation-only refusal for the first three variants. On `rethrow-changed`, it presents partial try/catch text with an explicit unsupported-handler explanation instead of a whole-method refusal; its Java 8 output compiles but fails the trace runner.

This evidence proves only these patched class files and runner paths. It does not strengthen the fixed JUnit assertion beyond its `containsOne("} finally {")` source check, and it does not imply that arbitrary malformed bytecode or other finally shapes are recoverable.

Replay writes generated artifacts to the supplied output directory or `/tmp` by default. Pass the Jarde source HEAD used to build the CLI so the result records its provenance:

```sh
cargo build -p jarde-cli --bin jarde-cli --target-dir /tmp/cf16-test13-negative-target
JARDE_SOURCE_HEAD="$(git rev-parse HEAD)" \
  openspec/evidence/java-syntax-2026-09-28/cf16-test13-multisegment/negatives/replay-neighbors.sh \
  /tmp/cf16-test13-negative-target/debug/jarde-cli
cargo clean --target-dir /tmp/cf16-test13-negative-target
```

The captured replay used the immutable source snapshot at `32155897af127badaa495979011d6c0838215400`; its toolchain SHA and statuses are in `toolchain.txt` and `results.txt`.
