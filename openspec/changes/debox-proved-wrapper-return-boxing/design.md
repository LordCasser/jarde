## Context

见 [proposal.md](proposal.md) 和 [EM-25 回放报告](../../evidence/java-syntax-2026-09-27/em25-boxing/report.md)。普通方法调用根据符号化 `CallTarget` 输出，目前没有通用的源码拆箱流程。Lambda 适配有自己的描述符证明，不能复用为普通返回值的证明。

## Goals / Non-Goals

**目标：** 当精确的标准包装类装箱调用是完整返回值，且 JLS 的装箱身份要求与该包装类 `valueOf` 的缓存保证重合时，直接使用已有的 primitive 表达式。保留 primitive 字面量类型、物理调用/返回来源；证明不完整时保守拒绝。

**非目标：** 简化拆箱调用、`Byte`/`Short`/`Long` 的 `valueOf`、任意非字面量参数、局部变量流、方法调用实参、条件合流、重载解析 cast、`instanceof` 收窄及非标准包装类 owner。

## Decisions

1. **只匹配直接返回，不匹配所有调用表达式。** 使用完整的方法 SSA 和符号化调用 owner/name/descriptor，定位一个静态 `valueOf` 调用，并证明其结果就是返回操作数。还要求参数是 primitive 字面量、包装类受支持、方法返回类型恰为该包装类或 `java.lang.Object`，并证明此调用的结果没有其他值消费者。现有表达式和 return 机制可以呈现 primitive 值；无需新增 AST 节点或通用转换框架。

2. **只接受规范与类库共同保证的 identity。** Java 8 的返回上下文会把 primitive 装箱；[JLS §5.1.7](https://docs.oracle.com/javase/specs/jls/se8/html/jls-5.html#jls-5.1.7)只保证 `boolean` 字面量、`-128..127` 的 `int` 字面量和 ASCII `char` 字面量重复装箱的身份。[`Integer.valueOf`](https://docs.oracle.com/javase/8/docs/api/java/lang/Integer.html#valueOf-int-) 与 [`Character.valueOf`](https://docs.oracle.com/javase/8/docs/api/java/lang/Character.html#valueOf-char-) 对对应范围有缓存保证，`Boolean.valueOf` 返回规范常量。只在这些交集内改写。`Byte`、`Short`、`Long` 即使固定 `javac` 编出相同调用，其自动装箱身份不由 JLS 保证，保留显式调用；不从使用位置推断重载目标。

3. **符号证据保持局部，失败时关闭优化。** 只识别 `Boolean`、`Integer`、`Character` 的精确标准 owner 和描述符，并检查字面量落在上述范围。不解析无关 classpath 条目，也不按字面量数值推断包装类。返回类型、owner、描述符、消费者、字面量类型或范围有歧义时，保留普通 `valueOf` 调用（若该表达式已被现有规则拒绝，则沿用现有拒绝行为）。

4. **减少输出操作时保留物理来源。** 返回的 primitive 表达式保留其字面量来源；return 语句保留 return BCI；简化掉的 `valueOf` BCI 仍作为派生来源记录。这样既保留两条物理指令，又不会输出一个随后被 Java 装箱重新构造的合成调用。

5. **不推广拆箱结果。** `TestDeboxing2` 的活动断言不要求删除 `.longValue()`，固定 JADX 输出也明确保留此调用；回放确认 Jarde 同样保留它，包括 null 到默认值的行为。共享的 lambda 装箱/拆箱规则遵循不同的方法句柄适配契约，保持不变。

## Risks / Trade-offs

- [删除调用会改变包装类或对象 identity] → 限定为 JLS 和标准类库共同保证的三种字面量范围，且要求精确 owner/descriptor 与返回上下文；完整源码回放再验证类型、identity 和值。
- [固定 JADX 的其它三种简写未追平] → `byte`、`short`、`long` 自动装箱没有相同的跨编译器引用身份保证；明确保留其 `valueOf`，待有更强目标编译器契约才另议。
- [调用在返回之外被静默删除] → 证明存在唯一的直接返回消费者；存在其它消费者或嵌套/实参使用的调用仍按普通调用处理。
- [把风格简化误认为语义修复] → 验收报告明确说明基线已能正确编译和运行；此变更只覆盖固定源码形式断言。
