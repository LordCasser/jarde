# Forward binding and exception-handler boundaries

Replay both fixtures from the repository root with:

```sh
python3 openspec/evidence/java-syntax-2026-09-22/interface-field-initializers/forward-binding-exception-edge/replay.py \
  --cli /tmp/jarde-cli-member-root-accepted
```

The script checks that the frozen Jarde CLI has SHA-256
`19ede7a6fe88b95530637e9765c7ae02b9f233520dc6a2e2e815f0dca58d0552`. It
compiles all original source with `javac --release 8 -g:none`, applies only
the fixture-specific classfile changes described below, and saves the full
original/JADX/Jarde source sets, compile logs, class bytes, `javap -v -p -c`,
class SHA-256, and runtime status. No external target code is downloaded or
executed. The tested patched class files pass `java -Xverify:all`.

## Qualified forward read

`forward-binding/original-source/ForwardProbe.java` uses a qualified read of
the later field:

```java
int EARLY = ForwardProbe.LATE;
int LATE = BoundaryEffects.value();
```

This is legal Java 8: the qualified name avoids the simple-name illegal
forward-reference rule. Since `LATE` is not a compile-time constant, the
initializer reads the field during interface initialization, before its
later `putstatic`; it sees the default `0`. `javap.txt` shows the original
`<clinit>` write order. The script swaps the complete `EARLY` and `LATE`
`field_info` records without changing Code. The input remains verifiable and
prints `L|0|9`, so physical field order does not remove the observable
forward binding.

JADX emits a complete source in the physical field order, which compiles and
passes `-Xverify:all` but prints `L|9|9`. The value differs because the
reconstructed declarations now initialize `LATE` first. Jarde emits its
uninitialized fields plus interface `static {}` representation; compilation
fails and no runtime is claimed. All generated text and diagnostics are
preserved without editing.

This pair makes the declaration-order requirement concrete: moving a valid
qualified field read across the target field's initializer changes the
result. The field expression can be represented as Java syntax, but safe
projection must retain the observed evaluation point. An unqualified
`EARLY = LATE` is a distinct source spelling and is rejected by javac as an
illegal forward reference.

## Exception handler in `<clinit>`

`exception-handler/original-source/ExceptionProbe.java` is first compiled as
a normal Java 8 class because Java source does not permit an interface static
initializer block. The script then transforms the resulting classfile into
an interface: it sets the interface and field access flags, makes the Java 8
static observer public, removes the class constructor, and retains the
initializer bytecode and its exception table unchanged. It does not alter
the handler code or its frame metadata. The final class has one
`<clinit>` exception-table entry, passes `javap` parsing and
`-Xverify:all`, and prints `ABEC|A|B`.

Both decompilers emit full class source, but the generated interface source
contains a static initializer block and fails Java 8 compilation. Their
complete outputs and compiler diagnostics are saved. This establishes a
verifiable JVM interface boundary with a real exceptional edge; it does not
claim that all handler paths are inexpressible through combinations of
field-initializer expressions. It does show that this handler-bearing
`<clinit>` is not safely handled by a proof restricted to a straight-line
single-write sequence.

See `summary.json` for exact checksums, compiler and verification exits, and
runtime outputs. For generated sources that fail compilation, the runtime
status is null because they were not executed.
