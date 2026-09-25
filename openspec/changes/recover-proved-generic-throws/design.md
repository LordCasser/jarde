## Context

见[提案](proposal.md)与[证据](../../evidence/java-syntax-2026-09-24/generic-throws-signatures/analysis.md)。reader 已有单一 `Signature` parser，`prove_method_signature_erasure_with_class_scope` 会把非空泛型 `throws` 后缀按顺序与物理 `Exceptions` 比对。`ordinary_parameterized_declaration` 已拿到同轮已发布类作用域、方法属性和完整无正文候选，但显式拒绝异常变量，最后从物理列表生成 `throws`。JADX 1.5.6 没有消费方法泛型 `throws` 后缀；这里只参考它的 Signature 读取路径，不继承其错误输出。

## Goals / Non-Goals

**Goals:** 让类级已证异常变量的无正文方法保留原 Java 泛型异常语义；在同一个方法候选里完成验证、拼写和发布，复用现有预算及局部拒绝。

**Non-Goals:** 不把该变更扩到方法自身 `<X>`、有正文方法或自定义异常类型的依赖解析；不引入新的 parser、异常类型全局求解器、JADX 算法副本或后处理字符串替换。上述扩大覆盖的机会记录在 roadmap。

## Decisions

1. **reader 证明与源码拼写分层。** 保持现有 reader 对参数、返回和非空 `throws` 的逐位置擦除证明；无泛型 `throws` 后缀维持现有“物理异常仍有效”语义。源码层只消费解析树和已发布类作用域，不猜测由 `Exceptions` 擦除来的变量。替代方案是单独重解析异常字符串，易与 reader 的语法及预算路径分叉，不采用。
2. **限制到无正文、类级变量。** 在现有 `ordinary_parameterized_declaration` 门后处理 `parsed.throws`；仅 `NoBody` 且变量已出现在 `class_scope` 才允许类型变量。继续拒绝有正文方法的异常变量，因为其抛出路径、catch 类型和调用绑定没有同轮完整兼容证明。方法自有 `<X>` 走另一声明路径，留给单独任务。
3. **异常合法性不从 descriptor 擦除单独推断。** 已发布类变量的 first-bound descriptor 必须是确定的 JDK 异常根 `Throwable`、`Exception`、`RuntimeException` 或 `Error`，这些类型本身就是 `Throwable` 子类；递归边界的擦除如落在这些根上也可接受。其他自定义 bound 没有当前类路径继承证明，即使 Java 实际合法，也局部拒绝。此白名单是最小可证明子集，不作为新的泛型类型系统。`reader` 已保证该 descriptor 与物理 `Exceptions` 同序一致，源码层仍按预算逐个验证和拼写。
4. **从结构化异常列表一次形成声明。** 有泛型 `throws` 后缀时，Class 类型沿原有简单无参数化类名门检查；变量型用现有 Java 标识符拼写门和类作用域。只有整个声明构造完成才调用当前 `project_generic`；若后缀为空沿用物理 `attributes.throws`。保留注解拒绝、同类 Methodref 门、来源记录及 essential/all 正文一致性。相比直接替换已经格式化的 `throws Exception` 子串，此法能保持位置、顺序和错误原子性。

## Risks / Trade-offs

- **自定义异常边界是真实合法源码** → 首片会拒绝；后续可在受控依赖解析中证明继承再扩大，无需复制解析器。
- **签名与正文的静态异常语义不一致** → 首片仅覆盖无正文方法；有正文方法保持既有拒绝。
- **只凭 `Exceptions` 属性不能恢复 `E`** → 没有已证类头与方法签名后缀时继续输出物理异常，避免伪造反射来源。
