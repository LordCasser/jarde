# 异常构造器实参中的字符串拼接

日期：2026-09-26。探针由 OpenJDK 23.0.1 使用 `javac --release 8` 编译。一个 `Probe` 类和 `main` runner 覆盖四种构造器形态：`throw new E(constant)`、`throw new E(label)`、`throw new E("arm-" + label)`、`return new E("arm-" + label)`；另有普通 StringBuilder 拼接返回作为链路对照。

## 三方结果

| 方法形态 | 原/JADX 源码 | Jarde 恢复文本 | 编译与运行 |
| --- | --- | --- | --- |
| `literalThrown`：常量实参 | `throw new ArithmeticException("arm-2")` | 完整 throw 语句 | 原/JADX 整类编译成功并输出 `arm-2`；Jarde 的方法文本也完整 |
| `labelThrown`：参数实参 | `throw new ArithmeticException(label)` | 完整 throw 语句 | 原/JADX 整类编译成功并输出 `arm-2`；Jarde 的方法文本也完整 |
| `thrown`：拼接实参后 throw | `throw new ArithmeticException("arm-" + label)` | 外层分配和 `dup` 回退，catch 路径没有 throw 表达式 | 原/JADX 整类编译、运行成功；Jarde 整类编译报缺少返回语句 |
| `constructed`：拼接实参后 return 新异常 | `return new ArithmeticException("arm-" + label)` | 无 return 表达式；同样的 new/dup 回退 | 原/JADX 整类编译、运行成功；Jarde 整类编译报缺少返回语句 |
| `concatenated`：普通拼接链返回 | `return "arm-" + label` | 完整返回表达式 | 原/JADX 整类编译成功；Jarde 输出 `return "arm-" + label;` |

原始 class、原源码重编 class、JADX 1.5.6 源码重编 class 的 runner 各输出五行 `arm-2`。原源码与 JADX 源码的重编 class 完全相同，SHA-256 均为 `d28a756995fba8e47b2b57ded7f47c9d240a5568184b8c52445dfc81a8905289`。因此本探针里 JADX 保留了行为和顺序。

为单独验收三个可恢复对照，`JardeControlProjection.java` 从 Jarde 输出的 `labelThrown`、`literalThrown`、`concatenated` 方法体原样摘出并加上 runner。它用 Java 8 目标独立编译成功，`java -Xverify:all` 输出三行 `arm-2`。所以这些方法各自没有被另外两个拒绝项连带破坏。Jarde 完整 `Probe` 仍不能编译，因为 `thrown` 和 `constructed` 各缺少一条有效返回路径；Jarde 对这两个方法不生成可执行结果，不能把后续 `ClassNotFoundException` 当作语义运行结论。

## 按 JVM 顺序核对

原 `thrown(Ljava/lang/String;)Ljava/lang/String;` 的关键字节码为：

```text
0:  new           ArithmeticException
3:  dup
4:  new           StringBuilder
7:  dup
8:  invokespecial StringBuilder.<init>()
11: ldc           "arm-"
13: invokevirtual StringBuilder.append(String)
16: aload_0
17: invokevirtual StringBuilder.append(String)
20: invokevirtual StringBuilder.toString()
23: invokespecial ArithmeticException.<init>(String)
26: athrow
```

也就是说 JVM 先执行异常对象的 `new`，随后才分配/构造 StringBuilder、计算完整消息、调用异常构造器，最后 `athrow`。JADX 源码写作 `throw new ArithmeticException("arm-" + label);`，重编 class 与原源码重编 class 逐字节相同，确认该源表达式保持了本样本的指令顺序。`constructed` 的关键顺序同样是异常 `new/dup`、StringBuilder 链、异常构造器，区别只是构造后直接 `areturn`。

## Jarde 首因与边界

Jarde JSON 报告直接记录了首个拒绝：`concat@1` 已接受 BCI 4 至 BCI 20 的 StringBuilder 链；之后 `new@1` 拒绝 BCI 0 的外层异常构造，诊断为 `jre_new_interleaved_effect`，并指出 BCI 4 是外层 `dup` 与异常构造器之间的 `Allocate { ty: "java/lang/StringBuilder" }`。`thrown` 与 `constructed` 给出同一条拒绝，证明首因是嵌套拼接的分配，不是 `athrow`、catch 边或 return 消费方式。

在 [`init.rs`](../../../../../crates/jarde-java/src/init.rs) 中，`sites` 接收 concat 所有权集合 `reserved`，用它避免把 StringBuilder 链再作为独立 `new` 呈现；但外层构造的 [`verify`](../../../../../crates/jarde-java/src/init.rs) 会逐条扫描 outer `dup` 到异常构造器之间的指令。白名单接受 Push、Load、Arithmetic、Negate，以及属于实参 SSA 依赖链的调用；其它操作走 `jre_new_interleaved_effect`。`Allocate` 没有“已被 concat 验证、可作为本构造器实参子表达式”的分支。也就是说 concat 已证明内层链，但 `new@1` 在检查外层范围时没有使用该所有权，这是首因。

throw 层位于后续：[`build.rs`](../../../../../crates/jarde-java/src/build.rs) 的 `Operation::Throw` 从 `athrow` 栈值调用 `render_value`。外层 allocation 和 duplicate 未被 `new@1` 认领后，渲染遇到 BCI 3 的未认领 Duplicate 并回退；catch 正常路径因此剩下一段没有 throw 和 return 的文本。常量及直接参数 throw 都经过相同 throw 呈现路径并成功，说明 throw 层不是最初缺口。

