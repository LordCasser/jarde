# CF-15 扩验巡查：跨 protected region 局部与异常包装实参（2026-09-30）

[CF-15 账本](../../jadx-feature-inventory-2026-09-27/control-flow.md)登记"空 catch/不可达 catch/异常变量写回"为弱验证边界（上游 `TestEmptyCatch`/`TestUnreachableCatch` 均 SmaliTest 关编译，只数文本头）。本目录是 JVM 侧首次构造性取证（主线 `d8205286`），固定转录见 [fixture](fixture/)（SHA 见 [results/fixture-sha256.txt](results/fixture-sha256.txt)），JADX Java-input 捕获在 [results/jadx-C*.java](results/)，主线基线 JSON 在 `results/*.base.json`。

## 结果矩阵

| 场景 | 形态 | 主线 Jarde | 固定 JADX |
| --- | --- | --- | --- |
| C1.swallow | 单个空具名 catch（unchecked 类型）+ 后续 return | 恢复（catch 体含 return，语义等价） | 结构恢复 |
| C1.five | **五个连续空具名 catch**，正文为语句式 `b.append('c')`（pop 结尾），builder 构造 store 前置于首行 | `five` 整方法回退："local 0 crosses a quoted fallback region" | 结构恢复 |
| C2.alias | catch 变量包装重抛：`new RuntimeException("w:"+m, e)` | 方法级不回退但 catch 体两条语句被引：参数 1 需 `IllegalStateException → Throwable` 上转型证据缺失 → 构造拒绝 → 局部声明与 throw 级联拒绝（**输出不可编译**） | 结构恢复 |
| C3.five / C3.popstmt | 对照组：连续空 catch（istore 正文）/ 纯语句式 append 无 try | 均完整恢复 | 均恢复 |
| C4.constructNamed | `new StringBuilder()` 构造 store + 具名 catch | 整方法回退（同 C1.five 家族） | 恢复 |
| C4.twrNamed | **真 TWR + 外层具名 catch**（表：两行 catch-all + 具名行 `[0,36)→39` 覆盖初始化） | 整方法回退："local 0 crosses a quoted fallback region"（CF-17 域，另案） | 恢复 |

## 根因（已用主线 scratch worktree 实验证实方向）

1. **具名行 + 完成 store 前置被当 TWR 资源头检查**：`guard.rs::resources` 门槛对 `before` 是 store 的行进入完整 TWR 证明，`jre_guard_handler` 拒绝后阻断 Catches 呈现。结构判别（[C4 字节码](results/C4-constructNamed.base.json)实证）：真 TWR 的具名行**覆盖资源初始化**（起点在构造之前，如 `[0,36)→39`），TWR 自身保护行恒为 catch-all；**起点紧跟完成 store 语句的具名行**在结构上不可能是 TWR 自己的保护行。实验补此判别后，C1.five/C4.constructNamed 的诊断前进到下一层（声明规划），twrNamed 不受影响。
2. **跨 protected region 局部的写值白名单只收 int 内联树**：`build.rs::all_reads_reach_presented_writes` → `presented_int_store_value` 要求写值为 `Value::Int` 的字面量/同块调用/加法树；构造 store（`new StringBuilder()`）直接失败 → `DeclarationPlacement::Incomplete` → 整方法回退。finally 家族此前以逐证书豁免旗标（`nullable_resource_lead || flag_conditional_lead || …`）绕过此层——本切片应扩展**写值形状**（同块单用途已证构造点）而非再加旗标。
3. **调用实参的引用上转型无证据通道**：`build.rs` 转换分派（约 20445–20496）只有 Object 目标/同名/数组闭集/`reference_overload_calls` 证明/`platform_reference_argument_widens`（当前仅 `List→Iterable` 一对）五个回答；异常包装重抛家族（`IllegalStateException → Throwable`）落入拒绝。

## 切片划分

- **Slice A（`recover-named-row-crossing-locals`）**：判别 1 + 写值扩展 2，闭合 C1.five/C4.constructNamed 家族；N1/P3StorePrefix 的"store 前置降级"钉死负例按判别 1 的结构论证**翻转为正例**（该形状不可能为 TWR），catch-all 行与非边界（劈开/吞初始化）降级保持。
- **Slice B（`recover-throwable-wrap-arguments`）**：判别 3 的 java.lang 异常类闭集上转型表（→ Throwable/Exception/RuntimeException 祖先），闭合包装重抛家族；用户类层级的一般证明登记为升级路径（触发条件：首个非平台类上转型场景）。
- C4.twrNamed（TWR + 外层具名 catch）登记为 CF-17 新缺口，随 TWR 家族扩验另片。

原 class 是行为基准；JADX 输出仅作结构参照。
