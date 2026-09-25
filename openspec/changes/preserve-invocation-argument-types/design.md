## Context

反例见 `../../evidence/java-syntax-2026-09-22/overloads/`，包括逐例运行结果及 `type-boundaries/` 的两个关键边界。`Builder::arguments` 当前调用 `meeting_position(..., Widening::Position)`；引用转换视为 Same，窄常量与赋值一样处理，未知类型原样通过。调用描述符虽存在，源码的重载选择却不受该事实约束。

审查实现时确认一个必要前置：`MethodFacts::parameter_types` 为早期boolean证明把所有引用/数组压成简单名Object；后来局部表达式的presented复用了该表，已不能真实描述参数的源码类型。应在该既有事实入口用现有type_of_component保留完整descriptor类型与slot位置，不在调用处用SSA类型替换已呈现的静态类型。

既有 `arguments` 已服务普通 invoke、new 构造和 this/super prologue。`Cast`、`Type`、OriginSet、描述符解析及 Lambda 工厂返回类型都存在。本项在这些边界处理，不建立全类重载搜索。

## Goals / Non-Goals

**Goals:** 对证据足够的参数固定源码静态类型，同时不增加原字节码没有的可失败检查。无法证明者拒绝该调用，并保留参数生产者。

**Non-Goals:** 不推导任意子类型关系，不恢复外部泛型声明，不读取 callee body，不把函数式工厂捕获参数当成实现方法参数。lambda 内部实现调用及 method-reference 内部目标重载的完整重建另案处理；本项只约束它们作为外层调用实参时的目标类型。既有 synthetic 名称冲突不混入。

## Decisions

1. 先使现有parameter_types如实携带参数descriptor的完整引用/数组类型；对应参数位置测试更新并保留boolean/双槽布局。调用实参只读Expr.presented，不能因为SSA某一值更精确就越过文本局部声明类型。隐式this仅在现有has_receiver与declaring_class事实齐全时由同一局部类型入口给出本类类型。调用是独立的类型消费位置。复用一个参数构造入口和小型局部归一化逻辑；不把 Widening::Position 全局改义，不给赋值/返回补无关 cast。保留既有 boolean 0/1 适配；描述符不可读或参数数量不一致时拒绝，不能以“无事实”为由默认目标已保留。真实 invokevirtual/invokestatic/invokeinterface/invokespecial、new 和构造器 prologue 共享此入口；verified accessor 已化为字段的形状不虚构调用。
2. 数值参数：同型稳定值原样；已证明的 widening 用既有 Cast 写出所需静态类型。范围内 int 常量供给 byte/short 时显式写 `(byte)`/`(short)`，char 可复用字符字面量；调用没有赋值上下文的常量窄化许可。越界、boolean/其它原始类型不相容及未知数值类型拒绝，不合成会改变值的窄化。真实转换表达式若已给出目标类型，不重复包装。
3. 引用参数采用明确的安全边界：null 可以写目标引用 cast；任何已有引用值到 `java.lang.Object` 的上溯可写 Object cast；源值已呈现为目标引用类型时可固定该相同类型。稳定的同型局部、字面量、字段/数组读、new 或已有同型 Cast 无需再包；Call 即使呈现类型相同也必须固定类型，因为外部泛型 poly invocation 可重新推断为更窄类型。其它不相等引用类型缺少层级证据时拒绝，不枚举或硬编码 JDK 继承表。
4. Lambda/MethodReference 的函数式类型来自已通过既有规则的工厂 descriptor 返回类型，借用现有 Expr.presented 携带；不能从外层重载候选猜。作为调用参数时先用此函数式类型的 Cast 固定目标，再按规则3处理所需类型。若外层参数为 Object，必须保留内层函数式目标，不能生成 `(Object) System::nanoTime`；表达式的构造和捕获次序不变。缺工厂返回类型则拒绝，不新增 SAM resolver。
5. 规则3的限制有实际 JVM 依据：把 `Object -> checkcast Runnable -> take(Runnable)` 的 checkcast 换成 nop 后，`java -Xverify:all` 接受且 `new Object()` 入参返回7。直接补 Runnable cast 将改成 CCE。因此 descriptor 不是任意引用转换的证明。已存在的显式 checkcast 按独立 cast change 保留，它给出的目标类型才是本层可消费的证据；不能仅因为文本不能编译就发明该检查。
6. 新包装沿用原操作数来源，并以既有 derived 来源记录调用消费位置；不伪造 cast opcode 的 direct BCI。子表达式和 invoke 的原始来源不丢。失败调用使用既有 quoted_bcis，递归保留其参数及普通 cast/调用生产者；不新增效果分析。
7. 不新增外部库。所需能力为现有描述符解析和 AST 构造；没有库能在不读外部类型事实时证明任意继承关系。维护、许可和预算成本均无理由增加。此处只改变 source recovery，不修改 parsing、dialect validation、runtime selection 或 verification/compilation 平面的状态。所有执行是授权的自写样例对照。

语义依据为 [JLS 调用上下文](https://docs.oracle.com/javase/specs/jls/se23/html/jls-5.html#jls-5.3) 与 [JLS 重载选择](https://docs.oracle.com/javase/specs/jls/se23/html/jls-15.html#jls-15.12.2)。现有 2c.29 中赋值/返回的隐式加宽仍成立；其中“参数位沿用池成员选择”被可执行反例推翻，应更新当前规划和测试说明，历史 archive 不改。

## Risks / Trade-offs

- 显式类型变多 → 仅在调用消费点约束；稳定同型值不重复包，真实 Cast 不叠同型 cast。正确目标优先于贴近 jadx 文本。
- 未知引用上溯产生更多诚实 fallback → 将边界固定为测试；后续若需要更多覆盖，先证明可复用的类型事实，不能在本项加 resolver。
- 泛型调用类型表面相等仍错选 → 使用外部 GenericFactory，保留其原始 helper，生成 probe 整类实际编译；预期由原 class 实测。
- 同型函数式目标仍被外层推断改变 → Runnable/Supplier 重载与 method reference 对照；Object 参数同时验证内部函数式目标仍可编译。
- 参数 cast 改变顺序或异常 → 多参数左右副作用、null、box producer 次数和异常对照，不重复调用。
- 调整旧断言掩盖退化 → 只更新由调用类型约束解释的文本差异；赋值/返回/concat 保持原测试，真实拒绝不得手改方法体绕过。
