## Context

证据在 `../../evidence/java-syntax-2026-09-22/instanceof/`。原始Object参数审计80行与JADX一致，扩展 type-boundaries 的合法Java源码又暴露四处JADX编译失败：源码 `(Object)` 加宽没有checkcast，指令不能独自保证Java左操作数静态类型可用于该目标。

decode尚无instanceof事实，AST没有类型测试。build共享boolean_proof负责表达式和局部初值的boolean判断，已有spell_reference、Cast、Not、最终消费点与quoted_bcis可复用；不能从JVM int栈形状推定Java int拼写。

## Goals / Non-Goals

**Goals:** 在已有区域内忠实呈现类型测试、boolean用途及合法引用操作数上下文，保持生产者效果、旧值与来源。

**Non-Goals:** 不恢复0/1 phi汇合或Java pattern matching，不扩展guard/loop准入，不解析外部继承图，不把测试折叠为true/false，不修复任意非法class。

## Decisions

1. 增加忠实的Operation::InstanceOf目标类型事实和ExprKind::InstanceOf{value,ty}，结果presented为Boolean。与CheckCast复用CP类目标读取和spell_reference，但保留不同操作事实，不能借Cast或拼字符串冒充类型测试。数组/多维数组沿用既有类型拼写，目标不合法时拒绝。
2. 操作数经既有single_stack_read和render_value在最终消费者at呈现，保持显式运行时checkcast及其顺序。为避免缺失层级信息导致不可编译的类型比较，已知具体Reference操作数通过已有Cast安全上溯java.lang.Object；本来为Object、Null或已有同型Object cast不重复包裹。不能向目标类型做收窄cast，不能用SSA引用形状覆盖源码presented类型。类型事实不足以证明是引用时按原契约拒绝，不新增resolver或String等类名特判。Object上溯不增加运行时类型检查，type-boundaries用真实编译执行固定这一选择。

   已恢复的lambda/方法引用需要先以已有工厂descriptor固定函数式目标，再安全上溯Object；不能输出 `(Object) Type::method`。该事实和双层Cast已在调用参数路径存在，复用同一有限逻辑，不另造函数式类型推断。type-boundaries的命名方法引用直接供给instanceof，javap证明中间也没有checkcast；本项只承接已有lambda模式，不扩大模式准入。
3. 将该操作作为共享boolean_proof的事实叶子，使boolean局部、返回、字段/数组写和实参使用同一类型判断；既有条件零分支极性自然得到测试或Not。发射按关系运算优先级处理instanceof及外层Not，不改变通用二元分组体系。boolean被用于非法Java整数算术的合法JVM边界必须继续拒绝，不能打印boolean+1。
4. 延期、真实消费者、唯一/重复消费和失败引用沿用现有引用表达式路径。instanceof结果被pop丢弃仍不能写成非法独立Java表达式语句或直接吞掉；保留测试和必要生产者引用。后续消费者拒绝时，quoted_bcis必须包含类型测试及延期call/cast/new；不新增消费表或值缓存。显式不相容类型也不折叠常量，以保留操作数效果和原始目标操作。
5. 新节点适配既有来源遍历、表达式类型与发射commit/replay。保留测试BCI和操作数来源，合成安全加宽用现有derived来源；默认无来源、正文/来源预算停止与失败不发布未支付节点的协议不变。
6. 不引入新依赖。现有读取、SSA、引用拼写、AST与boolean判据足以承接一个JVM操作；外部库不能替代缺失的内部表达式分支，新增库不值得其维护/许可/预算成本。JDK执行仅为受控测试，产品parse/dialect/runtime/verification/compilation平面不变。

依据：[JLS 15.20.2](https://docs.oracle.com/javase/specs/jls/se8/html/jls-15.html#jls-15.20.2)、[JVMS instanceof](https://docs.oracle.com/javase/specs/jvms/se8/html/jvms-6.html#jvms-6.5.instanceof)。取舍是少量Object加宽文本换取不依赖外部类路径的稳定合法拼写；当前不以额外层级推断压缩这类文本。

## Risks / Trade-offs

- 忽略静态类型，得到String instanceof Integer → type-boundaries整类重编译；用安全Object上溯而非目标收窄。
- 加宽覆盖原checkcast或吞掉调用 → 固定兼容/null/CCE、一次调用和producer自身抛错，内层原表达式不变。
- boolean叶子只接return，局部仍成int → 同一boolean_proof入口覆盖局部、分支及参数位置；非法整数消费者为负例。
- 看见未使用结果就删测试 → 固定pop/多消费者与来源引用；不引入非法Java语句形式。
- 把negated的0/1汇合冒充基础opcode覆盖 → 正面整类与该负面形状分开，禁止删生成方法后报整类通过。
- 与其它任务抢共享文件 → 完成numeric/throw/final字段交接后再派发生产实现。
