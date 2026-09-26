## Context

见 [proposal.md](proposal.md)、[行为合同](specs/java8-recovery/spec.md)及[冻结证据](../../evidence/java-syntax-2026-09-26/exception-constructor-concat/analysis.md)。`concat::plan` 已证明 StringBuilder 链并把 BCI 4–20 交给 `init::sites` 作为 reserved；`init::verify` 仍逐条检查外层 `dup` 至 `<init>`，只允许无效果运算及属于实参 SSA 依赖的调用，因此在内层 `Allocate` 处给出 `jre_new_interleaved_effect`。外层 `new` 失败后既有 `Throw`/`Return` 消费者无法呈现，完整类缺返回。`build::render_value` 已能按链尾 BCI 呈现拼接，不需要新表达式 AST。

本地 JADX 1.5.6 的 `ConstructorVisitor` 将 fresh `new`/`<init>` 组合为构造表达式，`SimplifyVisitor` 将 StringBuilder 链简化为拼接，然后 `InsnGen` 写出消费位置；冻结样本的原/JADX 源码重编 class 字节相同。可参考其先组合再呈现的顺序，但不复制可变 IR visitor 或其代码。Jarde 仍以不可变 SSA、各规则证书和物理 BCI 为事实来源。

## Goals / Non-Goals

**Goals:** 使已证拼接链作为同一块内对象构造器的真实参数，在 `throw` 与 `return` 两个消费位置完整恢复；保持求值顺序、唯一物理所有权、异常边界、来源与保守拒绝。

**Non-Goals:** 不接受任意嵌套 `new`/副作用，不修改 throw/return 规则或 SSA/CFG，不引入通用效果重排器。跨块、别名、多消费或不能证明的异常范围另案处理。

## Decisions

1. **交接完整的 concat 证明，而不把 reserved 当证明。** `report` 将同一轮 `concat::Plan` 传给 `init::sites`；后者仍用 `owned()` 排除链头作为独立 `new`。外层 `verify` 只在扫描到实际构造实参依赖中的链时读取 `value_at(tail)` 及其物理成员，不能因某 BCI 被任意规则 reserved 就放行。替代方案是让 `Allocate` 白名单接受 reserved，无法证明被调用的恰是该链的值，并可能移动无关效果。
2. **在外层构造区间内做依赖与边界闭包。** 对每条拟嵌入的链，核对其尾值对应一个真实构造实参、该值只有这一个被呈现的消费者、全部链 BCI 在 outer `dup` 与 `<init>` 之间且按 JVM 顺序执行，并且外层与内层异常处理覆盖一致。链的所有分配、`dup`、调用必须完整由 `concat` 拥有；外层 `Site` 仅认领自己的物理指令，不重复认领内链。对独立 void 调用、另一个分配、额外读者或跨边界情况仍给出原有保守拒绝。替代方案是只靠 SSA 的调用依赖放行；当前扫描已证明那不足以覆盖内链分配与副本。
3. **复用既有呈现和消费者。** `init::Site` 继续把构造参数生产者 BCI 交给 `render_value`；后者通过 `concat::Plan::value_at` 输出 `+` 表达式。`Operation::Throw` 和返回只消费已证构造值，不另造语法处理。保留构造外层先分配、随后拼接的 JVM 顺序；仅在可由 Java 表达式等价表示时接受。替代方案是增加新的 throw 或 nested-allocation AST，会复制现有结构且扩大证明范围。
4. **失败时保持完整引用。** 任何链身份、唯一消费、异常覆盖、预算或来源核对失败，外层 `new` 不成立；不得只呈现其中一半。默认/all 的正文和真实 BCI 来源必须一致。原样保留现有常量/直接参数构造及独立拼接路径。

## Risks / Trade-offs

- [链尾是参数依赖但链的别名另被消费] → 核对所有真实 SSA 读者和唯一呈现位置，以多消费负例拒绝。
- [异常表切割嵌入区间] → 按半开 BCI 覆盖比较外层构造与内链指令，差异时引用。
- [内链与外层双重认领或被重复发射] → `concat` 保持链指令唯一 owner，`new` 仅包含外层自身及参数值引用；核对完整来源。
- [独立 void 效果被错误折入参数] → 保留现有 statement-free 门槛；只放行证书中逐条枚举的链成员。

## Migration Plan

无格式或依赖迁移。若组合证明不成立，回退到已有 `new@1` 引用；冻结样本和负例继续记录该边界。
