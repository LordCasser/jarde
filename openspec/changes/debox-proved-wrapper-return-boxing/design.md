## Context

见 [proposal.md](proposal.md) 和 [EM-25 回放报告](../../evidence/java-syntax-2026-09-27/em25-boxing/report.md)。普通方法调用根据符号化 `CallTarget` 输出，目前没有通用的源码拆箱流程。Lambda 适配有自己的描述符证明，不能复用为普通返回值的证明。

## Goals / Non-Goals

**目标：** 当精确的标准包装类装箱调用是完整返回值，且声明的返回类型会执行相同包装转换时，直接使用已有的 primitive 表达式。保留 primitive 字面量类型、物理调用/返回来源；证明不完整时保守拒绝。

**非目标：** 简化拆箱调用、任意 `valueOf` 参数、局部变量流、方法调用实参、条件合流、重载解析 cast、`instanceof` 收窄、非标准包装类 owner，以及固定 JADX 拆箱测试未覆盖的包装类。

## Decisions

1. **只匹配直接返回，不匹配所有调用表达式。** 使用完整的方法 SSA 和符号化调用 owner/name/descriptor，定位一个静态 `valueOf` 调用，并证明其结果就是返回操作数。还要求参数是 primitive 字面量、包装类受支持、方法返回类型恰为该包装类或 `java.lang.Object`，并证明此调用的结果没有其他值消费者。现有表达式和 return 机制可以呈现 primitive 值；无需新增 AST 节点或通用转换框架。

2. **由 Java 返回上下文执行相同装箱转换。** 在对应包装类或 `Object` 返回上下文中，Java 8 重编会对 primitive 执行装箱。保留显式的窄 primitive 类型（`(byte)` 和 `(short)`），避免其意外变为 `Integer`。不删除其它 cast，也不从使用位置推断重载目标。

3. **符号证据保持局部，失败时关闭优化。** 只识别 `Boolean`、`Byte`、`Short`、`Character`、`Integer`、`Long` 的精确标准 owner 和描述符。不解析无关 classpath 条目，也不按字面量数值推断包装类。返回类型、owner、描述符、消费者或字面量类型有歧义时，保留普通 `valueOf` 调用（若该表达式已被现有规则拒绝，则沿用现有拒绝行为）。

4. **减少输出操作时保留物理来源。** 返回的 primitive 表达式保留其字面量来源；return 语句保留 return BCI；简化掉的 `valueOf` BCI 仍作为派生来源记录。这样既保留两条物理指令，又不会输出一个随后被 Java 装箱重新构造的合成调用。

5. **不推广拆箱结果。** `TestDeboxing2` 的活动断言不要求删除 `.longValue()`，固定 JADX 输出也明确保留此调用；回放确认 Jarde 同样保留它，包括 null 到默认值的行为。共享的 lambda 装箱/拆箱规则遵循不同的方法句柄适配契约，保持不变。

## Risks / Trade-offs

- [删除调用会改变包装类或对象 identity] → 限定为精确的标准包装类描述符和能由 Java 装箱恢复相同包装类的返回上下文；通过完整源码回放验证包装类、缓存 identity 示例、值和 null 行为。
- [必要的 primitive 窄化消失] → 将原字面量的 primitive 类型带入表达式，并要求保留固定的 `byte`/`short` cast；对错误返回类型和描述符增加负例。
- [调用在返回之外被静默删除] → 证明存在唯一的直接返回消费者；存在其它消费者或嵌套/实参使用的调用仍按普通调用处理。
- [把风格简化误认为语义修复] → 验收报告明确说明基线已能正确编译和运行；此变更只覆盖固定源码形式断言。
