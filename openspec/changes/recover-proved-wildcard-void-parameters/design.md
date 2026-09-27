## Context

参见 [提案](proposal.md)与[三方证据](../../evidence/java-syntax-2026-09-27/dt17-wildcard-void/report.md)。reader 已解析通配符、核对签名擦除；`class_source::spell_ordinary_signature_type_with_member_path` 已能写 `?`、`? extends`、`? super` 和基本类型数组。当前 `jarde-java::generic_return_candidate` 对单条 `return;` 额外要求零参数，因此 `ordinary_parameterized_declaration` 不能证明空正文和参数槽同时成立，类源码保留物理原始 `List`。

## Goals / Non-Goals

**Goals:** 在顶级普通 `Object` 子类的静态 `void` 方法上，对唯一的一个 `List` 通配符参数证明来源、正文和签名均完整，原子输出五个固定通配符形态，保留原始 `List` 控制。反射泛型类型与原 class 一致。

**Non-Goals:** 不扩成任意泛型方法体、多个参数、实例方法、字段或返回类型投影、用户定义成员界、复杂嵌套 Signature、type-use 注解，也不新增 Signature parser。

## Decisions

1. **复用同轮候选。** `GenericReturnValue::EmptyVoid` 已表达无效果 `return;`；将其参数槽列表从同轮 `NameTable` 和物理参数表完整采集，严格核对 Program 单语句、Code 唯一 `return`、无异常表、SSA 单块/无 phi、效果无读写且所有真实参数槽均未被读写。只因参数存在而丢掉候选没有语义依据；也不能仅检查最终源码文本。
2. **签名门只准入本次语法组合。** 在 `ordinary_parameterized_declaration` 的现有 erased Signature proof、声明合法性与候选参数槽检查之后，允许空 `void` 候选用于一个 `java.util.List` 参数。其唯一类型实参须是 `Any`、`Extends` 或 `Super`，界限于已可拼写的简单类或基本类型数组；方法静态、无异常/注解、类顶级且直接继承 `Object`。缺失/歧义 Signature、额外参数、同名本类调用与来源不完整时沿现有拒绝路径退回物理 descriptor，不改变 raw `List` 方法。
3. **按物理方法逐项提交。** 保持 `project_method_signature` 的完整声明一次计费、来源记录和停止语义。一个方法拒绝不会让其它物理成员被推测成功；预算/取消时没有半个参数类型。独立方法恢复仍以物理 descriptor 和同轮正文为准。

## Risks / Trade-offs

- 被省略的参数使用或副作用会让原始 `List` 与通配符语法的正文合法性不同 → 同时核对 Code、SSA 效果和 AST；读取、写入、调用、异常表和额外指令均拒绝。
- 通配符界虽可解析，却可能无法在当前源码词法位置拼写 → 首片只接受 JDK 简单界与基本类型数组，拒绝成员类型路径和类型注解。
- 局部成功不能证明 DT-17 全部变体 → 固定无 Signature raw 控制及反射行为，含正文和用户成员类界另列验收。
