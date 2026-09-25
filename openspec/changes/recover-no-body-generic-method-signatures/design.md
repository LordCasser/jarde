## Context

见[提案](proposal.md)与[三方证据](../../evidence/java-syntax-2026-09-24/method-local-generic-throws/analysis.md)。reader 的 `MethodSignatureErasureProof` 已给出方法自有变量第一界擦除，且可与类作用域、物理参数/返回和非空 `throws` 后缀逐位置核对。类源码的 `project_method_signature` 当前遇到 `NoBody` 且有方法变量就提前跳过；`generic_method_declaration` 仅适用于已恢复正文的静态直接参数返回。无正文方法不需要正文值流证明，但仍受源级覆写和本类调用绑定约束。

## Goals / Non-Goals

**Goals:** 让无正文抽象方法的一份已证 `Signature` 原子决定方法类型参数、参数、返回和异常位置；先交付顶层类或接口且无继承方法契约的子集，恢复泛型覆写与强类型调用。

**Non-Goals:** 不改变有正文泛型方法的 AST/SSA 证明；不把接口继承、泛型父类或外部依赖闭包的覆写推理塞进首片；不根据调用方反推签名，也不复制 JADX 对异常后缀的遗漏。

## Decisions

1. **共用 reader proof，移除无正文方法的提前跳过。** 无论是否含方法自有变量，都先完整解析并调用同一 `prove_method_signature_erasure_with_class_scope`；其结果只提供语法、作用域和擦除，不代表类源码自动可写。JADX 仅可借鉴类型参数在参数/返回前的投影顺序；其 `SignatureProcessor` 忽略泛型异常后缀，此处按 reader 完整树处理。`noak` 不提供方法 Signature parser，当前 reader 已是共享实现，新增依赖无收益。
2. **首片排除未知覆写契约。** 只接受顶层抽象类或不继承其它接口的顶层接口中 `NoBodyKind::Abstract` 的普通方法，物理父类精确为 `java/lang/Object`、接口列表为空；其方法名若可能覆盖 Object 的已知实例方法，也保守拒绝。同类 `Methodref` 仍走现有门，避免其它正文调用因方法变为泛型而改变类型检查或重载绑定。以后可用实际父类解析证明覆写兼容，不能凭物理 descriptor 相同假定源码关系相同。
3. **拼写一份完整方法候选。** 把 reader 已证的类/方法变量作用域合并供现有结构化类型拼写器使用，方法自有类型参数界按原 Signature 顺序输出；参数名沿 descriptor slot `argN`，返回与 throws 也从结构树逐位置拼写。类变量与方法变量不得同名由 reader 保证。异常变量仅当其已证第一界 descriptor 精确为 `Throwable`、`Exception`、`RuntimeException`、`Error` 之一才投影；无泛型异常后缀时继续用物理 `Exceptions`。采用与当前类头一致的 Java 标识符、非嵌套可拼写界、type-use 注解拒绝和预算门；完整候选构造与费用通过后才替换声明。
4. **局部拒绝与证据选择保持现状。** 来源继续指向同一方法 `Signature` 与物理成员；不修改独立方法报告、reader/query 结果或 class body。无正文方法若因继承、注解、上界、擦除或输出预算失败，不发布部分 `<X>`/`throws X`。essential/all 只影响细节证据，方法 Java 正文相同。

## Risks / Trade-offs

- **真实源码的抽象方法可能在继承其它类型的类或接口中** → 首片的父类/接口门会损失覆盖率；后续需用可用的父类型来源和实际覆写关系扩展，不以猜测放行。
- **类变量与方法变量拼写时作用域混淆** → reader 已拒绝重名，源码使用同一已证作用域而非按文本查找；回归覆盖两级变量及未绑定变量。
- **错误 Signature 在 JVM 可验证 class 上让 Java 源无法通过** → 逐位置擦除、异常根和继承门共同拒绝；负例不以 JVM 验证成功代替源码正确性。
