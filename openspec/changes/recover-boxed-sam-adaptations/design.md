## Context

字节码事实（`Patrol`，javac --release 8）：站点 BCI 2 `invokedynamic get:()Ljava/util/function/Supplier;`，BSM 参数含 instantiated MethodType `()Ljava/lang/Integer;`，impl MethodHandle 直指 `Patrol.supply()I` —— javac 不生成适配器，装箱由 metafactory 链接期 `asType` 完成。BCI 7 同形（`apply:()Ljava/util/function/Function;`，instantiated `(Ljava/lang/String;)Ljava/lang/Integer;`，impl `staticRef`）。`int[]::new` 站点的 impl 是真实合成方法 `lambda$arrayCtor$0(I)[I`，instantiated `(Ljava/lang/Integer;)[I`。

`lambda.rs` 原有适配证明只接受恒等、Object 检查、Object 上溯；上面的逐槽基本/包装配对全部拒绝，携带站点的方法整方法引用。2.1 已补两段配对，2.2 已在收到完整同类 `ClassMembers` 时证明合成分配方法 Code，但真实 `class-source` 尚未触发数组证明：`src/facade.rs::named_callee_candidates` 只收集 `accessor::candidates(ir)`，不收集 `invokedynamic` BSM 的实现句柄，因而该 helper 根本不在按需读入的成员表里。`PreparedClass` 已由类级请求持有；现有 `read_prepared_named_callees` 可以在同次预算下读取精确成员，无须第二次 class read。JADX 1.5.6 在冻结样例把 `int[]::new` 展开成 `x$0 -> new int[x$0]`，不是原样还原；此处的 `T[]::new` 是本变更希望做得更准确的语法恢复。

真实 Java 8 重编还揭示类级碰撞：若类源码同时保留物理 `lambda$arrayCtor$0(int)` 声明，并在 `arrayCtor()` 内呈现 `int[]::new`，javac 自己会生成同名同描述符 helper，报 `compiler-synthesized` 符号冲突。方法报告必须保留物理 helper；类源码只有在其用途被完整证明都已由数组引用替代时，才可原子省略该物理声明。用另一个词法方法名生成的 `lambda$f$0` 不构成这个正例的碰撞检验。

本地 JADX 1.5.6 的 `CustomLambdaCall.buildMethodCall` 从 bootstrap 的实现 `MethodHandle` 解析真实 `MethodNode`，仅同类 synthetic 方法可内联；`InsnGen.makeInvokeLambda` 再决定方法引用或 lambda 文本。这个“先沿精确句柄找到物理实现”的顺序值得复用。它找到同类 synthetic 后就标记 `DONT_GENERATE`，没有为本样例证明所有其它调用/句柄用途与源码目标类型；Jarde 不沿用该省略判定，而在完整类级证据之后原子提交。

仅有 helper 的 `(I)[I` 与 `load; newarray; areturn` Code 也不足以授予数组引用：`Supplier<int[]> s(int n) { return () -> new int[n]; }` 可以把长度作为站点捕获，而 `int[]::new` 必须从唯一 SAM 参数读取长度。准入还须证明零捕获、一个 SAM 参数。另一个独立源码门是目标类型：冻结 `BoxedSamProbe.arrayCtor` 的类级泛型 Signature 因相邻同类调用暂拒，声明写成 raw `Function`；Java 8 的 `Function arrayCtor() { return int[]::new; }` 编译失败（`Object` 不能转成 `int`）。因此类级省略 helper 还必须以可写且足够精确的函数式目标声明为前提；泛型签名恢复仍由原规则负责，本项不推断一套新泛型系统。

## Goals / Non-Goals

**Goals:** 逐槽配对判定扩展（包装/基本互转 + 数组构造器引用）；箭头呈现不变；负例保持；执行对照。

**Non-Goals:** 泛型签名投影门（`generic_call_binding_unproved`，属 ordinary-parameterized/generic-method-signatures 家族）；捕获值适配；特化函数式接口路径；LambdaMetafactory 之外 的 BSM（bridge/字符串_concat 用别的 BSM，不在本 change）。

## Decisions

