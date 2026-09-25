# `Interface.super` 选择与冗余直接接口边界

## 重放范围

本目录冻结两个 Java 8 探针。`InterfaceSuperProbe` 是普通 Java 源码正例：它的父类和两个无关直接接口都声明了 `value()`，类显式用 `DefaultLeft.super.value()` 解决默认方法冲突，也单独选用 `DefaultRight.super.value()`。`RedundantDirectSuperProbe` 的合法源同样使用 `RedundantChild.super.value()`；重放随后只改 class 常量池中 `InterfaceMethodref` 的 owner，把 `RedundantChild` 改为类文件也直接列出的 `RedundantParent`。类体的指令、直接接口表和其余池项不变。

从仓库根目录执行：

```text
python3 openspec/evidence/java-syntax-2026-09-25/redundant-interface-super/replay.py
```

脚本需要 PATH 上的 Java/JDK `javac`、`java`、`javap`、`jar`，以及 `/opt/homebrew/bin/jadx`、`/tmp/jarde-cli-audit-baseline`（原 CLI 构建的同 SHA 独立副本）；可用 `JADX` 和 `JARDE_CLI` 环境变量覆盖最后两个路径。编译目标固定 `--release 8`，分别建 `-g` 和 `-g:none` 正例。所有 `.class`、JAR、JADX 临时目录、重编产物都在 `TemporaryDirectory` 下；脚本结束时删除。只有源码、文本对照、运行结果、摘要和哈希留在本目录。

本次运行工具版本见 [`tool-versions.txt`](tool-versions.txt)，执行文件哈希见 [`tool-binary-sha256.txt`](tool-binary-sha256.txt)，所有冻结证据文件哈希见 [`SHA256SUMS.txt`](SHA256SUMS.txt)。主要重编 class 哈希见 [`classfile-sha256.txt`](classfile-sha256.txt)。

## 普通多接口冲突正例

原始 Java 8 源通过 `javac --release 8 -g:none` 与 `-g` 编译；两次 `java -Xverify:all` 输出均为：

```text
11
22
33
```

原 class 的 `value`、`chooseRight`、`both` 均以入口 `aload_0` 后接 `invokespecial InterfaceMethod`。常量池分别指向 `DefaultLeft.value` 和 `DefaultRight.value`，详见 [`positive-javap.txt`](positive-javap.txt)。此物理证据保留了 JVM 特殊调用的接口 owner，足以区分父类里的同名实现。

Jarde `class-source` 对 `-g:none` 和 `-g` JAR 都输出 `DefaultLeft.super` / `DefaultRight.super`，两份导出文本逐字相同。产物分别在 [`positive-jarde-InterfaceSuperProbe.java`](positive-jarde-InterfaceSuperProbe.java) 和 [`positive-debug-jarde-InterfaceSuperProbe.java`](positive-debug-jarde-InterfaceSuperProbe.java)；两份重编后在原有父类、接口和 runner 上运行，都输出 `11, 22, 33`。无调试表的重编 `InterfaceSuperProbe.class` 哈希与原 `-g:none` class 一致。报告仍标 `syntax_status = unchecked`、`verification = not_performed`；这是我们额外直接编译、运行确认的结果，不应误读为 class-source 自身执行了 Java 验证。

JADX 1.5.6 在 [`positive-jadx-raw.java`](positive-jadx-raw.java) 中将三处都写成裸 `super.value()`。移除其生成的 `package defpackage;` 后，导出源码确实可用 Java 8 编译运行，但结果变为 `7, 7, 14`，见 [`positive-jadx-run.txt`](positive-jadx-run.txt)。这是该输入上可观察到的分派错误；不能由一个样本推断 JADX 的所有接口 default 调用都错误。

Jarde 的机制保留了池项类别：[`decode.rs`](../../../../crates/jarde-java/src/decode.rs) 第 661–696 行区分 `MethodRef` 与 `InterfaceMethodRef`。非构造 `invokespecial` 进入 [`build.rs`](../../../../crates/jarde-java/src/build.rs) 第 12891–12905 行的 `special_receiver`；该函数第 12943–12977 行证明 receiver 是入口 `this`，再比较同次类头的直接父类/接口和池项类别，接口形式构造 `Super { qualifier: Some(owner) }`。[`emit.rs`](../../../../crates/jarde-java/src/emit.rs) 第 861–867 行将其写为 `Qualifier.super`。

