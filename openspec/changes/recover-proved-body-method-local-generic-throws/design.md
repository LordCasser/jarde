## Context

见[提案](proposal.md)与[三方证据](../../evidence/java-syntax-2026-09-24/body-method-local-generic-throws/analysis.md)。reader 已有单一方法 `Signature` parser、方法局部类型参数作用域及异常后缀对物理 `Exceptions` 的逐位置擦除证明。无正文泛型方法可结构化拼出 `<X> ... throws X`；有正文的类级 `throws E` 已引入同轮 `GenericReturnCandidate::EmptyVoid`，用 AST、Code、SSA、效果和 BCI 共同证明空 `void` 正文。现有有正文 `generic_method_declaration` 只接受静态、参数直接返回的局部类型变量，明确不处理本案的实例 `void` 方法和异常变量。JADX 的方法 Signature 读取不消费 `^TX;`，可借鉴其类型参数来源，但不能复制其物理异常输出。

## Goals / Non-Goals

**Goals:** 不新增正文证明或 Signature 机制，在一个已证空效果方法上同时保留 `<X>` 与 `throws X`；使无正文和有正文方法的结构化泛型方法头共享拼写规则，保持类源码预算、来源及局部拒绝一致。

**Non-Goals:** 不改静态参数返回泛型方法的准入；不处理有参数或非 `void` 的新形状、非空正文、方法变量与类变量同名遮蔽、参数化/自定义异常界、继承/覆写及跨类调用求解。

## Decisions

1. **严格复用 `EmptyVoid` 正文候选。** 只有 `Recovered` 方法拿到既有同轮 AST/Code/SSA 空效果候选、物理 descriptor 为 `()V`、零参数且方法 Signature 的返回为 `void` 时才进入本切片。候选本身不读 Signature，不从最终 `return;` 文本倒推；本轮不改动候选生成或普通方法报告。另建 void 方法 proof 或仅看 Code 一条指令都没有价值。
2. **方法局部作用域与异常擦除仍由 reader 先证明。** `Signature` 必须恰好声明一个方法类型变量 `X`，其 class first bound 是已证 JDK `Throwable`、`Exception`、`RuntimeException` 或 `Error`，无接口/额外界；异常后缀恰好 `^TX;`，物理 `Exceptions` 恰好同一 first-bound 擦除类型。首片同时要求类没有 `Signature` 属性且已发布类作用域为空：类头投影失败也会得到空作用域，不能把它误当作非泛型类。源码层仍验证 UTF-8 Java 标识符与完整界拼写。JVM verifier 容许未绑定/矛盾 Signature，故不能以“类可装载”替代此步。
3. **复用已有方法头拼写，而非复制第三套 formatter。** 把无正文泛型方法中“证明 NoBody/层级”与“结构化拼写 type parameters、返回、参数、throws、修饰符”分开，让本轮的空正文分支在自己的门通过后调用相同拼写器；无正文原有输出及有正文静态参数返回路径保持原规则。`project_method_signature` 的选择仅针对精确方法局部空 `void` 形状，候选不足时仍由既有路径拒绝；完整声明构造成功后一次调用 `project_generic`。相比在 `generic_method_declaration` 末尾直接替换 `throws Exception`，共享拼写路径保留异常次序、作用域与预算原子性。
4. **先拒绝无法证明的 Java 绑定。** 限顶层普通非泛型类、Object 直接子类、零接口，方法名不与已知 Object 实例方法相同；沿用本类同名 `Methodref` 拒绝门、注解/flag 完整拼写门及输出预算。即使原 class 可验证，未知父类/接口的覆写异常契约也不能从单方法 Signature 推断；保守物理回退比写出不可重编类更合适。

## Risks / Trade-offs

- **共享无正文 formatter 时改变旧输出** → 抽取时保持原 NoBody 门及输出格式，回归无正文方法自有 `<X> throws X`、类级 `E` 与静态参数返回方法。
- **局部变量作用域或擦除被伪造** → reader proof、唯一方法变量和同名异常后缀一起通过才发布；verifier-valid 反例必须保持物理回退。
- **类与调用绑定仍未通用求解** → 首片拒绝接口、非 Object 继承、Object 同名方法和本类 Methodref；合法复杂形状留给独立证据切片。
