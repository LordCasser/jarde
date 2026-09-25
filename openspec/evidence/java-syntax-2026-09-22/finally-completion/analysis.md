# `try/finally` completion audit

This is a source-only Java 8 audit using self-written methods. It checks the completion order of a `try` return or throw, the overriding completion from `finally`, and observable side effects. It does not propose recovery of `finally` syntax.

The current `present-proved-java-structure` change explicitly treats `catch_type == 0` as ambiguous between a finally handler and catch-all. Its proposal, design, and java8-recovery spec require leaving such regions as references, never spelling them as `finally` or `catch`. This fixture follows that boundary: generated catch-all rows in `javap.txt` are evidence about JVM completion semantics, not proof that Jarde should reconstruct a source-level `finally`.

`FinallyCompletion.java` and its runner compile with `javac --release 8 -g:none`. The exact class is 907 bytes, SHA-256 `2b8902d998065d2d1747abaeba386e006cdf5ff7f1311abb28506e8d8dc0cc94`, with six Code attributes (constructor, four tested methods, and `<clinit>`). `javap.txt` has the full disassembly. Key offsets:

- `normalReturn`: BCI 0–4 computes/stores the try return; 5–10 executes finally and returns the saved value; 12–19 is the exception handler copy.
- `finallyReturns`: BCI 0–4 is the try return computation; 5–9 returns the finally value; 10–15 handles an exception from the try and also returns the finally value.
- `finallyThrows`: BCI 0–4 computes/stores the try return; 5–14 executes finally and throws; 15–25 is its exceptional copy.
- `tryThrowsFinallyRuns`: BCI 0–9 throws the try exception; 10–18 executes finally then rethrows the same object.

The original class passes `java -Xverify:all`. The full output is in `original-run.txt`:

```text
normal:return:1:12
return-overrides:return:4:34
throw-overrides:throw:java.lang.IllegalStateException:try=false:finally=true:56
try-throw-preserved:throw:java.lang.IllegalArgumentException:try=true:finally=false:78
```

JADX 1.5.6 generated a complete class that compiles and passes verification. Its first two cases match. Its `finallyThrows` emits trace `566`, running the finally side effect twice; its `tryThrowsFinallyRuns` emits `8`, omitting the try side effect `7`. The full class and exact output are `jadx.java.txt` and `jadx-run.txt`; no source edits were made.

Jarde used the frozen `/tmp/jarde-cli-bitwise-root-after`, verified SHA-256 `88f2e7aa9b5b8172da02f5af5d4d2c029e72a52d9b3298b46b82774e0bd5fdfd`. The complete class output and report are retained verbatim in `jarde.java.txt` and `jarde-report.txt`. It contains eight `@bytecode` references; compiling the unedited full class fails with four “missing return statement” errors in the four tested methods. The exact compiler stderr is `jarde-javac.stderr`; therefore there is no Jarde runtime result.

Run `python3 run_audit.py` from any directory to replay. It checks the frozen CLI hash, compiles the source with Java 8 target/no debug metadata, runs the original, decompiles the same class with JADX and Jarde, and separately compiles/runs each complete recovered class when compilation succeeds. Work classes are recreated under `/tmp/jarde-finally-completion-audit`; evidence outputs are refreshed in this directory.

Root copied the entire audit directory to an independent temporary location and reran the script
with a separate `JARDE_FINALLY_WORK` directory. The replay exited successfully and reproduced the
907-byte/6-Code class hash, original four outcomes, JADX's two differing traces (`566` instead of
`56`, `8` instead of `78`), Jarde's four missing-return compiler errors and eight references, and
the unchanged frozen CLI hash. The script now resolves evidence paths relative to itself, asserts
the expected class hash and checks the CLI hash again after the run.

Architecture assessment: `guard::finally_copy` currently recognizes a straight rethrow handler
only as a refusal, because `catch_type == 0` alone says neither that normal and exceptional cleanup
copies are equivalent nor that every exit executes exactly one copy. The two overriding-completion
methods have handlers that return or throw a replacement rather than rethrow, so expanding the
existing TWR/monitor/typed-catch rule by label would be unsound. A recoverable non-overriding slice
must first prove copy equivalence, exact protected range and saved return/primary exception flow;
return/throw override needs a separate proof of completion priority. These are distinct planned
boundaries, not results this audit claims implemented.
