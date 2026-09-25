## Context

见[冻结对照](../../evidence/java-syntax-2026-09-25/instanceof-boolean-merge/analysis.md)和[规格](specs/java8-recovery/spec.md)。现有 `prove_conditional_value` 已核对双臂 straight、唯一 stack Phi、消费者与物理边；`build_conditional_value` 将两个 `iconst` 渲染为 `Integer`，所以 `ExprKind::Conditional` 呈现为 `int`。`return_expr` 对 `Z` 返回若未拿到 Boolean 证据，交由 `adapt_return` 的通用 `integer_low_bit_boolean`，因此输出 `% 2 != 0`。这保证任意合法整数返回的低位语义，应作为默认路径保留。

## Goals / Non-Goals

**Goals:** 仅为已证明的 Boolean 测试、精确 0/1 两臂和唯一 `ireturn Z` 消费生成更直接的 Java 8 布尔表达式，并保留全部 BCI 来源及一次求值。

**Non-Goals:** 不优化一般整数条件、非返回消费者或其它 `instanceof` 形状；不把 2/3 等 verifier 有效值归一化为常量 true/false；不增加 AST 类型、新 pass 或独立布尔值分析。

## Decisions

1. **在现有条件值提交边界缩写。** 只有 `ConditionalValueProof` 成功、consumer 为真实 `ireturn Z`、`test_expr` 的呈现类型为 Boolean、两臂 SSA 定义各自指向精确的 `Push(Integer(0/1))` 且两臂没有其它效果时，按 `when_true/when_false` 与实际分支对应构造原 test 或 `ExprKind::Not`。复用已有 proof 的前驱、唯一 Phi 使用及所有权核对；不得只看输出文本中的 `? 0 : 1`。对其它消费者继续原条件表达式及通用低位转换，避免改变字段、局部与调用实参的类型合同。
2. **保留物理来源。** 缩写后的表达式携带原 test、两个常量 producer、structural transfer、Phi consumer 和 return BCI；可沿用 `build_conditional_value` 已收集的 `OriginSet`，只改变表达式形状。若预算、取消或来源提交失败，遵循现有原子回退/停止契约，不输出缺少一臂的简写。
3. **只让已呈现的 Boolean 直接进入 `Z` 返回。** `return_expr`/`adapt_return` 的类型交接须接受这一个已证明 Boolean 表达式，不再为它施加 `% 2 != 0`；一般 `int` 仍使用 `integer_low_bit_boolean`。低位适配已在 [`recover-integer-boolean-returns`](../recover-integer-boolean-returns/verification-root.md) 对 2/3 反例验收，不能删除。比起在发射器中匹配字符串或无条件简化 `%`，此处具有 SSA/消费者和 Java 类型事实。
4. **以完整类行为而非文本快照独立验收。** Java 8 重编原/JADX/Jarde 的 `InstanceOfMerge`，在 null/String/Integer/Object 上核对八行结果及一次调用；已冻结的 `BooleanMergeControls` 同时覆盖正极性 1/0 与反极性 0/1，再以非 0/1 负例及现有整数低位返回、条件值来源/预算测试守住边界。JADX 的简写只作可读性参照，原 class 仍是行为判据。

现有 SSA、AST、发射器与来源层足以完成此局部投影；无需引入依赖。它不改变 classfile 解析、Java 方言选择、JVM verifier 或运行时解析，只改变经证明的恢复源码形状。

## Risks / Trade-offs

- `[风险]` 极性反向或测试重复求值 → 分别测 0/1、1/0 两种臂映射及带副作用操作数的调用次数。
- `[风险]` 把 2/3 当成源码 Boolean，破坏 JVM 最低位语义 → 仅识别精确 0/1 SSA 常量，复跑 verifier-valid 非 0/1 基线。
- `[风险]` 美化丢失跳转/常量 BCI 来源 → 复用完整条件值来源集合，逐 BCI 断言及预算/取消测试。
