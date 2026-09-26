# Proved Java 8 varargs call-site fixture

`VarargsCalls.java` is compiled with OpenJDK `javac 23.0.1` using
`javac --release 8 -g:none -d v8 VarargsCalls.java`. The resulting Java 8
`v8/VarargsCalls.class` is 1,845 bytes, SHA-256
`faee29aa9f47ce6cb7dc02537bffe217dfa4a1a08776f39176cbd3d7ddb7be53`.
`javap.txt` freezes its method descriptors and relevant call instructions. The
fixture deliberately has direct calls and array controls in the same class, so
the call target is a member of the selected physical definition.

The positive calls are `count(mark(1), mark(2), mark(3))`,
`strings("one")`, and `objects("a", "b")`. They cover `int...`, a one-element
`String...`, `Object...`, left-to-right side effects, and a throwing element.
The controls include a local-held array, an ordinary `int[]` target, and
same-name `overloaded(Object...)` / `overloaded(String)` binding. `nullElement`
initializes an `Object[]` with a single null. `arrayElement` passes a `String[]`
directly to `Object...`; flattening it changes the runtime array class. These
are intentionally non-expansion cases.

`explicitVarargsArray` is written as `count(new int[]{mark(4), mark(5)})` in the
fixture source, but that spelling is not recoverable from the class file: javac
lowers it identically to `count(mark(4), mark(5))`. Its purpose is a runtime
parity case for multi-element varargs lowering, not a requirement that the
decompiler preserve source spelling. By contrast, a local or parameter holding
an array remains distinguishable and must stay one array argument. A sole null
or array-valued element is conservatively retained because it can denote the
entire fixed array argument.

Baseline commands (run from the repository root):

```text
mkdir -p /tmp/jarde-proved-varargs-baseline
javac --release 8 -g:none -d /tmp/jarde-proved-varargs-baseline tests/fixtures/proved-varargs-calls/VarargsCalls.java tests/fixtures/proved-varargs-calls/VarargsCallsRunner.java
javap -classpath /tmp/jarde-proved-varargs-baseline -p -c -s VarargsCalls
java -Xverify:all -cp /tmp/jarde-proved-varargs-baseline VarargsCallsRunner
jadx --no-imports --no-res -d /tmp/jarde-proved-varargs-jadx /tmp/jarde-proved-varargs-baseline/VarargsCalls.class
jarde-cli class-source --input /tmp/jarde-proved-varargs-baseline/VarargsCalls.class --policy single-class --class VarargsCalls
```

`openspec/changes/recover-proved-varargs-calls/evidence/original-run.txt` is the original runner output. `baseline-jadx.java` is
the JADX 1.5.6 source for the class; `baseline-jadx-compile.java` removes only
the generated `package defpackage;` line, which JADX adds to a default-package
class and which does not compile beside the default-package runner.
`baseline-jarde.java` is the pre-change `class-source` text. It is compilable
after its comments are ignored and preserves every explicit array. Compiling it
with the runner and running with `-Xverify:all` produced the original output.
The frozen original, pre-change Jarde, JADX, and updated Jarde runner results
are in `openspec/changes/recover-proved-varargs-calls/evidence/` as
`original-run.txt`, `baseline-jarde-run.txt`, `baseline-jadx-run.txt`, and
`after-jarde-run.txt`. The two
Jarde runs and original output match exactly; the JADX run differs at the
documented null and covariant-array controls.

| case | original | JADX 1.5.6 | Jarde before | Jarde after |
| --- | --- | --- | --- | --- |
| `ordered` | `count(mark(1), mark(2), mark(3))` | same | `count(new int[]{mark(1), mark(2), mark(3)})` | expanded |
| `oneString` | `strings("one")` | same | `strings(new java.lang.String[]{"one"})` | expanded |
| `objectValues` | `objects("a", "b")` | same | `objects(new java.lang.Object[]{"a", "b"})` | expanded |
| `ordinaryArray` | `plain(new int[]{...})` | same | same | same |
| `explicitVarargsArray` | `count(mark(4), mark(5))` after javac lowering | expanded | explicit `int[]` | expanded |
| `heldArray` | `count(values)` | expanded to element calls | `count(local0)` | `count(local0)` |
| `overloadedArray` | `overloaded(new Object[]{"x"})` | `overloaded("x")` | explicit `Object[]` | explicit `Object[]` |
| `nullElement` | `shape(new Object[]{null})` | `shape(null)` | `shape(new Object[]{null})` | same |
| `arrayElement` | `shape(new String[]{"x"})` | `shape("x")` | `shape((Object[]) new String[]{"x"})` | same |

The original and pre-change Jarde runner outputs match exactly. The updated
Jarde projection also matches the original runner output; its expanded
`explicitVarargsArray` remains semantically identical because only the array
allocation is removed and its elements execute once, in order:

```text
ordered=3/3/123
one=1
objects=2
explicit=2/2/45
plain=2/2/67
held=2
overload=1
null=[Ljava.lang.Object;/1
array=[Ljava.lang.String;/1
exception=stop/2/19
```

JADX is a text-shape reference, not the semantic oracle for the edge cases:
its output changes `arrayElement` to `Object[]` and `nullElement` to a null
array, so the JADX-compiled runner reports `array=[Ljava.lang.Object;/1` and
`null-exception=java.lang.NullPointerException`. The simple positive cases and
ordinary-array negative case match. The implementation must preserve the
original/Jarde behavior on all controls even where JADX flattens them.
