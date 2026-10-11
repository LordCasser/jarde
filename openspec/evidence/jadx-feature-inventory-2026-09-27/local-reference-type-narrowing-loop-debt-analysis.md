# CF-11 `noCastAssign` 类型债务静态分析

## 结论

这是一个独立于 outer foreach 的现有局部类型决策债务。最小处理应另开窄片，扩展 `declarations` 里的 `decide_types`，让它能采用已存在的局部声明类型证据，而不是让“最早一次写入值的类型”决定整个 local 的源码声明类型。无需引入新 pass、类型服务或变量身份服务；当前 outer-foreach 改动不应吸收这项修复。

v2 通过把 `noCastAssign` 的首写从 String 常量改成 `Runner.seed(): Object` 改变了类型证据，不能算作 v1 类型债务已经修复。v1 的 `LocalVariableTable` 和 loop-header `StackMapTable` 都明确给 slot 1 `Object`；Jarde 仍将它输出为 `String`。

## 固定控制证据

`baseline-v1/capture/execution.json` SHA256 为 `27ec2eed14e3a547f274ae8eac4dae9fd91b9be549801a9e027cf9c1dfecdad6`，29 命令、58 个 raw stream。原始 javac8/javac23 两组与 JADX 四组编译和运行均为 0。Jarde default/all、combined/isolated 在两个 JDK 上合计 8 次编译均退出 1，未运行；每次都在 `target = iterator.next();` 报“Object 无法转换为 String”。

v1 javap 的 `noCastAssign(List):Object` 是：BCI 0 `ldc "seed"`，BCI 2 `astore_1`，loop header BCI 10，`Iterator.next():Object` 位于 BCI 20，BCI 25 再写 local 1，BCI 29/30 读出并返回。LVT 给 slot 1 `target Ljava/lang/Object;`，区间 `[3,31)`；StackMapTable 在 offset 10 的 append frame 将 slot 1 声明为 `java/lang/Object`。因此最早的 String 初始化 store 恰在 LVT 区间起点前一个 BCI，而声明类型和循环合流类型都是 Object。

`baseline-v2/capture/execution.json` SHA256 为 `07dd9ca9076993df96a2b00c481d656b059eb95558766732e77cd006cd04d783`，也有 29/58。四组 Jarde default/all × javac8/javac23 都编译、运行 0。v2 的 `noCastAssign` 改为 `target = Runner.seed()`，其 descriptor 返回 Object；它的 BCI 0 首写因此已是 Object。v2 javap 的 LVT 与 StackMapTable 仍为 Object。这个控制只证明首写为 Object 时旧策略通过，不能反证 v1 的首写优先问题已消失。

固定类证据 SHA：v1 javac8 class `f593c7a42c8326f6e030f437242354e564ee88fb2918512392e38e38074aaf93`；v2 javac8 class `28b92d58ce18fb08362bd39b0558de98c3c41830cc8cbc6bd60ec3a183b21956`。

## Builder 路径（Atlas 结构事实）

Atlas 1.6.1 已打开项目。对 `crates/jarde-java/src/build.rs`、`reuse.rs` 和 `names.rs` 的窄文件查询返回完整结构事实；当前项目 Atlas 状态为 structural，62 个文件 structural fresh，`lazy_dataflow.has_dataflow=false`。因此下面的 Builder 路径有 Atlas 源码结构依据，但没有 Atlas variable-trace/DF 证据可证明具体 SSA phi 的 type 内容。一次较宽的 `crates/jarde-java/src` search 一直停留在 Focus pending，后来过期；没有使用其部分结果。

