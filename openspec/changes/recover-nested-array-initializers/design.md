## Context

见 [proposal.md](proposal.md) 与[冻结对照](../../evidence/java-syntax-2026-09-25/nested-array-initializer/analysis.md)。reader 已将 `anewarray [I` 解成 `element=int, dimensions=1, total_dimensions=2`；`array_element` 已能从该事实得到外层 `aastore` 的 `int[]` 组件。现有 `ArrayInitializers::prove` 只准 rank 1，最终消费者不含 `aastore`，父层扫描也不能将已证明的子层 store 链作为一个元素表达式。`NewArray` AST 已有完整 rank、长度与可选初始化元素，类型呈现和发射只是把带初始化器限制在 rank 1。

JADX `ReplaceNewArray` 可参考“收集同一分配的写入并组合为数组值”的方向，但其 `TreeMap` 按索引重排会改变有副作用写入的实际顺序；冻结乱序反例已观察到原 class trace `12`、JADX 重编 trace `21`。直接复用本仓现有 SSA/效果/来源与 Java 类型规则，避免引入该算法或额外依赖。`javac` 用于验证生成 Java 8 语法，`java -Xverify:all` 对比运行；都只在本地冻结 fixture 的测试/验收中执行，不属于引擎对用户输入的运行时行为。

## Goals / Non-Goals

**Goals:** 让同一基本块内、分配与 store 链完整闭合的嵌套数组表达式通过既有 Builder 提交；至少覆盖带可观察元素的 `int[][]` 与 `String[][]`，并保持乱序边界现有正确行为。

**Non-Goals:** 不推断源程序原始大括号风格；不处理数组逃逸、跨块/循环链、稀疏或重写索引、无法证明的协变引用存储、动态长度或任意级别的局部别名；不扩展类型层级解析，也不重写 Region 认领机制。

## Decisions

1. **在原证明计划内自内向外组合。** 按块内分配 BCI 逆序尝试候选，子层先得到独立 `ArrayInitializer`。仅当子数组的最终 SSA 值被一个真实 `aastore` 作为值操作数唯一消费、组件类型可证明等价时，允许其把该 store 作为终端消费者。父层以子证明为一个有界元素区间核验；子层的分配、`dup`、索引、元素计算和 store 逐项计入该区间，但父子 `owned` 集合不能共享 BCI，父 store 仍只由父层认领。任何候选失败不改变已提交计划。若直接复用普通 `collect_expression_bcis`，它会在子层 `dup` 处失败；仅在精确匹配子证明的 final value、范围、终端 store 与单一 use 时把该子链视作表达式单元。相比新增全局别名/数组图，这一局部扩展最小且与既有规则一致。
2. **维持逐层物理顺序与异常集合。** 每层仍要求静态长度 N、按字节码顺序完整写入索引 `0..N-1`、同一 SSA 分配身份和无独立 effect 的区间。父层在每个元素的原位置验证子链，不因索引排序或临时值移动改变分配→元素→写入顺序；所有可能抛错的来源使用既有 handler 集合比较。额外 use、入口或异常边不满足时完整拒绝。候选扫描、子链验证与来源合并全部使用现有预算、取消轮询和深度上限。
3. **沿用一个 `NewArray` AST 节点。** 带初始化器时允许经证明的总 rank，类型为 `base + [] × rank`，发射 `new T[][]{new T[]{…}, …}`；内层独立呈现自己的 `new T[]{…}`。父层的每个元素仍经过 `array_initializer_element`，只接受准确的数组组件类型或该函数已有的安全准入。普通 `new T[n][]`、一维 `new T[]{…}` 与空数组的现有行为不变。不额外发明嵌套数组 AST、通用递归数组推断器或可疑强制转换。
4. **以 JVM trace 作为正确性裁判。** 对冻结正例及乱序反例独立读取原 class、JADX 1.5.6、Jarde 完整源码，重编并在验证模式比较值、trace 与异常。来源须覆盖每个真实 BCI，成功时不重复写普通数组赋值；失败时完整引用。JADX 的可编译性与最终数组相等都不足以证明求值顺序。

## Risks / Trade-offs

- **子层终端 `aastore` 与父层 own 的关系** → 子层来源可包含终端作为边界证据，但其 `owned` 不含该 store；父层独占 store，source map 去重并检查原真实 BCI 全覆盖。
- **元素类型或 handler 信息不足** → 宁可保持带来源拒绝；不通过 Java 强制转换偷换 JVM 的 `ArrayStoreException` 或首次异常时点。
- **嵌套深度及扫描成本** → 复用 Code/IR 尺寸、现有预算/取消和 `MAX_VALUE_DEPTH`，不得为任意输入无界递归或二次展开同一子链。
- **当前一维门槛保护现有负例** → 保留按物理 0..N-1 的准入，尤其冻结的 `result[1] = mark(1); result[0] = mark(2)` 必须输出同序或拒绝。
