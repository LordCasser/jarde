# EM-23 嵌套字段接收者链覆盖盘点

本盘点只核对活动 `TestFieldIncrement2` 的 receiver-chain 断言与既有 EM-23 完整类记录。它确认一个缺少同形完整类三方对照的候选，不据此断言当前 Jarde 对该形状有行为缺陷。

## 活动断言与现有证据

上游测试源为 [`TestFieldIncrement2.java`](</Users/lordcasser/workspace/testzone/jadx/jadx-core/src/test/java/jadx/tests/integration/arith/TestFieldIncrement2.java>)。`TestCls` 在第 10–24 行声明内部 `A.f`（第 11–13 行）、接收字段 `a`（第 15 行）和两个实例更新方法：`test1(int n)` 的 `this.a.f = this.a.f + n`（第 17–19 行），`test2(int n)` 的 `this.a.f *= n`（第 21–23 行）。活动 `@Test` 在第 26–32 行，源码断言分别要求 `this.a.f += n;`（第 30 行）和 `this.a.f *= n;`（第 31 行）。这是一个活动的 JavaInput 源码形状；两条断言处于同一个测试方法中。

EM-23 首次完整类重放并未覆盖它。[`report.md`](../java-syntax-2026-09-27/em23-field-updates/report.md) 第 3 行明确区分 `TestFieldIncrement` 的活动单字段断言与 `TestFieldIncrement2` 的嵌套接收者断言；第 5 行说明原/JADX/Jarde 完整类输入是 `Updates.java` 与 Runner。虽然 [`replay.py`](../java-syntax-2026-09-27/em23-field-updates/replay.py) 第 16–20 行把 `TestFieldIncrement2.java` 的 SHA 固定为 `5dd70254…`, 该哈希只证明重放所对照的上游测试版本，不能证明该测试类的编译产物被用于完整类对照。

后续 [`spell-proved-field-unit-updates` 验收](../../changes/spell-proved-field-unit-updates/tasks.md) 第 13 行再次完整重放同一 EM-23 `Updates` fixture。该 change 的 [`design.md`](../../changes/spell-proved-field-unit-updates/design.md) 第 9、13、19 行把嵌套 `this.a.f` 排除在范围外，并要求复杂接收者身份与求值次数另行证明；tasks 第 14–16 行也将嵌套接收者复合赋值保留为未覆盖项。[独立主线验收](../java-syntax-2026-09-27/em23-field-updates/root-acceptance-2026-09-27.md) 第 3 行同样只确认 `this.instanceField++`、静态字段 `--` 与字符串链，没有记录 `TestFieldIncrement2` 的完整类重编。因此这两个活动断言均未被现有完整类记录按同形动态覆盖。

## 一个最小下一候选

只做一个包含两条活动形状的完整 Java 类对照：保留实例方法里的 `this.a.f = this.a.f + n` 和 `this.a.f *= n`，由共同 Runner 通过可访问的初始化路径设置 `a` 并分别调用这两个方法。对原/JADX/Jarde 的完整源做同一 JDK 编译与 `-Xverify:all` 运行；比较原程序输出，并逐条记录生成文本是否呈现 `+=` 和 `*=`。Runner 应观察 `f` 的最终值，使用独立初始状态避免前一调用影响后一断言。保持内部 `A`/接收字段这一嵌套 receiver 结构，不改成 `this.f` 或局部别名。

该候选只验证活动测试的两条方法形状；它不顺带覆盖有副作用的 receiver 调用、数组 receiver、别名变化或并发修改。若完整类动态运行正确但输出仍是等价低级读写形式，应区分语义覆盖与该 JUnit 的精确源码拼写要求，不提前将其归类成语义回归。

## 现有实现接缝与边界

- `crates/jarde-java/src/build.rs:7669-7675` 在 body 构建时执行 `CompoundAssignments::prove` 与 `UnitFieldUpdates::prove`。`CompoundAssignments::prove_field_update`（`13813-13917`、`13936-14117`）从写字段定位同块更新，要求字段计划确认读写同一物理字段、读写 receiver 来自同一个 `dup` 复制的 SSA 值，并约束唯一用途和依赖指令前缀。receiver 的表达式依赖在 `14060-14084` 单独收集。这里已有“共享一次 receiver 值”的证据接缝，但该函数目前只把 `iadd`/`isub` 映射为复合赋值运算（`13964-13980`）；不能仅凭这个函数推断 `*=` 已获证明。
- 字段计划 `crates/jarde-java/src/field.rs:710-814` 使用 verifier-stated receiver type 和成员 owner/name/descriptor 检查具体字段身份；字段读写由 `Plan::claim` 提供给上面的证明。嵌套 `this.a.f` 需要两个字段层级各自保留正确身份。
- `StmtKind::FieldAssign` 的输出路径在 `crates/jarde-java/src/emit.rs:848-865`：receiver 作为一次 primary operand 输出，后接准确字段名与 assignment operator；普通字段表达式的递归 receiver 输出在 `1346-1350`。这说明 AST/emitter 已有嵌套字段书写接缝，但不替代验证 receiver `this.a` 在更新链里只计算一次的 SSA 证据。
- 现有精确更新证明以 `ValueId`/dup/read/write 的身份、唯一消费者和依赖闭合来约束 receiver。代码中没有单独名为“receiver stabilization”的通用机制；先用这组现有事实查看完整类报告与反编译文本，动态证据才能决定是否需要新增适配。此处不预设新机制。

Atlas 已打开项目并在 `crates/jarde-java/src/build.rs` 的限定 scope 找到 `CompoundAssignments`；源码位置和上述行号以当前工作树的直接文件内容为准。本盘点未构造输入或运行任何工具链。