- `declarations` (`build.rs:703`) 先用 `slot_uses(ssa, operations, reuse, ...)` 收集指令读写，再调用 `decide_types`，随后才调用 `prove_debug_generic_locals(debug_locals, ..., &decided, ...)`。普通 `debug_locals` 不作为 `decide_types` 的参数；泛型 debug 投影发生在类型决定之后。
- `slot_uses` (`build.rs:1927`) 从 SSA block instructions 的 `reads()`/`writes()` 建立每个 `LocalVariable` 的 `SlotUse`，用 `reuse.variable_at(slot, bci)` 归属变量。`SlotUse` (`build.rs:3631`) 保留 region path、BCI、read、written、stored；注释明确 `stored` 是 write 实际读取/存储的值，在普通 astore 路径可与被写 slot 的 SSA value 不同。
- `decide_types` (`build.rs:2433`) 对每个变量只从 `written.is_some()` 的记录中按 `(path, bci)` 选最早一次写作为 `FirstWrite`。引用类变量的类型最后优先来自专门化证据，否则从这次写调用 `written_type` (`build.rs:32105`)；它尝试 array、constant，再读 `ssa.value(value).ty()` (`value_type`, `build.rs:33726`)。之后扫描其它 writes 时只把 reference 与 primitive 之间的类别冲突标为 `Unknown`，不会在 String 与 Object 两种 reference 之间上推或合并。
- 对 v1，首写 String literal 使决定落为 `java.lang.String`；后写 `Iterator.next():Object` 仍属 reference，所以不触发当前冲突门。`Builder::write_statement` (`build.rs:25539`) 和 `decided_type` (`build.rs:26141`) 消费同一决定；`Builder::assignment` (`build.rs:25621`) 对普通非 boolean reference 值直接保留 `Assign`，不会插 cast 或将既有 String 决定改宽。实际产物因此保留 `String target` 与 `target = iterator.next()`，触发 javac 的 Object→String 错误。
- `write_target` (`build.rs:25732`) 只通过 `reuse.variable_at(slot, bci)` 取得 local identity，再从 `NameTable` 取名字。Atlas 的 `reuse::Plan::variable_at` (`reuse.rs:87`) 按 slot/BCI 选择 whole 或 split local。`SlotEvidence` (`names.rs:436`) 的形状是 `Unnamed`、`Whole(String)`、`Split(Vec<Option<String>>)`；它保存的是名称证据，不包含声明类型。它的注释也明确一种 inferred lifetime 可以早于该 slot 唯一 LVT 记录。
- LVT/StackMap 证据在本例并不缺失：LVT descriptor 和 StackMap merge 都给 Object。问题是当前普通局部类型决策不消费 LVT descriptor，也没有把 loop-header local merge 作为首写类型的约束。Atlas 的结构结果只能确认 `decide_types` 的消费路径；不能据此断言 SSA 的 phi `Value.ty()` 一定等于 Object。

## 最小后续方向

建议将该债务拆成单独类型决策窄片，只扩展现有 `decide_types` 的 reference-local 决策：在最早写类型之外，考虑同一 `LocalVariable` 已有的显式 LVT descriptor/范围证据，并用所有写入验证该声明确实可承载；v1 的 Object LVT 因而胜过首写 String literal。这个 class 的初始化写在 LVT start 前一个 BCI，需要规则明确允许“紧邻 LVT 起点、同 slot、同 reuse local identity”的初始化写归属；不能把任意相邻或 slot 重用的 debug entry 当成类型声明。

StackMapTable 的 Object loop-header frame 可作为验证合流类型的独立佐证，但不建议单独把粗略 verifier join 当成所有局部的 Java 声明类型：它可能比源类型更宽。LVT descriptor 是这个固定源变量的直接声明证据；若 LVT 缺失或范围/identity 不完整，则保留现有策略或拒绝精化。不要修改 iterator proof、foreach candidate、局部身份切分或 generic Signature 路径。

回归应保留 v1 原样 `Object target = "seed"; target = iterator.next();`，要求产物以 Object 声明并在 javac8/javac23 编译运行；再保留 v2 `Runner.seed():Object` 与 existing String-target control，确认两个来源都不会混淆。当前分析没有修改测试、产品文件，也没有运行任何构建或工具链。
