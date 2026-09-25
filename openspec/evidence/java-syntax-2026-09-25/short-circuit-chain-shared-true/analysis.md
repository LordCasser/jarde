# 三次短路测试的字段值：原件 / JADX / Jarde

冻结的 [Java 8 源与 class](../../../../tests/fixtures/p3-conditional-values/short-circuit-chain-shared-true/README.md)由架构师重新用 `javac --release 8 -g:none` 编译，class 逐字节相同且 `java -Xverify:all` 成功。`assign(ZZ)V` 的 BCI 1/5 两次 `ifne 14` 与 BCI 11 `ifeq 18` 汇入 BCI 14/18 的 1/0；BCI 19 唯一 `putstatic result:Z`。这是两个短路外层入口共享 true 生产者的三测试链，不是目前 `ShortCircuitValue` 固定外/内两次测试能表示的图。

JADX 1.5.6 的 [完整类](jadx-ChainOrField.java)输出 `result = z || z2 || rhs()`；Jarde **实现前**的 [完整类](jarde-ChainOrField.java)在 `assign` 中留下嵌套空 `if`，两次引用 BCI 19 并误报 `jre_region_loop`。[实现前全证据 JSON](jarde-report.json)将方法标为 `quality=fallback`、`representation=mixed`，source map 只有 0/1/4/5/8/11/19/22，遗漏 BCI 14/15/18，未给出完整拒绝证据。

原 class、JADX 和 Jarde 的完整类分别用 `javac --release 8` 与同一 Runner 重编、在 `java -Xverify:all` 下执行。原 class 与 JADX 的四行值/调用轨迹逐字一致：

```text
extra=true,left=false,rhs=false,result=true,calls=0
extra=false,left=true,rhs=false,result=true,calls=0
extra=false,left=false,rhs=true,result=true,calls=1
extra=false,left=false,rhs=false,result=false,calls=1
```

Jarde 实现前前三行的 `result` 均为 false；调用次数仍为 0/0/1/1，所以缺陷是字段汇合值与写入被丢弃，不是 RHS 被提前调用。实现前的编译及运行日志保存在本目录的 `freeze-*`、`original-javac.log`、`original-run.txt`、`jadx-javac.log`、`jadx-run.txt`、`jarde-javac.log`、`jarde-run.txt`。Jarde 当时的文本虽可编译，已有 fallback 明示不能当作等价 Java；漏掉生产者来源更是独立的证据所有权问题。

本地 JADX `IfRegionMaker.mergeIfInfo` 对可连续合并的条件按转移边选择 OR/AND，`TernaryMod` 在值折叠前校验汇合 Phi，这给出遍历顺序。Jarde 的最小长期改动是把现有局部短路 Region 的固定两个测试改成有预算的有序测试列表，再对每条物理边、两个生产者、唯一栈 Phi 和 `putstatic` 做闭合证明；表达式仍可由已有 `Conditional` 逐层嵌套。不能只照搬 JADX 的合并启发式：别的合法类已证其匿名类内联会改变类身份，且此处必须完整保留拒绝边界。

实现后，架构师用当前 CLI 重新生成 [完整类](jarde-after-ChainOrField.java)和[全证据 JSON](jarde-after-report.json)。`assign` 为 `quality=structured`、`representation=java`、`fallbacks=[]`，只写一次 `result`，无 `@bytecode`；source map 恰覆盖 BCI 0/1/4/5/8/11/14/15/18/19/22。原源码、JADX、Jarde 实现后完整类分别用 `javac --release 8` 重编，与同一 Runner 在 `java -Xverify:all` 下四行逐字一致；日志为 `original-after-*`、`jadx-after-*`、`jarde-after-*`。JADX 在无包名 class 上输出的伪 `package defpackage;` 仅在重编前去掉，类体和方法没有改写。该运行证明四条样例路径的等价性，报告本身的 `semantic_validation=unproven` 保持原样。

[重复消费者负控](../../../../tests/fixtures/p3-conditional-values/short-circuit-chain-shared-true-controls/README.md)从 Java 8 源重建且通过 JVM 验证，fresh recovery 引用全部 13 个相关 BCI、拒绝两个结构化字段赋值。[真实异常边负控](../../../../tests/fixtures/p3-conditional-values/short-circuit-exception-chain/README.md)同样可逐字节重建、经 JVM 验证并将 11 个受保护指令完整引用，catch 及续块保留来源。非 1/0 生产者的三测试补丁也通过内部聚焦测试；独立效果和真正额外入口对**三测试链**仍须补足控制，不能将这些局部成功扩张为任意布尔 CFG 的证明。
