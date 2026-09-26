## Context

字节码事实（`Patrol`，javac --release 8）：站点 BCI 2 `invokedynamic get:()Ljava/util/function/Supplier;`，BSM 参数含 instantiated MethodType `()Ljava/lang/Integer;`，impl MethodHandle 直指 `Patrol.supply()I` —— javac 不生成适配器，装箱由 metafactory 链接期 `asType` 完成。BCI 7 同形（`apply:()Ljava/util/function/Function;`，instantiated `(Ljava/lang/String;)Ljava/lang/Integer;`，impl `staticRef`）。`int[]::new` 站点的 impl 是真实合成方法 `lambda$arrayCtor$0(I)[I`，instantiated `(Ljava/lang/Integer;)[I`。

`lambda.rs` 现有适配证明只接受恒等、Object 检查、Object 上溯；上面的逐槽基本/包装配对全部拒绝，携带站点的方法整方法引用。

## Goals / Non-Goals

**Goals:** 逐槽配对判定扩展（包装/基本互转 + 数组构造器引用）；箭头呈现不变；负例保持；执行对照。

**Non-Goals:** 泛型签名投影门（`generic_call_binding_unproved`，属 ordinary-parameterized/generic-method-signatures 家族）；捕获值适配；特化函数式接口路径；LambdaMetafactory 之外 的 BSM（bridge/字符串_concat 用别的 BSM，不在本 change）。

## Decisions

1. **逐槽配对表。** 对 impl 与 instantiated 的 descriptor 逐槽（参数 + 返回）判定：同 descriptor 恒等；或一对 `(基本 B, 包装 W(B))` 双向（asType 语义：装箱 `B.valueOf`、拆箱 `B.value()`，链接期完成，源码不可见）；或既有 Object 检查/上溯。arity 不等或任一槽不配对即拒绝。备选「模拟 asType 全语义（含接口默认方法桥）」超出本轮证据，拒绝。
2. **数组构造器引用 = 合成 impl 的特例。** impl 是本类合成分配方法（`newarray`/`anewarray` 于参数）且逐槽对应时接受；箭头拼写沿用 `lambda.rs` 既有合成方法呈现规则（`T[]::new` 的目标拼写从 instantiated 返回类型读）。
3. **证明读 BSM 常量池事实。** instantiated MethodType 与 impl MethodHandle 都是 BSM 参数事实，不新增数据流；impl 读不到/非本类/非本 artifact 物理输入范围内时保持既有拒绝路径。
4. **负例与执行对照。** arity 错位、`String`↔`int`、null 接收者、边界 Integer 值经 SAM 往返。

## Risks / Trade-offs

- asType 的宽化（如 impl `()long` instantiated `()Long` 之外还有 `()Object` 收纳）→ 已由既有 Object 上溯类覆盖，本 change 只加基本/包装对，不扩宽化集合。
- 拆箱 NPE 语义在链接期 → 执行对照含 null 接收路径的负控。

## Migration Plan

无迁移；golden/语料计数若变重录。

## Open Questions

无。
