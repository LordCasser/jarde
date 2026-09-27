## Context

参见 [CF-05 对照](../../evidence/java-syntax-2026-09-27/cf05-numeric-condition/report.md)。`ConversionBasic` 已通过；`ConversionCases` 的 `castByte` 也已证明。`build.rs::byte_conditional_argument` 会在 `B` 形参下逐臂接受 byte 表达式或 byte 范围内常量；普通 `invocation_argument` 仅对单个直接常量提供窄化。`castShort`/`shortConstant` 在调用参数门因整体条件表达式呈 int 而拒绝，`byteField`/`shortField` 更早在条件值构建时因 byte/short 字段与 int 常量混合没有 Java 条件类型而拒绝。两门均在现有 SSA/CFG 条件值证书之后，不需重做控制流识别。

## Goals / Non-Goals

**Goals:** 让现有条件值在准确的 B/S 调用目标下，从两个叶子的本身类型或表示范围证明一个窄条件表达式；让字段+常量的 Java 类型在发表条件节点前闭合；保留所选重载和物理来源，失败原子拒绝。

**Non-Goals:** 任何 int 变量到 byte/short 的隐式窄化、DEX boolean 寄存器的泛化数值解释、所有 Java 数值条件规则、`final` 字段初始化、超出固定 CF-05 四处差距的转换语法。

## Decisions

1. **复用现有值闭包，分别证明叶子与目标。** 在条件值证书已确认测试边、phi、唯一使用和正常/异常边后，从字段 descriptor 或直读 int 常量取得每臂类型/值；被调方法 descriptor 只选择所需的 `B`/`S` 目标，不能代替叶子证明。对于常量要求 `byte`/`short` 表示范围；字段要求准确声明类型。若消费者不是唯一、调用签名不符、值来自计算/别名或类型来源不明，保持拒绝。
2. **在发表 AST 前解决混合臂类型。** `Expr::Conditional` 目前只接受相同类型或 null/reference；`byteField`/`shortField` 在此即拒绝。优先对已证字段+范围内常量的窄数值条件，在构建现有 `ExprKind::Conditional` 前为常量臂补源级 cast/字面量类型，或在现有条件类型规则中加入同等有界判断；必须保证 AST 的 `presented` 类型与实际 Java 文本一致。避免新增第二套条件 AST 或全局“int 就可窄化”规则。
3. **在调用点保持准确重载。** 将 `byte_conditional_argument` 的逐臂判定按同一模式覆盖 `short`，或抽成对 B/S 共用的最小 helper；只在现有 `invocation_argument` 路径和准确目标签名下使用。Java 语法上必要的 cast 应落在每个叶子，不能把可能有副作用/非恒定值的整个条件式强转。`ConversionBasic`、`castByte` 及正负例同时检验。
4. **来源、预算与弱证据边界。** 每个新增 cast 的 origin 包含实际常量 BCI 与调用 BCI 的 derived 关系；原条件测试及字段读取顺序不变。沿用现有预算/停止传播，不能在拒绝后发布半个方法。三项 Smali 测试只验证 JADX 的文本路径，不当作完整源码通过事实。

## Risks / Trade-offs

- [源级 cast 改变重载或值] → 在真实 `(B)I`/`(S)I` 目标和字段/常量范围证书下逐臂生成，三方运行覆盖 true/false 两路。
- [条件类型计算过早或过宽] → 仅对两叶已证且消费者唯一的窄形状启用，超范围/错 descriptor 负例必须仍拒绝。
- [额外预算计费改变固定停止点] → 新遍历按现有限额计费，测试取消与停止后空产物，不调整无关预算阈值。
