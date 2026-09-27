## Context

[CF-04 对照](../../evidence/java-syntax-2026-09-27/cf04-ternary/report.md)把普通数值/布尔 ternary 和两种构造器 `this(...)` 实参证明为已有能力。失败方法的 BCI 1/5/12 是条件测试，BCI 15/19 分别写入 int 常量 1/2，BCI 20 是唯一 `ireturn`。Jarde 的 `Region::ShortCircuitValue` 已准确拥有这些块；`build.rs::prove_short_circuit_value` 现在把两个 producer 分别限定为 1/0，`ShortCircuitConsumer::Return` 只接受 `Type::Boolean`，后续 `short_circuit_boolean_expression` 也默认 1/0。因此失败点在**已有值证书的类型边界**，无需另建区域算法。

## Goals / Non-Goals

**Goals:** 对无异常边的直接方法级共享尾，从精确两个 int 字面量 producer、唯一 stack phi 和唯一 int 返回消费者证明值闭包；生成按原测试边顺序求值的嵌套 `?:`；原 class/JADX/Jarde 完整 Java 8 源码重编、验证运行 13 行一致。

**Non-Goals:** 任意表达式 producer、字段/数组/调用的 int 值消费者、Boolean/Integer 装箱、异常 handler、循环/switch 内嵌套值、数值窄化及一般 ternary 美化。

## Decisions

1. **在现有证明入口区分值用途。** 保留 `Region::ShortCircuitValue` 的边、测试、gateway、producer 和 consumer 身份。布尔消费仍要求 1/0 及原适配条件；新增的窄路径仅在返回类型准确为 `int`、消费者为唯一 `ireturn` 时读取两个直接 `ConstantValue::Int`，要求不同 producer、同 stack 深度、唯一 phi/use 和原有完整边/预算证明。任何隐式 cast、额外消费者或非 literal 叶仍拒绝。
2. **生成整数条件树时跳过布尔化简。** 既有 `build_short_circuit_statement` 按物理 test edge 构造条件表达式；int 路径保留两个整数叶与嵌套 `?:`，经现有 `adapt_return` 写 `return`。`short_circuit_boolean_expression` 和共享布尔尾投影只处理原 1/0 布尔消费，不应把 int 1/2 当真假值重写。每个测试、producer、phi 和 `ireturn` 的来源仍按现有 AST/SourceMap 路径发布。
3. **证明完成后原子发布。** 若 SSA phi、所有前驱、表达式 Java 类型、来源、预算或停止有一项不闭合，保留整段物理引用和 `unproven` 报告。不能只发已恢复的其它方法并声称完整类可编译。基本形状 `TernaryBasic` 及布尔 1/0 sink 是强制非回归对照。

## Risks / Trade-offs

- [整数 1/2 被布尔归一化导致结果错] → int 消费显式绕过布尔表达式折叠，三方运行覆盖 true/false 两路及构造器不回退。
- [phi 消费或前驱不唯一时拼出错误条件] → 复用现有完整 SSA/CFG 证书并增加负例；任何缺失、额外边或重复使用仍原子拒绝。
- [扩大到任意值生产者引入求值次序问题] → 首片只处理两个直接 int 常量，无副作用 producer；测试条件效果只按原有 `test_edges` 生成。
