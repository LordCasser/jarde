# 局部 reference 写兼容：下一片最小设计建议

## 目标与证据

只解决 body local 的 reference 写入导致声明类型过窄这一编译障碍。当前控制 `noCastAssign(List)Object` 先把 String seed 写入 `target`，再把 `Iterator.next(): Object` 写入同一局部，merge 后返回 Object。

已接受的同 IR 诊断证据是 [`0.stderr.raw`](</private/tmp/jarde-cf11-local-type-debt-observation-run-root-v3/0.stderr.raw:68>)，其接受记录 [`acceptance-root-v5.json`](</private/tmp/jarde-cf11-local-type-debt-observation-acceptance-root-v5.json:1>) 的状态为 `observed_same_ir`，范围声明为诊断观察，不是产品修复或正确性验收。raw 显示：同一 `LocalVariable { slot: 1, index: 0 }` 有 BCI 2 String seed store、BCI 25 Object store、BCI 29 load；Phi `ValueId(2)` 的消费者是该 load。补充记录明确 Load 输出 `ValueId(14)` 在 BCI 30 被 `Return` 消费，方法返回 descriptor 是 Object（raw 73–75、68 行）。当前结构化输出仍声明 `String target`，再写入 `iterator.next()`。

诊断的 LVT facade 行 `descriptor=None` 不代表 raw LVT descriptor，也不作为类型依据。建议规则完全由 body 的 SSA、操作、reuse identity、return descriptor 和访问路径决定，因此应可在 `-g:none` 输入上执行。

## 建议的最小规则

仅对非参数、非 catch/header 声明的 body local 考虑。保留现有 boolean/char、nullable、数组创建、generic-signature 等专门类型证明优先级；仅当这些路径没有决定类型时，才检查 reference write compatibility。

沿用 `decide_types` 已有 `uses: BTreeMap<LocalVariable, Vec<SlotUse>>`：它由 SSA 每条局部 read/write 按 BCI 经过 `reuse.variable_at` 分组，包含 path、read ValueId、written ValueId 和写入栈值（[`build.rs:1934-1968`](</Users/lordcasser/workspace/projects/jarde/crates/jarde-java/src/build.rs:1934>)、[`build.rs:3630-3647`](</Users/lordcasser/workspace/projects/jarde/crates/jarde-java/src/build.rs:3630>))。不新增 identity 或 LUB 服务。对同一 identity 的所有物理写入，只在路径完整、所有写入均为 reference category、候选不涉及 primitive/null/caught 特例时，允许选用 `java.lang.Object` 作为最保守的可赋值声明类型；至少一个明确写入类型必须与 first-write 类型不同，避免无差异地扩大已有声明。Object 只是 reference 的通用上界，不推断具体父类/接口，也不改变 Phi 类型。

读端必须是闭合证明：逐一找出该 identity 的每个局部 read，沿其 SSA Load 输出检查所有 consumer。第一片建议只接受每个 read 都直接进入返回 descriptor 为 Object 的 `Return`；其他接收位置先拒绝候选并走既有 fallback，避免把“已接受的 Object 上界”泛化成未经验证的 consumer 规则。不要因为 reference 类别相同而假定所有调用 receiver、参数或数组操作在 Object 声明下成立。已接受诊断提供了正向所需的一个实例：Load 29 输出 14 → Return 30，方法返回 Object。

具体上，当前代码的 first write 是两条声明路径共用的种子（[`build.rs:2447-2512`](</Users/lordcasser/workspace/projects/jarde/crates/jarde-java/src/build.rs:2447>)）；普通推断的 fallback 使用 first stored 的 `written_type`（[`build.rs:2853-2871`](</Users/lordcasser/workspace/projects/jarde/crates/jarde-java/src/build.rs:2853>)）。后续复查只检测 reference/primitive 类别冲突，不检测 reference-to-reference 写入是否能填入声明（[`build.rs:2873-2909`](</Users/lordcasser/workspace/projects/jarde/crates/jarde-java/src/build.rs:2873>)）。最小实现可把 candidate 计算并入现有逐写工作，复用已有计费和 stop；读端扫描须同样逐访问 charge/poll，在 `StopReason` 时停止且不发布部分决定。

## 可复用规则与必须保持的边界

