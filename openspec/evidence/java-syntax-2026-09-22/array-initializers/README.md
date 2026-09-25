# One-dimensional array initializer audit

This source-only audit covers Java 8 `new int[]{...}`, `new String[]{...}`, empty initializers, and element expressions with observable effects and exceptions. `ArrayInitializerRunner.java` is compiled independently after each complete class variant; it prints array contents, the trace of element evaluation, and the exception class/message. No generated source was edited.

The input class was compiled from `ArrayInitializerProbe.java` with:

```text
javac --release 8 -g:none -d <classes> ArrayInitializerProbe.java
```

The original class has 1,281 bytes and SHA-256 `998bdb54c863d92cb63cc08c654df21bc674961c652fc46a5c3eaf7de2915c86` (`original-class-sha256.txt`). It has 13 `Code` attributes. The frozen jarde CLI is `/tmp/jarde-cli-deferred-accepted-7747`, SHA-256 `7747b60a17dc635f0e8402d867cb9e74f9472a62057eb4865dbd8f4a207dfb34`; the before/after files confirm it was unchanged.

The original class and the JADX whole-class recompile both pass `javac --release 8`, independent runner compilation, and `java -Xverify:all`. Their 14 output lines are byte-for-byte equal after the runner removes identity-based array `toString()` values; output SHA-256 is `f16eb426dc781c7ac365ff0f85b20c1452269145a306297802105f5eeb5a7349`. The output verifies left-to-right element evaluation: modes `0`, `1`, and `2` stop after traces `a`, `ab`, and `abc` respectively for `int[]`, and `A`, `AB`, and `ABC` for `String[]`; successful cases preserve all three values. Empty arrays have no trace. The normalized comparison is in `audit.json`, with raw outputs in `original-runtime.txt` and `jadx-runtime.txt`.

JADX emits the expected initializer expressions, including `new int[]{1, 2, 3, -4}`, `new String[]{"left", null, "right"}`, and the effectful element calls. Its separately compiled class is 1,292 bytes with SHA-256 `2068c424a728475bd17001f9785145dae5e9ef954ef7aa461b1c792977bec1ee`.

The frozen jarde CLI exits 0 and emits 38 bytecode markers. Its whole generated class fails `javac` with four `missing return statement` errors (`jarde-javac.stderr`), exactly at the four non-empty initializer methods:

| method | generated source | failure |
| --- | ---: | --- |
| `literalInts()` | line 69 | the `newarray; dup; index; value; iastore` chain is quoted, leaving no return |
| `literalStrings()` | line 99 | the `anewarray; dup; index; value; aastore` chain is quoted, leaving no return |
| `effectfulInts(int)` | line 129 | the three effectful stores are quoted, leaving no return |
| `effectfulStrings(int)` | line 152 | the three effectful stores are quoted, leaving no return |

The generated helper methods compile far enough to show that this run's only class-level compiler errors are those four initializer bodies. The independent runner and JVM are recorded as `skipped` for jarde because its class compilation failed; no jarde runtime result is claimed. Empty arrays are the existing recovered boundary: jarde writes `return new int[0];` and `return new java.lang.String[0];`, which preserves their values and effects but does not use initializer braces.

The architecture root cause is the real missing `new T[]{...}` recovery rule. `ExprKind::NewArray` in `crates/jarde-java/src/ast.rs` represents only an allocation with length expressions and explicitly leaves initializer chains as a sequence of stores. `build.rs` recognizes `Operation::NewArray` as a value-producing allocation, but the normal array emission path only renders a creation when its value reaches a reader. Each initializer `dup` is then rejected by `chained_pair`, whose accepted shape is only `dup; store; store`; the nearby comment explicitly calls out `newarray; dup; iconst_0; ...; iastore` as outside that shape. Consequently the creation, copies, and stores are quoted instead of being folded into one `ExprKind::NewArray` with element expressions. The missing capability is therefore a bounded array-initializer shape recognizer and AST/emitter representation, not a runner issue, an array element type issue, or an exception-order ambiguity.

Re-run the complete audit with:

```sh
python3 openspec/evidence/java-syntax-2026-09-22/array-initializers/run_audit.py
```

The script uses a bounded temporary directory, records every command's stdout/stderr/status, preserves the original class, and checks the frozen CLI hash before and after the run.

Root independently reran the same script in `root-7747/`: the class hash, 13 Code attributes,
14-row original/JADX equality, 38 jarde markers and four-method whole-class javac failure
were reproduced with the same frozen CLI SHA. No generated source was edited for the replay.
