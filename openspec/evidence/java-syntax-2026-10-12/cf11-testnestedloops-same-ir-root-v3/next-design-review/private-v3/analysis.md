# CF11 foreach 声明归属下一片：内层候选与拒绝边界

本稿只读源码与已有冻结运行证据，供后续 spec 参考。它不是实现，也没有重跑任何测试、工具链或运行 candidate。静态判断不冒充新的 same-IR trace。

## 已观察到的两个首拒

固定原始输入是 `cf11/UpstreamNestedLoops.test(Ljava/util/List;Ljava/util/List;)V`。baseline execution 为 `/private/tmp/jarde-cf11-testnestedloops-baseline-root-v1/execution.json`，SHA-256 `5635e60b5565a19ba4fab116d0b6ae73f72e62568e0761edeff2365291c95435`；JDK 23 原 class SHA-256 `8ece89e4d38751c20529622b57417afc8601368fc20078656ea2b24c17ed8231`。真实 Jarde default 输出在同 baseline 的 `jarde/javac23-jarde-default-UpstreamNestedLoops/UpstreamNestedLoops.java`，SHA-256 `e24607d81410e4d673b4d4d020c3eb19c1ebc0f417e0e2eb564a33bb825bfaf1`。它显示 `Iterator local5;` 在外层 while 之前，`local5 = l2.iterator();` 在外层 body 内、内层 while 之前；这证明声明与初始化语句不在同一输出位置。它不单独暴露运行期 `DeclarationPlacement` 的 `RegionPath`，该身份仍应从 Builder 本轮 facts 核验。

已有 guarded same-IR diagnostic v3 的 stderr raw `/private/tmp/jarde-testnestedloops-same-ir-diagnostic-root-v3/0.stderr.raw` SHA-256 `dd853ee46eff134645137cf80380d27418e9d64701c9ae9f53b7195785b1c850`，记录了准确顺序：

- BCI 42，`path=[1, 1]`，while，首 body statement 为 `Declare`，拒绝于 `post_entry_candidate_proof_incomplete`，子阶段 `iterator_declaration_before_init`。
- BCI 13，`path=[1]`，while，首 body statement 为 `Assign`，拒绝于 `body_first_not_initialized_declaration`。

因此，内层候选到达了声明查找，而外层候选甚至没有进入它。两个首拒不是同一个声明问题。此次设计只覆盖内层；外层的元素赋值 `s1 = (String) local3.next()` 保持现状，不因内层放行而改写。

## 最小设计界限

现有 `Builder::iterable_for_each_candidate` 在 `crates/jarde-java/src/build.rs:21224`。它先要求首 body statement 是带初始化值的 `Declare`（约 21242–21249），随后在当前 `self.stmts[..init_index]` 中反向找 `Iterator` 空声明（约 21356–21361）。真实内层形状的空声明已由更外层词法列表输出，当前列表只持有 `local5 = l2.iterator()` 及被构建的 loop，所以这个同列表索引不是合适的 owner 证明。

最小候选方向是：

1. 只对当前 init Store 的 slot 求已有 `reuse::Plan::variable_at(slot, init_store_bci)`，并核对 hasNext/next 两处 load 仍映射到同一个 `LocalVariable`。slot/name 相同不足以证明相同变量；`reuse::Plan::variable_at` 的 split-slot 行为见 `crates/jarde-java/src/reuse.rs:89`。
2. 用已有 `Declarations.placements` 和 `at_region` 证明该 `LocalVariable` 有完整、可见的词法 declaration owner，拒绝 `Incomplete`；如 placement 是 `Elevated`，还要求 owner 的 `at_region` 确实列出该变量。定义与验证见 `build.rs:441–478`、`1850–1912`。声明发射在 `Builder::declare_at`（约 `22662–22685`）。本片不依据名字搜索父 AST，也不扩张 planner 的类型或 reuse 推断。
3. 删除范围只限当前 `self.stmts` 里的已证明 iterator 初始化赋值。**不**把父 frame 的声明索引传给当前 caller，不跨 frame 删除、搬动或重写 declaration。父级 `Iterator local5;` 即便成为无用局部，也可保留；死声明清理单独处理。
4. 保留现有 `Object` foreach header 和 body cast。成功变换只将原 `next()` expression 节点替成 `Object` synthetic local；原元素绑定与 cast 留在 body（变换见 `build.rs:21684–21716`）。不增加 generic Signature、iterable element type 或 cast 推断。

这里的 owner check 是现有 plan 身份的消费，不是“看见祖先同名 `Iterator` 声明便接受”。需要确认当前父列表的 emitter 确实已安排同一 `LocalVariable` 的声明，并确认 owner 路径包住 candidate 的 init 与所有原 iterator 使用。若现有 facts 不能从 candidate 上下文证明 owner 可见，应继续拒绝，先增窄诊断，不把 path/name 猜测当成证明。

## 放宽声明门后必须实测的剩余证明

