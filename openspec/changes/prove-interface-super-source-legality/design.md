## Context

见 [proposal](proposal.md)、[直接接口冗余证据](../../evidence/java-syntax-2026-09-25/redundant-interface-super/analysis.md)、[父类链冗余证据](../../evidence/java-syntax-2026-09-25/superclass-redundant-interface-super/analysis.md)和[抽象方法证据](../../evidence/java-syntax-2026-09-25/abstract-interface-super/analysis.md)。`preserve-special-call-dispatch` 的 2.1–3.1 已落地：`MethodIr` 借用同次类头的直接接口，`CallTarget` 保留 `InterfaceMethodref` 类别，`Builder::special_receiver` 证明入口 `this` 后写出 `Super { qualifier }`；失败会经现有 region fallback 保存 BCI 与延期生产者。它尚未归档，3.2 的全库门禁债务不在本变更处理。

现在的直接接口名单没有接口之间的继承边、父类链已实现的接口，也没有目标方法声明。`RedundantDirectSuperProbe` 的 patched class 保持 `invokespecial` owner 为直接 `Parent`，但另一个直接 `Child` 继承它；JVM 运行可接受，javac 拒绝 `Parent.super`。另一组 patched class 的其它直接接口无关，但 `Base` 或更远祖先 `Root` 已实现 owner `A`，JVM 仍运行 `A.m()`，javac 同样以冗余接口拒绝 `A.super`。第三组 patched class 保持唯一直接 `Child` 与相同特殊调用，只把选定 `Child.value()` 换成抽象声明；JVM 验证通过但调用抛 `AbstractMethodError`，javac 仍拒绝 `Child.super`。源级合法性需要完整的选定类/接口关系和目标 default 两份额外事实，不能由运行输出或常量池类别替代。

本地 JADX `InsnGen.getClassForSuperCall`（`jadx-core/.../codegen/InsnGen.java`，约 1080–1112 行）沿当前类及父类问 `ArgType.isInstanceOf`，`callSuper` 若返回当前类就发裸 `super`；正常双接口反例中这抹掉了 `InterfaceMethodref` 的显式 owner，运行结果从 11/22/33 变成 7/7/14。它提醒要按需访问祖先，但不能复制其“可赋值即改限定名”的决定。Jarde 保留原池项的接口类别与 owner，只用选定层级事实判断这个限定名能否合法写出。

## Goals / Non-Goals

**Goals:** 对候选接口 `super` 的 qualifier 做有界、按选定定义的非冗余证明，包括其它直接接口和当前类的完整父类链，并在同一选定接口闭包中证明唯一可访问的 default 目标。直接声明、唯一继承和多个无关直接接口的正常显式选择仍可恢复；无法证明时保留具体缺口与停止状态。

**Non-Goals:** 不验证任意 classfile 的 JVM 合法性，不重新实现 method resolution、JLS 全部默认方法规则或 `Outer.super`；不组装接口源码、不扩张候选到非直接接口、不读取接口方法体。不把旧特殊调用 change 的全库 Clippy 债务并入本项。

## Decisions

