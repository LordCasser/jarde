# CF11 literal-seeded reference write compatibility（私有架构审查）

## 范围与判断

仅审查 `noCastAssign(Ljava/util/List;)Ljava/lang/Object;`：先把字符串字面量写入局部 `target`，后把 `Iterator.next()` 的 Object 写入同一局部，再将 merge 后的值作为 Object 返回。问题是首写的 String 目前成为局部声明类型，后续 Object 写入虽仍被归为 reference，却不能生成可编译的赋值。

窄结论：可以在 `decide_types` 已有的逐写扫描里识别“同一个 `LocalVariable` 的所有已知物理写入都是 reference”，并在此类局部需要容纳更宽 reference 写入时选用 `java.lang.Object` 作为赋值兼容类型；这不需要新建 LUB 服务或 identity 服务。只对 body local 生效，并沿用 `uses` 已有的 `LocalVariable` 键、path、budget 和 stop。`Object` 是所有 Java reference 的已知公共上界，不是一般 LUB：不能据此推断任何子类或接口共同类型。

这个候选必须再收窄到读点也能在 Object 声明下成立的形状。当前 fixture 的 merge load 直接送到 Object 返回，恰好满足这一点。不能把“全是 reference writes”单独当成所有局部都能声明 Object 的充分条件；成员访问、需要特定接口/类类型的调用参数、数组操作等读点会因静态类型变成 Object 而失效，除非现有输出在该位置已有独立、可靠的窄化证明。未作运行时检查，也不建议借写入类型推断插入未经证明的 cast。

## 已观测事实（诊断原始材料，不代表验收）

材料来自 `/private/tmp/jarde-cf11-local-type-debt-observation-run-root-v2/0.stderr.raw`。日志包含 test harness 的 `Finished test` 与该方法输出；但此次运行最终 free memory 为 `5,352,644,608` bytes，低于 5 GiB guard，root 已标记 resource-failed，因此以下只作为 raw observation，不是合格 CI/test 验收证据，也没有据此修改共享代码。

- 方法描述符为 `(Ljava/util/List;)Ljava/lang/Object;`。诊断 LVT 行给出 slot 1、name `target`、range `(3,31)`、descriptor `None`。这个 `None` 是 facade `DebugLocal::descriptor` 的值，不能当成 raw classfile LVT descriptor；本审查不使用 debug descriptor作为类型证明。
- 三次 slot 1 access 共用 `LocalVariable { slot: 1, index: 0 }`：BCI 2 store，BCI 25 store，BCI 29 load。该同一 identity 的边界摘要在三处均落在相同 identity。
- BCI 2 的 stored `ValueId(4)` 是 `Ref(Unknown)`，producer 为 BCI 0 string push；write result `ValueId(5)` 也为 `Ref(Unknown)`。当前选型 `Some(Type(Reference("java.lang.String")))`，access_count=3。
- BCI 25 的 stored `ValueId(12)` 是 frame-named `java/lang/Object`，来自 BCI 20；store result `ValueId(13)` 也是 Object。该值的 consumer 是 BCI 10 merge。
- BCI 29 load 读取 Phi `ValueId(2)`；phi 位于 BCI 10、slot 1，type `Ref(Unknown)`，两输入分别为上述 String producer 与 Object producer，唯一 consumer 是 BCI 29 load。
- 日志生成的局部为 `java.lang.String target;`，随后 `target = "seed";` 与 `target = iterator.next();`，最后 `return target;`。raw 输出带有 “presentation is not claimed to compile”。

## 源码事实

