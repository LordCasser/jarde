# CF-16: `TestTryCatchFinally4.TestCls.test()V`

This fixture is scoped to the single pinned JADX test method `TestTryCatchFinally4.TestCls.test()V`. It freezes the JADX source and compiled class, captures the complete class-source outputs from pinned JADX and Jarde, verifies a Java 8 source transcription against the pinned method's BCI/opcode/exception-row shape, and includes an explicitly separate helper-seam control for the cleanup paths.

The pinned JADX checkout is `2fb1b16386941660fda07e9017285aec40fcb37f`. The source file `original/TestTryCatchFinally4.java` has SHA-256 `57b52e955fdf2db03a6949b02acc3ce9a096f939ed33c44a05c5fc36b8e0e846`. The nested target class is major version 55 with SHA-256 `2bf1b8932e521aade29fa1d562269903f9b8e9942f86981eb458d4f552d67060`. Its exception rows are `[22,31) -> 34 IOException`, `[17,22) -> 38 any`, `[40,49) -> 52 IOException`, and `[38,40) -> 38 any`. `original.javap.txt` is the frozen `javap -c -v` output of that pinned class.

`original/TestTryCatchFinally4$TestCls.java` is the exact target method extracted into a standalone top-level class so it can compile without the JADX test framework. Compiling it with `javac --release 8` preserves the fixed method's BCI/opcode sequence and exception rows; `bytecode-shape.txt` records the check. The Java 8 output class SHA-256 from the captured replay is `4918d1b726506100c4b9213d2c7bbdf0fbffb38de7e4d7ee679de8b58e3f0a0d`.

The pinned JADX complete class source reconstructs the method as a body `try` with a nested cleanup `try/catch` in `finally`. Jarde's captured complete class-source presentation has no recovered `test()` statements. Its report exposes both the initial recovery refusal and the downstream source-scope refusal: the fallback diagnostics are `jre_guard_handler` at BCI 38 and `jre_region_uncovered_blocks` for five live blocks; the final explanation says local 1 crosses a quoted fallback region and cannot be bound as one lexical definition-use slice. Preserve both levels when interpreting this case: the initial guard/region proof fails first, and the local-binding message describes why the fallback presentation also cannot be emitted as ordinary statements.

The fixed target directly creates a `FileOutputStream`, so the pinned method offers no injection seam for forcing a body `IOException`, cleanup `IOException`, or cleanup `RuntimeException`. Its original source, pinned JADX output, and Jarde presentation compile with `--release 8` and verify-run with `java -Xverify:all`, but the target runner only observes normal completion; that result does not prove behavior recovery. The separate `Control.java` helper-seam probe drives four paths and logs operation order. Original and pinned JADX outputs match for normal completion, body `IOException`, cleanup `IOException`, and cleanup `RuntimeException`. Jarde's explanation-only control compiles as an empty method and produces empty event logs, so it does not match. This control exercises the source-level nesting semantics only: it has different bytecode and is not evidence that Jarde recovered the pinned `TestCls.test()` behavior.

Run the deterministic replay with a Jarde CLI built from this checkout and a fresh output directory:

```sh
./replay.sh /path/to/jarde-cli /tmp/cf16-test4-replay
```

The script verifies the pinned class/source hashes and JADX HEAD, compiles with `javac --release 8`, compares the target bytecode shape, decompiles with pinned JADX, invokes Jarde `class-source`, and runs the target and control variants with `java -Xverify:all`. The output directory retains the regenerated Jarde reports and run transcripts. It does not modify production code or build Rust artifacts.
