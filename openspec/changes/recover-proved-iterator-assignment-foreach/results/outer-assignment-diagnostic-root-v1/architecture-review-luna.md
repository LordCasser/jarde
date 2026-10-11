# CF11 outer13 assignment-to-foreach architecture review

只读审查；未运行构建、测试、Git 命令，也未改仓库。本文判断的是静态实现边界，不声称固定 71 单元或 CF11 UpstreamNestedLoops outer13 的门已通过。

## 结论

若同次 IR 上现有 iterable 完整 proof 对外层 loop 成功，支持首条 `Assign { value: Cast(next()) }` 的最小方向，是保留该赋值和 cast 原样，只把 cast 的 `next()` 值接到新的 Object foreach 元素局部：

```java
for (Object element : source) {
    target = (T) element;
    // 原循环体其余内容
}
```

这种形式不需要新增 `next()` 目标元素的 `LocalVariable` 身份证明。被引入的 `element` 是 foreach AST/emitter 新建的绑定，不是原 bytecode local；原赋值的目标 local、声明位置、类型决定及其它读写均留在现有 AST 中。关键是把原赋值 RHS 中的 `next()` 唯一替换为合成元素读取，并证明 `next()`（及 cast 结果）只到达这条首条赋值。证明应以 statement origin/BCI、指令与 SSA `ValueId` 链关联，不以 slot 或打印出来的 name 当身份。

目前 candidate 只接受首条 `Declare { value: Some(..) }`（`build.rs:21480-21518`）；成功后只改局部克隆 body 的初始化表达式为 Object local，并生成 `ForEach<Object>`（`21980-22011`）。所以 Assign 目前会直接拒绝。若扩大入口，不能把保留赋值误转换成“foreach 元素就是原 target”；后者会改变量声明/作用域并删除一次 cast/赋值。

## 现有 proof 覆盖与必须守住的不变量

`iterable_for_each_candidate` 已证明：Java 8+、`Iterator.hasNext()Z`/`Iterator.next()Object` 与特定 Iterable/List/Collection `iterator()` 的精确接口调用（`21480-21504`, `21590-21680`）；Iterator 初始化声明归属由 `iterator_declaration_for_loop` 依据 reuse variable、声明 placement/owner 和可见声明行检查（`build.rs:11854-12017`, 调用处 `21690-21725`）；来源只取 local 且 presented type 符合 owner；init 到 loop 之间的 gap 不承载有副作用的语句，且不写 source/iterator（`21726-21760`）。Iterator 初始化结果只被 store 消费；header 仅含已证的 load/hasNext/test；next block 在 next 前无额外指令；iterator phi 只有初始值和 loop 自身回边，phi 与初始 iterator 值只在 hasNext/next 的两处 load 使用；receiver 的 SSA 值逐一吻合；next 结果只由 cast 或 store 消费，inline cast 输出也只有目标 store 一个消费者（`21760-21870`）。最后 init/hasNext/next/test 以及可选 inline cast 的 exception handler 集合必须相同（`21896-21912`）。Iterator 初始化删除行和 iterator 自身声明（若在当前语句列表）随后才移除；内层索引只通过 helper 的已证 owner row 返回（`21934-21951`）。这些 iterator 证明维持原样。

对 Assign 最小扩展还需证明：

- 它是 loop body 的第一条语义语句，且其 `origin.primary().bci()` 正好是 `next` 的 store BCI；RHS 是精确的 `Cast(Call Iterator.next())`，不是仅同名/同 slot 的另一个表达式。
- `next` 返回的 SSA 值只有这条 cast 的消费者，cast 结果只有该 store 消费者。现有 `next_result`/cast 单消费者约束可复用；禁止以目标变量的 slot/name 代替此数据流证明。
- 只替换表达式树中的 `next()` 叶子为 synthetic Object local；保留 Assign 的目标、cast 类型、cast origin、statement origin 和其它语句不变；不删 target 的 declaration、不把 target 提升成 foreach header 绑定。
- 保留当前处理时序：原序列是 `hasNext` 成功、`next`、cast、对 target 的写、剩余 body。foreach 的迭代取值后进入 body，再由首条赋值执行 cast/write，顺序相同。异常集合必须仍覆盖 next 与 cast 相同；现有 proof 已对 inline cast 检查 handler equality。不得把 cast 移到 header 或删除它。
- 现有安全 gap 必须继续成立，尤其 ancestor owner 分支只允许空声明 gap；不能因为新目标是 Assign 而放宽成可跳过调用、字段写或其它可观察动作。源表达式保持当前 local-only 限制，iterator() 仍恰好一次且位于原 loop 前的执行点。

