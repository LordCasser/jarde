## Context

见 [proposal.md](proposal.md) 和固定 [DT-27 基线](../../evidence/java-syntax-2026-09-28/dt27-typed-functional/README.md)。`src/class_source.rs::project_method_signature` 已解析方法 `Signature`、证明擦除并能拼写普通参数化声明，但 `ordinary_parameterized_declaration` 要求同轮正文候选。`crates/jarde-java/src/report.rs::generic_return_candidate` 尚不接纳直接返回的 `invokedynamic`。`crates/jarde-java/src/lambda.rs::plan` 只有 erased SAM、instantiated SAM、implementation 和捕获事实，不读方法目标 `Signature`：它把 `Object→String` 算作方法引用不能容纳的适配，静态引用退化成 lambda，绑定实例引用为保护创建期空值行为而拒绝。`src/facade.rs` 在 class-source 成员恢复后才投影声明，因此独立方法恢复与完整类投影的目标信息不同。

## Goals / Non-Goals

**Goals:** 对固定的 Java 8 `Function<String,Integer>` 静态/当前实例引用和 `Supplier<String>` 当前实例引用建立声明与函数站点的同一证书；完整类重编、运行和反射泛型返回类型均一致。保留每个物理 BCI/CP 的来源，预算或拒绝不发布半成品。

**Non-Goals:** 不使泛型 `Signature` 在普通单方法恢复中自动可信；不支持任意 `Function<T,R>`、任意 SAM、重载选择、unbound receiver、外部实例的 nullable/副作用捕获、altMetafactory 或整体 DT-27 清项。`FinallyOnce` 与其它 CF 形态独立处理。

## Decisions

1. **泛型目标只在完整类的 class-source 边界进入。** 复用现有方法 `Signature` parser、擦除证明、类型拼写和 prepared member recovery；为确切 `Function<String,Integer>`/`Supplier<String>`、无方法类型变量、简单顶级类建立私有的目标证书。证书需核原始 Code 是无 handler/异常表、无分支的直接返回：静态 `invokedynamic; areturn`，当前实例 `aload_0; invokedynamic; areturn`，且 SSA/Program 的结果确为同一站点。禁止仅因 `Signature` 存在就投影；这个入口比新建全局泛型类型系统更贴近已有职责。
2. **先证目标与 bootstrap，再选表达式。** 证书核 LambdaMetafactory 普通 metafactory、factory 返回擦除接口、SAM erased/instantiated 参数及结果、实现 handle 的 owner/name/descriptor/reach 和已支持转换。`Function<String,Integer>` 的 instantiated 形状必须是 `(String)->Integer`；`Supplier<String>` 必须是 `()->String`。`Integer.parseInt(String)` 的 `int→Integer` 属于 Java 方法引用允许的装箱；`this::length`/`this::label` 的目标与结果必须精确可写。这里可复用 `lambda::plan` 的 descriptor/转换事实，但把 **erased→instantiated** 的调用方泛型适配与 **instantiated→implementation** 的成员适配分别审阅；不能全局忽略任一适配，也不能影响 raw SAM 的现有显式 cast 行为。
3. **绑定当前实例只用局部、可证明的来源。** 证书要求 capture 是实例方法入口 local 0 的原 `this`，没有其它 producer、别名写入或求值。该值非空，直接 `this::member` 在创建时保存同一对象；普通 nullable 接收者、`next()::member` 或之后可变的 holder 不进入此证书，保持现有拒绝。`Reach::Receiver` 与 capture 个数单独不证明 nonnull，不能把两者当作通用许可。
4. **声明和正文同一事务提交。** class-source 应在恢复函数正文前把已受证目标作为受限上下文传给现有恢复；完整 Program/SSA 及源码形状复核后，将参数化声明和 `MethodReference` AST 一并发布。若复核、输出预算或源码编译边界失败，就撤销这次投影并给出明确拒绝/不完整标记。不能留下 raw 声明配 typed-only `::`，也不能只换 header 配 `(Object p0)` lambda。可沿用现有 `GenericReturnCandidate`/class-source staging 与 Builder checkpoint；具体承载字段以少量私有数据为准，不引入新 pass 或平行 AST。
5. **先固定正反例，再推进实现。** 固定 class SHA 与三方基线；反例分别改变泛型参数/结果却保留物理 descriptor 与 indy、加入直接返回前的可观察副作用或 handler、以及使用可能为空/求值有副作用的绑定接收者。受支持形态须完整 Java 8 重编、`java -Xverify:all` 多路径执行并核反射泛型类型；反例只验安全拒绝或已证明的原有语义，不把 JADX 的输出本身当正确性证明。还核源映射、预算/取消和原非泛型 DT-27、descriptor 适配回归。
6. **不引入外部库。** 当前 reader 已有 Signature、BootstrapMethods、descriptor、SSA 与 AST/emit 工具；缺的是两个现有决策平面的连接。外部库不会替代这项同轮来源与捕获阶段证明，新增依赖增加维护和许可核验成本。解析、dialect 判定、运行时解析与验证状态保持各自边界，源码恢复不自动执行目标类。

## Risks / Trade-offs

- **泛型头与函数正文错配** → 事务式提交并用完整类重编、反射和不匹配 Signature 近邻验证。
- **错误地忽略擦除转换** → 对照既有 raw `ToIntFunction` 动态检查测试，只有受证目标允许 Java 编译器承担该转换。
- **绑定接收者空值失败时机改变** → 只准入口 `this`，nullable/副作用捕获继续拒绝；后续扩展必须另证创建时行为。
- **伪造 SSA/来源或预算半成品** → 精确 one-return、完整 Code、唯一站点、来源覆盖与取消/输出预算测试；失败走既有保守通道。
- **范围外泛型/SAM 成为隐性回归** → 证书先限固定 Java 8 目标，既有 raw SAM、lambda helper、数组构造引用和方法引用回归必须保持结果。