诊断只观察到首拒，不知道下面任一 gate 对这份 IR 的实际结果。后续若把声明归属门放宽，需用同一个冻结 class 与同一 Builder/Program 把第一个后续拒绝记录出来；不能从源文本或此前候选代码推断它们已通过。按现有 candidate 顺序，至少检查：

- **局部序列与位置**：`iterator()` 初始化语句到 inner while 前只有允许保留的无副作用 literal/declaration；init Store、`iterator()`、hasNext、next 的物理 BCI 与所消费 statement 一致。项目内 `self.stmts` 其他 statement 不得被误删除。
- **调用与 slot 身份**：接口 owner/descriptor 精确为 Iterable/List/Collection 的 `iterator()Iterator`、Iterator `hasNext()Z`、`next()Object`；init/test/next 的 Load/Store 指向同一个 reuse variable，无 split-slot 串线。
- **初始化结果和控制流**：iterator 调用返回值只有 init Store 一个 consumer；测试 header block 没有别的未建模指令；next block 在 next 前没有会被 foreach 改变次序的指令；header Phi 的输入必须恰为初始化值与自身回边，并且没有其他 iterator-slot 写。要记录 Phi/value IDs、inputs、uses（含无 BCI phi-use）和 block/path，而不只记录“phi 通过/失败”。
- **读写与唯一消费**：hasNext 和 next 都从同一 iterator value 读取；`next()` 结果只到原 cast 或原 element Store；cast 输出只到该 Store；元素 Store 的值只有原 cast-load reader。改写不可额外调用 next、删除 cast 或改变读取位置。
- **异常与 effect**：初始化、hasNext、next、loop test（以及若适用的 cast）具有 candidate 要求的相同 handler 集；记录各 instruction 的实际 handler/effect 与第一条不相等处。证明未完成时保留原 while。
- **AST/词法与结果**：内层 body 仍绑定既有 `s2` cast/类型；Object header 只替代原 next 位置；只从其当前 owner 列表删去 initializer；父列表声明、outer while、outer `s1` Assign、之后的 list mutation/cleanup 全部按原结构。核 default/all 的 RuleDetails 与最终文本保持同一 site decision。

这些 gate 分别对应当前实现约 `21363–21373`（序列与重复 slot）、`21380–21440`（BCI/call owner/descriptor）、`21442–21570`（slot、SSA、CFG、Phi 与 reader）、`21571–21634`（next/cast/store 消费者）、`21635–21642`（handlers/effects）。准确行号随代码改动可能变化，应以对应源码 pin 为准。

## 必须保留的对抗控制

- **inner positive target**：BCI 42 从 `iterator_declaration_before_init` 继续后，应能看到后续每个 gate 的实测 verdict；最终只有在全证书通过后才允许投影。不能因仅证明父声明存在就先返回 foreach。
- **outer negative target**：BCI 13 首 statement 是 Assign，继续拒绝 `body_first_not_initialized_declaration`；inner 的 owner 放宽不触碰 `s1` next/cast，也不把 outer candidate 扩成产品范围。
- **same-name/split-slot refusal**：slot 在 init/test/next 映射为不同 `LocalVariable`、reuse 返回 None、声明只靠同名字符串匹配或有多个可见 owner 时拒绝。
- **incomplete/quoted/exception owner refusal**：placement 为 `Incomplete`，owner 不包含所有 consumer，跨 fallback 或 catch/handler 所有权未证实时拒绝；不允许为拿到可见声明而跨 guard/handler 迁移声明。
- **extra iterator write/read**：header 外的额外 iterator-slot 写、初始化值第二消费者、hasNext/next 使用不同 iterator、next 结果多消费者或 cast/store 不是唯一链时拒绝。
- **order/effect negative**：header/next block 出现额外指令，关键调用 handler 不同，或 iterator() 不在原 inner-loop 入口求值时拒绝；不能把 iterator 创建提前到 outer-loop 外，也不能延后到改变 `s1`/list mutation 顺序的位置。
- **generic boundary**：普通 generic source refusal 保持原样。Obj header + body cast 的既有结构不要求恢复 List<String>，也不构成恢复它的证据。

## 确认与未确认

已确认：冻结 generated source 中 `local5` 的 hoisted declaration 与 inner-body iterator assignment 分离；same-IR v3 观察了 inner BCI 42 和 outer BCI 13 的准确首拒；当前 candidate 在声明门之后已有完整的一组 SSA/consumer/effect 检查；现有 planner 使用 `LocalVariable`、`DeclarationPlacement` 和 `at_region` 表达身份/owner。

静态推断：内层目标可以由已证明 owner 读取、仅删当前 initializer 语句来继续尝试；保留一个未使用的 hoisted Iterator declaration 不改变语义。这个推断尚未在放宽后的 same-IR Builder 上验证。

未知：inner 42 越过 declaration gate 后的首拒/是否全部通过；candidate 上下文是否能直接映射到 `at_region` owner；生成 foreach 后 exact class compile/runtime、default/all report 是否与文本同意。只有实际诊断与后续构建/运行能回答这些问题。
