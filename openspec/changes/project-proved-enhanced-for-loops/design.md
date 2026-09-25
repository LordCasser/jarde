## Context

见 [proposal.md](proposal.md) 与 [当前三方证据](../../evidence/java-syntax-2026-09-24/enhanced-for-current/analysis.md)。`jarde-java` 已有 `Region::Loop`、`ForHeader` 的单前驱／单 latch 证明、SSA 用途、数组读取与长度事实、`StmtKind::For` 及 `break`／`continue` 绑定。数组样本当前是行为正确的计数 `for`，`Iterable` 是行为正确的 `while`。旧 2026-09-22 审计中的 `hasNext()` 编译失败不是当前状态。`for-each` 与手写下标循环可编译成同一字节码，投影只能主张可观察等价。

## Goals / Non-Goals

**Goals:** 第一切片让可证明的 Java 8 数组遍历输出增强 `for`，保留正确的普通循环拒绝路径；第二切片在类型证据闭合后限定 `Iterable` 投影。任何切片均须保留真实来源、预算与整类可执行性。

**Non-Goals:** 不恢复唯一原作者写法；不扩大当前 Region 对嵌套循环、异常局部或任意 loop shape 的接受范围；不把通用集合／泛型类型求解塞入循环投影；不修整仓既有语料指纹与 Clippy 债务。

## Decisions

1. **复用已有循环结构，添加最小展示节点。** 数组候选只在现有 `Region::Loop` 与 `ForHeader` 已接受后进一步证明；已证明时 `build` 构造一个 `StmtKind::ForEach { label, element type/name, iterable, body }`，`emit` 只负责 Java 8 拼写。失败保留现有 `For`／`While`。不新建 IR、crate、全局循环 pass 或依赖。直接用字符串替换计数头会失去类型、来源和 `continue` 的结构信息。`ForEach` 是必要的一个 AST 语句形态，不是新机制层。
2. **先局部证明，再原子投影。** 从既有 SSA/Operation、loop 的物理 BCI 和前驱／latch 关系核对：索引初值零、唯一 +1 更新、`index < length`、长度与元素读取同一数组 SSA 身份，数组捕获只求值一次，索引和长度缓存无外部消费者；元素绑定恰好在循环体入口，并可合法赋给声明类型。额外索引用途、不同数组、读写数组引用、越界或异常顺序变化均拒绝。`ForHeader` 对带副作用 latch 和早期 `continue` 的拒绝边界继续生效；`break`、标签及异常边不重新推断。先生成完整候选与来源，再从当前语句序列移走被折叠的捕获/索引/元素绑定；证明或构建失败不修改当前普通循环。数组表达式需从原捕获语句取得；若保留临时数组声明更能证明单次求值，也可写 `for (... : temp)`，但不可重复调用供应者。
3. **`Iterable` 是后续独立准入，不跟数组同投影规则。** `iterator()`、`hasNext()`、`next()` 的签名及使用次数只是必要条件；还需以方法／类源类型和元素转换证明 `for (T x : expr)` 可以编译。第一批候选宜先限定字节码确实调用 `java/lang/Iterable.iterator()Ljava/util/Iterator;`、`java/util/Iterator.hasNext()Z`、`next()Ljava/lang/Object;`，并证明 iterator 值仅供这两次调用使用、`next()` 是每轮体内首个可观察动作、容器表达式仅求值一次。仅有同名 `iterator()` 的非 `Iterable` 类即使通过 JADX 的短签名检查，也没有合法的增强 `for` 源类型。原始 `Iterable` 参数不能把增强 `for` 的元素直接声明为 `String`，但可尝试 `for (Object item : raw)` 并在循环体原位保留 `String value = (String) item`。这一写法在[手工 Java 8 探针](../../evidence/java-syntax-2026-09-24/iterable-raw-projection/analysis.md)中合法，原/探针的十条执行轨迹只有 helpful-NPE 的临时变量描述不同；因此泛型方法头投影并非此子集的硬前置。新的 `Object` 绑定必须有不冲突的局部名称；已共享的 `ForEach` AST 字段应从 `array` 改名为 `iterable`，使节点准确表达数组和 `Iterable` 两种来源，不增加另一节点。是否能把 `next()` 的隐式执行、cast 与已有 SSA 局部绑定原子移交，仍须 4.1/4.2 验证。不得通过无来源的泛型强转或更改 inferred type 强行让输出编译；没有完整证据时保留 `while`。
4. **算法参考而非复制 JADX 的变更模型。** 本地 JADX `2fb1b1638694` 的 `LoopRegionVisitor` 使用双输入 phi、归纳变量用途、同一数组长度与 `aget` 关系，值得借鉴；它在识别后将旧指令标成不发射。Jarde 的来源与停止契约需要原子候选，不采用可变指令隐藏。[受控 Java 8 对照](../../evidence/java-syntax-2026-09-24/iterable-raw-projection/jadx-iterable-for-claim/analysis.md)表明 JADX 对同一 `Iterable` 方法有调试局部表时输出增强 `for`、`-g:none` 时退回 `while`，即使字节码指令和泛型方法 `Signature` 相同。Jarde 的后续证明应以 SSA/类型/作用域为准，不把调试表当硬前置；可同时追平有调试信息的 JADX 子集，并覆盖其无调试信息的保守缺口。无外部库能替代已有 SSA、Region 与类源码类型检查，新增依赖既无能力收益也增加维护和许可证审计成本。
5. **验收区分解析、验证和源码恢复。** classfile 解析与 JVM 验证沿用已有 reader/JVM 事实，`javac --release 8` 验证输出的源方言合法，`java -Xverify:all` 对照原 class 的值、调用数和异常类型／顺序；JADX 只是固定版本的质量参照，不是行为真值。对象数组、一次求值、空/null、抛错、索引逃逸、错配数组和非 `Iterable` 探针分别占用正反例。带 helpful-NPE 的诊断文本由 javac 选择临时变量而变，不把该字符串当等价判据；异常类型与可观察调用轨迹必须匹配。

