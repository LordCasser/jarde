## Context

参见 [proposal.md](proposal.md) 与 DT-14 [实测报告](../../evidence/java-syntax-2026-09-27/dt14-enum-init/report.md)。当前 enum 常量前缀证书和 `Measure` 的单一赋值后缀都在 class-source 装配阶段；这次观察到的后缀是数组读、循环、实例方法调用和 Map 写组成的顺序效果。

## Goals / Non-Goals

**Goals:** 只证明并投影固定的 Map 初始化/`values()` 遍历 suffix，和已证明 enum 常量前缀原子结合。

**Non-Goals:** 泛化成任意 `<clinit>` source recovery；接受未证明循环、调用序列、静态字段副作用或 Java 之外的 target 逻辑；改变非 enum 类初始化路径。

## Decisions

- 仅在完整 enum 常量组、唯一 enum `<clinit>` 与同次完整 Code/结构候选都确认后解析后缀。标准 enum prefix 的事实不能被用来推断 suffix；suffix 的每个 array/loop/call/write 节点均要有确切 BCI 与同一执行次序证据。
- 首片只接纳 fixture 所示的一个 Map 字段：确认字段身份、声明类型与可写/可见性；确认其初始化值来自无参 Map 构造；随后 `values()` 的返回数组唯一地驱动单一增强 for-each，循环体仅将当前元素的 `name()` 和自身按同次序写入该 Map。
- 输出只复用已有 Java initializer/statement rendering 能实际呈现的完整语句节点，不从 fallback 文本删 enum 物理前缀，也不按方法名、`HashMap` 名称或 javap 文本匹配。
- 在发布 enum source projection 前同时验证标准 prefix 和 suffix；任一步拒绝或停止都不提交半成品。物理 fields/methods 与原 `<clinit>` source map 保持原样报告。

## Risks / Trade-offs

- [编译器可将循环 lowering 成多种 CFG] → 首片限制为一个可证明的直线前缀加有唯一控制头/回边/出口的基本 `values()` 遍历，不匹配便整组拒绝。
- [Map 写可观察字段引用、顺序和异常] → 通过运行时 identity/size 检查，并逐项绑定被遍历值与 put 参数；不以内联或常量折叠替换调用。
- [枚举类其他静态字段造成顺序依赖] → 只投影首片证书覆盖的一个 Map 字段和顺序，任何邻接写入拒绝，不默默重排字段初始化。
