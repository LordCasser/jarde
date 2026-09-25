# Interface field initialization phase boundary

This evidence isolates the `ConstantValue` versus `<clinit>` distinction for interface fields. The source in `original-source/PhaseProbe.java` is legal Java 8. `javac --release 8` emits `EARLY` as a `getstatic PhaseProbe.LATE` followed by `putstatic PhaseProbe.EARLY`, and emits a call for the `LATE` initializer. The experiment changes only the three-byte instruction at class-file offset `483` inside `<clinit>` Code: `invokestatic BoundaryEffects.value()I` becomes `sipush 9`. The class file length is unchanged; `summary.json` records the SHA-256 values and the changed byte offsets.

`PhaseProbe.class` is the frozen patched artifact; `PhaseProbe.prepatch.class` preserves javac's class before that one instruction edit. `javap.txt` is the disassembly of the frozen class. The frozen class passes `java -Xverify:all` and prints `0|9`: `EARLY` observes the JVM default value of `LATE` before the later `putstatic` stores 9. Neither interface field has a `ConstantValue` attribute.

The frozen class was decompiled with the CLI identified in `summary.json` and JADX 1.5.6. Each complete emitted source and matching support classes was **attempted** with `javac --release 8`; only a successful compilation was run with `-Xverify:all`. Jarde's source failed compilation (exits: 0/1/None); javac rejects the uninitialized interface fields and the interface `static` block, so there is no Jarde runtime result. JADX's source compiled and printed `9|9` (exits: 0/0/0). This is observably different from the original class's `0|9`: JADX presents `LATE` as a literal, allowing javac to fold `EARLY` to 9.

`javac-constant-mechanism-only/JavacConstantControl.java` is a hand-written mechanism control, explicitly **not Jarde output**. In that source, `EARLY = JavacConstantControl.LATE; LATE = 9;` makes both fields compile-time constants. The class has `ConstantValue` attributes and the runner prints `9|9` because javac inlines 9. This demonstrates why using a literal `static final int LATE = 9` in the projected interface would erase the original default-value observation.

## Replay

From the repository root, run:

```sh
python3 openspec/evidence/java-syntax-2026-09-22/interface-field-initializers/constant-phase-boundary/replay.py
```

To direct generated logs and frozen outputs to a fresh directory, pass `--out /path/to/new-empty-dir`; the executable inputs remain in the requested evidence directory. To use an explicit temporary compilation area, also pass `--work /path/to/work-dir`. The script refuses to overwrite a nonempty alternate output directory.
