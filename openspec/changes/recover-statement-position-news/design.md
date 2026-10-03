## Context

[巡查证据](../../evidence/java-syntax-2026-10-03/statement-new-patrol/README.md)：B6 四形判别与 B5.main 复合。既有切片 `refuse-unconsumed-construction-invokes` 的拒绝位与其 CST 反例（`ordinary-new-void-effect`——实参含真实调用的 `CST` 序）。**第一个取证义务**：读该切片的语句位判定处——"实参依赖真实调用"判定的精确位置与当前语句位 new 的完整拒绝路径；确认实参分类数据（常量/局部读/消费链）在哪一层可得。

## Goals / Non-Goals

**Goals:** 无序敏感实参语句位 new 呈现；B5/B6 恢复行为一致。**Non-Goals:** CST 形（实参真实调用）保持拒绝；接口注解形（interface 无 ctor）；数组 new 语句位；消费位通道。

## Decisions

1. **判据（沿既有证明）**：new@1 构造证明通过 ∧ 全部实参∈{常量、局部/参数直读、已证消费链表达式}→ 呈现 `new X(args);`；任一实参含调用副作用 → 保持原拒绝文本（CST 保护不变）。
2. **验收锚定**：B5 复合（main 三 new + init 块平铺不变）、B6 四形；变体（多语句 new 混合消费位、静态嵌套类语句 new）；负例（CST 冻结反例逐字不变）。

## Risks / Trade-offs

- **消费链实参的序**（`new B6(new B6(1).n);` 内层已证）→ 内层按既有消费链证明，外层仅看自身实参类别；序由内层证明保证。
