## Context

`MixedArrayValue.one(ZZI)V` 先求 `array(nullArray)` 与 `index(pos)`，再经 BCI 9/15/21 测试、24/28 的 1/0 producer，把栈值 Phi 交给 BCI 29 `bastore`。此时栈还有数组和下标两个先求出的值。当前导出在 BCI 24 的重复 Region owner 处原子引用。[独立 Region/SSA 跟踪](../../evidence/java-syntax-2026-09-25/mixed-short-circuit-array/region-trace.md)已经确定更早的失败点：外层候选的图、producer 与前驱证明通过，却在消费者锚点因未识别 `ArrayStore` 退出；通用 `If` 随后真实重复认领 BCI 24。`prove_short_circuit_value` 的单操作数消费者集合与普通 `array_write` 的三操作数路径也未接合。

## Goals / Non-Goals

**Goals:** 以原始 class 的 24 条值/调用/异常轨迹恢复一个 `boolean[]` 元素写入；保留数组、下标、RHS 的求值顺序、一次性与来源；所有前置证据不闭合时原子拒绝。

**Non-Goals:** 不推断任意 `bastore` 是 Boolean、不支持 `[B` 或多维/别名未知数组、不折叠任意控制流、不把实例 `putfield`、局部 Store 或数组初始化器并入此 change；不复制 JADX 的全局类型传播。

## Decisions

1. **先拆开所有权与消费者。** 已验证 BCI 24 分别被 `If(15)` 的 `Straight [24,29]` 与 `If(21)` 的 `Fallback [24]` 认领；真正首因是闭合短路候选未把 BCI 29 的 `ArrayStore` 当消费者锚点。只为该可证候选增加锚点并使其一次认领完整图；若有外部前驱或冲突，仍维持 overlap 拒绝，不能删掉防线。此步骤必须先于 `bastore` 发射。
2. **在原私有短路图中证明第三操作数。** 仍要求有界无环测试、真实 taken/fallthrough、1/0 producer、唯一同槽栈 Phi。`bastore` 的 `reads()` 应精确为按 stack depth 排列的 array/index/value 三项，且 Phi 只占 value；其唯一直接 use 是该 store。另两个操作数从其 SSA 生产链证明身份、类型、可呈现、求值先于测试、无重复 owner/第二 effectful consumer。
3. **仅接受 proven `[Z`。** `bastore` 的操作码也适用于 byte 数组，必须通过现有 `array_element` 的数组源类型证明 Boolean 组件并检查 opcode。若数组值类型未知、是 `[B` 或元素类型来自不可靠猜测，拒绝。
4. **按赋值语义一次发射。** 复用现有数组/下标表达式和 `IndexAssign`，按 array、index、RHS 顺序嵌入 Java；null/bounds 检查仍发生在 RHS 后的实际写入处。借已有 Boolean 适配把短路值写成 Java boolean；全部相关 BCI 有来源，预算/取消只给原子结果。

## Risks / Trade-offs

- `[Region 先验重叠]` 不能因未来消费者可写而忽略重复 owner；先证明归属再扩消费者。
- `[求值/异常次序]` 提前检查数组或把 RHS 调用复制到两个表达式会改变 null/越界轨迹；以正常/null/越界 24 路径的数组、下标、b/c 调用数与异常类别为门槛。
- `[类型歧义]` JVM verifier 的 int-like 值与 `bastore` 不区分 `[Z`/`[B`；只从数组源类型证明组件。
