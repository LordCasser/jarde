# CF16 TestTryCatchFinally13 observable baseline

This read-only fixture audits the fixed `TestTryCatchFinally13.TestCls.test(I)V` shape. The fixed source and its compiled test class are preserved alongside a probe whose `test(int)` token sequence is checked against the fixture by replay. Only `doSomething1/2/3/4` and `logError` are instrumented: they append events, sites 1/2/3 can throw three distinct runtime exception types, and `logError` records the exact type of the exception retained by the probe.

`fixed-class.javap.txt` comes from the pinned JADX test build; `probe-class.javap.txt` is the `javac --release 8` probe. The instruction BCI sequence, every opcode, and all five exception-table entries match. To keep Java 8 target invocation opcodes identical, the probe declares the five called helpers package-visible while the fixed class declares them private; helper visibility is a documented fixture-only difference. The fixed class has class-file major 55 and the probe has major 52. The exact comparison is in `bytecode-shape-check.txt`.

The runner covers the `-12` early return; `>10`, ordinary else, and `-1` branches; and one exception from each of `doSomething1`, `doSomething2`, and `doSomething3`. Original and pinned JADX sources compile with `javac --release 8`, pass `java -Xverify:all`, and produce identical seven-line traces. Each path records exactly one `finally` event. The exception rows preserve the three concrete runtime type names through `logError`.

Jarde CLI returns a Java presentation, but marks `test(I)V` explanation-only and emits no executable statements. The presentation compiles as Java 8, then fails the runner's first early-return assertion with an empty trace. This is the current safe-refusal baseline for this probe. It does not strengthen the fixed JUnit assertion beyond its single source-text check `containsOne("} finally {")`, and does not establish behavior for other finally shapes.

Replay sends generated outputs to a caller-supplied directory or a fresh `/tmp` directory. Build the CLI from the checked-out root into a dedicated Cargo target when needed:

```sh
cargo build -p jarde-cli --bin jarde-cli --target-dir /tmp/cf16-test13-jarde-target
openspec/evidence/java-syntax-2026-09-28/cf16-test13-multisegment/replay.sh /tmp/cf16-test13-jarde-target/debug/jarde-cli
cargo clean --target-dir /tmp/cf16-test13-jarde-target
```

The captured run used Jarde HEAD `dd35384c204f960e4be61b7d47adf3350f96fc71` and pinned JADX HEAD `2fb1b16386941660fda07e9017285aec40fcb37f` (`dev`). `toolchain.txt` and `results.txt` record tool versions and status codes. `jarde-baseline-report.txt` summarizes the method-level refusal; the full machine-readable report is intentionally left in the replay output directory to keep this frozen evidence small.
