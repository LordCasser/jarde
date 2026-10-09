# 协变 child-array 规划的对抗审查

本审查只读 proposal/design/spec/tasks 与当前实现，未执行 Cargo、Git、Java 或任何测试，也未验证 proposal 中的前置 CI。前片产品仍冻结在规划指定身份；本文是规划输入，不代表 apply 或验收已开始。

## 结论

设计的核心拆分成立：当前提前拒绝确实是结构 proof 中 `component != child_type`（`crates/jarde-java/src/build.rs:13232–13251`），而已有构造事实仍足以确定 child allocation、parent store 和 child producer interval。移除这项类型相等要求后，Builder 已有基于 child 实际类型与精确 store BCI 的 Java assignability 门（`build.rs:26680–26747, 29440–29471`）。无需新增 pass、AST、walker 或类型表。

我没有发现需要扩大产品范围的结构不变量漏洞。需要在实现验收中把几项笼统的负控制进一步锚定到 **child-array 组合路径**，否则其他更早的拒绝门可能让测试看起来通过，却未实际检验本片边界；另外不要要求现有 snapshot 类型 proof 的拒绝必须以特定预算诊断呈现，因为 facade 目前把该 proof 阶段的 BudgetExceeded/Cancelled 转为空 proof 列表。

## 现有不变量与允许的最小改动

- **父数组不变量应保持。** `prove_array_initializer` 在读取 child 前，先用真实父 `NewArray` 的 element/rank 算 `component`，并在 `build.rs:13196–13202` 将该组件与实际 `array_value` 类型及真实 array-store opcode 对照。这个检查不是本片要删的门。待移除的条件只在 `build.rs:13244` 将 parent component 与从 child `Operation::NewArray` 还原的 child type 相等比较；这里的 child 实际类型应继续用于呈现/赋值判定，不能改写成父组件类型。
- **child 身份与 consumer 是精确的。** `build.rs:13215–13218` 以父 `aastore` 写入的 `stored_value: ValueId` 查 `child_values`，并要求 `child.consumer == store.bci()`；`13245–13249` 又要求 child 的其它 source 均处在父 element 的 `dup/index` 与该 store 之间。最小实现应只去掉相等拒绝，仍确认 `operations[child_at]` 是 `NewArray`，不放宽 ValueId、consumer 或 source interval。不能把 operation guard 顺手删掉后，将任意 producer 重新解释成 child array。
- **每层 retained value 仍须单一、有序消费。** `array_initializer_reader`（`build.rs:13700–13767`）校验 sole-use、同 block、reader 在 store 后；reader 的其他栈操作数必须由其前面的表达式完整产生，且 source interval 集必须精确等于这些依赖。父层最后一个 store 另验证 reader 是否是允许的普通 consumer 或确切存 child retained value 的 `aastore`（`build.rs:13444–13462`）。额外 reader、跨 block、乱序或区间内无归属 effect 仍应失败。
- **公共提交仍是闭合链提交。** 反向扫描先在 block-local `candidates/candidate_sites` 保存完整候选；普通 consumer 根才发起链遍历。对每个 node 的 `owned` 与每个 Site 的 `owned`，先用 `claimed` 和已提交计划做冲突检查；只有整条链闭合且无冲突后，才移出 chain Sites 并写 `proved.allocations/aliases/owned`，再 `commit_site`（`build.rs:12735–12772, 12775–12864`）。本片不得在 child 被 parent 识别时提前写入 committed plan。因 `StopReason` 的 `?` 从该构建函数返回，局部 `proved` 随失败销毁；但这一逻辑性质不能代替实际 child-chain Stop 负控制。

实现审阅时要核实被删除的 `child_type` 局部变量是否仍只用于早拒绝。当前线索显示它在该 branch 内只用于 `component != child_type`；如果移除后变量无其它使用，应保留对 `Operation::NewArray` 的形状/存在性校验，不制造为了传递 child type 而新增的事实载体。

## 呈现类型与异常行为的边界

`Builder` 通过 allocation BCI 找到 `ArrayInitializer`，按元素 `(ValueId, store_bci)` 渲染原值，再以同一 `store_bci` 调 `array_initializer_element`（`build.rs:25299–25367`）。若 element 是已提交 constructor Site，还会核对 Site 的 `finished_value` 与 initializer 精确值一致（`25314–25331`）。child 类型应由该值实际指向的 `NewArray` 表达式生成，不能从目标组件反推。

`array_initializer_element` 对 reference element 接受 null、精确类型和 Object；其余要求 `value.presented` 是 reference，并调用 `initializer_reference_widens(..., store_bci, ...)`（`build.rs:26721–26747`）。该 helper 使用 Java 数组形状、release/platform 闭表和完全相同 store BCI 及 source/target 的 snapshot widening（`29446–29470`）。因此结构候选闭合后仍可能在 Builder 拒绝；结构记录、`NewRecord.presented` 或 Local SourceMap 锚点都不等于成功产生完整 Java initializer。验收必须同时读 body quality、representation、正文 fallback/diagnostic，并对预期成功者做完整源集编译运行。

Snapshot proof 的 producer 读取同一 BCI `aastore` 的 SSA stack operands，并要求数组引用、索引和值三个读取在栈上连续；同 loader、合法 reference descriptors、source/target 数组 rank 相等才继续（`src/facade.rs:25060–25159`）。之后仅在同一 BCI 产生完整 Java source/target spellings，沿所选 snapshot hierarchy 证明（`25169–25215`）；method-only recovery 明确不给此 family proof（`40795–40801`）。这直接支撑 ownGrid 中 `DerivedA[] -> Base[]` 的 `DerivedA -> Mid -> Base` 和 `DerivedB[] -> Base[]` 的直接链。

