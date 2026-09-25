## Context

见[提案](proposal.md)、[行为规格](specs/java8-recovery/spec.md)与[三方对照](../../evidence/java-syntax-2026-09-25/static-method-local-generic-throws/analysis.md)。reader 的单一 `Signature` parser 已识别 `^TX;`、方法局部变量作用域，并逐位置核对 descriptor 与物理 `Exceptions` 擦除；query 共享 reader 而不介入源码投影。`class_source::project_method_signature` 已先执行该证明及同类 Methodref 门，`generic_method_declaration` 已消费同轮 `GenericReturnCandidate::Parameter` 并写出静态 `T` 参数/返回，但其异常循环只允许普通类，把本例整个头退回物理形态。无正文和空 `void` 子集已有结构化局部异常拼写。JADX 的 `SignatureProcessor.parseMethodSignature` 只应用类型参数、参数和返回，`MethodThrowsVisitor` 另以物理异常集合构造 throws；其结果说明可参考类型来源，但不能照搬异常算法。

## Goals / Non-Goals

**Goals:** 在现有 reader 证明和同轮参数来源 proof 之上，为一个源级可靠的静态直接返回形状写出完整方法头。保持原子替换、预算、拒绝来源及其它泛型方法路径不变。

**Non-Goals:** 不新增 Signature grammar、运行时 javac 判定、跨方法绑定推断或类层级解析；不扩至条件返回、非静态方法、非空效果方法、自定义异常界、类自有泛型变量、参数化异常或隐藏/覆写关系。本类 Methodref 导致的物理回退可能令整类不可重编，作为独立架构债务记录。

## Decisions

1. **沿用 reader 的语法/擦除判据。** 继续先 `parse_method_signature` 和 `prove_method_signature_erasure_with_class_scope`，由它们拒绝尾部语法、未绑定变量及 descriptor/`Exceptions` 矛盾；class-source 只做 Java 8 源级准入。新的异常变量必须是本方法局部变量，异常后缀恰好一个 `^TX;`；它的唯一 class first bound 是 `java/lang/Throwable`、`Exception`、`RuntimeException` 或 `Error`，无接口/附加界。其它方法变量沿用现有可拼写与擦除证明；异常变量可以与直接返回变量相同，`<T extends Exception> T echo(T) throws T` 是合法 Java，名字不同不是证明条件。若 `Signature` 无异常后缀，保持现有物理 `Exceptions` 输出。选择严格 JDK 根是因为单 classfile 的可装载性不足以证明自定义类在 Java 源中是 Throwable 子类；后者需要独立依赖/层级证明。
2. **在现有静态方法投影内增加一条条件门。** 只有同轮候选确为 `GenericReturnValue::Parameter`，参数槽与 `T` 返回的既有证明成立，且类为无 `Signature` 的顶层普通 Object 直接子类、零接口、无同名本类 Methodref 时接受 `throws X`。方法 flags、注解、姓名及标记继续走现有拒绝门；物理 `Exceptions` 次序仍由 reader proof 保证。通过门后用已解析变量名构造 `throws X`，与 `<T, X>`、参数/返回组成一个声明再调用现有 `project_generic` 原子发布。无需让正文候选读 Signature，也无需通过输出文本倒推返回来源。现有纯 class throws 和无后缀方法仍走原路径。
3. **不采用 JADX 的物理异常集合。** `MethodThrowsVisitor` 合并物理异常会丢泛型异常变量，且集合可改变顺序；本方案使用 Signature 作为源级位置，`Exceptions` 只作擦除核对。已有 [throws-order](../../evidence/java-syntax-2026-09-25/throws-order/analysis.md) 反例约束这一选择。亦不在 reader 加源级形状门：reader/query 需要保留完整有效 grammar，只有 class-source 才决定当前 Java 输出能力。
4. **验证以三重语义为准。** 原/JADX/Jarde 各在 `-g`/`-g:none` 下重编完整类；比较运行结果、泛型反射及原强类型调用是否重编。负例区分 reader 先拒绝与 source shape 拒绝，禁止误把同类调用回退后的整类编译失败记为本项成功。Rust 定向回归覆盖相邻无正文、空 `void`、普通类异常及预算/取消；不因一项门槛扩张其它泛型能力。

## Risks / Trade-offs

- **合法自定义异常界暂时拒绝** → 保持物理声明和原因；日后有可信层级证明再扩展，不能猜 `Throwable` 子类型。
- **泛型头更改调用与隐藏规则** → 首片限普通 Object 直接子类、无接口/类签名及同名本类 Methodref；边界负例和完整类回编检验。
- **受检异常虽无运行时抛出仍影响 Java 类型检查** → 同时比较显式泛型调用方和反射，不能仅用 `echo("ok")` 值相同作为验收。
- **方法头部分更新或预算越界** → 维持现有临时声明组装、`project_generic` 一次交付及停止传播；拒绝不得吞预算错误。
