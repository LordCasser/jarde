# Build implementation record

## Scope

本次只调整 `crates/jarde-java/src/build.rs` 中 `prove_array_initializer` 的 child-array 结构分支，并在同文件私有测试区增加真实 fixture focused 测试。没有改动 array candidate 的收集、唯一 parent store、`ValueId` reader、interval、深度、计费、handler 闭包或整条 chain 的原子提交逻辑。

## Production change

child 分支继续要求其 allocation 对应 `Operation::NewArray`，并继续要求 child sources（除其 parent store）落在 parent 当前 element interval 内。删除了 `component == child_type` 的结构层相等判断。child 的 Java assignability 仍由 Builder 在携带确切 store BCI 的 element presentation 中检查；结构闭合不等于正文可呈现。

## Focused evidence added

新增 `direct_child_array_chain_is_structural_and_commits_only_as_a_whole`，直接分析冻结 direct fixture 的 `ownGridDirect`，分别覆盖 javac8 与 javac23。正例锁定外层/内层数组 allocations BCI `1/7/28`、parent stores `24/45`、最终 consumer `46`、constructor allocations `12/33`，并对照 `aastore` 的实际 SSA stored `ValueId` 与 child final value、Site finished value。测试还检查三个数组节点与两个 constructor Sites 的 ownership 不重叠、各 Site 只提交一次。

同一 classfile `Code` 上派生三类局部变体：父索引交换、child retained value 后的 `dup; pop` reader、child store 与 parent store 之间的 `mark` effect。每种变体在两套 compiler leg 都要求结构候选与 pending constructor Sites 均为空，验证失败不会半提交。`dup` 控制按 JVM SSA 语义表述：dup 输入 ValueId 的 reader 是该 dup；dup 输出两个不同 ValueId，不把它误说成同一个 ValueId 有两个 reader。

另将 child allocation 的 class operand 改成 fixture 已有的 `java/lang/Object` Class 项。该 `Object[]` 对 parent `Base[][]` 的 `aastore` 是 verifier 可接受的 reference store，但 Java initializer element 不能据此赋给 `Base[]`。focused build 测试只断言结构/ownership 仍闭合；完整正文的拒绝应由集成测试依据 Builder 的确切 store/type 检查证明，不能由本结构测试代替。

预算证据分开记录。child-array instruction scan 的 `IrItems` 控制使用实际 `ownGridDirect`，为 effects census、BCI 28 allocation/length-use 和其后所有指令计费后，精确在 child `dup` BCI 31 停止。另一个 `AnalysisSteps` 控制精确停在构造参数验证的 BCI 37；这是 constructor argument scan，不代表每个 child-array 节点按 `AnalysisSteps` 计费。预取消断言使用同一 proof 入口及其真实首个 effects BCI。

新增 `direct_child_array_builder_uses_exact_store_type_proofs_and_stops_atomically`，通过同一方法的真实 canonical/SSA、已提交 `ArrayInitializers` 和 `sites_after_array_composition` 进入 `build`。正确的 snapshot widening 项精确对应 `DerivedA[] → Base[] @24` 与 `DerivedB[] → Base[] @45`，正文必须完整且不 ragged。Builder 正文还分别拒绝缺少全部 widening、BCI 24 上 source 错为 `DerivedB[]`、BCI 24 上 target 错为 `LocalInterface[]`，以及 BCI 错为 23 的证明；这些用例将结构闭合与具体 store 类型证明分开。充足预算先完成 build 并读取同一 `Budget` 的真实 `IrItems` 用量，再将上限设为完整用量减一重跑，要求 `build` 返回 Stop、不返回 `Program`，并锁定实际停止 BCI 46。另在合法 arrays/Sites 已准备后取消 token，再从真实 `build` 入口调用，断言它在 guard ownership 初始 poll 以 `at: None` 停止。这个控制证明该 build 请求不能提交部分 Program；它没有证明递归 `render_value(NewArray)` 每个子节点各自 poll/计费。

现有 `array_store_opcode_matches_the_exact_primitive_component` 继续锁定 primitive store opcode/component 的同型正例和错 opcode 拒绝；本次没有另造 primitive child-array fixture。没有新增 observer、测试开关或 production instrumentation。

## Validation status

本 agent 未运行 Cargo、rustfmt、Java 或 JADX。root 的 `root-focused-builder-v1` 编译在该测试文件报错：取消断言对 `Result<ArrayInitializers, _>` 使用 `assert_eq!`，且从 `Budget` 直接调用不存在的 `counted_usage`。已改为带 guard 的 `matches!`，并从 `budget.usage().counted_usage(...)` 读取实际用量；这些修正尚未重编译。曾误运行只读 `git diff --check` 和 `git diff --stat`，没有 Git 写操作。两套 direct class 的输入字节、Code 长度以及无 exception handlers/Code 子属性由测试 helper 读取当前 class facts 后检查；变体仅更新自己的 Code bytes、Code length 与 Code attribute length，未改 canonical fixture。