数组赋值和异常控制需要如此区分：同秩的合法引用协变可以写为 nested initializer；primitive leaf 不参与引用层级提升，`array_reference_widens` 对 primitive component 保持 invariant（`build.rs:29519–29560`），当前 helper 也固定拒绝 `int[][] -> long[][]`、错误 rank，接受 Java 形状要求的 `int[][] -> Object[]`（`33586–33612`）。对于 verifier-valid、运行时会触发 `ArrayStoreException` 的不兼容 `aastore`，结构 child proof 可以存在，但 Builder 不得输出把它写成成功兼容 initializer 的正文。负例需先确认字节码确实可 `-Xverify:all`，否则 verifier 早拒不覆盖本片呈现门；可记录原程序抛错作为事实，故意不执行一个会改变结果的 recovered source。

## 计划中最值得收紧的验收表达

1. **Extra-reader、重排、effect 控制须明确命中子数组。** Tasks 2.1/2.2 目前列了这些否例，但应避免用只改父数组最终 reader 的旧控制冒充 child retained value 的 extra reader。最低验收是派生 child candidate 后，让它的 final ValueId 出现第二个实际 reader，同时保持目标 child `aastore` 和其它语法仍可识别；另一个负例改变 parent 的 child store 次序；effect 负例放在 child allocation/element stores 至唯一 parent consumer 的组合区间。报告预期拒绝的 BCI/consumer，说明未被无关 unsupported opcode 掩盖。
2. **primitive/rank 负例需要结构与类型两层证据。** 旧同型 primitive nested-array 正例证明删除相等门不会破坏合法路径；但 helper 的 `int[][] -> long[][]` 单测本身不证明嵌套结构候选实际抵达 Builder。至少保留一个真实 child-array 类型不兼容/秩不匹配 control，先证明结构 Site/source 事实确实形成，再证明完整方法因 Java 赋值证据不足而不成为成功正文。验证 fixture 仍为合法可读、可验证输入，且拒绝不是由不支持的语法提前触发。
3. **“未知/错证明”分清 helper 与 class-source。** 既有 `initializer_snapshot_widening_requires_the_exact_store_and_full_type_pair`（`build.rs:33614–33648`）已经覆盖错 BCI、target 与方向；不要新造证明注入架构或把 unit helper 通过冒充完整 method 正例。实际 selected-family 的 missing-Mid 是自然的 class-source 负控制：DerivedB 关系可证明，但 DerivedA 链不完整时，整个 ownGrid 方法仍不得成功。若另加不透明/错误 pair，应证明该输入确实有结构候选、类型证明缺失是最终门，整个正文不声称可编译。
4. **预算停止准确测试结构公共 meter，不借错误类型回退推断。** `prove_array_initializer` 与公共 commit 使用会传播 `StopReason` 的 Budget；应在 child-parent candidate/commit 路径注入真实 BudgetExceeded 与 Cancelled，核实没有半份 ArrayInitializer、child Site 或 pending Site 返回。注意 snapshot producer 的 `snapshot_hierarchy_widenings_presented` 会在 `src/facade.rs:25314–25324` 将 BudgetExceeded/Cancelled 转成空 proof 列表。因而当前规划可以验收“缺类型证明时不输出成功 Java”，但不能只凭这个数组赋值拒绝宣称精确 Stop reason/location；若本片将其作为明确断言，需先确认后续 build 是否实际传播停止，或将该断言限定为已有 meter 的结构组合路径，不为此顺带重做 facade 的证明错误策略。

## 验收分母与冻结边界

- 目标 direct 输入的原始类为 `Base`、`DerivedA`、`DerivedB`、`LocalInterface`、`Main`、`Mid`；两腿各六份 class-source，总计12份。目标面是 `numberGridDirect`、`collectionGridDirect`、`ownGridDirect` 在两真实 JDK 腿的完整源集都可隔离编译、`-Xverify:all` 运行且 exit/stdout/stderr 与对应原始腿相同；不能只测 Main、只编 Main 或删除 Mid。报告必须把两个 JDK 结果分别列明，不用单个“2/2”掩盖输出数与运行数。
- 旧direct失败 baseline 仍是双腿0/2，三 grid 的局部候选不得计为成功。`collectionGridDirect` 同时出现 `ordinary_generic_source_unproved` 和 fallback；前片 `src/class_source.rs:7229–7237` 的 Signature 完整正文门槛说明相关性，但当前并未证明这是独立阻塞。先恢复 child 后重跑完整 family；不得提前放宽 Signature 门或把当前诊断作为 covariance 失败归因。
- 保留设计中列出的其它回归分母：旧factory双腿2/2、18矩阵16/18、数值转换家族2/2及 BigDecimal 两腿既有失败边界。legacy/JADX 旧产物按原始 manifest/hash 引用；JADX 8 profile 中只有 `--rename-flags none` 四腿与 frozen 原始流匹配，default 四腿不匹配，不能从语法观察数或 compile/run exit 反推 semantic accepted。
- 已知前置 CI `37968418985` 尚处等待时，本审查不授权也不建议任何产品改动；proposal/design/spec/tasks 继续保持规划态。此处所有负控制均是建议的验收条件，不是已执行或成功的测试。