- `decide_types` 按 `uses` 中同一 `LocalVariable` 的写入扫描并为每个写支付 `AnalysisSteps`、poll budget；它同时统计完整 path 与 first write。first write 当前是两条声明路径共用的类型种子（[`build.rs:2434-2512`](</Users/lordcasser/workspace/projects/jarde/crates/jarde-java/src/build.rs:2434>)）。
- body local 的默认类型来自特殊 proof 或 first stored value 的 `written_type`；参数则直接以 method descriptor 类型定型，源码注释明确写明 body 的写不能 widen parameter（[`build.rs:2655-2669`](</Users/lordcasser/workspace/projects/jarde/crates/jarde-java/src/build.rs:2655>)、[`build.rs:2853-2871`](</Users/lordcasser/workspace/projects/jarde/crates/jarde-java/src/build.rs:2853>)）。
- 已有冲突复查循环再次遍历所有写，但只拒绝 reference/primitive 类别冲突，不比较 String 与 Object 的 reference assignability（[`build.rs:2873-2909`](</Users/lordcasser/workspace/projects/jarde/crates/jarde-java/src/build.rs:2873>)）。因此 String 与 Object 都是 reference 时，first-write String 决定仍保留。
- `direct_reference_write_type` 能从 String/Class 常量、调用返回 descriptor、field descriptor、allocation 和 array creation 中取精确写入类型；它刻意不跟随 Load/Store、Duplicate、Phi 或任意 Unknown 链（[`build.rs:3008-3061`](</Users/lordcasser/workspace/projects/jarde/crates/jarde-java/src/build.rs:3008>)）。`null_leading_reference_type` 只处理 first stored value 真正为 null 的场景，要求每个非 null write 独立给出相同 exact reference type；本例首写不是 null，故不适用（[`build.rs:3063-3106`](</Users/lordcasser/workspace/projects/jarde/crates/jarde-java/src/build.rs:3063>)）。
- `written_type` 按 known array/constant facts 优先，否则读 frame `value_type`；`value_type(Ref::Unknown)` 和 `Null` 都给出 `Object`。注释及 `reference_lifetime_type` 明确指出该 Object fallback 可供普通声明输出，但不是 reference lifetime 的证据；后者只接受 named frame type，或由 array/direct constant 精确补出的 Unknown（[`build.rs:32107-32126`](</Users/lordcasser/workspace/projects/jarde/crates/jarde-java/src/build.rs:32107>)、[`build.rs:32128-32152`](</Users/lordcasser/workspace/projects/jarde/crates/jarde-java/src/build.rs:32128>)、[`build.rs:33735-33747`](</Users/lordcasser/workspace/projects/jarde/crates/jarde-java/src/build.rs:33735>)）。本案可以用“验证器给出的 reference 类别 + Object assignment 的普遍性”讨论局部赋值兼容性，但不得把 `written_type` 的 fallback 反说成精确 producer type、source lifetime 或 LVT 证据。
- 写入端 `write_statement` 用同一个 decided type 执行 `meeting_position`；它认可 reference 到 reference 的位置赋值，不会额外比较子类关系。之后 `assignment` 发布声明后的普通写（[`build.rs:25540-25610`](</Users/lordcasser/workspace/projects/jarde/crates/jarde-java/src/build.rs:25540>)、[`build.rs:25613-25687`](</Users/lordcasser/workspace/projects/jarde/crates/jarde-java/src/build.rs:25613>)、[`build.rs:30873-30920`](</Users/lordcasser/workspace/projects/jarde/crates/jarde-java/src/build.rs:30873>))。对本案 String 与 Object 写入，声明 Object 即可由 Java reference assignment 接受。
- receiver/call 参数有各自的 descriptor-aware 规则。`invocation_argument` 对 Object→特定 reference type 不以“二者都是 reference”为准，会要求相同类型或有闭合层级证明；未知关系拒绝（[`build.rs:27542-27669`](</Users/lordcasser/workspace/projects/jarde/crates/jarde-java/src/build.rs:27542>)）。已有层级规则是刻意有限的 release-specific table，不是全局类型层级/LUB（[`build.rs:31286-31315`](</Users/lordcasser/workspace/projects/jarde/crates/jarde-java/src/build.rs:31286>))。因此不可把普通写入兼容修复扩大成无条件 Object 声明并假定调用仍可编译。
- `LocalVariable` 身份是 `(slot,index)`；reuse plan 按 BCI 把访问放到拆分后的变量，不能只按 slot 合并类型。reuse 的拆分依据是无歧义 LVT scopes 或窄 SSA/CFG 生命周期证明，也可在没有 debug 表时执行；证明不足时保留为一个变量（[`names.rs:396-429`](</Users/lordcasser/workspace/projects/jarde/crates/jarde-java/src/names.rs:396>)、[`reuse.rs:1-38`](</Users/lordcasser/workspace/projects/jarde/crates/jarde-java/src/reuse.rs:1>)、[`reuse.rs:84-101`](</Users/lordcasser/workspace/projects/jarde/crates/jarde-java/src/reuse.rs:84>))。任何新增候选必须使用现有 identity 和其 complete paths，不能按名字、slot 或 LVT descriptor 另作归并。
- 类型分析按 write 逐项收费和 poll；冲突循环也再次收费/poll。当前 `decide_types` 以 `Result<..., StopReason>` 传播 stop。若将 reference compatibility 合入现有循环，应维持每次处理前的收费/poll、path completeness、handler/catch 行为和 stop传播，不缓存一个只由 LVT 支撑的部分答案（[`build.rs:2456-2468`](</Users/lordcasser/workspace/projects/jarde/crates/jarde-java/src/build.rs:2456>)、[`build.rs:2873-2894`](</Users/lordcasser/workspace/projects/jarde/crates/jarde-java/src/build.rs:2873>))。catch value 有独立 header，当前冲突复查明确跳过 `Definition::Caught`；新检查需维持该特殊语义并重新审查 handler joins，不能把缺失写入误当成 complete evidence（[`build.rs:2887-2894`](</Users/lordcasser/workspace/projects/jarde/crates/jarde-java/src/build.rs:2887>)).

## 边界反证