- `direct_reference_write_type` 只从直接 producer 取得精确类型，如字符串/类常量、调用返回 descriptor、field descriptor、allocation、array creation；它有意不跟随 Load/Store/Phi（[`build.rs:3008-3061`](</Users/lordcasser/workspace/projects/jarde/crates/jarde-java/src/build.rs:3008>))。可复用它确认确切写入类型，不能扩成通用值图遍历。
- `null_leading_reference_type` 是首写确为 null 时的既有专门规则，要求每个非 null 写入都独立给出同一精确 reference 类型（[`build.rs:3063-3106`](</Users/lordcasser/workspace/projects/jarde/crates/jarde-java/src/build.rs:3063>))。本规则不重定义 null-leading 行为。
- `written_type` 会把 `Ref(Unknown)` 和 null 的 frame spelling 落到 Object；`reference_lifetime_type` 明确说这个 Object fallback 可用于普通 emitter 声明，却不是生命周期边界证据（[`build.rs:32114-32132`](</Users/lordcasser/workspace/projects/jarde/crates/jarde-java/src/build.rs:32114>)、[`build.rs:32133-32152`](</Users/lordcasser/workspace/projects/jarde/crates/jarde-java/src/build.rs:32133>)、[`build.rs:33735-33747`](</Users/lordcasser/workspace/projects/jarde/crates/jarde-java/src/build.rs:33735>))。候选 Object 只能由“所有实际写入是 references，且 Object 对这些值可赋值”成立，不能把 Unknown 推成 String、接口或生命周期证明。
- 参数类型由 method descriptor 固定，body 写入不得 widen（[`build.rs:2655-2665`](</Users/lordcasser/workspace/projects/jarde/crates/jarde-java/src/build.rs:2655>))。
- `write_statement` 会让 initializer 与决定类型相遇，之后的 assignment 复用同一类型决定；reference assignment 的 source position 本身不提供类层级推断（[`build.rs:25540-25610`](</Users/lordcasser/workspace/projects/jarde/crates/jarde-java/src/build.rs:25540>)、[`build.rs:25613-25687`](</Users/lordcasser/workspace/projects/jarde/crates/jarde-java/src/build.rs:25613>)、[`build.rs:30873-30920`](</Users/lordcasser/workspace/projects/jarde/crates/jarde-java/src/build.rs:30873>))。
- `LocalVariable` 是 `(slot,index)`；reuse plan 可凭 LVT 或 SSA/CFG 证据分离同一 slot 的生命周期，也可在无 debug 表时工作。类型兼容只消费每个既有 identity 的访问；identity 不明确、任一访问无 path、catch value、handler join 或 split placement 不完整时不扩大判断（[`names.rs:396-429`](</Users/lordcasser/workspace/projects/jarde/crates/jarde-java/src/names.rs:396>)、[`reuse.rs:1-38`](</Users/lordcasser/workspace/projects/jarde/crates/jarde-java/src/reuse.rs:1>)、[`reuse.rs:84-101`](</Users/lordcasser/workspace/projects/jarde/crates/jarde-java/src/reuse.rs:84>))。
- 当前逐写 proof 在每次读取证据前 charge/poll；写冲突复查也 charge/poll，并将 `Caught` 留给 catch header 自己处理。新增 read proof 必须遵循同样的 budget/Stop 纪律，不忽略不完整 consumer，不缓存部分 Object 决定（[`build.rs:2456-2468`](</Users/lordcasser/workspace/projects/jarde/crates/jarde-java/src/build.rs:2456>)、[`build.rs:2873-2909`](</Users/lordcasser/workspace/projects/jarde/crates/jarde-java/src/build.rs:2873>))。

## 合法反例与拒绝条件

接口反例：`static int interfaceUse(boolean b) { CharSequence target = "seed"; if (b) target = new StringBuilder(); return target.length(); }`。两个写入都合法填入 `CharSequence`，且 length 是接口方法；无条件将局部声明成 Object 会让 `target.length()` 无法编译。该规则不计算 String/StringBuilder 的接口公共类型；若 read consumer 要求 CharSequence，第一片必须拒绝 Object 候选，保留 fallback/拒绝输出。

数组反例：`static Object arrayUse(boolean b) { CharSequence[] target = new String[1]; if (b) target = new CharSequence[1]; target[0] = "x"; return target[0]; }`。这是合法的局部数组生命周期：初始化、重赋值、数组元素读写都需要数组静态类型；声明成 Object 会让下标读写无法编译。第一片不通过 Object 兼容消掉数组 shape。若需要验证物理 slot reuse，须另用 classfile/BCI 事实，不能由 source snippet 推断 javac 复用 slot。

其他拒绝：primitive/reference 混写；非 Object 返回或 invocation 参数/receiver 要求窄类型；数组读写；无法证明所有 load consumers；未解析的 slot reuse identity；参数 slot；catch/handler write；stop/budget 造成的部分扫描。尤其不要插入未经 bytecode/现有 widening proof 支持的窄化 cast。

## 最小任务与验收条件

1. 在 `decide_types` 既有 first-write/reference 冲突路径加入局部兼容候选，范围只限非参数、普通 body reference local；不改 frame/SSA identity、不追 Phi 建 LUB、不接入 LVT descriptor/LVTT，也不改变循环 foreach 类型规则。
2. 为 `noCastAssign` 加固定输出断言：局部为 Object，seed 与 `next()` 两次写保留，Return 用法保留。输入同时有 debug 与 `-g:none` 变体；两者必须由同一 IR/operation/return 事实决定，局部名允许按输入证据变化。
3. 加合法 CharSequence receiver 与 CharSequence[] 生命周期拒绝控制。它们不得输出 `Object` 局部后留下不可编译的 `length()`/`target[0]`；应保持既有保守 fallback 或拒绝局部。
4. 加至少一个不同 `LocalVariable` identity 复用同 slot 的控制，确认一个生命周期的 writes/reads 不渗入另一个；参数、catch/handler、预算 stop 的现有拒绝条件保持。
5. 成功要求精确 positive source 和两个合法 negative source 的行为固定，read proof 覆盖每个相关 Load output 的全部 consumers，生成的 Java 8 source 可由项目后续验收编译通过。实现验证时另按工程 gate 跑必要测试；这份提案本身未运行工具链，不能算修复完成。

## 待 root 代理决策

- 首片是否严格只允许所有 Load outputs 直达 Object Return？建议如此；其他已知 Object 接收点留待证据和需求明确后扩大。
- 对 source type 是 `Object` 但 downstream 使用更窄 class/interface 的合法源代码，第一片是否明确拒绝整个 affected region，而不是试图推接口公共类型？建议拒绝并把 LUB/hierarchy 分析拆成独立债务。
- slot reuse 控制是否要求无 debug 也实测，还是先只验证同一 SSA/reuse identity 的隔离？建议实现 fixture 同时覆盖无 debug 正向与一个 split-slot 隔离控制。

这只是设计提案。当前 foreach 产品仍处最终 CI/clean 交付阶段；该状态不代表局部引用写兼容修复已实现或验收。
