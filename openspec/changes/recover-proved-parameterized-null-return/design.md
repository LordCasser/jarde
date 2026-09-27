## Context

证据见 [DT-18 报告](../../evidence/java-syntax-2026-09-27/dt18-parameterized-members/report.md)。`class_source::project_method_signature` 已调用 reader 的 `prove_method_signature_erasure_with_class_scope`；`ordinary_parameterized_declaration` 已把同轮参数直接返回投影为 `List<String> id(List<String>)`。同一类的 `empty` 正文已恢复为 `return null;`，却因没有候选而拒绝 Signature。DT-16 的独立变更在原有 `GenericReturnCandidate` 中证明精确 `aconst_null; areturn`，本变更以该候选完成验收为前置条件。

## Goals / Non-Goals

**Goals:** 当一个顶级普通 `Object` 子类的公有无参实例方法确切返回 `null`，且唯一方法 `Signature` 的结果是单段 `java/util/List<java/lang/String>`、物理 descriptor 是 `()Ljava/util/List;` 时，完整类源码保留参数化返回类型。物理成员、独立方法恢复和停止状态不变。

**Non-Goals:** 新增候选提取机制、推广任意类/界/通配符返回类型、带参数/throws 的方法、字段读取与局部变量推断、同类调用重绑定或成员类源位置。

## Decisions

1. **复用 DT-16 的同轮 null 事实。** 仅在候选由完整 Program、Code、SSA 证明为无参数、无效果、准确 `aconst_null; areturn` 时进入普通参数化声明门。Jarde 不从输出文本或仅凭 `Signature` 推断正文。
2. **方法头仍由现有 reader 和 class-source 门一次决定。** 先证明 `Signature` 唯一且完整、擦除等于 `()Ljava/util/List;`，返回只有 `List<String>` 这一单段参数化形态；再检查顶级普通类、实例公有方法、无注解、无同名重载/本类 `Methodref` 和完整源码正文。通过 `record.project_generic` 原子提交；失败保留物理 raw 声明及拒绝来源。
3. **用独立消费者观察行为。** 三份完整类型源码和同一 Runner 在 Java 8 下重编并由 verifier 运行，反射返回类型必须与原 class 逐字一致。raw 字段、raw 方法以及已有参数直接返回保持原状。预算和错误 Signature 负例验证没有局部发布。

## Risks / Trade-offs

- 相同 `return null;` 文本可能掩盖前置效果，故必须消费 DT-16 已验收的 Code/AST/SSA 候选，而不是解析生成文本。
- 这个固定形态窄于 JADX 的 `TestGenerics2` 和 `TestGenericFields`；其成员类、字段读取和局部变量推断仍需各自对照，不把 DT-18 整项标为追平。
