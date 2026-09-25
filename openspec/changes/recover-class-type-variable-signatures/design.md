## Context

见 [proposal](proposal.md) 和[已冻结的类变量三方对照](../../evidence/java-syntax-2026-09-24/ordinary-parameterized-signatures/boundaries.md)。reader 已有带预算的 `ClassSignature` / `MethodSignature` 树和按方法位置的擦除证明；类声明目前由物理 `super_class`、`interfaces` 和 flags 直接拼写，方法证明只接受方法自己的类型变量。JADX 1.5.6 的 `SignatureProcessor` 先解析类签名、再更新方法类型；这个顺序可借鉴，但其 `fixTypeParamDeclarations` 会从父类/接口的使用处补造未声明变量，本项目不能把修补后的源码当成原 class 的证据。

## Goals / Non-Goals

**Goals:** 让同一 class `Signature` 的类型变量声明和成员 `Signature` 的引用共享一份已证作用域，先完成顶层普通类/接口、无泛型实参的物理父类/接口、直接参数返回以及可证明的无正文成员，重编后的 class/method 泛型反射和独立调用方一致。

**Non-Goals:** 不猜匿名/局部/成员内类从外层捕获的变量；不为缺失的类 `Signature` 反推变量；不从字段正文或泛型调用点反推类变量；不处理带泛型实参的父类/接口、字段 `Signature`、合成构造器参数、type-use 注解路径或增强 `for` 元素类型。编译 classpath 缺失仍按[独立对照](../../evidence/java-syntax-2026-09-24/ordinary-parameterized-signatures/nested-missing-signature-reference.md)记录，不把通用依赖闭包问题并入类变量投影。

## Decisions

1. **reader 完成类级事实证明，source 层决定能否拼写。** 在已有 `ClassSignature` 解析后，用同一擦除原语校验类变量名唯一且边界可终止、父类与接口的数量/顺序/擦除和物理 class 完全一致。proof 保留类变量及其第一边界擦除；不加载目标类，也不由 reader 判断 Java 语法。修改既有方法擦除入口，显式接收已证类级作用域；方法自己的同名变量若无法区分绑定，首片拒绝而不猜。替代方案是在 `class_source` 复制一套 Signature 解析/擦除，容易与 query 分叉，故不采用。
2. **类头与成员头使用同一次类级候选。** `src/facade.rs` 读取选中类唯一的 `Signature` shell，在方法循环前让 `src/class_source.rs` 构造类声明候选，并只把成功发布的类变量作用域交给同类方法。类头候选从 flags、物理父类/接口和结构化泛型类型重建，不对已拼好的字符串做替换；类型变量的 `extends Object` 可省略，其余 class/interface 边界按真实顺序拼写。带泛型实参的父类/接口可能要求整组继承成员一起改写，首片保守拒绝；类候选失败时保留原物理头，成员不得孤立写出 `U`。这样既不新增全局 pass，也不让方法结果反向修改类事实。
3. **成员投影沿用现有直接返回、无正文和调用门。** 普通参数化声明的类型拼写器只在收到已发布类作用域时写 `TypeVariable`，否则继续拒绝。正文参数槽/返回值与同类 Methodref 仍按已有门证明；`NoBody` 走既有 flags/分号路径。方法自有 `<T>` 与类变量同名、类变量参与复杂表达式或影响重载绑定而无证明的情形保持物理头与拒绝，避免泛型头改变 Java 编译时静态绑定。
4. **原子发布与预算。** 类签名属性读取、递归类型、变量作用域、物理擦除、声明文本和来源逐项计费/轮询。类级候选完整后才替换 `ClassSourceDeclaration.declaration`；成员级候选分别在完整证明后发布。停止沿现有报告路径传播，不发半个 `<...>` 或只写成员中的未声明变量。essential/all 共享最终文本。

## Risks / Trade-offs

- **类级 `<U>` 与方法 `U` 分别成功会制造孤儿作用域** → 方法只接收类头已发布的证明；类候选拒绝时方法按物理 descriptor 输出并保留原因。
- **JADX 会补造未声明变量以求编译** → 本项目只读 classfile 自身的变量声明；非法/不一致的类 `Signature` 不能用源码修补掩盖。
- **有界类型参数或泛型父类改变源码绑定** → 先核对物理父类/接口的逐位置擦除，再检查源码可拼写及成员正文/调用；缺证据整项拒绝而不局部投影。
- **内类共享外层变量或方法同名遮蔽** → 先明确拒绝，留独立作用域任务；不扩大本次类作用域模型。
- **泛型引用类缺失导致源码不能编译** → class-source 本身不承诺完整项目 classpath；保留引用事实，不把未提供的类误判为不存在。本项只验收依赖齐备的 class/调用方。
