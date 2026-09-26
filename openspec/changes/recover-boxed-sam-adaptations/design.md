## Context

字节码事实（`Patrol`，javac --release 8）：站点 BCI 2 `invokedynamic get:()Ljava/util/function/Supplier;`，BSM 参数含 instantiated MethodType `()Ljava/lang/Integer;`，impl MethodHandle 直指 `Patrol.supply()I` —— javac 不生成适配器，装箱由 metafactory 链接期 `asType` 完成。BCI 7 同形（`apply:()Ljava/util/function/Function;`，instantiated `(Ljava/lang/String;)Ljava/lang/Integer;`，impl `staticRef`）。`int[]::new` 站点的 impl 是真实合成方法 `lambda$arrayCtor$0(I)[I`，instantiated `(Ljava/lang/Integer;)[I`。

`lambda.rs` 现有适配证明只接受恒等、Object 检查、Object 上溯；上面的逐槽基本/包装配对全部拒绝，携带站点的方法整方法引用。具体有两道门：`plan` 在 `adaptation_plan` 前对捕获和全部 SAM 实参调用 `shape(bound) == shape(expected)`，使 `Integer`→`int` 连适配器都进不去；`adaptation_plan` 的返回分支直接比较 impl 与**擦除 SAM** 返回，尚不能表达 `int`→instantiated `Integer`→擦除 `Object` 两段。`is_generated_body` 目前只认 `lambda$` 名称，`build.rs` 对该 form 仍会发一个合成方法调用，并无已证的 `T[]::new` 发射规则。JADX 1.5.6 在冻结样例把 `int[]::new` 展开成 `x$0 -> new int[x$0]`，不是原样还原；此处的 `T[]::new` 是本变更希望做得更准确的语法恢复。

## Goals / Non-Goals

**Goals:** 逐槽配对判定扩展（包装/基本互转 + 数组构造器引用）；箭头呈现不变；负例保持；执行对照。

**Non-Goals:** 泛型签名投影门（`generic_call_binding_unproved`，属 ordinary-parameterized/generic-method-signatures 家族）；捕获值适配；特化函数式接口路径；LambdaMetafactory 之外 的 BSM（bridge/字符串_concat 用别的 BSM，不在本 change）。

## Decisions

1. **逐槽配对分两段。** 捕获仍须与站点/impl descriptor 精确一致；SAM 实参从 `shape(bound) == shape(expected)` 的提前拒绝中移出，交给 `adaptation_plan` 对擦除 SAM→instantiated 的检查和 instantiated→impl 的配对判定。返回方向为 impl→instantiated→擦除 SAM。每段只准恒等、已证明的 Object 检查/上溯，或唯一包装类与基本类型配对；`int`→`Integer`→`Object`、`Object`→`Integer`→`int` 必须把两段都证明，arity 不等或任一段不符即拒绝。源码可通过 Java 8 的自动装箱/拆箱呈现，但报告仍记所证明的转换，不把它误记为恒等。备选「模拟 asType 全语义」超出本轮证据，拒绝。
2. **数组构造器要证明合成 impl Code。** 在现有 `ClassMembers` 的同物理类成员表中选定唯一目标，要求精确 MethodHandle 绑定、完整无 handler 的 Code、仅参数 load→`newarray`/`anewarray`→`areturn`，目标数组类型与 instantiated 返回一致，长度参数与 impl descriptor 一致，无额外调用/字段/副作用。`lambda$` 名称和 synthetic 标志只提供候选，不授予投影权；成员事实缺失或 Code 不完整时拒绝。成功时在 Builder 增加这个已证 form 的 `T[]::new` 发射，保留物理合成方法报告并检查类源码中的同名方法与新编译器合成方法不冲突；不能靠字符串替换已发射源码。
3. **证明读 BSM 常量池事实。** instantiated MethodType 与 impl MethodHandle 都是 BSM 参数事实，不新增数据流；impl 读不到/非本类/非本 artifact 物理输入范围内时保持既有拒绝路径。方法级请求未提供 `ClassMembers` 时不推测数组 impl 语义；普通非数组装箱站点仍只需 BSM 的类型事实。
4. **负例与执行对照。** arity 错位、`String`↔`int`、错误包装类、合成方法附加效果/handler/错误数组类型、null 拆箱、负数组长度及边界 Integer 值经 SAM 往返；捕获边界原拒绝不放宽。

## Risks / Trade-offs

- asType 的宽化（如 impl `()long` instantiated `()Long` 之外还有 `()Object` 收纳）→ 在两段转换中逐段复用已有 Object 上溯，本 change 只加基本/包装对，不扩宽化集合。
- 拆箱 NPE 语义在链接期 → 执行对照含 null 接收路径的负控。

## Migration Plan

无迁移；golden/语料计数若变重录。

## Open Questions

无。
