## 1. 接收者身份与拼写

- [x] 1.1 在 `jarde-java` 的命名/发射路径中把"该槽位是接收者"作为独立输入：非 `static` 方法的槽位 0 一律按接收者处理，不经过关键字别名规则；用合成类固定三件事——有调试名 `this` 的实例方法写 `this.f`、无调试信息的实例方法不再写 `arg0`、静态方法的槽位 0 仍按既有 `arg<slot>` 规则拼写（反例）。验证：文本级断言 + 既有 fixture 对照（A10；A13）。 证据：实现提交 `93be08c`；`MethodFacts::has_receiver()` 只读 `ACC_STATIC`，`NameTable::build_with_receiver` 在 `report.rs` 按事实二选一；新增 `tests/p3_instance_receiver.rs`（4 项）与全语料成员级可逆检查（bcprov 前 40 类 113 处变化，静态成员 0 变化、接收者位置之外 0 差异）。
- [x] 1.2 确认接收者形态覆盖所有位置：字段读取、实例方法调用、作为实参、数组/长度与 `synchronized` 等既有呈现路径；分组与括号规则不变（`java8-recovery` 既有 receiver 分组要求继续通过）。验证：既有 receiver 分组样本与新增实例方法样本的输出对照（A13）。 证据：所有读取位置都经同一 `ExprKind::Local(name)` → `emitter.put`；既有 receiver 分组样本（`tests/p3_eval_context.rs`）与拼接/布尔样本继续通过；位置普查（120 个真实类）`this.` 602、`(this)` 11、`this == ` 2、`this;` 8。

## 2. 文本重基线与回归

- [x] 2.1 枚举受影响的文本预期（golden fixture 与断言具体文本的测试），逐个确认差异只发生在接收者位置；不得借机放宽断言。验证：失败清单 + 可逆差异检查（只允许 `this_`/接收者位置的 `arg<slot>` 变成 `this`）。 证据：4 个文件共 33 个 hunk，30 个只改接收者拼写（`arg0.`/`self.` → `this.`），1 个新增测试、1 个注释更新；可逆脚本只把接收者拼写归一后逐条比对，未发现放宽或删除。
- [x] 2.2 更新 `openspec/specs/java8-recovery` 的示例拼写（例如 receiver 分组场景里的实例方法样例），保持要求语义不变。验证：OpenSpec strict 校验与人工逐条比对。 证据：主 spec 的既有示例拼写未被本次修改影响（其样本均为静态成员或参数位置），因此本 change 未改主 spec；新增要求由 delta `specs/java8-recovery/spec.md` 承载。

## 3. 执行对照与门禁

- [x] 3.1 用受控 JDK 编译执行对照固定实例方法样本：原 class 与呈现文本在同一输入集合上结果一致，且文本编译通过（A13）。验证：`cargo test --test p3_execution_comparison --locked -- --ignored`（受控样本集内）。 证据：批量与单方法两个入口的对照都覆盖接收者读取——新生成样本 `ReceiverField`（`value()` 读自身字段、`bump()` 写回再读、`scaled(I)I` 静态对照）经 javac 编译、执行并与原类逐 trace 相同（5 行 trace）；反例（把判定改回序数命名）在 wrapper 编译处变红。
- [x] 3.2 全量门禁：`cargo fmt --all`、`cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`、`cargo test --workspace --all-targets --all-features --locked` 全绿；确认 `jarde-reader`/`jarde-jvm`/`jarde-query` 的 fact 与身份计数未受影响（A18）。 证据：`cargo test --workspace --all-targets --all-features --locked` 全绿（提交 `93be08c` 时 1443 passed；此后各轮合计 1494 passed / 0 failed）；`jarde-reader`/`jarde-jvm`/`jarde-query` 的事实与身份计数未因本 change 改变。
- [x] 3.3 记录 verification：哪些断言是本次有意改变、哪些是未动的既有契约；对真实语料（bcprov、S2-009 WAR）各抽一个类做前后文本对照，并写明本 change **不**解决的可读性差异（无 import、无类型推断名）。 证据：本节各条；真实语料前后对照（`HistoricalControlFlow` 逐字节相同、bcprov `ASN1OutputStream` 13 行全部接收者位置）；未解决的可读性差异（无 import、无类型推断名）记在同处。

## Acceptance Map

| ID | 主题 | 主要外部判据 |
| --- | --- | --- |
| C01 | 接收者身份 | 非静态方法槽位 0 一律写 `this`，静态方法不受影响 |
| C02 | 文本一致性 | 受影响预期只发生接收者位置的差异，无断言放宽 |
| C03 | 行为不变 | 受控执行对照与全量门禁通过，fact/身份计数不变 |
