## Context

见 `proposal.md` 和 `../../evidence/java-syntax-2026-09-22/immediate-functional-receivers/`。三份 Java 8 完整类经冻结 CLI `8b86c729…756e9` 两次独立重放，`summary.json` 字节相同。即时数组构造器与 `Math::abs` 方法引用均零 bytecode 引用，却各因裸露的 lambda/方法引用接后缀调用而编译失败；同型对象先赋给有类型局部的对照，原/JADX/Jarde 均编译执行且结果相同。仅在手写机制对照里，把现有 Jarde 接收者加上函数式接口 cast 后，两份即时调用才编译并与原类执行一致；这不是生产恢复结果。

`build.rs::call_expr` 把实例调用的真实 SSA receiver 交给 `render_value`，随后直接放进 `ExprKind::Call`。LambdaMetafactory 路径已从工厂 descriptor 把 `Lambda` 或 `MethodReference` 表达式的 `presented` 设为函数式引用类型；但 Java 8 的这种表达式是需要目标类型的 poly expression，`emit.rs` 目前只为它作为调用接收者补括号。`invocation_argument` 在函数式表达式作为实参时已经复用 `cast_argument` 补目标类型，并将 cast 的来源接到真实消费 BCI。

## Goals / Non-Goals

**Goals:** 在即时调用 receiver 位置补足由工厂自身证明的函数式目标类型；仅当它与当前调用的 pool owner 同型且可拼写时恢复，保持工厂与调用一次求值及来源。

**Non-Goals:** 不恢复泛型 Signature、不推断接口继承、不重新选择重载、不改 lambda 体内参数适配或创建阶段捕获、不解决合成 `lambda$` helper 命名冲突，也不把任意表达式强制转换成函数式接口。

## Decisions

1. **在现有调用构造处处理目标类型。** 只有 `render_value` 交来的直接 `Lambda`/`MethodReference` 接收者需要这项补足；`CallTarget` 的 pool owner 和当前表达式的 `presented` 共同证明同型，字段/局部/已显式 cast 的接收者照旧。具体检查先拼写 owner，再比对完整引用名；没有类型事实或 owner 不同则拒绝，而不是凭 JVM 引用槽形状猜接口继承。单靠 emitter 为所有 lambda 加括号已经证实不能生成 Java 目标类型。
2. **复用 `Cast`，不增加 AST 或局部。** 用现有 `cast_argument` 将函数式目标包在 `Lambda`/`MethodReference` 外，再由统一 emitter 写为 `((IntFunction) lambda).apply(...)` 或 `((IntUnaryOperator) Math::abs).applyAsInt(...)`。cast 只给源码 poly expression 提供目标类型，不对应新的 JVM `checkcast`；原工厂 BCI 保持 primary，真实调用 BCI 作为 derived。手写两份控制源码的 Java 8 重编/验证执行与原始类逐行一致，说明此受限形状不需要新求值机制。把 lambda 存入新临时局部虽也可编译，却会重复现有声明/保存机制并扩大作用域处理，因此不采用。
3. **继续使用既有预算和拒绝路径。** 新判断和 cast 节点在 `call_expr` 的一次构造中发生，不重读 class、不重新解码或恢复。沿现有 IR/工作/深度/取消与 emitter 输出计费，无法证明时让该调用的原有 fallback 追溯同时保留工厂和消费者；essential/all 只影响证据物化，不改变 cast 决策。`preserve-lambda-descriptor-adaptation` 将来处理函数体内适配，与本项外层调用接收者互不代替。
4. **不引入库。** reader 的 invokedynamic、pool owner、SSA receiver 和 Java AST 都已有；外部 Java 解析器无法补出缺失的目标类型证明，增加维护与许可面也不提供收益。此处属于源码恢复，不改变 class 解析、Java 8 dialect 校验、运行时选择或目标代码验证。

## Risks / Trade-offs

- **接收者工厂与调用 owner 不是同一接口** → 暂时明确拒绝；若要接受接口继承，须独立提供层级证据，不能用 raw 形状推断。
- **源码 cast 被误当运行时检查** → 来源标为真实工厂及调用，不虚构 `checkcast` BCI；重编完整类并实际执行三种输入验证异常。
- **为简单局部调用重复 cast 或重新求值** → 只处理直接函数式表达式，冻结的局部对照保持文本/行为稳定。
- **一般显式 lambda 的合成 helper 名冲突** → 独立探针已观察到这项相邻债务；本项的两份直接正例与局部对照均不借删 helper 规避它，不扩大范围。
