# EM-23 剩余活动测试形状：下一次动态基线盘点

本盘点只按 `expressions-misc.md` 的 EM-23 测试映射筛选明确活动断言；没有修改产品或创建 OpenSpec。JADX 测试来自固定 checkout `/Users/lordcasser/workspace/testzone/jadx`（账本固定基线 `2fb1b163`）。Jarde 结构事实使用已打开的 Atlas 项目，仅查询 `crates/jarde-java/src/build.rs`，没有全仓索引。

## 推荐顺序

### P1：增强 for 中条件执行的局部 `i++`

**JADX 原测试。** `/Users/lordcasser/workspace/testzone/jadx/jadx-core/src/test/java/jadx/tests/integration/variables/TestVariablesDefinitions2.java` 的 `TestCls.test(List<String>)` 在第 14–24 行先设 `i = 0`，遍历非空列表，仅当 `str.isEmpty()` 时执行 `i++`，最后返回 `i`。第 27–35 行的活动测试 `test()` 明确要求 `int i = 0;`、恰好一个 `i++;` 和 `return i;`，并拒绝 `i2;`。这不是 TestArith 的未实现断言：`arith/TestArith.java:39–46` 的精确 `+=`/`++` 断言标有 `@NotYetImplemented`；其活动 `test()` 在第 33–37 行只要求能产生源码。

**现有完整类证据的边界。** EM-20 的 `LocalScopes` 已有完整类三方对照，涵盖经典 `for` 头中的 `iinc`；Jarde 输出 `for (local2 = 0; ...; local2 = local2 + 1)`（[报告](../../java-syntax-2026-09-27/em20-local-scopes/report.md)，[已重编的 Jarde 源码](../../java-syntax-2026-09-27/em20-local-scopes/acceptance/source/jarde-LocalScopes.java:34)，对应原 `iinc` 在 [javap](../../java-syntax-2026-09-27/em20-local-scopes/acceptance/javap-LocalScopes.txt:41)）。这证明了相邻的循环局部更新形状，不覆盖增强 for 主体内、受 `if` 控制的计数器增量；该报告也明确把 `TestVariablesDefinitions2` 留作 EM-20/EM-23 交叉形状。现有 EM-23 `Updates` 完整类只覆盖字段单位更新、字符串追加及参数更新（[报告](../../java-syntax-2026-09-27/em23-field-updates/report.md)），不含这个循环。

**可复用的 Jarde 接缝。** `build.rs:27158` 的 `Builder::unit_field_update_statement` 是字段单位更新路径，不应拿来证明局部变量。局部 JVM `iinc` 走 `Operation::Increment` 的语句发射分支（`build.rs:23053–23056`）：通过 `write_target` 取得当前变量名和类型，再构造 `local + 1`/`local - n` 的写入。Atlas 在 `build.rs` 的结构查询确认了 `PostfixUpdates::prove_local_snapshots`（`build.rs:11571`）负责 `iload; iinc; consumer` 快照形；它要求加载与增量读取同一 SSA 值、该旧值恰有有界消费者且更新值在消费者前未被读取。这是值被表达式消费时的专门路径。此候选的 `iinc` 是循环体中的独立语句，核心应由普通 `Operation::Increment` 发射和循环/局部作用域路径覆盖；快照证明不能替代完整类运行证据，也不要求为了追逐 JADX 的 `i++` 拼写而新增语法节点。

**最小完整类基线。** 新建一个顶层 `VariablePostfixLoop` 类，只保留 `public static int countEmpty(List<String> values)`：`int i = 0`、null guard、增强 for、空串条件、`i++`、`return i`。独立 `Runner` 依次调用 `null`、空列表、含两个空串和一个非空串的列表，打印每次计数；不添加其他字段、更新方法或外部 helper。原类 stdout/exit/stderr 是比较 oracle，随后对原/JADX/Jarde 完整类源码与同一 Runner 做 fresh 重编和验证运行。

**先验不变量（待原类实际运行确定具体 raw）。** 每个满足 `isEmpty()` 的元素恰计一次；非空元素不改计数；null 和空列表返回 0；方法调用之间没有静态或实例计数残留。目标是动态验证局部作用域、增强 for、条件分支和普通 `iinc` 语句组合，而不是要求 Jarde 文本必须采用 `i++`。

## 不列为本轮候选的相邻测试

- `arith/TestFieldIncrement.java:30–37` 的活动断言已由 EM-23 `Updates` 完整类与接受记录覆盖：实例 `++`、静态 `--`、字符串追加和双输出均有完整类运行证据。
- `arith/TestFieldIncrement2.java:26–32` 的活动断言就是显式双读 `this.a.f = this.a.f + n` 与 dup 接收者 `this.a.f *= n`。其先前完整类基线记录在 [README](../em23-receiver-chain-next/README.md)；乘法变更当前仍在推进中，按任务要求不重复提案或将其称为已验收。
- `arith/TestArith.java:39–46` 精确局部赋值断言未实现标记；不把它提升成下一条基线要求。
- `arith/TestFieldIncrement3.java` 虽包含浮点 `/=` 源形状，但其活动断言针对的是差值赋值而不是 `/=`；它不在 EM-23 映射的四个测试文件内。本次不把该形状扩进账本任务。

## 结论边界

唯一建议的下一条动态基线是 P1。当前相邻架构和 EM-20 证据说明这段 Java 8 形状理论上可由现有局部 `iinc`、循环与作用域机制覆盖；没有新完整类证据不等于产品失败，也不构成实现缺口结论。只有原/JADX/Jarde 的完整类三方结果才能决定是否需要后续工作。EM-23 仍是部分完成，不能据本盘点宣称整项闭合。