本机 JADX 1.5.6 的源码链路也与结果吻合：[`ConstructorVisitor.java`](/Users/lordcasser/workspace/testzone/jadx/jadx-core/src/main/java/jadx/core/dex/visitors/ConstructorVisitor.java) 把 `<init>` 调用改写为 `ConstructorInsn`；对 fresh instance，它移除 `NEW_INSTANCE` assignment chain 并把 metadata 交给 constructor node。 [`SimplifyVisitor.java`](/Users/lordcasser/workspace/testzone/jadx/jadx-core/src/main/java/jadx/core/dex/visitors/SimplifyVisitor.java) 识别 StringBuilder 构造、append 链与 `toString`，按 append 顺序收集实参、构建 `STR_CONCAT` 并移除 builder 指令。生成阶段 [`InsnGen.java`](/Users/lordcasser/workspace/testzone/jadx/jadx-core/src/main/java/jadx/core/codegen/InsnGen.java) 的 `InsnType.THROW` 直接输出 throw 操作数。这个样本不需要单独的 throw visitor 才能解释 JADX 输出。

这份证据只确定当前缺口：concat 已验证的内层分配未能成为普通构造实参表达式的一部分。它不证明应该允许任意嵌套分配或其它效果。

## 复现命令

以下命令从仓库根目录执行。Cargo target 独立放在 `/tmp`，结束时清理；编译输出也放在临时目录。

```sh
EVIDENCE="$PWD/openspec/evidence/java-syntax-2026-09-26/exception-constructor-concat"
TMP_DIR=$(mktemp -d /tmp/jarde-exception-concat-replay.XXXXXX)
BUILD_TARGET="$TMP_DIR/cargo-target"
mkdir -p "$TMP_DIR/original" "$TMP_DIR/recompiled-original" "$TMP_DIR/recompiled-jadx" "$TMP_DIR/recompiled-jarde" "$TMP_DIR/jadx-src"
javac --release 8 -g -Xlint:-options -d "$TMP_DIR/original" "$EVIDENCE/Probe.java"
jar cf "$TMP_DIR/input.jar" -C "$TMP_DIR/original" .
java -Xverify:all -cp "$TMP_DIR/input.jar" Probe
javap -classpath "$TMP_DIR/input.jar" -c -v Probe
javac --release 8 -g:none -Xlint:-options -d "$TMP_DIR/recompiled-original" "$EVIDENCE/Probe.java"
java -Xverify:all -cp "$TMP_DIR/recompiled-original" Probe
jadx -d "$TMP_DIR/jadx" "$TMP_DIR/input.jar"
sed '1{/^package defpackage;$/d;}' "$TMP_DIR/jadx/sources/defpackage/Probe.java" > "$TMP_DIR/jadx-src/Probe.java"
javac --release 8 -g:none -Xlint:-options -d "$TMP_DIR/recompiled-jadx" "$TMP_DIR/jadx-src/Probe.java"
java -Xverify:all -cp "$TMP_DIR/recompiled-jadx" Probe
mkdir -p "$TMP_DIR/jarde-src"
CARGO_TARGET_DIR="$BUILD_TARGET" cargo build --locked -p jarde-cli
"$BUILD_TARGET/debug/jarde-cli" class-source --evidence all --input "$TMP_DIR/input.jar" --policy plain-jar --class Probe > "$TMP_DIR/jarde.java" 2> "$TMP_DIR/jarde.json"
cp "$TMP_DIR/jarde.java" "$TMP_DIR/jarde-src/Probe.java"
javac --release 8 -g:none -Xlint:-options -d "$TMP_DIR/recompiled-jarde" "$TMP_DIR/jarde-src/Probe.java"
# 以上 javac 对 Jarde 文本预期失败：thrown 与 constructed 缺少返回语句。
javac --release 8 -g:none -Xlint:-options -d "$TMP_DIR/recompiled-jarde" "$EVIDENCE/JardeControlProjection.java"
java -Xverify:all -cp "$TMP_DIR/recompiled-jarde" JardeControlProjection
cargo clean --target-dir "$BUILD_TARGET"
python3 -c 'import shutil,sys; shutil.rmtree(sys.argv[1])' "$TMP_DIR"
```

预期：原源码、JADX 源码各成功输出五行 `arm-2`；Jarde 完整 class 源码编译失败两处；Jarde 对照 projection 独立编译成功并输出三行 `arm-2`。`SHA256SUMS.txt` 固定输入 Java/class/JAR、JADX 源码及两种重编 class、Jarde 源码/报告与 control projection 的哈希。`javap.txt` 保存原 class 指令和异常表；`javap-recompiled-jadx.txt` 保存 JADX 重编 class 指令。

## 文件说明

- `Probe.java`：Java 8 fixture 与 runner；`input.jar` 和 `original-classes/Probe.class`：冻结输入；`recompiled-original/Probe.class`、`recompiled-jadx/Probe.class`：同哈希的两种源码重编输出。
- `jadx-Probe-raw.java`、`jadx-Probe.java`：JADX 原始输出与去掉 `defpackage` 后的编译输入。
- `jarde.java`、`jarde.json`：Jarde class-source 输出和完整报告。
- `JardeControlProjection.java`：只包含 Jarde 已成功恢复的参数 throw、常量 throw、普通 concat 三个方法及 runner。
- `javac-*.txt`、`*-run.txt`、`javap*.txt`、`jadx.log`：编译、运行与反汇编记录。
- `tool-versions.txt`、`SHA256SUMS.txt`：工具/源码版本和冻结输入/输出哈希。