`StmtKind::Assign` 本身只有 `{ name, value }`，`Declare` 则带类型、可选 source spelling 和初始化值（`crates/jarde-java/src/ast.rs:780-814`）。emit 对赋值逐字输出 name 和 RHS（`crates/jarde-java/src/emit.rs:759-765`），而构建期 `write_statement`/`assignment` 已按 `LocalVariable` 的类型决定检查赋值值（`build.rs:25532-25611`）。既然转换保留该节点，就不需要重新决定 target 类型或变量身份；candidate 需要验证它确实保留了绑定语句和数据流，type checker 仍由既有构建路径负责。

## 提交/回滚边界与对抗控制

loop body 先在临时 `stmts` 中 walk，再恢复外层 statement list 和 `declared` 集合（`build.rs:19135-19147`）；walk 出错后立即传播。candidate 在该恢复之后只读 proof，直到全部条件通过才修改局部 body clone；拒绝返回 `None` 时不会部分删除 statement。成功后 loop caller 才按返回索引倒序移除 iterator declaration/init row，再 push 新 loop（`build.rs:19236-19277`）。若扩展 Assign，candidate 不应触碰 target 的 declaration/assignment accounting；拒绝路径仍应原样留下 while 与 body，任何跨列表 owner 移除仍只能由当前 iterator helper授权。

对抗性拒绝场景至少包括：Assign 不是 body 首条语句；首条赋值的 BCI/store 与 next store 不同；同名 local 来自不同 reused variable；next 结果被 dup/分支/第二消费者读取；cast 值另有读取；cast 与 next/hasNext 的 handler 区间不同；source 是调用表达式或 source/iterator 在 gap 被重写；gap 中有可观察调用/字段写；iterator 声明是当前列表无法唯一定位的 ancestor；target 声明需移位或 target 是本循环中另一个槽生命周期实例。控制要证明错误形状拒绝，不能静态假定外层目标走正例。

## JADX 对照与未知事实

本地 `/Users/lordcasser/workspace/testzone/jadx/jadx-core/src/main/java/jadx/core/dex/visitors/regions/LoopRegionVisitor.java:246-309` 的 `checkIterableForEach` 先要求条件只有一个 register、iterator SSA var 未用在 phi 且 use list 恰为两个，再识别 `iterator()/hasNext()/next()`；分别处理 wrapped cast 与直接 next result，检查元素变量只在 loop 内使用/赋值，然后隐藏 iterator 和 next 相关指令并建立 `ForEachLoop`。它的 `fixIterableType` 在 `:312-358` 处理可用的泛型类型。对本任务有参考价值的是明确分开 wrapped cast / next 结果路径及单消费者约束，不是照搬它对可变 SSA/debug register 的宽松操作；Jarde 当前 Object 扩展无需引入泛型推断。

实际未知：我没有运行 root 正在使用的固定 71 单元/outer13 同次 IR 诊断，也没有看到该目标的精确 loop body AST、BCI、SSA use 集合、声明 placement 或 handler 集合。因此只能给条件判断：完整 iterator proof 通过后，保留 inline Cast(next()) 的 Assign 在架构上可用最小转换支持；目标是否满足上述 Assign/store/第一条语句与 handler 条件，仍由 root 的实测门决定。既有 main 的实现位置是 `crates/jarde-java/src/build.rs:21480`；没有用旧的 `UpstreamNestedLoops4`（这是另一个目标且现有报告记录为 fallback）冒充当前证据。
