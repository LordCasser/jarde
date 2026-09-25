# Root 独立验收：连续条件调用实参

使用最终生产代码独立构建 `/tmp/jarde-carried-root-target/debug/jarde-cli`，SHA-256 `dd5ff212acc464c02960e2c2e387e055195c3c077e4da7480766ce98d76bbff0`。冻结输入 class 的 SHA-256：单条件 `2bf349eb148816d67ad14ef8ffe7ec0639333e9e7c4cec9331a77cc83605e770`，双条件 `4e587e0a6c7f81c7b1e9f2a9b71683427a3d78c689068ae5ad2a381fac29c0d4`。最终完整类 JSON/Java 在同日 [对照目录](../../evidence/java-syntax-2026-09-25/constructor-conditional-delegation/analysis.md)的 `root-after-final-*` 文件中。

| class | Jarde 恢复 | 来源 | 原源码/JADX/Jarde 完整类 Java 8 重编 | `java -Xverify:all` |
| --- | --- | --- | --- | --- |
| `ConstructorConditionalProbe` | `java`，0 引用 | 8/8 BCI | 三份均成功，所得 class SHA 均等于冻结输入 | 三份各 4/4 路径，与冻结原 class 逐行一致 |
| `ConstructorPairProbe` | `java`，0 引用；输出 `this(arg2 == 1 ? arg1 : "", arg2 == 0 ? "" : arg1)` | 14/14 BCI | 三份均成功，所得 class SHA 均等于冻结输入 | 三份各 6/6 路径，与冻结原 class 逐行一致 |

四个 verifier-valid 负例的最终 CLI 报告均为 `quality=fallback`、`representation=mixed`、`syntax_status=not_java`，未发布双条件 Java 表达式；来源分别覆盖 `CarriedTypeMismatch` 13/13、`CarriedThirdArgument` 15/15、`CarriedInterveningEffect` 15/15、`CarriedNonPrologue` 19/19 指令 BCI。独立效果控制明确引用 BCI 12 的独立指令。`CarriedMethodCalls` 完整 Jarde 类成功 Java 8 重编，替换原类在 `ControlRunner` 的 20 行 `-Xverify:all` 结果与冻结原类逐行相同，覆盖静态及实例调用的四种布尔组合和 `A/B` 后 `C/D` 的效果次序。

代码审读确认第一个条件的 BCI 12 `Stack(1)` 值 7 经第二测试块及两臂原样传递，BCI 22 同值 Phi `[7,7]` 被 SSA 折回 7；值 7 恰有两项 Phi operand use 和一项真实调用 use。第二条件独立证明其 `Stack(2)` 变化 Phi；准备阶段核对最终调用的两个相邻实参、descriptor、类型、独立指令和构造器前导身份，两个 `Folded` 计划在证明与表达式构造全部成功后一起登记。拒绝时两段 Region 的逐指令引用先收集、计费，再登记；SSA 块缺失时保守引用块首 BCI。测试以完整恢复 `IrItems` 用量减一及 `IrItems=1` 检查停止时文本与 source map 均为空，并覆盖预取消。

Root 回归：`jarde-java` lib 174/174，`p3_carried_conditional_arguments` 4/4，`p3_conditional_values` 2/2，`p3_proved_boolean_conditional_returns` 3/3；根包 `p3_invocation_arguments` 3/3（另有一项既有 ignored）、`p3_region_owner_overlap` 1/1、`p3_short_circuit_transfer_gateway` 6/6、`p3_two_exit_return` 7/7。格式、diff-check、OpenSpec strict 均通过；`cargo clippy -p jarde-java --lib` 退出码 0，仍有仓库已有警告。Root 私有 Cargo target 清理后删除，未触碰并行任务的 target。

准入范围仅覆盖第二段条件自身可证明、两段相邻且最终是一次调用消费的场景。第二段条件自身缺证时尚无冻结的 verifier-valid source-map 缺口，应另案复现后决定是否扩展；不因此引入全局 Region 引用机制。共享 corpus 指纹及其它未提交测试的 Clippy 构建问题同样不混入本 change。