- **成员调用/receiver：** 声明 Object 后 `target.member()` 无法仅凭原先 String write 保证可编译。receiver 可能需要某个 class/interface 的静态类型；插入窄化 cast 还可能引入 class-cast failure。只有实际 receiver renderer 已持有同一 BCI 的可靠 owner/widening 证明时才可以让 Object 类型通过，否则拒绝该写入形状。
- **接口或窄参数：** Object 局部传入 `takes(SomeInterface)`/`takes(String)` 不满足 Java 静态调用类型。当前 argument gate 要求证明 widen，Object 到窄类型没有此证明；不得由“曾经有 String 值”给 invocation 补 cast，因为控制流 merge 的另一路可为其他 Object。
- **数组：** Object 局部不能 `target[i]` 或访问数组成员。现有数组 covariance 逻辑依赖明确 array shape/rank/element；把一组 reference 写入统称 Object 会丢掉 shape。数组写入可因 runtime covariance 抛异常，不能与标量 reference 的赋值兼容合并处理。
- **Reference Unknown：** Unknown 表示 frame 没有具体 class，不等于某个可任意使用的具体类。它足以说明 Object 引用赋值的类别边界，但不能支持窄类型、生命周期分割、成员调用或接口选择。Null 也只在 Java null assignment context 下兼容，并不单独提供公共子类类型。
- **参数：** 参数类型来自 method descriptor，改为 Object 会改变公开/override 方法签名；当前代码已经显式固定它。本窄规则必须只对非参数 body local 应用。
- **slot reuse：** 不同 `(slot,index)` 代表分离的 source locals；slot 上可能曾有 String var 与后续 Object var。若 reuse 已证明 split，就分别处理；若证明没有完成，则不能拿 LVT 单行/descriptor 替代 missing identity proof。
- **handler 与 stop：** catch header 和 ordinary slot write 不同；已有冲突逻辑跳过 caught definitions。若 path 缺失、Phi 输入不全或 budget stop，不能宣称“所有写均 reference”。失败/停止应沿现有 `StopReason` 路径退回，不应部分应用 Object 类型。

## 源码推论与最小闭环建议

1. 本案真正的故障是“首写是 String”被直接拿来作整个局部的声明类型，而 later Object reference write 只参加类别冲突检测。现有循环已经枚举所有同一 `LocalVariable` 的 writes、逐项 poll/charge，并且能判断写值是否 reference；类型系统已存在 universal `java.lang.Object` 的 source spelling。这个特定 String + Object 的目标形状可在现有循环/当前 key 上把候选兼容声明类型从 first String 提升为 Object，不需要层级查询，也不需要给 Phi 推导 LUB。
2. 不能将第 1 点推广成“所有仅 reference writes 的局部都声明 Object”。更小的闭环是：仅当某写入确实比 first-write candidate 更宽/不可被 first type 接收时考虑 Object；条件限制为 body local、所有已知写入均为 reference、path/write evidence 完整、没有 primitive/null 特例误用，并且该变量各实际读点都能以 Object 静态类型输出。若现有 read position 已有明确 target descriptor，可按那一处的既有规则接受/拒绝；若需新增全局 consumers scan 或隐式 casts 才能使调用成立，则这超出当前窄片。
3. no-debug 必须依赖 `uses`/SSA/Operations/CFG 与 `LocalVariable`，不能把 LVT 的 raw descriptor、`DebugLocal::descriptor()` 或 LVT name 当类型证明。fixture 应同时有 debug 与 `-g:none` 形式，验证决定相同；名字在 debug 版按既有来源输出，无 debug 版允许 ordinal 名称。
4. Java 8 最小 fixture 建议一个类含三个小方法：正向 `noCastAssign(List<?>)`（String seed → Object `next()` → Object return），无 debug 的同形版；两个必须拒绝或仍保持原式的控制：reference-only 局部先 String 后 Object 但以窄 receiver/窄参数消费，以及 reference-array 局部有数组读/写。再加一个参数 slot 与 caught/handler 写的控制只有在现有 fixture harness 可直接表示、不扩成新的验证面时才放入本片。正向没有必要引入自定义接口层级、源码通用 LUB 或新 metadata。

## Atlas 与验证范围

Atlas `project(open)` 已打开该仓库，scoped search 对 `crates/jarde-java/src` 的 `decide_types`、`direct_reference_write_type`、`null_leading_reference_type`、`written_type`、`reference_lifetime_type` 触发了 Focus，但查询仍处于 `focus_structural_not_ready`，没有发布局部结构结果。故本报告不把 Atlas 未完成查询当作源码证据；以上引用由 scoped 源码逐段核实。只读 `/private/tmp` 与指定 root raw 文件，没有运行任何工具链/CI，没有编辑共享工作区。前一阶段提到的 roadmap 规模（71/612 fixed）作为 root 给定上下文；本审查不据此声称完成度或提出大路线图。
