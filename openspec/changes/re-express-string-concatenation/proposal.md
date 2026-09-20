## Why

两个发现共享同一种表示（左深 `Binary::Add`），本 change 分别验收、一次重述（[完成复核](../../completion-review.md) 的 T1 与 T4 行及其两节；本 change 采用这两行记录的数值）：

**T1（P0，进程 abort）**：源码形状 `new StringBuilder().append(s)` 重复 N 次后 `.toString()`（复核样本 `DeepConcat2048`，`javac -J-Xss32m --release 8 -g:none`，class 摘要 `a5b32735…`）在 debug 构建上：**N = 1536 与 2048 退出 `134`、stdout 为空**、stderr 为 `has overflowed its stack`／`stack overflow, aborting`；N = 1024 完成。同一字节的 optimized release CLI 在主线程上对 2048、4096、8192、15000 均完成——两条是**不同的构建边界**，本 change 分别记录。LLDB 把递归定位在表达式打印机（`crates/jarde-java/src/emit.rs::expr` 的二元臂，复核记 `emit.rs:476`），不是 `Drop`；根因是 `crates/jarde-java/src/build.rs::concat_expr`（约 2561 起）从第一个片段起左折叠构造一棵深度等于 append 数的树，每个片段各自满足 `MAX_VALUE_DEPTH`，但那条界只约束**值**的嵌套，约束不了这棵树在构造、打印、克隆与释放各路径上的深度。上一轮 superseded 方案只在构造前加深度界，把左深树与转换缺陷留在原地，本 change 不接受该形态。

**T4（P1，值错误）**：`new StringBuilder().append(a).append(b).append("!")` 被恢复成 `arg0 + arg1 + "!"`：`arg0 + arg1` 是**数值加法**，随后才与字符串相加。输入 `(1, 2)` 时原 class 返回 `"12!"`、恢复文本返回 `"3!"`，报告仍为 Complete/Structured（结构平面看不见转换）。`concat.rs::keeps_its_conversion`（`concat.rs:654`）逐 overload 的判断（`int/long/float/double/boolean/String/Object` 接受，`char`/`char[]`/`CharSequence` 等拒绝）只在**字符串上下文里**成立；`concat_expr` 直接从第一个原始值开始左折叠，没有让第一个 `+` 进入字符串拼接。缺陷与 receiver 分组修复无关：`fa6dc6e` 已存在。

缺陷的底层形态（复核的框架）：**类型被多个位置各自猜、表达式被多个位置各自建**，于是修一处不会让别处停止产出矛盾结果。本 change 把这一个决定交回唯一所有者：**拼接链的呈现由一份片段表示拥有**——片段各自带着自己的表达式、`append` 的参数类型与 origin，转换在片段自己的位置发生，链长只增长序列、不增长这一层的递归。

## What Changes

- 被判为拼接链的呈现 MUST 逐片段保留该 `append` overload 自己的转换，转换发生在片段自己的位置；链的呈现 MUST 从一个**字符串上下文**开始，MUST NOT 先把相邻数值片段加成数值再转换（`(1, 2)` 的 `"12!"` 是本条的验收值）。
- 布尔片段 MUST 按 `append` 的参数 descriptor 拼写：参数为 `boolean` 的片段里的 `0`/`1` 字面量 MUST 是 `true`/`false`，MUST NOT 是 `1`/`0`；`null` 片段与对象片段按其 overload 的转换写出。
- **片段表示**（例如带片段序列的节点）由 design 选定：增加一个片段只增长序列，MUST NOT 增长本层在构造、发射、克隆或释放路径上的嵌套深度；片段内部真正嵌套的子表达式仍按既有 `MAX_VALUE_DEPTH` 受限。MUST NOT 用放大线程栈替代这一性质，MUST NOT 把「2048 个片段必须停止」写成产品契约：深链能呈现时 MUST 正常完成。
- 求值次数、对象到字符串的转换、副作用与异常次序 MUST 与链一致，每个片段保留自己的 BCI/source-map 锚点；MUST NOT 任意平衡、合并或重排片段——两个片段 `append(a).append(b)` 是 `"" + a + b`、**一个**加法片段 `append(a + b)` 是 `"" + (a + b)`，两种形态 MUST 同时成立（同一输入 `(1, 2)` 分别得 `"12!"` 与 `"3!"`）；链的识别与今天被拒绝的 overload 集合 MUST 逐字不变。
- 验收：T4 以 `(1, 2)` 的多顺序执行对照（含 boolean、`null`、对象与一个可观察/可能抛异常的片段）；T1 以**子进程级**检查覆盖 debug 与 release 两个构建，既要**正常完成**也要覆盖预算停止后的清理，MUST NOT 出现 abort、空 stdout 或 signal；变异恢复左深折叠（构造/打印/释放任一处的深度）必须让检查变红；既有全字符串链与「数值在最后」的链逐字不变。

