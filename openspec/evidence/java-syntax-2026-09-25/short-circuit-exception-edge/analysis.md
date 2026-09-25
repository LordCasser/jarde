# 异常保护区中的短路字段写入

冻结的 [Java 8 样例](../../../../tests/fixtures/p3-conditional-values/short-circuit-exception-edge/README.md)在 `try` 内执行 `result = left && mayThrow()`，`RuntimeException` 处理器读取原字段。`assign(Z)Z` 的异常表为 `[0,18) -> 21`；BCI 0/1 与 4/7 是两次测试，10/11 与 14 是值生产者，15 是唯一 `putstatic`。`mayThrow()` 增加 `calls` 后总抛异常。Runner 在调用左真分支前把字段设为 `true`，所以左真路径进入 catch 并返回旧值，不是成功写入的新值。原 class 用 `java -Xverify:all` 的实际输出是：

```text
left=false,returned=false,field=false,calls=0
left=true,returned=true,field=true,calls=1
```

架构师重新运行 `javac --release 8 -g:none`，冻结 class 与新产物逐字节一致；原件与 [JADX 1.5.6 完整类源码](jadx-ExceptionShortCircuit.java)分别以 Java 8 模式编译，JVM 验证下两行输出逐字一致，日志见本目录 `original-*`、`jadx-*`。JADX 输出带有 `Code duplicated` 警告，展开了 false 臂，并把字段赋值放在 try 外。本样本的运行对照没有证明该移动会产生行为错误，因此不能把它当作 JADX 语义错误；它只说明不能照搬这段结构重排。Jarde 对异常边应有更强的物理所有权与 SSA/异常路径证明，证据不足时完整拒绝。

实现前的聚焦回归 [p3_short_circuit_exception_edge.rs](../../../../tests/p3_short_circuit_exception_edge.rs)故意失败：Region 报告先结构化 `[0,4,10,15,26,21]`，再将 26 记为 `jre_region_loop`、14 记为 `jre_region_uncovered_blocks`；source map 遗漏 BCI 10/11。问题在短路候选构造前跳过所有受保护 frame，普通嵌套 `If` 先夺取共享生产者和消费块的所有权。修复边界是仅在同一受保护区内一次认领候选，并在其真实异常边上整体引用；不把异常路径纳入现有无异常的字段赋值证明。

实现后，架构师用当前 CLI 重新生成 [Jarde 完整类](jarde-after-ExceptionShortCircuit.java)和[全证据 JSON](jarde-after-report.json)。`assign` 的 Region 一次认领 BCI 0/4/10/14/15 及 handler 21，标为非结构化 `jre_region_exception_edge`；引用行覆盖 BCI 0/1/4/7/10/11/14/15/18，source map 连同 handler/后续返回覆盖全部必要指令。方法仍为 `quality=fallback`，没有虚构字段赋值。聚焦回归 1/1 通过，`jarde-java` lib 161/161 通过。

这份 Jarde 文本可用 Java 8 模式编译，却只有引用注释，因而不能拿它的运行结果当语义验收：同一 Runner 的左真分支输出 `calls=0`，原 class 和 JADX 均为 `calls=1`。这个明确差异展示了“完整保守拒绝”和“已恢复行为”之间的区别。下一步要恢复异常边上的短路赋值，须另证处理器覆盖、调用位置与字段写入路径；本 change 不为追平 JADX 的文本而猜测这些路径。
