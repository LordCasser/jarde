## Context

见 [proposal.md](proposal.md)、[两个行为规格](specs/java8-recovery/spec.md)及[家族装配审计](../../evidence/java-syntax-2026-09-26/named-member-outer-receiver/family-assembly-analysis.md)。本变更以 `assemble-proved-member-class-family` 已验收的根/成员双物理身份、捕获值证书、嵌套 writer 和派生来源为前置，不能在该阶段缺失时绕过它。冻结 `OuterReceiverCases` 的 Member BCI 38 调用 `OuterReceiverCases.access$101(this$0)`；桥方法体对直接父类 `ReceiverBase.value()` 做 `invokespecial`。把该调用生搬为 `access$101(Outer.this)` 的源码不可编译，把它改成 `Outer.value()` 则目标不同。

本地 JADX 先通过 `ClassModifier`/inline visitor 识别 synthetic helper，再由 `InsnGen.callSuper` 沿父类链写 `Class.super`。其顺序可参考，但“可赋值于某个父类”和可内联标记不是 Jarde 的接收者、直接父类目标或使用闭包证明；生产侧不引入 JADX 或通用内联 pass。

## Goals / Non-Goals

**Goals:** 在一个已证命名成员家族内，对无分支、单调用/单返回的准确方法桥和所有捕获 Outer 调用点做局部证明；以一个原子文本投影保留分派、异常、参数和来源。

**Non-Goals:** 任意 synthetic accessor 内联、跨层祖先或 interface default 的限定 super、static nested/local/anonymous、一般方法句柄等价改写、开放世界中未知消费者的全项目兼容承诺。未知 method handle/bootstrap 使用是拒绝条件，不是默认安全。

## Decisions

1. **沿准确物理定义验证桥，不靠 `access$` 名称。** 候选必须解析到所选 Outer 的唯一 synthetic/static 方法，descriptor 的第一参数是 Outer，且完整 Code 只含局部读取、对 Outer 直接 superclass 方法的准确 `invokespecial` 和相应返回；参数/返回 descriptor、异常表、隐藏类初始化效果及额外指令都须被核对。一个目标不能同时由多个混淆定义承担。替代方案是按方法名或 `ACC_SYNTHETIC` 直接内联；两者都允许错误目标或额外效果。

2. **调用点逐个证明捕获接收者和有序实参。** 使用第一阶段的唯一 capture SSA 证书，把每个物理调用的首参追到 Member 的 `this$0` 值；普通 `other` 即使 descriptor 同为 Outer 也拒绝。核对调用点到桥体、桥内目标的参数映射和表达式求值/异常顺序；不把 `special_receiver` 对当前 entry-this 的证书挪用到外层捕获值。替代方案是只由常量池 owner 或调用 receiver 静态类型决定 `Outer.super`，同类型参数反例会变成错分派。

3. **完整使用闭包先于源码删除。** 在选定物理视图和请求可见的解析闭包中，沿已有 XRef/类引用事实枚举对该准确 `PhysicalMethodId` 的调用、method handle/bootstrap 和其他引用；每条调用必须落在已证家族方法中并具有自己的投影，其他引用一律拒绝整桥删除。不能因只看到了一个 Member 调用就把 helper 从 Outer 的文本中省掉。预算/取消贯穿扫描，部分覆盖不能升级为完整闭包。开放世界无法保证无未知字节码链接，因此报告只承诺选定 source unit/输入闭包，已知外部使用必须阻止投影。

4. **局部表达式投影，双端物理来源保留。** 在第一阶段的 nested writer 和家族方法投影接缝中按证书输出 `Outer.super.method(args)`；唯一引用闭合后才把 helper 从**文本**省略，`ClassSourceMethod`、原恢复结果与 source map 均保留。派生记录同时标记 Member caller BCI、Outer bridge body 的 `invokespecial` BCI、物理目标方法引用与生成表达式位置。若来源/文本预算停止，不发布半个改写；根/child 的同号 BCI 仍由 `PhysicalMethodId` 区分。替代方案是修改不可变 SSA 或在完成的源码字符串上替换调用；前者扩大 IR 机制，后者丢失身份和原子性。

5. **把分派结果作为验收判据。** 冻结原/JADX/Jarde 的全类 Java 8 编译、`-Xverify:all` 四段结果；添加显式 `other`、不同直接父类、额外调用点、method handle、桥内效果和异常覆盖的 verifier-valid 或直接 proof-unit 反例。JADX 只是算法/呈现对照；原 class 的实际调用目标、异常顺序和行为决定准入。

## Risks / Trade-offs

- [桥只有等价外观却调用不同目标] → 解析 Outer 的直接父类与准确方法 descriptor，核对 `invokespecial`，用 Base/Outer/MemberBase 三种返回值区分。
- [隐藏 helper 后仍有外部物理使用] → 预算内完整扫描选定输入及可见依赖；任何未投影或不透明引用拒绝文本删除，物理方法始终留报告。
- [多处调用中的一处失败] → 按桥整体原子提交，拒绝时全部调用维持物理来源，不发布混合的半套 `Outer.super`。
- [异常处理器或参数效果被移动] → 对桥体、调用点逐条核对覆盖和有序值依赖，不成立就拒绝。

## Migration Plan

该改动只扩大已证家族的源码投影，不改变原物理方法/字段身份。若某桥的闭包或顺序证明不成立，按现有家族拒绝状态保留物理文本；无需迁移或保留旧启发式行为。
