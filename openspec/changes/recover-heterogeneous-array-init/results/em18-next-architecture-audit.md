# EM-18：异构引用数组初始化器架构审计

## 判断

这片的最小实现边界是 Builder::array_initializer_element 的 reference 分支：元素已被证明是数组表达式中的一个 aastore 值，组件类型和元素的呈现类型已有明确来源；只需在现有精确类型、Object 组件、null 快路径之后，调用同模块已有的 platform_interface_argument_widens(self.profile.java_release, actual, expected)。命中时原样返回元素表达式，不构造 cast，不新增类型关系、LUB 或层级遍历。

这个 helper 的名字容易误导，但当前 NUMBER_FAMILY 六行确实位于其中；release_reference_argument_widens 则是 java.util 集合和两条 java.io reader 边的传递闭包，不含 Integer -> Number。因此 EM-18 planning 里把复用点写成 platform_reference_argument_widens 是旧名/错表，必须更正为当前 platform_interface_argument_widens（限定 JDK 8 的封闭行表）。

## 现有事实与代码路径

- ArrayInitializers::prove（crates/jarde-java/src/build.rs:12663）从已发布 SSA、decoded Operations、effects 和字段计划建立证明。它只进入有 newarray/anewarray 的方法；prove_array_initializer 从 Operation::NewArray 读取元素/维度，得出 component，并沿同块物理指令按预期索引顺序核对 dup; index; value; aastore。它还检查 allocation 与 store 的 handler 相同、索引/数组 copy/元素依赖的 use 闭合及唯一消费者。组件不是从 aastore opcode 推出来的：aastore 只证明 reference store（element: None）；数组形状由创建指令和 array_element(...) 得到。
- 表达式构造位于 crates/jarde-java/src/build.rs:24868：对每个已证明的元素调用 array_initializer_element(value, element_expr, component, at, store_bci)，最后仍生成带原 component 的 ExprKind::NewArray。
- 拒绝点是 crates/jarde-java/src/build.rs:26193-26246。Reference 分支目前只接受 null、元素 type 等于 component、或 component 为 Object；其余类型报告 array initializer element ... while the array component ...。Primitive/boolean 两条证据路径是另一问题，不应随本片改动。
- CT 的组件事实来自 cov() 的 anewarray java/lang/Number：Operation::NewArray 报出 Number；两个 aastore 分别填入 Integer.valueOf(1) 与 Long.valueOf(2L)，元素表达式的 presented types 来自其调用结果描述符。当前保存的完整类输出确实在 cov() 的首元素 BCI 23 因 Integer 对 Number 不等而拒绝，随后 local0 声明也没有生成。证据在 openspec/evidence/java-syntax-2026-10-05/heterogeneous-array-init-patrol/results/jarde-CT.txt。
- platform_interface_argument_widens 定义于 crates/jarde-java/src/build.rs:29355；NUMBER_FAMILY 是该函数内的闭表，精确列出 Byte/Short/Integer/Long/Float/Double -> java.lang.Number（29404-29411），并且只接受 java_release == 8。函数现有单测 the_boxed_number_rows_reach_exactly_their_pairs（约 32721）验证六对正例，以及 BigDecimal、BigInteger、AtomicInteger/AtomicLong/LongAdder、反向方向、数组、泛型、primitive 和其他 release 的拒绝。
- 同一现有 helper 被 invocation 参数投影调用（约 25511），并且也被数组形态的 invocation 参数 helper 复用（platform_array_argument_widens 约 29024）。但不能把 invocation 的 cast_argument 渲染动作照搬进数组元素路径：本片只读取 predicate 的证明，命中后保留元素原有呈现类型，依靠 Java 数组初始化器的赋值上下文完成 widening。

## ASE 语义边界

主锚是新建的确切 Number[]，不是一个静态声明为 Number[]、运行时可能实际指向 Integer[]/String[] 的别名。源和生成表达式都保留同一 Number[] allocation、同一元素求值次序与每项赋值；Integer/Long 到 Number 是无运行期检查的 widening reference conversion。数组初始化仍对元素执行运行期 array-store 语义，而闭表命中的确切 wrapper 不能触发 ArrayStoreException。不插入每元素 cast，也不把已证明的 store 检查移到表达式求值之前。

这个结论依赖现有 initializer proof 保持的物理顺序和新数组身份。JADX ReplaceNewArray.processNewArray（本地 checkout /Users/lordcasser/workspace/testzone/jadx/jadx-core/src/main/java/jadx/core/dex/visitors/ReplaceNewArray.java:77-181）把 stores 放进 TreeMap，再按 index 添加 filled-array 参数；它的 verifyPutInsns（184 起）检查同块及数组没有被当作 store value 使用，但这不是 Jarde 可以借用的副作用顺序证明。Jarde 的既有 prove_array_initializer 正按物理顺序要求 index 0、1、…，且 EM-18 报告记录了乱序副作用反例，所以不要改为按 index 重排。
一般协变数组写入由独立的 widen_covariant_store_receiver（约 29814）处理：它面对可能的运行时窄数组时通过 receiver cast 保留原 aastore 的动态检查。本片不改变那条路径，也不以 component 的静态 assignability 证明任意别名数组不会抛 ASE。