1. **在 facade 按需证明选定类/接口关系，recovery 层只消费结论。** `recovery_from_with_class_candidates` 已持有物理方法、`ResolutionEnvironment`、内容快照和预算；`jarde-java` 不应依赖解析器或读取其它类。仅当指令表含非构造 `invokespecial InterfaceMethodref` 且 pool owner 命中当前类的直接接口时，才枚举唯一候选 owner。对每个候选先选定目标接口 `I` 的完整头和成员表。若当前类有其它直接接口 `D`，逐个在同一环境与 loader/调用方上下文中选定其声明、确认 `ACC_INTERFACE` 和内部名，再沿 `interfaces` 边做有界 DFS；遇到 `I` 即证冗余。当前声明是 class 时，还沿同源直接 `super_class` 选定完整父类链，确认每个节点是 class 且内部名匹配，并遍历各父类 `interfaces` 的选定闭包；任何接口等于或继承 `I` 即证冗余。当前声明是 interface 时不走 class 父类链。父类链走到 JVM 固定根 `java/lang/Object` 时终止：普通 plain-jar 环境不包含 JDK 类定义，不能为了核对一个不存在的外部 snapshot 而拒绝全部正常类；只对这个 JVM 根应用零接口的固定事实，任何其它父类缺定义仍未知。全部可证、均未遇到才证非冗余。重复直接名、未选定、歧义、属性停止、错误类别、缺边或关系循环均不能当作无继承。即使只有一个直接接口，也须读选定 `I` 以证明目标 default；父类链证明同样不能由当前类直接接口名单替代。
2. **证明源码可绑定的 default，而不伪称完成 JVM method resolution。** 同一选定接口闭包已有 `ClassMemberFacts.methods` 的名称、descriptor、flags 与 Code 属性壳，无需读 Body。对池项 `name+descriptor`，从 `I` 沿选定父接口图查找最接近的同签名声明：同签名唯一 public、非 static、非 abstract 且有 Code 的方法可作为 default；`I` 自己或中途更近接口的抽象声明立即否决，不能绕到更远祖先。多源冲突或缺任何边拒绝。为避免原物理调用在 Java 源被另一个重载夺走，首片要求参与的接口闭包无其它同名不同 descriptor 声明；这会保守拒绝 Java 8 可通过显式 cast 合法绑定的重载，留待后续同一接缝证明参数的源级绑定。物理重复、桥接/泛型签名或不能证明的源码绑定也拒绝。保守拒绝是来源明确的覆盖边界，不是把 JVM 调度错误地改写成另一个 owner。与单靠 pool owner 或实际异常分类相比，这一局部门能同时保留合法继承 default 并拒绝抽象覆盖。
3. **复用现有选定定义读取链，不建立全局继承图。** `resolve_class_source_dependency_read` 已把符号解析、选定定义、材料化/摘要匹配及成员表停点放在同一请求中；其结果含接口头、完整成员表与属性壳，不读取方法体。把选定结果用于本次关系和方法 DFS，按唯一物理定义缓存本次请求已访问的接口，预算/取消与解析的 `ExecutionReport` 传播；每个结点在访问前占用现有深度/读取/步数预算。可抽取现有选择辅助函数以免重复，不能让不完整解析结果冒充“无父接口/无重载”。库函数替换不了本仓库带 loader 身份的选定定义和物理来源，新增外部依赖没有相应能力收益。
4. **以窄候选结论交接证明，缺省安全。** facade 将本请求已证可合法拼写的精确调用目标（至少含 owner、name、descriptor）作为短生命周期借用传入 `RecoveryRequest`/`build::Inputs`。`Builder::special_receiver` 保留当前 pool 类型、直接 owner、入口 this 与 private 优先级判断；接口分支必须命中证明集。空集表示未证明，不能被当作放行。不能把同 owner 的另一个重载或同名祖先的可用性借给当前调用。直接调用 recovery 而没有选定环境时保守拒绝接口 `super`；无需新增 AST/Statement/通用 pass。
5. **拒绝仍走现有缺口。** 已证冗余、抽象覆盖与关系/方法未知产生说明 qualifier 或目标 default 不可证的 `special_receiver` 错误，包含 BCI 与 owner；由已有嵌套表达式回退追踪 receiver/args 的延期生产者。预算或取消通过 `Result`/执行报告终止，不伪装成某个接口的正常恢复。`class-source` 和单方法入口共用该门；仅 source 质量受影响，不修改物理 facts、绑定结果或 JVM dispatch 报告。
6. **验收以源码重编和行为双向约束。** 先复核三组冻结 Java 8 `-g`/`-g:none` 正例、JADX 错值/不可编译及 patched 负例，再对新 Jarde 运行。正例 class-source 和单方法保持各自 `Left.super`/`Right.super`，完整源码重编输出 11/22/33；唯一直接接口的自声明 default 与继承 default 分别输出 4、3。直接接口冗余、父类链冗余与抽象 patched 负例均不能再发布非法 `I.super`，缺口引用 BCI 1，带实参效果的变体确认生产者来源保留。另用缺少选定定义、同名重载、多个父 default、重复/歧义选择及紧预算控制未知/停止；class-super、private 回归不受影响。`-Xverify:all` 仅证明 JVM 运行或抛出运行时异常，不作为 Java 源码合法性的替身。

## Risks / Trade-offs

- **接口闭包不完整或含复杂重载/泛型签名，可能保守拒绝本来合法的调用** → 只在选定定义链和目标方法均完整时声明可写，并在报告中显示缺失来源；后续可在已有接缝逐项证明绑定，不能按名称或方法签名猜测。
- **父类链增加选定读取和循环风险** → 只对实际接口特殊调用且当前声明为 class 时沿直接父类逐级读取，每个节点先按现有预算/取消/选定身份核验，完整遍历后才能证明非冗余。
- **为每个候选重复读取接口头，预算激增** → 先筛候选、同请求按定义去重，DFS 访问前轮询并使用现有深度/读取预算；不得遍历方法体或整个 classpath。
- **把未知误判为可写，导致 Jarde 文本 javac 失败** → 所有接口调用缺省空证明集，冗余与抽象负例以及丢定义/歧义/预算控制均须拒绝或停止。
- **把已正确的双接口调用一律拒绝** → 无关接口正例逐个重编并执行，同时核对同名父类结果 7 没有替代 11/22。
