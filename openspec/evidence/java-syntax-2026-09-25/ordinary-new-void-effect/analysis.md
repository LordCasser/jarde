# `new` with an intervening void effect

This fixture freezes a Java 8 class whose `new Target` and `dup` occur before a void call. The JVM initializes `Target` at the `new` instruction, so the original execution order is class initialization (`C`), the intervening call (`S`), then the constructor (`T`). Both JADX and Jarde present the call before `new Target(1)`, producing `SCT` instead. This records the ordinary `new@1` candidate as presented; it does not claim behavioral equivalence.

## Frozen inputs and outputs

The source inputs are `Generate.java`, `Runner.java`, `Side.java`, `Target.java`, and `Trace.java`. `VoidBetween.class` is generated directly with ASM because the bytecode ordering under investigation is not emitted by a Java source expression. `input.jar` contains that class and the four runner/helper classes. `VoidBetween.java` is JADX 1.5.6 output; it declares `package defpackage`, which must be removed for this default-package fixture to compile. `jarde.java` is the frozen Jarde presentation and is copied to the required filename `VoidBetween.java` before compilation. `jarde.json` preserves the class-source report.

The input class SHA-256 is `cba6ef63e220e3b8ec1d1abead07b2e7acb93182b8790bb6ced27b5a237989b7`; the input JAR SHA-256 is `722447f654a6df87f2ace283008b6494ea1a0d7063940e0a3b24e14085457b50`. The two recompiled decompiler outputs both produced SHA-256 `4c3ba4d37690c3e68bc01fdbc3e66e8f830ff65b59820439421d978fbb9148b4` in the replay below.

`Generate.java` was independently replayed with OpenJDK 23.0.1; its output compared byte-for-byte equal to the frozen `VoidBetween.class` (both SHA-256 `cba6ef63e220e3b8ec1d1abead07b2e7acb93182b8790bb6ced27b5a237989b7`). From this directory, the exact regeneration commands are:

```sh
TMP_DIR=$(mktemp -d /tmp/jarde-void-new-generator.XXXXXX)
mkdir -p "$TMP_DIR/generator" "$TMP_DIR/generated"
javac --add-exports java.base/jdk.internal.org.objectweb.asm=ALL-UNNAMED -d "$TMP_DIR/generator" Generate.java
java --add-exports java.base/jdk.internal.org.objectweb.asm=ALL-UNNAMED -cp "$TMP_DIR/generator" Generate "$TMP_DIR/generated/VoidBetween.class"
cmp VoidBetween.class "$TMP_DIR/generated/VoidBetween.class"
```

## Bytecode and recovery evidence

Exact inspection command, run from this directory:

```sh
javap -classpath input.jar -c -v VoidBetween
```

The method body is:

```text
0:  new           #8   // class Target
3:  dup
4:  invokestatic  #14  // Method Side.effect:()V
7:  iconst_1
8:  invokespecial #18  // Method Target."<init>":(I)V
11: areturn
```

`jarde.json` records method `make()LTarget;`, rule `new@1`, and one construction record with `head: 0`, `arguments: [7]`, `constructor: 8`, `presented: true`, and `refusal: null`. The Jarde source presents `Side.effect();` before `return new Target(1);`.

For JADX 1.5.6, the relevant local source is `jadx-core/src/main/java/jadx/core/dex/visitors/ConstructorVisitor.java:72–110`: it rewrites `<init>` to a `ConstructorInsn` and removes the originating `NEW_INSTANCE`; `jadx-core/src/main/java/jadx/core/dex/instructions/ConstructorInsn.java:31–46` copies the constructor arguments. Inferring that the rewrite can lose ordering relative to an intervening void effect is a **local source inference**, not a claim about the entire visitor pipeline. The observed `SCT` is established independently by compiling the frozen JADX source and executing it under `-Xverify:all`.

On OpenJDK 23.0.1, the original class run with `java -Xverify:all` printed `CST`. Recompiling either decompiler output with `javac --release 8 -g:none -Xlint:-options` and running with `java -Xverify:all` printed `SCT`.

## Replay

From this directory, the following reproduces the original and both reconstructed runs. The temporary output directories contain only Java class files and are outside the repository.