## 规划材料需要改正的陈述

- recover-heterogeneous-array-init/design.md:9 所说的复用点 platform_reference_argument_widens 不存在于当前 build.rs。应改成 platform_interface_argument_widens 的 NUMBER_FAMILY 六行；若要描述另一个表，现名是 release_reference_argument_widens，其直接边表只陈述 java.util/两条 java.io 关系，不能证明 Integer -> Number。
- recover-boxed-number-widening/proposal.md 的 Why 也把旧缺口写成 platform_reference_argument_widens 缺 java.lang rows；这是同一 stale helper 名。当前实现把 java.util/reader 边放在 release_reference_argument_widens，并把六个 boxed -> Number rows 放进 platform_interface_argument_widens。EM-18 应引用实际代码与表位置，不从这个旧称反推实现入口。
- recover-heterogeneous-array-init 的提案/需求把范围表述为任意“元素呈现类型是组件子类即可接受”。现有证据不是开放层级图：NUMBER_FAMILY 只证明六个 JDK 8 boxed types。需求应绑定“现有关系 helper 明确证明的赋值方向”，避免暗示 BigDecimal 或任意用户类也会被新层级规则接受。BigDecimal 确实是 Number 子类，但当前闭表刻意不包括它；这是保守拒绝的边界，不应写成 Java 赋值本身非法。
- design.md:15 把 component 描述为 anewarray/aastore 两者共同事实不准确。CT 的 component 来自 anewarray Number（reader/Operation::NewArray）；aastore 只表明 reference store，实际 component 仍由数组值形状读出。元素的 Integer/Long 则来自各元素表达式的调用结果描述符/呈现类型。
- proposal.md 的“更准确而非更宽”只有在闭表范围内成立；若保留“任意子类/超接口”措辞，它实际上比当前已证明的 platform relation 表宽。最小决策应明确不扩成任意 subtype checker。
- 任务里的“up() 同构 String 数组零回退”适合作为 whole-class golden；CT.up 实际源是 Arrays.asList("a","b") 的 String[] varargs 形，被呈现为 Object[] argument cast，不要把它写成 Number widening 对照的直接测试。

## 有界验收例

1. 正例：冻结 CT 两 compiler legs（javac 23 --release 8、真实 javac 8），只要求 cov() 与整类完整源/重编/运行通过，运行输出对照原始 class。它覆盖 component=Number、异构元素 Integer 与 Long、有序 aastore。
2. 零回退：同一 CT 的 up()、io()、io2() 完整文本逐字保持；再保留 EM-18 已冻结的同构 String[]/计算 int[]/对象数组实参输出。
3. 封闭表负例：另一个 Number[] initializer 让元素表达式呈现为 java.math.BigDecimal（也可用 AtomicInteger）时仍拒绝合并，因为现有 NUMBER_FAMILY 不陈述该关系。注明这只是“当前证据闭集之外的保守拒绝”，并非声称 Java 不允许 BigDecimal 赋给 Number。
4. 真实 ASE 负例：如需测试运行期异常边界，用独立的 verifier-valid classfile/混淆产物构成 anewarray String 后 aastore Integer.valueOf(...) 并捕获 ArrayStoreException；它应保持逐步写入/拒绝 initializer folding。普通 Java 源码不能直接表达 new String[]{Integer.valueOf(1)}，不要把无法编译的源片段当作 class fixture。另可用 Number[] component + static Object element 作未知兼容性拒绝。它们与 BigDecimal 闭表负例含义不同。

## JADX 用例的界限

EM-18 首片报告 openspec/evidence/java-syntax-2026-09-27/em18-array-initializers/report.md 使用 TestArrayFill、TestArrayFill2、TestArrayFillNegative、TestArrayTypes 做 Java class 对照；其生成输出与 Jarde/原 class 的固定数组形态和运行结果有明确记录。JADX TestArrayFill2.test2 单独是 new int[]{1, a++, a * 2} 并标 @NotYetImplemented；本地文件 /Users/lordcasser/workspace/testzone/jadx/jadx-core/src/test/java/jadx/tests/integration/arrays/TestArrayFill2.java 可直接看到注解。它属于后缀副作用位置，不是 CT.cov 的异构 Number varargs 数组，不能作为本片正例、反例或对 Number widening 的 JADX 能力证据。已有独立记录在 openspec/evidence/java-syntax-2026-09-25/array-postfix-element/analysis.md。
