# Java 8 interface field initializers

This fixture contains the smallest permanent positive input for recovering
ordinary Java 8 interface fields whose non-constant initializers are
represented by `<clinit>` writes, plus two controlled classfile boundaries
that require complete effect and source-order proof before projection.
`CONSTANT` has its own `ConstantValue`;
`FIRST`, `SECOND`, and `TOTAL` invoke helpers during interface initialization
and leave an observable order trace.

The checked-in sources and classes are self-contained. Rebuild and compare the
runtime golden from this directory with:

```sh
javac --release 8 -g:none -d v8 InterfaceInitProbe.java InitEffects.java InitRunner.java
java -Xverify:all -cp v8 InitRunner | diff -u runtime.stdout -
```

The positive sources were compiled with `javac 23.0.1`; their three checked-in
class files are version 52. SHA-256 values are listed below.
`runtime.stdout` records execution of the original class files under
`-Xverify:all`:

```text
ABT|A1|B2|4|7
```

The output order demonstrates the two field initializer calls, the later
aggregate call, and the preserved `ConstantValue`.

| File | Size | SHA-256 |
| --- | ---: | --- |
| `v8/InterfaceInitProbe.class` | 771 | `e04abe561a505d9022039776b8b29de22e35f6d4dba45a08f19a9b5c07afb527` |
| `v8/InitEffects.class` | 726 | `5e6bf1d10ff1d783988f3574df274a0d7a1dde949085d906ee3a28ff22ec0e05` |
| `v8/InitRunner.class` | 391 | `f5bf3e25708437b412a2e8a0380eeacafade55caf0399339248c06bf3f3b9690` |

The negative inputs begin as the legal Java sources under `negative/`, are
compiled with the same javac release and debug flags, then receive the
documented classfile patches. Their patched class bytes are checked in under
`v8/negative/`. A Java source compile alone cannot produce either patched
class; `replay_negative.py` preserves the patch generator, verifies the
pinned bytes, then runs each saved negative with `-Xverify:all` and compares
its output with its local `runtime.stdout` golden. To intentionally recreate
the checked-in patched bytes, run this from the fixture directory:

```sh
python3 replay_negative.py --write-patched-classes
```

Ordinary replay, which verifies the pinned class bytes and runtime goldens,
is:

```sh
python3 replay_negative.py
```

`extra-effect` appends a real `invokestatic BoundaryEffects.independent()V`
at BCI 16 after both interface field stores. The helper reads
`BoundaryProbe.SECOND`, so the post-store effect observes `B`; folding this
effect into a declaration runs it before a field store and changes the
observation. The legal source has the helper-call anchor, while only the
controlled patch adds the independent `<clinit>` call. The resulting
interface class verifies and prints `ABXB|A|B`.

`forward-read` starts with the legal qualified forward read
`EARLY = ForwardProbe.LATE`, followed by the non-constant `LATE` initializer.
The patch swaps the complete `EARLY` and `LATE` `field_info` records while
leaving `<clinit>` Code unchanged. The class verifies and prints `L|0|9`:
the early read sees the JVM default. Reordering source declarations to the
patched physical field order would move `LATE`'s write earlier and change the
value to `9`; projection must preserve the proven evaluation order or refuse
the group. This is
not an invalid classfile or an arbitrary unsupported Java spelling: it is a
legal Java 8 source compiled to a Java 8 class, then physically reordered as
a controlled, verifier-checked classfile input.

The three positive classfiles have 8 Code attributes total. The six patched
negative classfiles add 16, for 9 classes and 24 Code attributes across this
fixture. Every class is major version 52.

| Negative class | Size | SHA-256 |
| --- | ---: | --- |
| `v8/negative/extra-effect/BoundaryProbe.class` | 629 | `ddbeb07c02d68beddc307e70684cb9c672cf7ffa47ff4ec86719bef6467ebeac` |
| `v8/negative/extra-effect/BoundaryEffects.class` | 606 | `9ce3ee4f3a39f15502b721cbedc382bd8f9ca715c50f40484e5a5ad132a1f2aa` |
| `v8/negative/extra-effect/BoundaryRunner.class` | 390 | `dd75c26f3ee1ddb34c2a160e56fa2c32ede2c2699b383ed46345a6a61a73dba7` |
| `v8/negative/forward-read/ForwardProbe.class` | 548 | `916d57e880cd8bd99c5c705fab87dea60d41365afa3fee3604e5d2dd3a8f7b29` |
| `v8/negative/forward-read/BoundaryEffects.class` | 445 | `a7d5ca4b0a0d32defffad557210761956826861dff49db1c57f57a8dc9441a00` |
| `v8/negative/forward-read/BoundaryRunner.class` | 389 | `dad065664803cadd530c1ddd46256317e33dcb0fca328076cb6a2c62120892b9` |
