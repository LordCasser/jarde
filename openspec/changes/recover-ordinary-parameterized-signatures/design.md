## Context

见 `proposal.md` 与冻结的 [三方证据](../../evidence/java-syntax-2026-09-24/ordinary-parameterized-signatures/analysis.md)。reader 已按需解析 JVMS `Signature` 的普通参数化、数组与通配符，并提供逐位置擦除证明；query 共用该树。`src/class_source.rs` 只按 descriptor 拼方法头，现有泛型方法投影对 `type_parameters` 为空的普通参数化签名整项拒绝。JADX 的 `SignatureProcessor` 在类型推断前把解析类型写回方法节点，值得借鉴其先保留类型事实的顺序；其 `validateParsedType` 的非冲突判断不能替代严格擦除：Verifier 可接受的直接 List→Tree 元数据变体仍被 JADX 打印为错误源类型。擦除同为 Map 的嵌套 `Tree` 不会被该证明排除。

## Goals / Non-Goals

**Goals:** 让完整类方法头在真实 `Signature`、descriptor、源码拼写及正文/调用绑定同向证明后保留普通参数化类型；静态、实例和无正文成员走同一事实与发布口径。失败时解释并保留物理声明。冻结 identity 样本须在泛型反射、完整类与独立调用方编译执行上闭合。

**Non-Goals:** 不在 reader 中判断源级合法性；不由方法体的 cast 推断不存在的 `Signature`；不新增泛型求解器或 JVM 依赖；不改变独立方法体与增强 `for` 元素类型。当前单类请求也不提供目标 Java 8 平台及外部 classpath 的类型存在性证明；合法源名不等于该引用在重编环境可解析。类/字段泛型头及类变量作用域、合成构造器参数映射不在本项臆测，缺证明时明确拒绝并由后续独立 change 承接。方法自有 `<T>` 由在途 `recover-generic-method-signatures` 负责，本项复用其装配接缝，不重写其结论。

## Decisions

1. **同一物理成员的一次读取，解析与投影分层。** 从当前 `MemberHeader` 唯一 `Signature` shell 读取原字节，沿用 reader 的有界 `parse_method_signature` 与 `prove_method_signature_erasure`；无属性直接保留 descriptor。只在 class-source 判断可拼写性、类头类型变量上下文、正文与调用。替代方案是复制 JADX `SignatureParser` 或让类源码依赖 query；两者都会产生第二套 grammar 或反向依赖。`noak` 不提供本项目已具备的完整 source proof，故不加依赖。
2. **从结构化类型构造整个声明候选，不替换成品字符串。** 将 `SignatureType` 的 base/array/class 和 `TypeArgument` 的 exact/any/extends/super 递归写成 Java 8 源类型；每个 `ClassTypeSegment` 的完整限定名、嵌套关系和标识符须有无歧义拼写，无法确认的 `$`/嵌套路径整项拒绝。参数类型按 reader 擦除结果与 descriptor 逐位置配对；位置名称沿用同轮 `RecoveryFacts`/`NameTable` 的槽映射，`ACC_VARARGS` 只在最后数组层改写 `[]`，`Exceptions` 仍取原属性。方法/参数/type-use 注解沿既有装饰路径逐位置附着，无法证明时整项拒绝。替代的文本替换会破坏数组、嵌套通配符、注解或同名类型位置，不采用。
3. **正文证明与调用绑定是发布条件。** 对已恢复方法，使用同轮 Program/SSA 的参数槽、来源和表达式消费构造源级兼容侧证据；至少支持无变更的参数直接返回、相同来源的安全合流及无受影响消费者的正文。参数化后会改变静态接收者或返回类型的调用位置，须与实际 Methodref 目标交叉证明或保守拒绝；当前类对目标方法的调用也须保留原绑定。不能靠“擦除同一”推断 Java 重载选择不变，不能调用 `javac` 作为产品证明。`NoBody` 的 abstract/native/interface 方法没有 Program，改由旗标、声明位置和类装配关系证明；未恢复或混合正文不假装有兼容证据。现有 `GenericReturnCandidate` 可承载直接参数返回，但扩展时应复用同一 Program/SSA 事实，不按生成的文本或每个 fixture 制作字符串模式。
4. **原子类级发布。** 在一次预算内完成属性读取、语法/擦除、类型与声明构造、正文/调用核对、marker/来源文本计费，再替换 `ClassSourceMethod.declaration` 与 `text`；保留 `item` 物理身份、独立 `RecoveryReport`、body、原属性和参数名。`Recovered` 与 `NoBody` 采用各自现有块/分号模板，不绕过 `ClassSourceOutcome`；停止和拒绝都不发布半个头。essential/all 只改变证据详细度，不改变正文。
5. **类型声明与增强 `for` 后续分开。** 本项会使 `Iterable<String>` 方法头可信，但不立即把 `String` 传播到 `iterable_for_each_candidate`。后者在方法体阶段构建 AST，目前 `Object` 头加原 cast 是可执行的；将来须以同一参数槽的已证签名事实进入 builder，再证明 `next()`、cast、异常处理器与副作用位置，不能在最终类文本上替换 `Object` 或为 raw 来源插入 unchecked 转换。

## Risks / Trade-offs

- **合法 `Signature` 却无法写成单类 Java 源** → 保留 descriptor 头与原始属性/拒绝原因；不把 classfile 的合法性等同于当前源码上下文的可呈现性。
- **嵌套泛型实参的类名存在性未证明** → `Map<String,Tree<Integer>>` 的外层擦除仍是 `Map`，即使 `java.util.Tree` 不存在；当前来源名检查只证标识符/内类路径可拼，不读取目标 Java 8 平台或外部依赖。对此变造 class 的输出不能宣称可重编，须作为独立的环境类型解析债务记录；本 change 的重编正例限于已备齐类型的 Java 8 夹具。
- **泛型声明改变表达式静态类型和重载目标** → 使用同轮 AST/SSA/Methodref 事实逐点准入；缺证明时拒绝，即使反射签名因此暂时未追平。
- **内类、构造器合成参数及 type-use 注解需额外元数据** → 严格按已有来源决定映射，不能靠 `$` 或参数数量猜；缺口单列，不塞入本 change 的成功子集。
- **现有方法自有 `<T>` change 同时编辑 class-source** → 先独立验收该接缝，再合并普通参数化分支与共同发布代码；不得用“支持普通泛型”覆盖方法局部类型变量证明。
- **预算/取消与来源回归** → 候选构造、类型递归和调用扫描逐项计费并轮询，原子写回；测试检查 essential/all、低预算、取消、物理记录及来源。