1. **逐槽配对分两段。** 捕获仍须与站点/impl descriptor 精确一致；SAM 实参从 `shape(bound) == shape(expected)` 的提前拒绝中移出，交给 `adaptation_plan` 对擦除 SAM→instantiated 的检查和 instantiated→impl 的配对判定。返回方向为 impl→instantiated→擦除 SAM。每段只准恒等、已证明的 Object 检查/上溯，或唯一包装类与基本类型配对；`int`→`Integer`→`Object`、`Object`→`Integer`→`int` 必须把两段都证明，arity 不等或任一段不符即拒绝。源码可通过 Java 8 的自动装箱/拆箱呈现，但报告仍记所证明的转换，不把它误记为恒等。备选「模拟 asType 全语义」超出本轮证据，拒绝。
2. **数组构造器要证明合成 impl Code 与参数来源。** 在现有 `ClassMembers` 的同物理类成员表中选定唯一目标，要求精确 MethodHandle 绑定、完整无 handler 的 Code、仅参数 load→`newarray`/`anewarray`→`areturn`，目标数组类型与 instantiated 返回一致，长度参数与 impl descriptor 一致，无额外调用/字段/副作用。站点还须零捕获、一个 SAM 参数，证明长度来自调用而非创建时的捕获。`lambda$` 名称和 synthetic 标志只提供候选，不授予投影权；成员事实缺失或 Code 不完整时拒绝。成功时 Builder 才可增加已证 form 的 `T[]::new` 发射；不能靠字符串替换已发射源码。
3. **从同次 BSM 补精确按需成员候选。** `named_callee_candidates` 除既有 accessor 候选外，只对本规则支持的 LambdaMetafactory 站点，从同次 IR 的 BSM MethodHandle 解析出同类、`REF_invokeStatic`、`lambda$` 的精确 owner/name/descriptor 与 Indy BCI，再交给现有 `CalleeCandidate` 和 `read_prepared_named_callees`/`read_named_callees`；保持既有候选顺序并去重。候选只授权按预算读取，不授权发射；2.2 的完整 Code 证明仍是唯一准入。类级请求复用已持有的 `PreparedClass`，方法级请求沿原按需读取，无额外类扫描或新 reader。普通非数组装箱站点仍只需 BSM 类型事实。
4. **类源码按已证用途原子处理 helper 碰撞。** `T[]::new` 的同次计划携带被选中 helper 的精确原始 owner/name/descriptor、物理方法身份、站点 BCI/CP 与已证数组类型；方法报告、JSON 与独立方法请求始终保留该成员。复用已有 enum-switch 的私有 same-run AST sidecar：方法恢复时保留含普通 helper 调用的 typed `Program` 与候选，类级装配在完整用途与类型证明后克隆该 AST，按站点身份替换为已证 `T[]::new`，复用现有 emitter，不改写已生成的字符串或公开报告。按 helper 分组先 stage 所有相关方法的替代正文与 helper 省略标记，预付全部输出预算后一次性提交；任一步停止则原始 lambda 正文与物理 helper 一起保留。

   用途普查取本次 `PreparedClass` 的方法表、每个已分析方法的同次 IR/常量池/BSM 事实，并要求全部声明的 Code 方法已完整分析；逐个查同物理 helper 的直接 invoke、动态站点 BSM 实现句柄、LDC MethodHandle 及其它可达句柄消费。只有每个用途都精确属于已证可投影的站点、且至少有一个站点，才省略 helper。`jarde-query::xref::scan_candidates` 有完整覆盖与 exact item 的可借鉴验收规则，但它会从 snapshot/scope 再枚举并读取 class，当前没有 `PreparedClass` overload，不用它做本次同类 census。不能以 `lambda$` 名称、合成标志或候选个数当作用途完整证明。

   第一轮类级 Java 8 源目标门限于无需尚未投影泛型 Signature 的直接返回/赋值：零捕获、单参数，擦除 SAM 参数与 instantiated 参数之间无 `CheckCast`，允许 `int` 恒等或 `Integer` 经已证 unbox 到长度 `int`，数组返回恒等或 Object 上溯，并核对实际已拼写的函数式声明。raw `Function` 的 Object→Integer 检查必须拒绝；非泛型 `ArrayMaker.make(Integer)` 可作同名 helper 正例。多站点或其它消费者只有在全部被同一批次完整证明时才投影。
5. **负例与执行对照。** arity 错位、`String`↔`int`、错误包装类、合成方法附加效果/handler/错误数组类型、null 拆箱、负数组长度及边界 Integer 值经 SAM 往返；捕获边界原拒绝不放宽。

## Risks / Trade-offs

- asType 的宽化（如 impl `()long` instantiated `()Long` 之外还有 `()Object` 收纳）→ 在两段转换中逐段复用已有 Object 上溯，本 change 只加基本/包装对，不扩宽化集合。
- 拆箱 NPE 语义在链接期 → 执行对照含 null 接收路径的负控。

## Migration Plan

无迁移；golden/语料计数若变重录。

## Open Questions

无。
