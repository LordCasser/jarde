## Context

见[提案](proposal.md)和[三方证据](../../evidence/java-syntax-2026-09-24/body-generic-throws/analysis.md)。reader 已有唯一方法 `Signature` parser、类作用域解析和异常后缀对物理 `Exceptions` 的逐位置擦除证明；无正文 `throws E` 已在结构化声明路径保留。`jarde-java` 当前把有正文的参数返回值证明作为 class-source sidecar 交给 facade，`ordinary_parameterized_declaration` 却明确拒绝有正文的异常变量。JADX 1.5.6 的 `SignatureProcessor.parseMethodSignature` 不消费 `^TE;`，本轮只把它当作差异定位，不照搬其物理异常回退。

## Goals / Non-Goals

**Goals:** 以同轮完整空正文事实支持一个无参 `void` 方法的类级 `throws E`；将正文、签名、类头与调用绑定在一次声明投影前检查，保持预算、取消和来源记录一致。

**Non-Goals:** 不扩大到会读写局部或抛异常的正文、有参数或非 `void` 方法、方法自有 `<X> throws X`、自定义异常界、继承/覆写求解、跨类调用类型求解，亦不引入新的 Signature parser 或独立恢复 pass。

## Decisions

1. **复用现有 class-source 候选接缝，只增加严格的空 `void` 证明形状。** 在现有同轮方法候选中表示“无返回值且无效果”，只有 Program 完整、非 ragged，AST 精确一条 `Return(None)`，Code 无异常表且仅有 `return` 指令，SSA 单块、无 phi、该指令无读写和抛出效果时产生候选。只对含方法 `Signature` 的受控 class-source 运行收集并计预算；普通方法恢复不改变。相比仅看最终文本 `return;`，这还能排除被省略的副作用；相比另加一整套 sidecar/恢复 pass，扩展当前候选足够表达这一证明。
2. **reader 负责签名与擦除，源码层只证明作用域与合法 Java 拼写。** `Signature` 的 `^TE;` 必须同物理 `Exceptions` 逐位置擦除相同；`E` 必须来自已发布类头，其第一界限于已证 JDK `Throwable`、`Exception`、`RuntimeException` 或 `Error`。物理 `Exceptions` 不反推出 `E`。方法 `Signature` 缺失、无异常后缀、未绑定变量或异常位置不匹配均保留现有局部拒绝/物理回退。外部库无需引入，现有 parser 已覆盖字节码语法；JADX 的缺口是源恢复而非 reader 能力。
3. **先封住未证明的 Java 绑定。** 首片只接受顶层普通类，物理父类是 `Object`、零接口，方法无参数、`void`、名称不与已知 `Object` 实例方法相同，且不存在本类同名 `Methodref`；保留现有注解与成员 flag 的拼写门。这样无需假定未知父类/接口的 throws 覆写契约或本类调用方的编译绑定。保守拒绝虽少覆盖合法类，但比因签名局部正确而输出不可重编类更可审计；后续扩围须独立证明具体绑定。
4. **完整声明一次构造后原子发布。** 在 `ordinary_parameterized_declaration` 的结构化异常列表路径中，仅当同轮空 `void` 候选和上述门都成立才允许有正文 `SignatureType::TypeVariable`；其余参数、返回与异常源仍由原有规则生成。源码标记明确写空正文 proof，而非“参数返回 proof”。失败沿现有 `refuse_generic`，预算/取消沿现有操作停止传播，不对最终文本做 `throws Exception` 字符串替换。

## Risks / Trade-offs

- **被隐藏的正文效果** → 同时核对 AST、Code、SSA 与效果表，并以真实抛出、处理器和控制流负例检验。
- **Signature 可解析却非源码合法** → reader 擦除之外继续检查异常根界、变量作用域和类/方法的源码绑定；verifier-valid 伪造签名不得直接发布。
- **过窄门损失真实覆盖** → 将含参数、非空正文、自定义异常界及继承场景记录为后续独立切片，不在本轮增加泛用异常推断机制。
