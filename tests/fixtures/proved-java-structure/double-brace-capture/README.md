# Double-brace capture: the companion constructor's capture order

`DB.java` is the patrol's own source (`openspec/evidence/java-syntax-2026-10-05/double-brace-patrol/fixture/DB.java`): a
double-brace initializer without a capture (`DB$1`) and one with a capture (`DB$2`,
`new ArrayList<String>() {{ add(s); }}`). javac writes the capture class's constructor with the
synthetic store **before** the superclass constructor call — `aload_0; aload_1; putfield val$s;
aload_0; invokespecial java/util/ArrayList.<init>()V` — which is legal JVM bytecode and not legal
Java source (the assignment before the call is a *flexible constructor body*, a preview feature
javac refuses under `--release 8`).

The fixture freezes **both legs** of that compilation, because the order is the fact the
presentation is about and both javacs have to state it:

| leg | compiler | command |
| --- | --- | --- |
| `v23/` | the ambient javac, `--release 8` | `javac --release 8 -g -d v23 DB.java` |
| `v8/` | a real javac 8 | `<jdk8>/bin/javac -g -d v8 DB.java` |

`freeze.py` compiles both, asserts in each leg that the `putfield val$s` line precedes the
`invokespecial` line in `DB$2`'s constructor, runs both legs under `java -Xverify:all` and
requires `2/z`, then writes `SHA256SUMS`. `JARDE_JAVAC8` names the JDK 8 install (a directory or
the `javac` binary) and defaults to the Corretto install this repository's evidence records.
The only difference the two legs carry is the class flag javac 8 sets on an anonymous class
(`final class DB$2` against `class DB$2`), which the presentation reports faithfully.

`run.sh` recompiles `DB.java` and runs it from a temporary directory — the reproduction tool,
not a guard: the guards are `tests/double_brace_capture.rs` (the presentation, the recompiled
family and its run) and `tests/ctor_reorder_dispatch_guard.rs` (the order-sensitive controls).
