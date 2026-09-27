## Context

[EM-01 单 child 冻结对照](../../evidence/java-syntax-2026-09-27/em01-declarations/report.md) 证明单个 `SingleAbstract.A` 在没有构造站点时也被根类漏写；原/JADX 完整 Java 8 根源码运行 `true:1`。当前 `prepare_class_source_member_family` 已读取同一选定环境的双方 `InnerClasses`，但静态 `StaticNoCapture` 必须持有构造目标；`project_class_source_static_member_family` 要求非空站点，且 writer 只有在该目标存在时才改写 child 构造名和记录成员头来源。因此关系/声明证明与构造使用证明耦合。见 [行为合同](specs/java8-recovery/spec.md)。

## Goals / Non-Goals

**Goals:** 一个普通根类与一个直接静态抽象 child，在无分配点时输出完整根级嵌套声明；保留两份物理报告和准确成员来源，并维持原静态构造形态。

**Non-Goals:** 第二 child、接口/注解/枚举 child、成员字段、泛型 Signature/父接口、跨根使用改写、桥方法、局部/匿名类或任意无 Code 方法的泛化投影。它们分别继续由既有单元或后续任务证明。

## Decisions

1. **复用现有家族证明，拆开两种责任。** 双向 `InnerClasses`、唯一选定 root/child、成员 flags 与物理完整性先产出声明型准入；构造目标及调用站点只是需要改写 root 代码时的附加证据。让现有静态无捕获状态允许“无构造目标”，不新增全局嵌套扫描或 JVM IR。比起伪造一个不存在的分配点，这准确反映输入只有声明的事实。

2. **严格限制可装配 child。** 首片要求唯一 child、准确非泛型非接口静态抽象类、无字段，完整普通默认构造器和已声明无 Code 的抽象方法；其它成员/注解/Signature/未知 Code 或根里的未证 `A` 源级使用均拒绝。现有构造型分支继续消费其 `ProvedStaticMemberTarget` 和完整站点证书，声明型分支不调用构造改写。根/child 各自恢复在同一预算和停止状态下完成，所有证明失败仅留物理报告。

3. **writer 以关系而非构造目标定位成员身份。** 已证 child 的 `PhysicalDefinitionId` 与 relation 足以为嵌套头、`A()` 构造名提供派生锚点；不应让这两处基础声明依赖分配站点。嵌套修饰符使用成员关系 flags，按 `public static abstract class A` 顺序写出；抽象 `test2();` 保留合法无 Code 声明。原 `ClassSourceMethod` 及 child text 不变，最终根 text 是一次性衍生投影。现有 source map/derived 计数随这两个关系锚点调整，不把物理 child 的 BCI 冒充根 BCI。

4. **三方验收及拒绝。** [replay.py](../../evidence/java-syntax-2026-09-27/em01-declarations/replay.py) 的 `--fixture single` 记录固定 JADX 测试哈希、原/JADX/Jarde 完整 Java 8 根源码重编和验证运行；`--fixture multi` 保留第二 child/泛型未支持对照。定向负例破坏关系、增加 child/字段/Signature、截断方法或降低预算；现有 `static-member-basic` 构造型案例必须通过。现有 Rust reader、预算和 writer 足够，无需引入第三方库；JADX 仅作为外部参考。

## Risks / Trade-offs

- [仅有成员关系但正文不可呈现] → 准入要求根/child 物理完整与合法抽象无 Code，失败时保留原结果。
- [构造型回归] → 明确把无站点声明分支与既有非空站点分支分开，执行原冻结测试。
- [来源映射失真] → 头与构造名由关系及 child 方法身份锚定，根方法 BCI 不复用。
