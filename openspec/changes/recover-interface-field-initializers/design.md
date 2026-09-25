## Context

证据在 `../../evidence/java-syntax-2026-09-22/interface-field-initializers/`：771 B 的 Java 8 接口含一项 `ConstantValue` 与三项有副作用的运行时初值。原 class、JADX 在完整编译/验证下输出 `ABT|A1|B2|4|7`；Jarde 的三个无初值接口字段和一个 `static {}` 使 javac 报四错。交换两个完整 `field_info` 后，Code 与原执行不变，字段表顺序却与写入顺序相反。`constant-phase-boundary/` 的合法 496 B class 两字段均无 `ConstantValue`，先读默认值再以 `sipush 9` 写入，原 JVM 输出 `0|9`；JADX 的完整源码重编译后输出 `9|9`，手写的机制对照显示 javac 为字面量声明生成 `ConstantValue` 并内联前向读取。JLS 8 [§9.3.1](https://docs.oracle.com/javase/specs/jls/se8/html/jls-9.html#jls-9.3.1)要求接口每个字段有初值，且非编译期常量确实在接口初始化时求值；[§15.28](https://docs.oracle.com/javase/specs/jls/se8/html/jls-15.html#jls-15.28)界定了会被提前折叠的常量表达式。

当前 `src/facade.rs::class_source_with_evidence` 按字段表构造 `ClassSourceField`，字段只有自己的 `ConstantValue` 可变成 `=...`；方法逐个经 `recover_prepared_member` 产生 `ClassSourceMethod`。核心 `jarde-java::report::recover` 在生成 `RecoveryReport` 时已经丢掉结构化 statement，只留下正文文本和 source map。`<clinit>` 的原始 Code/SSA 在成员恢复接缝仍可借用。当前 `ClassSourceReport::fields` 保留物理表序，`source_text` 则也照该顺序发射；本项只有源文本需要在完整证明后改序。

## Goals / Non-Goals

**Goals:** 在一次既有类读取/成员恢复内证明普通接口的直线、单次字段初始化组，原地把可拼写的值移动到声明，并使物理报告仍能审计该移动。失败全组拒绝，保留真实来源、预算和停止。

**Non-Goals:** 不实现普通类或注解接口字段初始化、枚举投影、任意控制流/异常等价、跨类类型解析、Java 源反解析、通用初始化器 IR，亦不把方法体中任意字段写当成声明赋值。

## Decisions

1. **类级证明不可省，但只做局部私有侧车。** 单字段声明和单方法恢复都无法知道“每项接口字段都有初值”及跨字段效果顺序；`RecoveryReport.text` 又不能反解析。于已有 `MethodIrAnalysis::ir()` 到 `jarde-java` Builder/Emitter 的一次恢复路径上，为类源码可选地带出 `<clinit>` 的有界候选：每条真实静态字段写的已证目标（owner/name/descriptor）、右值结构、来源 BCI、求值顺序和剩余语句状态。它与正常报告同次生成，不复跑分析或发射；类装配层只短暂持有，最终报告保留必要投影归属。可考虑直接扫描字节码并从正文截字符串，但前者缺语义表达式，后者丢失身份/顺序/来源，均不采用。若邻近的 `recover-proved-enum-constants` 已落下同类接缝，复用其私有通道，不并行发明第二套。
2. **整组准入而非逐字段搬移。** 只接受 `ACC_INTERFACE` 且非 `ACC_ANNOTATION` 的普通接口，字段名字/descriptor/flags 无歧义且 Java 可声明。每个字段要么已由自身 `ConstantValue` 给出可拼写初值，要么在唯一的 `<clinit>()V` 的已证直线正常路径上被一次写入；不能同属两组。每个写须由现有 field plan 证明为当前接口的对应声明、值单次求值且赋值类型等价。整个 `<clinit>` 除这些赋值与尾部完成外不得有未认领 statement、异常边、分支、逃逸或未呈现 effect。任何一点失败，所有运行时初值保持原报告/明确拒绝，不先发布部分声明。
3. **顺序与初始化阶段均由执行事实决定，物理事实仍原样保存。** 对非 `ConstantValue` 字段按已证 `putstatic` 执行顺序排列源声明；对应原字段条目仍以物理表索引存于 `ClassSourceReport::fields`，并公开足以对照字段索引与来源 BCI 的投影结果。现有字段注解与值、标记随其声明移动，不脱离原字段。常量字段由 JVM 在普通初始化前备值，保持自身 `ConstantValue` 来源。原本仅由 `<clinit>` 写入的字段若被写成 Java 常量表达式，会变成新的 `ConstantValue`，甚至使其它字段的读取在编译时内联；因此每个运行时 RHS 必须用既有 AST 与字段事实证明其源码不是 JLS 常量表达式，不能证明则整组拒绝，不为通过门禁发明求值屏障或常量计算器。对可能受源码前向引用、名称遮蔽或声明位置影响的表达式，同样先经既有命名/类型/求值位置规则证明或拒绝，不假定字段表即源码顺序。重排只作用于组装文本，不改物理读取顺序或恢复 `<clinit>` 的原始 `RecoveryReport`。
4. **仅投影完整 Java 表达式。** 右值沿用已有 AST/发射器/输出预算与 source origin，不能把 `local` 临时变量、独立前缀调用、未恢复 fallback 或数组 store 文本拼进 `=...`。若一个独立效果实际先于右值，应先证明现有 deferred 保存已经包含在该表达式的真实求值位置，否则拒绝。成功投影后，源文本不另写接口 `static {}`，但报告保留原方法身份、分析/恢复结果及明确“已投影到哪些字段”的状态。失败则维持保守、有标记的原呈现，不把不可编译文本标成编译通过或语义验证。
5. **预算与职责边界。** 候选遍历的每个字段、statement、SSA use、来源和发射字节走现有 budget，持续轮询取消；停止不变成空组。reader 的 classfile 解析及 Java 8 方言事实不变，JVM 运行时选择/验证不由此功能执行；javac/JVM 只在受控 fixture 验收中运行。现有 noak/SSA、AST 与 emitter 已提供所需事实和呈现，外加解析/反编译库既不能消除跨成员证明，也会增维护与许可负担，因此不新增依赖。

## Risks / Trade-offs

- **字段表与执行顺序不同** → 使用冻结的 field_info 交换样本；源码按写入顺序，JSON 保留表序和来源。
- **运行时赋值被 javac 升级为 ConstantValue** → `constant-phase-boundary/` 原类 `0|9` 与 JADX 重编译 `9|9` 已证；在既有 AST 上保守判断 RHS 是否可能成为 Java 常量表达式，不能证明非恒定则拒绝整组，不仅检查写入顺序。
- **部分投影掩盖额外效果** → 将 `<clinit>` 的完整已恢复 statement 与异常边作为全组条件；注入额外调用/重复写的合法 classfile 负例。
- **前向引用或常量早期初始化改变值** → 比对原 class 与修后整类的调用 trace、初值、异常；无法证明的表达式拒绝，不补猜测性限定名。
- **预算或输出耗尽时留下半组** → 先收集并证明有界计划，提交时按现有输出计费；中止不发布已搬移声明，原始报告及 stop 保持可见。
