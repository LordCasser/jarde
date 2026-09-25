# 接口 `super` 的默认方法可用性边界

本目录把同一个 Java 8 `Probe.value()` 的接口限定调用分成三种目标头。`default/Child.java` 自己声明 default，原 class 返回 4；`inherited/Child.java` 不声明方法而继承 `Parent` default，原 class 返回 3。两者源均是合法的 `Child.super.value()`。第三种先用 default 版编译 `Probe.class`，仅以 `abstract/Child.java` 编译出的同名 `Child.class` 替换目标定义；`Probe` 的常量池、指令和直接接口表不动。这是用于观察恢复边界的 classfile 组合，并非声称存在等价的 Java 源。

从仓库根目录运行：

```text
python3 openspec/evidence/java-syntax-2026-09-25/abstract-interface-super/replay.py --jarde-cli /tmp/jarde-cli-audit-baseline
```

脚本使用 JDK 23.0.1 的 `javac --release 8`、JADX 1.5.6 与指定 Jarde CLI，分别重放 `-g`/`-g:none`。所有 class、JAR、重编目录在 `TemporaryDirectory` 中自动清理；保留源码、`javap`、反编译文本、诊断和哈希。版本见 [`tool-versions.txt`](tool-versions.txt)，输入/输出文件见 [`SHA256SUMS.txt`](SHA256SUMS.txt)，当次 class 摘要见 [`classfile-sha256.txt`](classfile-sha256.txt)。

## 原 class 与两家输出

合法的直接 default 版在 `java -Xverify:all` 下输出 4；合法的继承 default 版输出 3。对继承版，Jarde 写出 [`Child.super.value()`](inherited-jarde-Probe-g-none.java)，完整 `Probe` 源在原接口 JAR 上 Java 8 重编，替换执行仍输出 [3](inherited-jarde-run-g-none.txt)。JADX 把同一调用写成裸 [`super.value()`](inherited-jadx-Probe-g-none.java)，去掉其导出用的 `package defpackage;` 后 Java 8 仍以 [“找不到方法”](inherited-jadx-javac-g-none.txt)拒绝。因此不能为了与 JADX 外观一致而去掉接口限定；当前 Jarde 的这一正例行为应保留。

替换为抽象 `Child.value()` 后，`Probe.class` 的 `InterfaceMethodref Child.value:()I` 与 BCI 1 的 `invokespecial` 不变，见 [`hybrid-javap-g-none.txt`](hybrid-javap-g-none.txt)；选定 `Child` 头中同签名方法现在有 `ACC_ABSTRACT`，没有 Code。`java -Xverify:all` 接受该 classfile 并走到调用处抛 [`AbstractMethodError`](hybrid-run-g-none.txt)。这仅证明 JVM 验证和执行边界，不说明成功值；该组合没有合法的等价 `Child.super.value()` Java 8 源码。

Jarde 当前只看池项是接口、owner 等于唯一直接接口且 receiver 是入口 `this`，仍输出 [`Child.super.value()`](jarde-Probe-g-none.java)。用选定的抽象 `Child.class` 在 classpath 上重编，会得到 [“无法直接访问抽象方法”](jarde-javac-g-none.txt)。JADX 对该组合又输出裸 `super.value()`，也不能重编；它不是可借鉴的修复方案。有无调试表得到相同判断。

## 架构结论

“owner 是唯一且非冗余的直接接口”只证明限定名形状，尚未证明该限定调用所选方法可作为 Java 默认方法访问。已有的 [`prove-interface-super-source-legality`](../../changes/prove-interface-super-source-legality/design.md)应在同一选定定义链上增加方法可用性门：准确方法声明若为抽象就拒绝；当前接口未声明时，须沿已选、完整的父接口关系找到唯一可用 default，保持合法继承版。不能从 `InterfaceMethodref` 类别、方法名或运行时抛错反推 Java 源可写；遇到未选定义、歧义、重载/桥接或预算停止要保守处理。复用 facade 的按需头读取和已有特殊调用 fallback 足够，不需要另建全局语法 pass。

本证据只覆盖零参 `int value()` 与上述三个类头。它没有证明所有默认方法继承、重载、泛型替换、桥方法、私有接口方法、解析器/运行时异常或任意非法 class 的规则；这些情形需要独立控制，不能用本反例替代 JVM/JLS 全部解析。
