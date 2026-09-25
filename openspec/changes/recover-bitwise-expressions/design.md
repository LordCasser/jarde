## Context

实证位于`../../evidence/java-syntax-2026-09-22/bitwise/`：基础211项与type-flow的56项完整类对照均为原class=JADX，jarde均javac失败；mixed-types的8项证明JVM的Int形状不能决定Java boolean合法性。

当前Operation缺六个位运算事实；BinaryOp只有算术/比较，binary_type把所有非算术都认作boolean。boolean_proof只读descriptor等叶子，decide_types以既有局部工作队列传播复制关系；语句构造不能事后改变这个决定。统一emitter已有绑定强度和左右操作数分组。Atlas本轮scoped查询仍未交付结构结果，以上判断以实际源文件和javap复核。

## Goals / Non-Goals

**Goals:** 在现有事实→SSA值→Binary→emitter路径补齐位运算，并让已有boolean与局部类型判断在该表达式上闭合。

**Non-Goals:** 不建立一般类型求解器，不跟随任意phi/未知store猜boolean，不添加全图pass、跨方法解析、优化器或新缓存；不处理移位、0/1分支汇合及源码风格化简。

## Decisions

1. 为iand/land、ior/lor、ixor/lxor增加忠实操作事实，操作符可用小枚举区分And/Or/Xor；结果的Int/Long形状继续读取现有事实，不复制frame机制。AST直接扩展BinaryOp三个成员，不增加第二种Binary节点。`~a`编译成与-1异或时保留异或即可，不需专用识别机制。
2. 显式区分Binary的算术、关系及位运算类型。位运算只接受两侧boolean，或两侧可作整数提升的类型；不得因所有非算术历史上都是比较就默认boolean，也不得把float/double送进整数位运算。保留byte/short/char到int与long运算的实际提升宽度，不靠覆盖presented标签把不合法正文说成合法。
3. 扩展现有共享boolean证据查询，使其有界检查位运算的两个实际SSA操作数，并允许调用方提供“此位置读取的局部是否已决定boolean”的同源查询。它仍只接受已证明叶子、位运算组合及适用的0/1字面值；不跟随任意phi/store。至少一个操作数须有独立boolean证据，另一侧须有boolean证据或适用literal；纯0/1组合不能无上下文地启动一个boolean局部。最终渲染在已证明boolean位置做literal拼写，不向普通int表达式传播boolean需求。
4. 局部类型沿用decide_types唯一决策和已有工作队列。将第一写入表达式里受支持的位运算局部依赖纳入现有readers关系；某个依赖获证时，对该写入重新执行同一证据查询，只有组合完整成立才把目标入队。每个局部至多从未证明变为已证明一次，不能仅因一个依赖boolean就把`a & integerLocal`认成boolean。依赖收集与查询都受现有深度限制、预算和取消约束；不新建第二个类型pass，也不在AST完成后全树改类型。
5. render_value读取两个真实操作数，递归传入最终消费at及深度；在boolean模式呈现两个boolean值，在整数模式校验整数类型。消费延期与失败生产者追溯须同一准入，防止有副作用调用先独立发射、再嵌入表达式执行两遍。已恢复前缀不因后续失败整体退化；来源覆盖可由正常语句或引用共同承担。
6. 在统一binary_binding中增加Java实际的`&`高于`^`高于`|`、且低于相等运算的级别，保持一套左右分组规则。来源仍挂真实运算与操作数，source map commit/replay和节点收费不另写路径。已有纯值循环如需接纳，只追加本操作事实，不能借机扩大调用/guard/存储准入。
7. 不引入依赖。外部解析/反编译/类型推理库不能替代已读取的SSA与descriptor，新增维护、许可及运行成本没有收益。受控javac/java只用于自写审计和测试；产品不执行目标代码，不改变parse、dialect、runtime、verification、source compilation的报告承诺。

依据：[JLS 15.22](https://docs.oracle.com/javase/specs/jls/se23/html/jls-15.html#jls-15.22)、[JLS 15.7.2](https://docs.oracle.com/javase/specs/jls/se23/html/jls-15.html#jls-15.7.2)。

## Risks / Trade-offs

- 局部声明与return各自猜类型 → 共享证据查询，固定nested/copied/hoisted/passed四类真实字节码及混合descriptor负例。
- 0/1把整数局部误导为boolean → literal不作独立种子；加纯整数0/1组合与boolean加literal的成对回归。
- 组合依赖导致无界扫描/递归 → 沿用已有局部队列，明确每条依赖的有界检查与预算收费，深链及取消测试覆盖。
- 运算符加入后括号或类型默认分支漂移 → 对byte/char提升、long边界、混合分组和实际javac结果验收，不只比较运算符字符串。
- 调用消费或拒绝丢失效果 → 左右一次、左侧false仍调用右侧、每侧抛错、旧局部和dup/pop拒绝分别对照。
- 多项改动共享build/facts/AST → fixture可独立准备，生产必须等root串行移交，保留其它change的完整实现与证据。
