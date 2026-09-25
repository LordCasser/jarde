## Context

见 proposal 与 `../../evidence/java-syntax-2026-09-22/generic-method-references/`。普通源码的两个工厂在 descriptor 上都返回 raw ToIntFunction，但 bootstrap 的动态参数分别为 String 与 Object。class-source 当前仅呈现 descriptor；无需先恢复泛型 Signature 才能保留这一区别。

`lambda::plan` 已解析 SAM、instantiated、implementation 描述符，但将参数压成引用/primitive shape 比较，丢弃返回部分，并把 instantiated 参数直接用作 LambdaParam。`Builder::lambda_expr` 直接构造 Call/New，没有经过普通调用参数的类型约束入口。MethodReference 没有可写的参数 cast；完整类中使用 raw SAM 时，两条路径分别造成错选重载或不合法的显式 lambda 参数。

root 的 `wider-target/` 又用3项完整类执行固定了动态 String、实现 Object 的区别：原/JADX均在Integer输入上先CCE且实现调用0次；jarde漏检查、返回7且调用1次。这是额外已证实缺陷，不仅是设计假设。

## JADX 源码对照与当前架构落点

本地 JADX `2fb1b163` 在 `CustomLambdaCall.buildMethodCall` 读 `LambdaMetafactory` bootstrap 中的实现 handle 和有效方法原型，只在有效参数与实现方法参数相同时设置 `useRef`；`InsnGen.makeInvokeLambda` 据此在方法引用、显式 lambda 与内联合成方法之间选择。这个“先比较参数，再选择表达式形态”的顺序可借鉴，但不能把 `sameArgs` 当成本项目的完整适配证明：它没有在该判断里同时证明擦除 SAM 参数、动态参数、实现参数三者的检查次序与捕获阶段。当前冻结 JADX 类保留 `ToIntFunction<String>` 等泛型声明，所以 `LambdaAdaptationSupport::pick` 和 `v0 -> wider(v0)` 交给 Java 编译器时已有 `String` 静态目标；Jarde 当前类声明仍为 raw SAM，直接照搬相同方法引用/无检查 lambda 会改变重载选择或漏掉调用前的 CCE。本 change 因而从已有 bootstrap 事实显式构造受限检查，再固定实现 Methodref；方法引用只在无需适配时选择。JADX 的成功执行可作对照，不能代替这一层证明。

## Goals / Non-Goals

**Goals:** 修复受限引用参数适配：擦除 SAM 的 Object 参数在调用时检查为动态引用类型，再以精确实现 descriptor 调用。覆盖静态方法、无捕获的实例接收者与构造器；保持已支持同型 primitive/no-adaptation 对照。

**Non-Goals:** 不扩展任意继承证明、boxing/unboxing、数值 widening、返回窄化、捕获 producer 准入、特殊 handle dispatch 或 altMetafactory 附加 flags。不新建 lambda body/泛型 AST，不因 javac 拒绝而猜任意转换；这些边界必须明确拒绝或保留已证明的原写法。

## Decisions

