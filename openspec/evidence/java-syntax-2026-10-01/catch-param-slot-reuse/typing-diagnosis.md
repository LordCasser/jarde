# 诊断记录：catch 参数槽复用的实参类型来源（recover-caught-value-argument-typing）

基线 `59126c7e`，隔离 worktree。fixture SHA 复核：`S1.class` =
`e372f219aa6b556e6717b10648d99ff3d5f4bdaca5a4b4f68233d2f157b3c765`（与
[fixture-sha256.txt](results/fixture-sha256.txt) 一致）；基线恢复文本
[S1.jarde.java](results/S1.jarde.java) 逐字重放一致。

## 1. 实参呈现类型 Actual 来源（诊断结论）

`twrHelper` 的 `tag(e)` 实参拒绝消息（BCI 40，"presents `S1`"）只能产自
`crates/jarde-java/src/build.rs` 的 `invocation_argument`（约 20630/20703 行一带）：实参
`Expr` 的 `presented` 字段为 `Some(S1)`。该实参的表达式来自这条链：

1. `return tag(e);` 的调用作为 return 语句的操作数渲染：`call_expr`（invokestatic@40）经
   `stack_operands` 取操作数 → `render_value(value, at=43, depth=1)`（`at` 是 return 的求值点，
   不是 40；诊断探针实测）。
2. 操作数值定义为 **aload@39 这条 load 指令自身**（`Definition::Instruction{bci:39}`）。
3. `render_value` 的 `Operation::Load` 臂（约 19496 行）→ `self.local(variable, name, bci)` →
   `decided_type(variable)` → `decision(variable)` → `decide_types` 的**槽决策**。
4. 槽 0 是 TWR 资源槽（P3 2.4 头部声明；同时是子句参数槽，P3 3.4 不拆分）→ 整个方法只有
   `LocalVariable::whole(0)` 一个变量；`decide_types` 取**方法序第一个写**——BCI 7 的资源
   `astore_0`（`S1`）——于是 handler 体内的参数读取呈现 `S1`。

即：**实参呈现类型 = 槽决策（`decide_types` 对 `LocalVariable::whole(0)` 的首次写决策），
不是值定义**。子句头拼写（`catch_header`，异常表行 `catch_type` 的池拼写）一直正确。

SSA 事实（探针实测，`Definition::Caught` 的 `bci` 是喂 handler 的异常边指令点）：

- plainHelper（handler 只有一条贡献边）：绑定 `astore_0`@7 的栈读值定义就是
  `Definition::Caught{block 0, bci 1}`，ty = `Named(java/lang/IllegalStateException)`。
- twrHelper（touch@9、close@13、athrow@34 三条边同入 handler@38）：绑定 `astore_0`@38 的
  栈读值是 `Phi{block 38, Stack(0)}`，ty 同为 `Named(IllegalStateException)`——单行子句的
  合并类型即该行的类；多行（multi-catch）子句即 javac 对参数在 handler 体内自身的 lub。

## 2. 修改点

`build.rs` 新增 `caught_clause_parameter_type(slot, read)`（放在 `decided_type` 之后） +
`render_value` 的 `Operation::Load` 臂对 `self.local(...)` 的呈现类型覆盖。条件（三者同时
成立才覆盖，其余读取保持槽决策）：

1. 读取值的定义是 **handler 入口 store**（`Operation::Store{slot == 读取槽}` 的指令值）——
   副本 store 或其他写不触发；
2. 该 store BCI ∈ `clause_parameters`（该行被呈现为子句头——14673/13530 既有集合）；
3. store 的栈读值（handler 入口引用）ty 为 `RefType::Named`——catch-all 行无名可取，保持
   既有呈现。

覆盖值 = `value_type(入口引用.ty())`——`Definition::Caught`/`Phi` 既有类型事实通道
（frame.rs `caught_reference` 的行类型；`value_type` 与 `spell_reference` 同一拼写路径，
与子句头逐字一致）。

## 3. 变体边界（前后输出见 [variants/](variants/)）

| 变体 | 前 | 后 |
| --- | --- | --- |
| S1.twrHelper（TWR 单子句槽复用） | handler 体被引（presents `S1`） | `return tag(local0);` 完整恢复 |
| S1.plainHelper（对照） | 完整恢复 | 逐字不变 |
| S1V.clausesHelper（TWR 双具名子句） | 整方法被引（local 0 crosses a quoted fallback region） | 不变（区域级既有拒绝先于实参检查，属"不可呈现形状保持既有拒绝"） |
| S1V.multiHelper（TWR multi-catch `A \| B`） | 整方法被引（同上） | 不变（同上） |
| S1V.copyHelper（handler 先存副本再传参） | 完整恢复（副本槽决策 = 副本 store 的 written_type = 行类型） | 逐字不变 |
| S1V.scopeHelper（非 handler 槽复用读取） | BCI 23 拒绝（presents `S1V`） | 逐字不变（读值定义非入口 store，槽决策保持） |
| S1V.rethrowHelper（复用 handler 内 rethrow） | 整方法被引（区域级） | 不变（rethrow 通道零变化） |
| S1V.plainClauses（无 TWR 双具名子句，参数同占槽 0） | 子句 1 恢复、子句 2 被引（presents 子句 1 的行类型——同一归属缺陷） | 两个子句都恢复（同一值归属规则的直接受益，非槽复用 TWR 场景） |

## 4. 回归测试

`tests/p3_caught_value_argument_typing.rs` + 固定 fixture
`tests/fixtures/p3-caught-value-argument-typing/v8/`（同一 1221 字节 S1.class）。已验证：
撤掉 build.rs 修复后 `the_slot_reusing_handlers_argument_presents_the_rows_own_type` 在
`return tag(local0);` 断言处失败；修复后两用例全绿。
