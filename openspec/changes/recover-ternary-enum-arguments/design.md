## Context

见 [proposal.md](proposal.md) 和 DT-14 [重放报告](../../evidence/java-syntax-2026-09-27/dt14-enum-init/report.md)。`TestEnumsWithTernary` 的 Java enum class initializer 在每个 constructor call 前包含一个分支汇合，int literal 控制 fixture 与现有 int source-argument 证书只在此处不同；测试自带的 String 参数则属于另一边界。

## Goals / Non-Goals

**Goals:** 将有界 int conditional 值恢复成同一 enum constructor 实参中的 ternary，且不破坏条件求值次数、两 arm 选择、字段/常量顺序。

**Non-Goals:** 通用 CFG 表达式变换；String 普通/varargs 参数；任意 `<clinit>` 逻辑；与方法内 CF-04 status 合并。

## Decisions

- 在 enum initializer 的类级原始 Code 证书中验证 branch diamond：唯一 predicate producer，条件分支左右路径各有一枚 literal，路径无其他操作，并在唯一 join 按原 stack value 进入唯一匹配构造器；保留每个 branch/arm/join/constructor BCI。
- 对 predicate 只复用同次可读 expression evidence；不得从输出文本解析、凭相邻 BCI 将未证明调用提升成条件。已证明的条件 effects 必须在打印表达式处执行恰一次。
- 让新 argument variant 仍进入既有完整 enum group；与 constructor descriptor、物理常量字段、`putstatic` 位置和 `$VALUES` 顺序闭合。任一常量不满足形状则整组拒绝。
- 不把 JADX 的 String ternary 用于证明 int gate；它只说明组合形态和 DT-11 的简单 String constructor argument 未分别验证。实施边界仍以 `TernaryInit` 与 `LiteralInit` 控制为准。

## Risks / Trade-offs

- [条件表达式运算可能是有副作用的调用] → 只接受可完整拼写、唯一使用的同次条件值并执行行为对照；不复制或移动它。
- [Javac 会因常量宽度选择不同 opcode] → 用 descriptor 与 literal value 验证等价 int，覆盖 iconst/bipush/sipush/ldc；不以某一固定 opcode 序列猜来源。
- [加入后通用 enum 证书可能吃掉未知控制流] → 仅确认为一个有界 diamond 的节点集合，要求全 Code 无剩余未解释指令并做正反 verifier-valid fixture。
