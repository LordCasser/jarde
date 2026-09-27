## Context

固定 `Outer<T>` 与唯一 `Inner` 的 classfile 有准确双方 `InnerClasses` 关系；家族报告已到 `prepared`，capture 已 `proved`。根方法 Signature 是 `()Ldt19/Outer<TT;>.Inner;`，child 方法 Signature 是 `(TT;)TT;`，两者擦除分别匹配物理 descriptor。`src/facade.rs` 现有 `family call source path requires non-generic root and member headers` 门阻止源调用投影；`src/class_source.rs` 在没有选定成员路径时拒绝嵌套 Signature，child 独立恢复时没有外层 `T` scope。现有 `assemble-proved-member-class-family` 的物理身份、捕获与 writer 是基础，不重新建 class graph。

## Goals / Non-Goals

**Goals:** 唯一根与唯一直接、非静态、命名 child；根具有一个无额外界的 `<T extends Object>`，child 无自有类型参数，方法 `T id(T)` 是同一个参数的无效果直接返回，根 `make()` 经已证构造调用产生该 child。外部 `Outer<String>.Inner` consumer 完整编译验证运行。

**Non-Goals:** 多子类、多层内嵌、成员自有类型参数、接口/bridge、泛型本地推断、跨类调用传播、复杂正文、原始类型混用或从二进制 `$` 名猜作用域。

## Decisions

1. **先证明家族，再开放词法 scope。** 只有双向 `InnerClasses`、选定 child 物理身份、唯一捕获参数/字段及完整扫描均成立，才读取根唯一完整 class `Signature`。外层 `T` 的名称、界与擦除以 reader 现有 Signature 解析与物理字段/方法 descriptor 校验；child 不凭自身 `(TT;)TT;` 创建自由类型变量。错误、重复、冲突或缺失根签名均拒绝泛型家族，独立 child 请求仍保留原物理表达。
2. **在现有源路径上限定泛型成员。** 复用 `ProvedMemberInnerTarget.source_type_path` 和成员构造证明，为根方法返回类型选择 `Outer<T>.Inner` 两段路径；`generic_diamond` 不因泛型外层而自动打开。只有该根/child 精确家族的调用与构造 identity 闭合后才允许越过现有非泛型门，不能直接删除 guard。body AST/SSA 参数直接返回必须与 child `(TT;)TT;` 及外层 T 同轮一致。
3. **一次性写家族源码与双 owner 来源。** 根 writer 只输出一个 package 和 `Outer<T>`，child writer 在里面写 `Inner`/`T id(T)`，隐藏捕获构件须由现有证书驱动。完整声明、构造和方法 body 的位置在家族文本提交前统一计费，child 物理报告及 root/child source map 身份继续可查；任何阶段停止或拒绝保持保守两份物理文本，不出现只改了一半的泛型头。
4. **以完整源码和反例验收。** 重放固定原/JADX/Jarde Java 8 全源码与同一个 API consumer，在 `-Xverify:all` 下逐字比输出；再构造 verifier-valid 错外层变量、错 InnerClasses、第二成员/额外捕获读写、错误构造目标与低预算/取消负例。不能以文字 `Outer<T>.Inner` 出现代替重编和物理来源核对。

## Risks / Trade-offs

- 类型变量 `T` 对 child 是词法继承而非 child class Signature 自声明。只有家族身份通过后才提供作用域，避免把其它同名物理类的 T 错绑。
- 泛型返回类型与构造语法必须同时合法。已存在的 class-source writer/方法 Signature 接缝应复用；若无法保持原子输出，先缩小实现切片并修订本规格，不做源码字符串替换。
- 本片外部 consumer 使用泛型成员类型，但不要求 Jarde 从无 debug 的局部 SSA 重新推断 `String`；那是 DT-19 的下一切片。
