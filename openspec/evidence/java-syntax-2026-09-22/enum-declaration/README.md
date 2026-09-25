# Minimal source-only enum declaration audit

`Stage.java` is a hand-written Java enum compiled as Java 8 by javac 23.0.1 with `--release 8 -g:none`. It has exactly two constants, each with a constructor argument, and one observable instance field. `StageRunner.java` checks `values()` order, both `valueOf` results, the constructor-backed values/methods, and reflected enum-constant fields and their identities. The original class passes under `java -Xverify:all`; `original-run.txt` records the result.

The fixture is compiled in full, then every class is decompiled together with JADX 1.5.6 and the specified Jarde CLI. JADX's complete source set compiles under `--release 8` and its runner passes `-Xverify:all` with the same result. Jarde's complete generated source set is preserved as `jarde-Stage.java` and `jarde-StageRunner.java`; its javac failure and attempted runtime are preserved in `jarde-javac.log` and `jarde-run.txt`. The compiler reports one error: the enum constant field is presented as an ordinary field declaration (`public static final Stage START;`) inside the enum body, where javac expects enum constants. No other compiler diagnostic is present. The runner cannot load because the enum class set did not compile.

`class-sha256.txt` freezes both Java 8 class files. `javap-Stage.txt` and `javap-StageRunner.txt` are full `javap -v -c -p` captures, including the private constructor and helper, so method code, descriptors, flags, and class-file ordering are available for replay. `jarde-cli-sha256.txt` records the executable hash: `/tmp/jarde-cli-static-root-after`, SHA-256 `8806d9aa06ad3b5fbcfe347144d09765dfbf3c9e172ee374eddf9313df893a44`.

## What the class file states and what source needs

The class header has `ACC_ENUM`, which the current source layer already uses to emit `enum Stage`. The two constant fields occur first in the field table, each with `ACC_PUBLIC | ACC_STATIC | ACC_FINAL | ACC_ENUM` and descriptor `LStage;`. The ordinary `ordinalValue` instance field follows. In the current Jarde text, however, the constant fields go through the generic field formatter, which ignores `ACC_ENUM` and emits them as field declarations instead of the required enum-constant list.

The methods and class initializer show the other boundary. The bytecode constructor descriptor is `(Ljava/lang/String;II)V`: the first two parameters are the JVM's enum name and ordinal, and the last is the Java-declared `ordinalValue`. The Jarde text currently exposes all three parameters and calls `super(name, ordinal)`, while Java source enum constructors only declare the last one. `<clinit>` constructs `START` then `FINISH`, passing names, ordinals, and source arguments, and assigns them to the enum fields. It then stores `$values()` in the synthetic `$VALUES` array. The compiler-generated `values()` clones that array; `valueOf(String)` delegates to `Enum.valueOf`; `$values()` builds the array. Jarde emits these artifacts as ordinary fields/methods and leaves `$values()` without a recovered body.

For this boundary, the class-file reader already publishes the class flags, field flags/table order, method descriptors, and method code needed to recognize an enum and inspect this pattern. The header itself is already correct. The missing piece is an enum-aware source model that gathers enum constants in their declared order, associates each `ACC_ENUM` field with its constructor arguments from `<clinit>`, and distinguishes compiler-supplied enum machinery from source-declared members. It must also hide the name/ordinal constructor parameters and avoid spelling the implicit backing field, helper, `values`/`valueOf`, and `<clinit>` as ordinary source members. That is a new source-layer structure/pattern, not a new class-file reader production for this fixture. Recovering more complex enum initialization remains beyond this two-constant example.

## Replay

From this directory, run:

```sh
python3 run_audit.py
```

The script is relative to its own location and was verified from a copied directory. Set `JARDE_CLI` if the frozen executable is elsewhere. It regenerates original class files, SHA-256 values, `javap`, JADX output, Jarde text/JSON, and all compiler/runtime logs and statuses. Required tools are `javac`, `java`, `javap`, and `jadx` 1.5.6.

The root agent independently replayed this directory from `/tmp/jarde-enum-root-VlHaFV` on 2026-09-23. The two class hashes, original and JADX runner output, and Jarde text were identical. Jarde's sole compiler error was identical apart from the replay directory path. The initial `javap -v -c` capture omitted private methods; the script and checked-in capture were then changed to `-p` and rerun without changing either class hash or decompiled output.