```sh
set -eu
EVIDENCE="$PWD"
TMP_DIR=$(mktemp -d /tmp/jarde-void-new-replay.XXXXXX)
mkdir -p "$TMP_DIR/jadx-src" "$TMP_DIR/jadx-classes" "$TMP_DIR/jarde-src" "$TMP_DIR/jarde-classes"
sed '1{/^package defpackage;$/d;}' "$EVIDENCE/VoidBetween.java" > "$TMP_DIR/jadx-src/VoidBetween.java"
cp "$EVIDENCE/jarde.java" "$TMP_DIR/jarde-src/VoidBetween.java"
java -Xverify:all -cp "$EVIDENCE/input.jar" Runner
javac --release 8 -g:none -Xlint:-options -d "$TMP_DIR/jadx-classes" "$EVIDENCE/Runner.java" "$EVIDENCE/Side.java" "$EVIDENCE/Target.java" "$EVIDENCE/Trace.java" "$TMP_DIR/jadx-src/VoidBetween.java"
java -Xverify:all -cp "$TMP_DIR/jadx-classes" Runner
javac --release 8 -g:none -Xlint:-options -d "$TMP_DIR/jarde-classes" "$EVIDENCE/Runner.java" "$EVIDENCE/Side.java" "$EVIDENCE/Target.java" "$EVIDENCE/Trace.java" "$TMP_DIR/jarde-src/VoidBetween.java"
java -Xverify:all -cp "$TMP_DIR/jarde-classes" Runner
shasum -a 256 "$EVIDENCE/VoidBetween.class" "$EVIDENCE/input.jar" "$TMP_DIR/jadx-classes/VoidBetween.class" "$TMP_DIR/jarde-classes/VoidBetween.class"
```

Expected output lines are `CST`, `SCT`, and `SCT`, followed by the hashes recorded above.

## 修复后 Jarde 验收（2026-09-25）

修复后使用当前源码重新构建 `jarde-cli`，同一 `input.jar` 的 class-source 结果分别冻结为 `after-jarde-essential.json`、`after-jarde-all.json`、`after-jarde-range.json`。最后一份使用合法的指令边界 `--evidence-bci 0..11`。三份 `make` 正文逐字一致，均为 `representation=mixed`、`quality=fallback`，且含 `// @bytecode 0`、`// @bytecode 3`、`Side.effect();` 和引用构造 BCI 8 的拒绝注释；不再发射 `return new Target(1);`。三份均报告 `jre_new_interleaved_effect` 并点名 BCI 4。`essential` 不物化 `news` 详情；`all` 与范围选择各有同一条 BCI 0 记录，`presented=false`，requirement 为 `StatementFree` 的公开描述。集成测试还检查 BCI 0、3、4、8 的来源映射。

这次修复选择保守拒绝，所以修复后正文不宣称可独立编译运行；原 JVM `CST` 仍是唯一可执行语义基准。新的负例验收是“不再将 `SCT` 当等价完整源码”，正例则继续确认真实调用实参可以呈现一次。

## SHA-256 manifest

```text
682ef5e36f67cf18276541c86fcaa502e2fed9565e164dedf5a775f57167c26a  Generate.java
a78b2f700536749f1389fc2a7dede83c2770f5dddc6866a193245fe61e6e5579  Runner.java
5365f2ae95be880daf60f5cf183d89fd223274f755f20d894cd206dcd21f5fd4  Side.java
ff3783e91c79acf92a5ed0898a2a480c2124841851901dd8a8f8cbb74d6547b4  Target.java
68dc4eb66554d38b53e1177585e776d870baed21f847a6e48e6e16c5801aea44  Trace.java
cba6ef63e220e3b8ec1d1abead07b2e7acb93182b8790bb6ced27b5a237989b7  VoidBetween.class
5a04ee932b586bdbad44b259f6c060c2b6d61761ecdfbc8cade8734ac373eaf8  VoidBetween.java
722447f654a6df87f2ace283008b6494ea1a0d7063940e0a3b24e14085457b50  input.jar
ff1b5b7b7b9a425e703950464d05a45369a29f259d569f6dd7f5ad8876d433f5  jarde.java
146a3ce6cc945a95502529b163e14acb0b63c9712cbe1e865acf5cf727817601  jarde.json
```
