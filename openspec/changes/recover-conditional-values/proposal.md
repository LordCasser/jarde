## Why

普通 Java 8 `condition ? a() : b()` 会把两个分支的值汇合到同一栈位置。jarde 已识别 `if` 双臂，却把汇合处 SSA Entry/Phi 作为“没有指令产生的值”引用，导致真实整类缺少返回语句。最小 424 B/6 Code 样本的原 class 与 JADX 均可重编译执行，两行一致；冻结 jarde 的整类编译失败。完整 1414 B/15 Code 样本的 16 行原/JADX一致，七个条件值方法同样编译失败。

## What Changes

- 对现有 `Region::If` 和 SSA 汇合做有界、精确的两臂值证明：同一 join、两条唯一前驱、一个被消费的栈 Phi，每臂各提供一次值，测试与值的来源明确。
- 只在此证明成立时复用已有表达式/消费者构建一个条件值表达式，写出 `test ? whenTrue : whenFalse`；保持未选臂不执行、已选臂只执行一次。
- 复用现有类型/调用目标、区域归属、预算与来源；对循环、switch、异常边界、外部跳入、类型不明及附带不可折叠语句的汇合维持完整拒绝。
- 不引入通用 Phi 求解器或跨方法类型层级解析；确实新增一个表达式形状，因为既有二元/`if` 语句节点无法作为局部、算术或调用实参的值。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：从已证明的双臂栈值汇合恢复 Java 条件表达式。

## Impact

涉及 `build` 的 `Region::If`/SSA 值交接、一个 AST 条件表达式及 emitter 分组、来源与相邻测试。根证据见 `../../evidence/java-syntax-2026-09-22/ternary-values/` 与 root 独立重放 `root-7747/`。本 change 只处理条件值；其他未证明的 Phi 仍保持既有拒绝。