本机 JADX 源码 checkout 为 `/Users/lordcasser/workspace/testzone/jadx`（1.5.6 对应的 `2fb1b163`）。[`InsnGen.java`](/Users/lordcasser/workspace/testzone/jadx/jadx-core/src/main/java/jadx/core/codegen/InsnGen.java) 第 883–887 行将所有 `InvokeType.SUPER` 转交 `callSuper`；第 1080–1116 行用 `getClassForSuperCall` 查找目标，并在返回当前 class 时写裸 `super`。本例中当前 class 已实现两个接口，类型关系检查把接口 owner 视为当前 class 的 supertype，于是输出落到父类同名方法。源码说明了与该输出相符的路径，运行值也确认了错误；单个探针没有逐 pass 跟踪，不足以描述其它 owner 形状或所有 JADX 调用的选择结果。

## Verifier-valid、但不能拼成合法 `Parent.super` 源码的反例

原始 [`RedundantDirectSuperProbe.java`](RedundantDirectSuperProbe.java) 实现两个直接接口：`RedundantParent.value()` 返回 3，`RedundantChild extends RedundantParent` 覆盖为 4。正常 Java 源明确调用 `RedundantChild.super.value()`，能编译并在 `-Xverify:all` 下返回 4，见 [`negative-legal-original-run.txt`](negative-legal-original-run.txt)。`javac` 明确拒绝把它改成 `RedundantParent.super.value()`：Parent 虽在类的直接接口表中，但它被另一个直接接口 Child 扩展，是语言规定的冗余 qualifier。拒绝日志见 [`negative-jarde-javac.txt`](negative-jarde-javac.txt)。[`RedundantIndirectNegative.java`](RedundantIndirectNegative.java) 另记录了更简单的反例：只直接实现 Child 时，不能用非直接父接口 Parent 作 qualifier；对应 javac 错误在 [`negative-indirect-javac.txt`](negative-indirect-javac.txt)。

重放只改合法 Child 调用所引用的 `CONSTANT_InterfaceMethodref` 的 owner index。`javap` 前后材料分别在 [`negative-javap-before.txt`](negative-javap-before.txt) 与 [`negative-javap-patched.txt`](negative-javap-patched.txt)：指令仍是 BCI 0 `aload_0`、BCI 1 `invokespecial`、BCI 4 `ireturn`，常量池目标由 Child 变成 Parent。`java -Xverify:all` 接受改写后的 class，并运行调用 Parent default 得到 3，结果见 [`negative-original-run.txt`](negative-original-run.txt)。这证明该输入是 JVM 可执行的 classfile；它不证明输入来自合法 Java 源，实际上 javac 源级表达式不存在。

当前 Jarde class-source 只看到当前类头中的直接接口名字和该条池项的 interface 标志，输出 [`negative-jarde-RedundantDirectSuperProbe.java`](negative-jarde-RedundantDirectSuperProbe.java) 中的 `RedundantParent.super.value()`。当前类报告虽是 `unchecked` / `not_performed`（见 [`negative-jarde-report-summary.txt`](negative-jarde-report-summary.txt)），恢复正文却像正常源码；我们额外用同一 JDK 编译，javac 以冗余接口错误拒绝。Jarde 判定位置是 [`build.rs`](../../../../crates/jarde-java/src/build.rs) 第 12959–12977 行：这里只确认 owner 等于某个直接接口，未检查另一直接接口是否为它的子类型。

## 结论边界

普通 javac Java 8 输入中的显式冲突选择已可恢复，无需新增 AST 或 pass。缺口限定在 verifier-valid、但不是合法 Java 源编译产物的 interface invocation：当前“池项为接口引用 + owner 出现在 direct interfaces + receiver 为入口 this”不足以证明 Java `I.super` qualifier 合法。可在尚未完成的 [`preserve-special-call-dispatch`](../../../changes/preserve-special-call-dispatch/) 中补充语言合法性门：需要足够的直接接口继承关系证据证明无冗余更具体接口；当该关系未知或输入范围缺少定义时应保留带 BCI 的 fallback。判定 transitive 子类型关系可能要求有界读取接口头并受现有环境/加载身份约束，不能仅凭相同方法签名或 owner 直接匹配猜测。负面补丁正好隔离此门；两个无关接口各自显式选择的正例确保新拒绝门不一刀切。

这些结果只覆盖上面冻结的两个探针、OpenJDK 23.0.1 `javac --release 8`、JADX 1.5.6 及所记录 Jarde CLI。`-Xverify:all` 是运行时可接受性证据，不等于一般 classfile 校验器；也没有据此推断任意工具、继承图、桥方法、间接别名或更复杂接口默认分派都能恢复。
