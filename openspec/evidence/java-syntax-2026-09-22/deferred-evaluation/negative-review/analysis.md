# deferred-order 实现边界审查

本审查只读当前 `crates/jarde-java/src/build.rs` 与
`openspec/changes/preserve-deferred-value-order/` 的规划，不运行 Cargo、不改生产代码或
Rust 测试。当前 CLI 固定为父任务给出的
`feed5c377a439490efb12a2a46bdfa0e94c32dffd3a83bd1583b70fe89317350`；本目录没有伪造
JVM 输入或把未执行的结果写成执行证据。

## 已闭合的局部契约

`prepare_deferred_bindings` 先要求生产值有一个 reader、生产者与 reader 在同一
`CanonicalBlockId`，再由 `has_independent_boundary` 检查区间中的非透明指令。普通同块直线
形状因此有清楚的候选边界。`bind_value` 先递归呈现初始化式，再检查
`Expr::presented`，最后经 `push(Declare)` 检查声明确实进入当前语句列表；失败时写入
`binding_refused` 并让后续 `render_value` 返回拒绝。这条失败 Declare/无类型路径不会留下
一个可被后续正文引用的成功 `Binding`。`render_value` 成功后优先读取 `bindings`，因此同一
SSA 值不会在成功保存后再次内联。

## 发现一：跨 region/guard 的证明没有进入候选判据

规划明确要求不能只以 BCI 或 CFG block 代替 Java 作用域，并要求未证明的 guard、循环、
phi/跨 region 保留来源拒绝。当前候选判据在 `build.rs:2175-2207` 只读取
`value_facts.uses().len() == 1`、`reader > last`、`block_of[anchor] == block_of[reader]`
和 `has_independent_boundary`；`BindingPlan` 也只有 `value/anchor/producer/name`，没有
`RegionPath`、guard body range 或循环迭代上下文。

`arm`（`build.rs:2093-2105`）只替换 `self.stmts`，而
`bindings`、`binding_plans`、`binding_refused`、`synthetic_names` 等状态仍跨 arm 共用。
guard 的 `region`（`build.rs:1431-1510`）则依次用 `range(plan.lead())`、
`body_range(plan.body())` 把同一 Builder 的语句切成 header/body。由此可见，当前实现没有
一个源码级不变量能证明“声明所在列表覆盖 reader 且不跨兄弟 arm/guard body”。

这不等于每个合法输入都已经能触发错序：SSA 的局部存储或 phi 往往会先把跨块值变成另一
种形状而被跳过。但当同一 canonical block 覆盖 try/guard 边界时，`block_of` 相等会直接
放行；当不相等时又没有拒绝记录，只会落入下一节的旧延期路径。task 1.2/2.4 要求的是
证明或明确引用，当前代码两者都没有完成。应把 region/guard 可见性作为候选成功条件，
否则在未知位置把该值标记为拒绝并让来源引用覆盖生产者和 reader。

## 发现二：未获候选的未知位置仍回到旧的 inline/deferred 路径

`prepare_deferred_bindings` 对不同 block、多个 use、无 reader、unsupported producer、无
independent boundary 都直接 `continue`，没有写入拒绝集合。之后 `instruction` 的 Invoke
分支（约 `build.rs:2600-2620`）仍执行：只要
`produced_value_reaches_a_reader` 为真，就把 `(value, at)` 放进 `deferred` 并跳过生产者
语句。最终 reader 调用 `render_value` 时，若没有 `Binding`/`binding_refused`，会在 reader
的 `at` 递归重建生产表达式。

因此“无法证明保存位置”与“证明不需要保存”在状态上没有区别。独立语句位于 reader 前、但
producer 到最终嵌套 consumer 的直接 reader 不是 producer 自身时，就会把调用、可变 field、
array、cast 或检查延后到 reader；这正是 spec 的 “unknown position 不得默认透明” 所禁止
的旧错序路径。`composite-boundary` 已把 `call -> iadd/ineg -> mark -> return` 固定成这种
形状；该审查不重复修改其证据，但当前接口仍应对所有未获 plan 的支持 producer 走明确
refusal，而不是继续 `deferred` 后在 `render_value` inline。

同一条件也覆盖了重复消费：候选只接受 `uses().len() == 1`（`build.rs:2178`），但多
reader 值没有对应的拒绝状态。只要某种合法 SSA 形状让一个支持 producer 同时到达两个
可呈现 reader，生产者就会被旧路径记一次 `deferred`，两个 reader 又各自从 producer
重建；即便该形状最终因当前栈复制规则很少出现，代码没有把“多 reader”闭合成一次求值或
来源完整的 fallback。task 1.2 要求的是明确拒绝边界，不能只把多 use 从候选列表静默
删掉。

## 发现三：依赖证明的深度停止没有向上返回“不完整”

`collect_dependency_bcis`（`build.rs:2327-2358`）在
`depth > MAX_VALUE_DEPTH` 或 `seen` 命中时返回 `Ok(())`。调用者只看到一个可能不完整的
`dependency_bcis` 集合，随后仍可以在区间中找到 boundary 并生成 binding plan。真正的
`render_value` 会在同一深度界限返回错误并 fallback，这使部分输入最终保守，但证明阶段已
把“不完整证明”当作正常候选继续走了。spec 要求达到深度限制时有界停止且不能把未完成
顺序证明当作安全内联；这里至少需要一个“不完整/拒绝”结果沿调用栈传播，不能只依赖后续
呈现偶然失败。

## 发现四：新增名字冲突循环未计费

候选阶段（`build.rs:2211-2229`）只为每个候选收取一次 `IrItems`，但
`NameTable::free_name` 以及与 `synthetic_names`、`lambda_params` 冲突时的下划线循环每次
都没有 `poll`/`charge`。`free_name` 本身也遍历完整 reserved/name 集合。若已有名字制造冲突，
实际工作量可以随冲突数量和集合大小增长而不出现在新增 binding 的预算中；这与 task 2.5
要求候选、绑定条目和命名冲突共享既有预算不一致。失败的 Declare 路径虽不会发布 Binding，
但已经消耗的命名工作同样没有被计量。

## 结论与边界

失败声明和成功绑定的单值提交顺序目前是可读的局部闭环；本审查没有把它误报为已经丢失
声明名。尚未闭合的是证明失败后的状态：跨 region/guard、直接 reader 之外的嵌套依赖、
深度截断和未计费命名都不能默认为安全 inline。以上边界应在现有 Builder 状态上补成
“成功证明才建立 Binding；否则保留完整来源的拒绝/引用”，不需要新 CFG/SSA/pass 或扩大
guard 准入。

## root 审读限制

此文是初版实现的静态检查，不把“缺少某字段/集合”本身等同于已执行的作用域错误。生产选择可用既有区域事实完成证明，不要求为了审查意见新增RegionPath副本或作用域注册表；具体switch反例另行实测。相同block是否覆盖一个安全呈现区间，需按既有区域构造和消费时机核实。

“所有未获plan的producer都拒绝”过宽，不作为实现指令：完整证明可直接有序内联的嵌套调用必须保留，只有未能完成内联/保存证明时才明确拒绝。保存不是每个表达式的必要前提。共享DAG节点再次出现在seen中也不必然意味着证明不完整；已完成的子树可正常去重，真正的深度截断/未处理节点才需要传播未完成事实。root已经执行确认的错误目前是composite-boundary的15项中11项，不能把本审查列举的每条风险虚报成独立运行反例。