## Capabilities

### New Capabilities

无。不新增 crate、公共模块、报告平面或停止分支。

### Modified Capabilities

- `java8-recovery`：新增「拼接呈现逐片段保留 append 的转换、并从字符串上下文开始；片段的转换按其参数类型拼写（含 boolean/null/对象）」的要求；「Recovery recursion is bounded and a stop is published」补上「链长 MUST NOT 变成这一层的递归深度（构造/发射/克隆/释放），深链 MUST 返回报告，默认线程栈是验收条件」的段落。
- `recovery-validation`：新增「拼接转换以受控执行对照验收（多顺序、含 boolean/null/对象与求值次序）」的要求；「A signal is not an acceptable answer」补上深链的**子进程级、debug 与 release 两构建**回归、预算停止后的清理、以及「放大线程栈不是修正」的约束。

## Impact

实现落在 `crates/jarde-java`：`build.rs::concat_expr`（片段表示与字符串上下文的起始）与 `emit.rs`（打印机按片段序列迭代；若片段表示参与克隆/释放路径，那两条路径也在本 change 的证据面内），必要时 `ast.rs` 新增一个纯增量的片段节点；`concat.rs` 的识别与 `keeps_its_conversion` 只读。本 change 依赖 [unify-local-type-decisions](../unify-local-type-decisions/proposal.md)：当一个片段的值是一次对局部变量的读取时，「这个值是不是 boolean」由该 change 的局部类型决定给出，本 change MUST NOT 在拼接路径里再作一次局部类型判定；它也消费 [decide-comparison-contexts](../decide-comparison-contexts/proposal.md) 定下的「值自身的证据 / 目标位置的要求 / 字面量适配」三分（`append` 的参数 descriptor 是片段的目标位置）。三者都改 `crates/jarde-java`（`build.rs`、`emit.rs`），MUST 按此顺序**串行**实施（本 change 为第 3 个），彼此不并行改动同一文件。

交付包含两类受控 fixture：转换形状用兄弟源码 + 真实编译产物（`tests/fixtures/p3-concat-conversion/`，javac 可产出）；深链用**测试内生成器直接产出的字节**（说明见 design 决策 6：`javac -J-Xss64m` 只是复核产出字节的手段，不能进入回归，而链长是检查的参数）。两者都带来源 README、`tests/fixtures/README.md` 登记、fingerprint 再生成与 reader fixture census 更新。修正前的反例（T1 的 signal 与 T4 的两侧值）MUST 各记录一次。

非目标：不新增 crate、依赖或 verifier；不改拼接规则的识别与 `keeps_its_conversion` 的接受集合；不新增报告平面或停止分支；不 spawn 工作线程、不使用 `catch_unwind`（栈溢出无法捕获）、不放大线程栈、不引入增长栈的库；不做一般表达式类型推断；不重开 R8/R9 与已归档的递归界、分组、boolean、数组类型修正；不因本 change 改动 `decide-comparison-contexts` 或 `unify-local-type-decisions` 的决定；不声称任意链长安全或覆盖整类表达式缺陷；不做性能工作（`optimize-demand-workloads` 保持 0/22）。当前仅完成修正规划，实施任务全部待办；历史归档与既有验证记录保持原状。
