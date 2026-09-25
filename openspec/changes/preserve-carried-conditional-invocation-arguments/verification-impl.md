# 携带条件实参的实现验证

## 1.2 冻结 SSA 与首次拒绝

在 `ConstructorPairProbe.<init>(String,int)` 的 Java 8 class（SHA-256 `4e587e0a6c7f81c7b1e9f2a9b71683427a3d78c689068ae5ad2a381fac29c0d4`）上，使用真实 JVM IR、canonical CFG、SSA 与 Region 生产器记录：

| 位置 | 真实前驱 | 栈槽/值 | 事实 |
| --- | --- | --- | --- |
| BCI 12 | BCI 6、10 | `Stack(1)` Phi `ValueId(7)` | 输入分别为 BCI 6 的 `ValueId(19)` 与 BCI 10 的 `ValueId(20)`；BCI 12 的 entry/exit 均为 7。 |
| BCI 12 → 16/21 | BCI 12 | `Stack(1)` `ValueId(7)` | BCI 12 的 `iload_2`/BCI 13 的 `ifne` 只处理第二条件的 `Stack(2)`；两个条件臂的 entry/exit 均保持 7。 |
| BCI 22 | BCI 16、21 | `Stack(1)` Phi `ValueId(12)` | 两输入精确为 `[ValueId(7), ValueId(7)]`，trivial Phi 的 `replaced_by` 为 7；BCI 22 entry 和 `invokespecial` 实际读 7。 |
| BCI 22 | BCI 16、21 | `Stack(2)` Phi `ValueId(13)` | 两输入分别是 BCI 16 的 `ValueId(21)` 和 BCI 21 的 `ValueId(22)`；调用按 JVM 栈从 `Stack(2)=13`、`Stack(1)=7`、`Stack(0)=14` 读取，参数顺序为第一条件 7、第二条件 13。 |

全部相关边均为 normal：`0→6/10→12→16/21→22`。旧 `prove_conditional_value` 对 `ValueId(7).uses()` 要求单一 use，实际有 BCI 22 的同值 Phi 两个 operand use（各自 `bci=None`）和 BCI 22 `invokespecial` 的一个指令 use，共三项，故**首次拒绝**是 `PhiUseCount`；尚未执行同 join 消费检查。基线 `constructor_call` 随后在 BCI 22 拒绝 `Stack(1)` 的 Phi/entry 渲染。基线 source map 仅覆盖 6/14 条 BCI，遗漏 `[6,7,10,12,13,16,18,21]`，说明不能单独提交第二条件折叠。

## 准入、拒绝与停止

实现只在相邻的两段已证明 `Region::If` 及其直接后继调用上尝试携带证明，嵌套 `Region::Sequence` 复用同一准备过程。第一次汇合的值必须沿第二测试块和两条 normal 臂的同一栈槽原样传递；第二次汇合的同值 Phi 必须恰有两个原值输入、`replaced_by` 指向原值，且自身无残留 use。原值的两次 Phi operand use 和一次最终调用 use 必须精确匹配。第二条件测试块中任何不属于测试表达式的独立指令都拒绝；调用核对实参位置、descriptor 类型、实例 receiver 或静态形式，`<init>` 还需已有 `init@1` 前导证明。两段表达式、测试效果和调用先全部通过，才同时登记折叠计划。拒绝时两段 `Region::If` 均引用各自 SSA 块的全部指令 BCI；收集/计费完成前不发布任一计划。

`javac --release 8 -g:none -Xlint:-options` 编译以下 verifier-valid 控制和正向调用类，`ControlRunner` 在 `java -Xverify:all` 下成功执行。控制 class SHA-256：

| 控制 | class SHA-256 | 定向断言 |
| --- | --- | --- |
| `CarriedTypeMismatch`：第一条件的 `Object`/`String` 臂没有本层可证明的统一 Java 表达式类型 | `ad98eba5244a1ebc3c35e13154315a7dbb4f804051009dfb3395f37bf8174e8c` | 两段均引用，13/13 指令 BCI 有来源。 |
| `CarriedThirdArgument`：最终 descriptor 有第三参数 | `0ec89b058576017d464649dca3b97348f80fde8b183f6798ecd0e27988bc7f5d` | 实参闭合拒绝，两段均引用，15/15 BCI 有来源。 |
| `CarriedInterveningEffect`：两条件之间执行独立的 `touch()` | `a028fb2fb834138de3f581463c28168be3d45c61a80a655cf8c68128477bae1c` | 明确报独立指令拒绝，两段均引用，15/15 BCI 有来源；原 class 执行观察到每次增加一个 `X`。 |
| `CarriedNonPrologue`：两条件是普通 `new Nested(...)` 的参数，目标构造调用不是当前 `this/super` 前导 | `89884c8aa705bc8fadf63df4be16ff10fbd8ad1f16266a4b39a5149f6cc33300` | 前导身份拒绝，19/19 BCI 有来源。 |

正向 `CarriedMethodCalls`（SHA-256 `191ffb3d25197407f1100354cc57aef58d840893d2f2256589bcc04309b1f1ad`）覆盖静态调用与实例 receiver：首参数两臂记录 `A/B`，次参数记录 `C/D`。四种布尔组合的 `-Xverify:all` 结果分别是 `BD:bd`、`BC:bc`、`AD:ad`、`AC:ac`；两种调用各自逐项相同，定向恢复均生成同序双条件调用且无引用。

## 本轮回归与门槛

- `cargo test -p jarde-java --lib`：174/174；包括旧条件值 Phi、单次消费和拒绝控制。
- `p3_carried_conditional_arguments`：4/4；正向构造器 14/14 BCI、静态/实例调用顺序、四项 verifier-valid 拒绝的逐 BCI 来源、低预算/预取消。低预算测试将 `IrItems` 设为完整恢复用量减一，验证停止后文本/来源皆为空；同时验证 `IrItems=1` 与预取消。
- 邻接的 `p3_conditional_values` 2/2、`p3_short_circuit_chain_controls` 1/1、`array_invocation_widening` 5/5；这些断言覆盖条件值、调用类型和旧 Phi 消费者。上述定向测试与全部 lib 测试合并运行均通过。
- `cargo fmt --all -- --check`、`git diff --check`、`openspec validate preserve-carried-conditional-invocation-arguments --strict` 均通过；`cargo clippy -p jarde-java --lib` 通过，保留仓库已有 17 条 lint 警告。`cargo clippy -p jarde-java --lib --tests` 在其它未提交测试的 `recover_for_class_source` 缺少新增第三参数处编译失败，属共享工作区并行改动，未混入本 change；共享 corpus 指纹债务亦另案处理。
- 本任务专用的 `/tmp/jarde-carried-conditional-target`（清理前约 1.8 GiB）已删除；共享 Cargo target 未触碰。

Root 独立 CLI 曾在本实现的正向代码上复编完整原/JADX/Jarde 单条件类与双条件类，分别完成 4/4、6/6 `-Xverify:all` 路径，并核对双条件 14/14、单条件 8/8 BCI。最终源码的独立复核归任务 4.1–4.2。

本次局部双图原子引用以第二段 `Region::If` 的条件值证明成立为候选门槛。第二图本身缺证时不会发布第一条件值，但仍走原有 `If` 引用路径；其来源是否存在独立缺口尚无冻结的 verifier-valid 复现，需以真实 class 与逐 BCI 差异另案判定，不在本 change 推广全局 `region_quote`。