1. 沿用同一个 lambda Plan。保留后续表达式构造真正需要的擦除参数、动态参数与 implementation 参数，不增加第二套 descriptor parser、通用 adaptation framework 或 resolver。现有 evidence 的三份 descriptor 仍是报告事实，不能把报告字符串反向解析为另一份决策。
2. 显式 lambda 的参数取擦除 SAM 的类型。对每个调用参数，按 bootstrap 中的顺序构造动态检查：同型不变；Object 到合法引用/数组类型使用现有 Cast。其它不相等的引用擦除类型暂不推断继承关系。检查的 origin 是真实 invokedynamic/site CP，不能虚构 checkcast opcode；原 capture 来源继续保留。
3. 动态检查之后才固定 implementation 参数的源码静态类型。先覆盖同型和到 Object 的确定上溯，复用现有类型/显式 cast 构造，避免另造“所有引用都可转”的规则。Object 上溯必须包在动态 String 检查之外，不能因最终形参是 Object 删除中间检查。捕获参数按 factory descriptor 与实现前缀核对，不能把 SAM 动态转换套在 capture 上。无证据的转换以既有 lambda refusal 明确停止。
4. implementation 的接收者也占据该有序输入列表。无捕获实例引用用第一个 SAM 参数检查/固定接收者，其余参数保持顺序；构造器用已有 New，按构造参数处理，结果类型是 owner，不能把 `<init>` 的 void 返回当成对象工厂无结果。cast 不得复制或提前执行 body、receiver 或参数。
5. 方法引用只在当前实际输出目标下可证明不需要参数适配且实现参数类型固定时使用。需要上述检查者改写为现有 Lambda；不能仅凭“capture 数量等于 receiver 数量”选择 MethodReference。与泛型目标 Signature 同步恢复相比，此路径只消费已有事实，修改更小且能保留调用时 CCE。
6. 返回路径不能照搬参数路径。`expanded/return-probe/` 实证 implementation `()Object`、SAM `()Object`、instantiated `()String` 的 raw Supplier.get 可以返回 Integer，只有 typed caller 自己的 checkcast 失败。原/JADX/jarde 完整类在 String/Integer 两状态均相同。因此本项核对实际 implementation→擦除 SAM 的返回适配：primitive 同型、引用同型、确定的 Object 上溯及已有 `Runnable` 等 `void` SAM 对非 `void` implementation 的结果丢弃可保留；需要其它真实返回转换则明确拒绝。instantiated 返回仍作为原 bootstrap 事实保留，不能仅因它较窄就凭空在 lambda 体加入检查，或把本已正确的 raw Supplier 控制降为引用。
7. 保持捕获与调用的阶段区别。现有 replayable 检查保留；对需要从 bound 方法引用改为 lambda 的形状，必须已有证据保证原 capture-time 非 null 检查和 receiver 保存仍在创建阶段。没有这项证明就明确拒绝，不新增一次 getClass/requireNonNull 或把异常推迟到调用。不得把 `preserve-deferred-value-order` 的存在当成自动允许所有捕获。
8. 使用既有非槽命名入口；避免已占局部/字段/lambda 参数。计划新增描述符工作、向量和 cast 节点应走现有共享工作/IR 预算与取消，essential/all 必须产生相同正文；不能在 source-map replay 重新选择适配形状。根代理实测证明“后置来源交付耗尽时没有半个 lambda”，但不能据此说内部新增工作已计费。当前 `lambda_expr`/`render_value` 的 `String` 拒绝通道无法安全传播 `StopReason`，已拆到 [propagate-value-rendering-stops](../propagate-value-rendering-stops/design.md) 以最小 typed failure 接上现有构建停止平面；本项 2.4 在该依赖验收前保持开放。
9. 不引入外部库。descriptor、类型和 Lambda/Cast/Call/New 已有，缺口是已有事实被丢弃后的错误决策，库无法在缺少外部类型事实时补出合法继承证明。无需新增维护与许可成本，也不修改解析、dialect、runtime selection、verification/compilation 平面。

bootstrap 的捕获/调用分期与动态参数限制参考
[LambdaMetafactory](https://docs.oracle.com/en/java/javase/23/docs/api/java.base/java/lang/invoke/LambdaMetafactory.html)。本设计的受限证明和拒绝边界来自当前架构及执行对照，并不宣称实现该 API 所允许的全部适配。

返回控制另与 [OpenJDK 23 ForwardingMethodGenerator](https://github.com/openjdk/jdk/blob/jdk-23%2B37/src/java.base/share/classes/java/lang/invoke/InnerClassLambdaMetafactory.java#L470) 核对：参数转换使用动态参数类型，返回转换使用擦除 SAM 返回类型。此实现证据用于解释当前原 JVM 观察，不能把 API 概述中“动态结果限制”直接翻译成额外 cast。

## Risks / Trade-offs

- 只固定 implementation 类型会丢掉动态输入检查 → 独立使用“动态 String、实现 Object”反例，错误类型必须在进入实现前失败。
- 显式 String 参数与 raw SAM 不兼容 → 擦除参数和参数体内检查分别呈现，完整类 javac 验收。
- 改成 lambda 后 bound null 延迟失败 → 缺创建阶段证明时拒绝，保留原 bound-null 对照，禁止手改生成正文。
- 返回适配引入无依据的 CCE → 本项限制为确定同型/Object 上溯，泛型返回单独实测并登记。
- helper 合成名冲突掩盖目标语义 → 正面夹具优先普通方法引用、不生成 lambda$ helper；既有 helper 冲突另案，不删除生成方法绕过编译。
- 生产文件与延期值实现重叠 → 先验收 `preserve-deferred-value-order` 再交接本项，测试夹具可独立准备；不把 lambda 捕获扩展夹进延期值任务。
