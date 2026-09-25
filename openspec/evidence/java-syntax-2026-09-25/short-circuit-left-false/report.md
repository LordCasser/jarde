# Left-false short-circuit control for conditional field writes

This control fixture exercises the short-circuit value immediately consumed by a static boolean field write. `ShortCircuitFalse.assign(boolean)` compiles to an `ifeq` from BCI 1 to the false producer at BCI 14, invokes `rhs()Z` only on the left-true edge at BCI 4, converges at BCI 15, then writes `result:Z` with `putstatic`. `rhs()` increments `calls` before returning true, making accidental RHS evaluation visible.

Run `reproduce.sh` from any working directory. It rebuilds Jarde with an isolated temporary `CARGO_TARGET_DIR`, regenerates the fixture and checks the regenerated class byte-for-byte against the frozen class, then saves disassembly, source, logs, and run output here. The temporary directory (including Cargo build output) is removed on exit. Environment: OpenJDK and `javac` 23.0.1; JADX 1.5.6.

Frozen owner class SHA-256: `a766f7a14a7ef71e255555314b88386c11be924ff27bdb3ec880cf09a607cb9d`.

The original frozen owner, JADX's recovered owner, and Jarde's recovered owner all produce the same observable trace under `java -Xverify:all`:

```text
false-result=false,calls=0
true-result=true,calls=1
```

Jarde `class-source` completed (`outcome=performed`, `execution.status=complete`). Only its recovered `ShortCircuitFalse.java` was compiled with `javac --release 8 -g`, against a jar containing the frozen class; execution put the replacement class directory before that jar. In `assign`, Jarde emits `left ? rhs() ? 1 : 0 : 0` as the value for the static field write. The left-false arm does not evaluate `rhs`, and the left-true run evaluates it once. This control therefore reaches the conditional static-field assignment and verifies the behavior after replacing the owner class.

JADX's only recovered source file was likewise compiled against the frozen jar and executed; it writes `left && rhs()` directly and yields the same trace. Compilation and execution logs, Jarde's complete JSON report, generated sources, hashes, and `javap` output are saved alongside this report. The replay does not exercise thrown-exception ordering because the RHS control is a visible counter increment that returns normally.
