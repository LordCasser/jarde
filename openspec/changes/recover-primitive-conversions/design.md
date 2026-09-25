## Context

`numeric-conversions/root/` 重放三个普通 javac 类，15 个转换 opcode 的180项原始结果、class hash 和 JADX13行差异与初审一致。jarde 每组完整源码仍 javac 失败。`chains/` 的独立 ConversionChain 为601 bytes，9个Code，SHA-256 `8c1bd3a85cbe47829fa5ff85b77621e80531a3e21b78ef60356c35ad043d828a`；73项中 JADX7项因删除中间浮点舍入而错，jarde24引用、javac失败。

已有 ExprKind::Cast 给出目标 Type、单操作数及实际优先级，也可直接陈述转换后的源码静态类型。缺少的是 `0x85..=0x93` 的忠实事实和 Builder 接入，不是 AST 表达能力。现有 invocation_argument 已按 callee descriptor 固定重载，必须让它看到转换后类型。decide_types 对非 boolean 局部仍以 frame 形状决定，不能因此宣称所有 byte/char/short 声明已经保留。

## Goals / Non-Goals

**Goals:** 覆盖明确的15个转换指令，保持每一步数值语义和静态类型，闭合直接返回、现有同型局部、调用实参和嵌套组合。

**Non-Goals:** 不做转换消除、常量计算、boxing/unboxing、范围分析或一般类型求解；不精化 byte/char/short 局部类型，不改 ireturn 按返回 descriptor 隐式窄化的既有边界。后两项另行审计，不以本项15条转换支持冒称任意窄整数方法完整恢复。

## Decisions

1. 在 decode 读取已有有效 opcode，产生一条明确的 primitive-conversion 操作事实；源/目标种类由 opcode 决定。复用现有 primitive Type/数值种类，避免再建可扩张的转换规则注册表。SSA/frame 继续陈述栈类别，i2b/i2c/i2s 的结果仍是 JVM Int，不能伪造新 frame 类型。
2. render_value 在真实指令处获取唯一 stack operand，核对已呈现类型与 opcode 的源类别：Int 接受 byte/short/char/int 的整数提升，拒绝 boolean；Long/Float/Double 要求相应数值类型。未知/矛盾证据保持显式拒绝，不把 boolean 按0/1任意数值化，也不只覆盖 presented 标签来隐藏非法正文。
3. 用现有 Cast 表达目标类型，沿用最终 at、深度与原运算 BCI。i2b/i2c/i2s 的目标分别是 byte/char/short，不能因结果 frame 为Int就省掉转换；后续调用参数规则读取该类型。允许最后的赋值/返回沿既有 Java 隐式 widening 规则呈现，但本项不新增删 cast 优化。
4. 转换链逐层保留，不按首尾类型合并。`(int)(float)16777217` 与原值不同；`(long)(double)9007199254740993L` 同样舍入。long→float 和 float→double 属于 widening conversion，但前者可能损失精度，后者会影响重载，不能混称无意义或可任意删除。float/double→整数的NaN、无穷、越界截断/饱和及随后窄化交给 Java 的相应 Cast 语义，不用宿主数值计算替代。
5. producer reader 及失败追溯接通这一元表达式，操作数求值一次。沿用已验收的共享内联位置/值保存，不能因转换不抛异常便越过独立语句或失效局部。已有纯值区域如要接纳，只加入此事实，不扩张 guard、phi 或调用准入。
6. 浮点常量支持若已落地，转换不得绕过其NaN准入与闭合运行表达式防折叠规则；在现有有界闭合表达式查询中识别这一元节点，并复用已有保存机制。若不满足证明，保持明确拒绝；不得另建浮点 evaluator 或按当前宿主预计算结果。参数、调用及其它运行值的Cast可以独立于浮点字面值支持完成。
7. 操作数与转换/消费者保留真实BCI和成员来源，节点和新增查询沿现有IR/工作/深度/取消预算；默认与完整证据只差元数据，commit/replay不重选转换文本。原型中的其它无类型局部失败仍保留必要生产者引用，不修改无关返回/声明规则以让测试通过。
8. 不引入依赖。读入事实、Cast、Type与统一emitter足够，外部库不能替代当前 SSA 和调用 descriptor 约束，增加维护及许可成本无收益。受控自写输入的 javac/java 只用于验收，不改变产品 parse、dialect、runtime、verification 与 compilation 平面。

依据：[JVMS conversion instructions](https://docs.oracle.com/javase/specs/jvms/se23/html/jvms-6.html#jvms-6.5.i2b)、[JLS 5.1.3](https://docs.oracle.com/javase/specs/jls/se23/html/jls-5.html#jls-5.1.3)。

## Risks / Trade-offs

- 用首尾同型消除舍入 → 独立转换链73项必须真实恢复，固定2²⁴及2⁵³附近值。
- 丢弃窄整数类型或 widening cast 改重载 → 180项完整类对照含 byte/short/char/long/float/double overload，不以JADX作为唯一oracle。
- 直接cast无效boolean或未知类型 → 合法descriptor变体经JVM验证后固定拒绝边界，不能靠非法class通过测试。
- 前置调用被独立呈现又嵌入Cast → 左右调用和抛错对照、生产者来源及共享延期值回归检查一次求值。
- 窄局部/ireturn问题被混入 → 当前类型与返回位置限制另列；本项不新建窄范围求解，原字节码明确转换仍必须恢复。
