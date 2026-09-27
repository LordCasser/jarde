## Context

固定 CF-10 的 `StepIndex.everyOther([I)I` 在 Java 8 class 中，BCI 4/5/6/7 依次是索引读取、数组读取、`arraylength`、`if_icmpge 22`；循环体 BCI 10～19 包含数组读取、累计与 `iinc 2,2`，BCI 22 返回。现有 `recover --evidence region_details` 报告 `loop@1` 在 BCI 6 因 `StatementFree` 拒绝，随后 BCI 10/22 成为 uncovered blocks，声明阶段才报 `local 1 crosses a quoted fallback region`。根因位于 Region 准入而非局部声明或 foreach 识别。

`Region::test_is_pure` 已用同块 SSA 从终端分支回溯 `condition_value_bcis`，只在该集合内允许有潜在效果的调用和字段读取；它没有相应接受 `Operation::ArrayLength`。`Builder::render_value` 已能由 SSA 单数组操作数生成 `ExprKind::ArrayLength`；循环构建与非 unit-step 的普通 `for` 已有路径。JADX 固定 `TestArrayForEachNegative` 仅断言无 `:` 且禁用编译；本设计以独立完整类的编译、验证运行作为更强验收。

## Goals / Non-Goals

**Goals:** 只修复 Region 对同块、真实被条件消费的数组长度生产者的判定，保留可抛异常的求值位置；让现有 Builder 和局部声明流程自然完成该步长循环。

**Non-Goals:** 新建循环类型或 SSA 机制、放宽不相关数组读写、处理 `List→Iterable` 调用、改写 foreach 识别、以局部变量文本补丁遮盖 Region 回退。

## Decisions

1. **复用条件值回溯。** `ArrayLength` 只可与已允许的调用/字段读取一样在 `condition_value_bcis` 中出现时通过测试块准入。单纯把所有 `ArrayLength` 当作无效果算术会误接纳未使用的可抛异常读取；单独新造纯度判定会复制同次 SSA 证据。
2. **保留 Builder 现有源表达式路径。** Region 只决定所有权，表达式仍由 `render_value` 解析精确单数组读取和原 BCI；如该证据不足，Builder 原子拒绝，不从 descriptor 或 JADX 文本合成 `array.length`。
3. **以三方完整类和保守负例验收。** 正例包括隔离的 `StepIndex` 与原 `ForeachCases` 组合；检查输出 `4`、组合输出 `10/abc/4`，且源文本不是 foreach。负例以可验证 class 或定向 Region 构造证明未消费、复用或额外效果不会通过；预算/取消仍遵循既有停止路径。固定 JADX `TestArrayForEachNegative` 的无冒号断言仅作对照，不作为语义证明。

## Risks / Trade-offs

- [数组长度可能在每次测试时抛 NPE] → 仅允许同块分支消费链中的精确生产者，并以 `null` 数组行为对照确认求值次数与位置。
- [Region 通过后仍有 Builder 或声明失败] → 不再放宽其它门槛；记录新诊断，另开独立差距，不以替代打印绕过。
- [负例字节码可能不由 javac 生成] → 定向测试须验证 class 可加载或明确作为内部 Region 判定测试，不把不可加载 class 当作运行语义证据。