`Iterable` 的候选提交应在 `Region::Loop` 已写出原 `while` 后进行：从紧邻前驱的 iterator 赋值、测试表达式和体内首个元素绑定分别取 AST 与真实 BCI，交叉核对 `CallTarget` 的 owner／descriptor、SSA 值的独占消费和调用所在的异常处理区。先构造新的 `ForEach`、原位保留的 cast 与来源，再一次移除 iterator 赋值和旧 `while`；任何一项证明失败都不改变已构造的普通循环。若将 `next()` 从体内表达式移到增强 `for` 的隐式取值会越过先前的调用、写入或 `try` 边界，拒绝投影。此处只重写 AST 中被证明的 `next()` 叶节点，不做文本替换或事后修改 SSA 类型。

[负例当前重放](../../evidence/java-syntax-2026-09-24/enhanced-for-algorithm/analysis.md)发现两个手写数组循环在 Jarde 仍为 explanation-only；这些输入只用于本变更的“不误投影”门槛，不能伪造 Jarde 运行等价。非 `Iterable` 探针在带同输入嵌套类的 classpath 下，Jarde `while` 可重编并运行一致。数组普通循环可编译性留给结构基底工作，不混进数组语法投影。

[4.1 受控边界](../../evidence/java-syntax-2026-09-24/iterable-raw-projection/remaining-4-1/analysis.md)还给出两种 JADX 语义错误：无调试表时把 cast 后的副作用挪到 cast 前，坏元素轨迹 `touches=0→1`；有调试表时把原本在 `try` 内的 `next()` 放进增强 `for` 隐式头部，异常从被捕获变成逸出。[root 复核](verification-root-iterable-boundaries.md)确认原始与重编 class 的运行差异。Jarde 当前对后一受保护区仍 explanation-only，属于独立结构债务；本投影只认处理器集合相同、取值和转换次序完整的子集，不因 JADX 能写 `for` 而降低门槛。

## Risks / Trade-offs

- 数组变量在捕获后被重赋值，或长度与元素读取只表面上同名 → 必须比对 SSA 值与消费，不按局部槽号／名称认领。
- `continue` 跳过 latch 的其它效果，或 `break` 后索引仍被读取 → 沿用 `ForHeader` 边证明，并拒绝逃逸消费。
- 删除旧头部指令时丢失 BCI、重复打印副作用或预算超支 → 构造完整来源与计费后的候选，一次提交；失败保留原表示。
- `Iterable` 的泛型签名、raw 类型与元素 cast 不能在当前类源码中保持合法 → 优先检验 `Object` 元素绑定加原位 cast；不能完整证明时保留已可执行的 `while`，单列类型缺口，不扩大数组改动。
- 第一批只认 `java/lang/Iterable.iterator()` 会漏掉合法的 `List`／`Collection` 源码：`javac --release 8` 对这两种静态参数类型分别发出 `java/util/List.iterator()`、`java/util/Collection.iterator()`。现有类型层不作通用接口继承判断；扩展时须从可信类层级证据证明源类型可用于 Java 8 增强 `for`，不能把 JADX 的“任何 owner 的 `iterator()` 短签名”放宽直接照搬。

## Migration Plan

无持久数据迁移。先固定基线与拒绝样本，再发布数组子集；任何证明失败均走当前普通循环。逐项验收后才开启 `Iterable` 子集，回退只需撤销该局部投影入口与最小 AST 分支。
