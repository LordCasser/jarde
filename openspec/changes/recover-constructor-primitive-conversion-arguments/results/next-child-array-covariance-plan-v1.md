# 子数组协变后续验收规划

这是一份独立后续 change 的验收输入，不属于当前数值转换实现范围。未修改产品、测试、canonical 或 OpenSpec 计划，也未运行 Cargo、Git、rustfmt 或 Java。

## 冻结证据与当前失败面

直接输入来自本 change 的 `results/legacy-regressions-root-v1`：`manifest.json` SHA256 为 `dc8a09f2b9d34bd50f2e8b43f5fbcfda23e2f7bfdca58c9970cf81be962528eb`。两腿 Main class-source 原始报告分别保存在 `logs/fixture_direct_javac8_render-Main.stdout`（SHA256 `93af7dcce7e9d57a6a05ef4f438d82014d4841f65e5ad6cf2ad013ffd49332d7`）和 `logs/fixture_direct_javac23_render-Main.stdout`（SHA256 `c6400523d7cedcce6f53eabd6a2dfd738bf3b806435190532a2c583a59efcd86`）。对应输入均为 direct 六类 family，javac8 使用 Corretto 8，javac23 使用 OpenJDK 23；报告由 manifest 指定的冻结 CLI `3e4b615e0131d041ecc47c72c0131a3bbd0f5fc9042d8f91b2aa21f26baf6a95` 生成，不代表当前未运行产品。

manifest 两腿的 `Main` 报告均把 `numberGridDirect`、`collectionGridDirect`、`ownGridDirect` 留在 `Fallback/Mixed`。前两个方法没有被保留的 construction `NewRecord`，正文只有解释性 bytecode quote；`ownGridDirect` 保留两个未呈现构造记录：`DerivedA` 分配 BCI 12 的值只被 quoted store BCI 23 读取，`DerivedB` 分配 BCI 33 的值只被 quoted store BCI 44 读取。记录分别以 `jre_new_shape` 拒绝。它们说明 child-array ownership 链没有闭合，不是 Java 赋值关系不兼容的证据。两腿完整 direct 六源集的候选编译都以 exit 1 结束：`logs/fixture_direct_javac8_compile-candidate-sources.stderr` SHA256 `b7f19001081f5e37f62a1c823c1c2c6f1b870269ccde9b633ea7bc547bcece5e`，`logs/fixture_direct_javac23_compile-candidate-sources.stderr` SHA256 `572c4619ae408cea4a73ba1b4aa5273e4ae5ff8f4165c660835c5a459d55433c`；因此该基线没有 candidate runtime comparison。

三个目标的类型关系分别是 `Integer[]` / `Long[]` 到 `Number[]`、`ArrayList[]` / `HashSet[]` 到 `Collection[]`，以及 `DerivedA[]` / `DerivedB[]` 到 `Base[]`。前两组有当前闭表的 leaf 关系；第三组需要所选 snapshot hierarchy 证明。此前审计 `openspec/changes/recover-heterogeneous-array-init/results/child-array-covariant-architecture-audit.md`（SHA256 `1a53e39dfde167504d5d11c093a20baaf922b9bcca4158e68451cee5dc2a354d`）已将 `ownGridDirect` 的 parent/child store reader 与类型判定分开。其 listing 的 482–512 是 javap **行号**，不是 BCI；方法的有效 BCI 为外层 store 24/45、最终 return 46，child 的内部 store 为 23/44。

`collectionGridDirect` 的旧 class-source 输出还带有 `ordinary_generic_source_unproved` Signature projection refusal，但目前没有证据证明它是 child-array 正文失败以外的独立阻塞。canonical 原始方法在 `openspec/changes/recover-heterogeneous-array-init/evidence/heterogeneous-array-initializers-v3-pathfix/direct/Main.java:102–106` 返回 `Collection<?>[][]`，两个子数组分别是 `ArrayList<?>[]` 与 `HashSet<?>[]`。冻结 CLI manifest 的 direct/javac8 `Main` 报告（见本节所列 manifest；对应方法项 `candidate_source_reports`）同时记录该方法 `body_quality: fallback`、`body_representation: mixed`、`new_records: []`，并列出“recovery run ... produced no statement”与 Signature refusal 两条 marker。这个观测只说明同一方法的正文未完整恢复时同时出现 Signature refusal；不能据此认定后者是独立缺陷。

当前拒绝分支也支持这种保守归因：`src/class_source.rs:7229–7237` 的 `ordinary_parameterized_declaration` 仅在 method outcome 为 `Recovered` 且 projection/record marker 为空，或是有明确种类的 `NoBody` 时继续；否则返回“method body or no-body declaration has no complete source proof”。该代码把 Signature 声明投影建立在完整正文/无正文证明上，但单凭这条通用门槛仍不能证明 collection 的唯一原因。正确结论是**未证明独立 Signature 阻塞**：先只解除 child-array ownership/呈现失败，再对完整 collectionGridDirect 和完整六类 family 复跑；若正文已恢复而 refusal 仍存在，再凭那次报告和 projection state 追查是否有独立 Signature 问题。不要放宽 Signature gate，也不要仅凭当前诊断拆出额外类型推断工作。

## 最小目标与复用边界

最小目标是在结构层不把 `component == child_type` 当作 child-array ownership 闭合的必要条件。当前 gate 位于 `crates/jarde-java/src/build.rs:13244`，false 时 `prove_array_initializer` 返回无候选；它把 Java 可赋值性误当成 child reader/identity/interval 事实。后续改动只应让已有结构 proof 能保留一个准确的 child array candidate 与 parent-store pairing，继而由 Builder 判定该 initializer 能否作为完整 Java 表达式呈现。

结构端复用现有 `array_initializer_reader`（`crates/jarde-java/src/build.rs:13700`）：保留同 block、唯一 reader、reader 晚于 store、其它操作数表达式闭合、store 到 reader 区间无未归属指令等约束。child 的 `consumer` 必须仍是其确切 parent `aastore`，final ValueId 仍须与该 store 的 value 操作数身份一致；child/parent arrays 与关联 Sites 仍在完整 parent chain 闭合后一起 commit，Site 只从 pending census 移动一次。不要以协变为由放宽 extra reader、跨 block、顺序或 interval 约束。

呈现端复用 `Builder::array_initializer_element`（`crates/jarde-java/src/build.rs:26685`），其中 reference component 检查调用现有 `initializer_reference_widens`（`build.rs:29446`），调用处把 `initializer.elements` 中与值配对的准确 `store_bci` 传入（`build.rs:25360–25368`）。该 helper 先处理 null、精确类型、Object 和现有闭表；再检查平台/数组关系及**同一 BCI 且 source/target 完全匹配**的 snapshot proof（`build.rs:29451–29470`）。未知、错误或缺失关系必须使整个正文 fallback，不能把结构候选成功等同于 Java 正文成功，不能只输出 child 或丢掉 parent store。

snapshot proof 已由 class-source family 路径产生，无需另造 subtype walker 或类型表：`src/facade.rs:25108–25220` 从实际 Code 中的 `aastore`（opcode `0x53`）读取同一 SSA 指令的 arrayref、index、value 栈操作数，确认 loader 与 rank，再以该 `aastore` 的 BCI/source/target 读取所选 snapshot class-header chain；`DerivedA -> Mid -> Base` 和 `DerivedB -> Base` 可由该链证明。method-only recovery 不带此 family proof，`src/facade.rs:40798–40800` 只在 class-source assembly context 中计算它。Builder 侧 `report.rs:130`、`report.rs:7199–7203` 已接收这组 facts。沿用这条已有入口即可。

结构/ownership 与 Java 呈现仍是两个结果面：结构闭合后类型未知，可以保留真实 Site/array 来源事实，但整个方法必须 fallback；只有 store 的实际类型 pair 可被 Builder 证明赋值，才允许成功 Java initializer。不能把“结构阶段不提交未知类型”作为实现约束，也不能仅凭 `NewRecord.presented` 或 diagnostic 的“presented as new”声称构造表达式已写进正文。

## 验收分母与正例

完整 direct 输入是六个原始 class/source：`Base`、`DerivedA`、`DerivedB`、`LocalInterface`、`Main`、`Mid`。两条编译器腿共 **12 个完整 class-source 输出**；每腿必须从完整 family 运行，不可只给 `Main` 或删除 `Mid`。正例应覆盖以下三个已冻结 direct grid；其中 `numberGridDirect` 和 `ownGridDirect` 是本 slice 的核心类型关系。`collectionGridDirect` 当前的 Signature refusal 只作基线记录，不计为已证实的独立 blocker；child-array 恢复后以完整输出重新判断，且不能通过放宽泛型 proof 来冒充 covariance 验收。

- `numberGridDirect()`：呈现 `Number[][]`，元素按原序为 `Integer[]`、`Long[]`。用平台 scalar widening 的同秩数组复用证明；断言 child/store 顺序、trace `12` 和完整源码。不得要求 class hierarchy proof。
- `collectionGridDirect()`：预期关系是 `ArrayList[]`、`HashSet[]` 到 `Collection[]`。现有平台接口关系可供 Builder 复用。当前 Signature refusal 尚未证明是独立 blocker；先验证 child-array 正文恢复并运行完整 family。只有在正文已完整恢复后仍被 Signature 路径拒绝，才根据新报告把它登记为独立问题；不得删除 Signature gate。
- `ownGridDirect()`：呈现 `Base[][]`，child 分别为 `DerivedA[]` 和 `DerivedB[]`。准确外层 `aastore` proof 为 BCI 24 与 45；前者必须沿完整 selected chain `DerivedA -> Mid -> Base`，后者为 `DerivedB -> Base`。断言 child 顺序、内部构造与外层写入各一次，trace 保持 `12`，并验证完整正文。

每条腿都用已有完整六源集一次性编译：javac8 的原编译参数是 `-source 8 -target 8 -g:none`，javac23 使用对应冻结命令；输入 classpath/sourcepath 保持空，运行仅用新生成 classes。必须逐字节比较原输入冻结的 stdout/stderr/exit，而不是只看 javac 或 Java exit 0。原始 manifest 已保存这两腿的输入与 comparison protocol，不要覆写历史记录。

## 必须保留的拒绝边界

- **未知类型、缺失证明**：结构/ownership 可以闭合，但 Builder 没有 assignability facts 时，正文保持 fallback；不能成功产出 covariant initializer。class-source proof 缺失、method-only 请求或不透明引用名都不能从类名拼写猜继承关系。
- **错 store BCI 或错类型 pair**：proof 的 BCI 必须精确等于 parent `aastore`，source/target 必须完全相等。当前 `initializer_reference_widens` 单元覆盖精确 pair 与错误 BCI/target（`build.rs:33616–33644`）；该边界不因嵌入 initializer 而变宽。
- **缺少 `Mid`**：如果 selected family 不含 `Mid`，`DerivedA -> Base` 链不完整，`DerivedA[]` 的 store 无证明；尽管 `DerivedB -> Base` 仍可证明，整个 `Base[][]` 方法必须 fallback。不要让一项成功覆盖另一项不明。
- **primitive array invariant**：不得把 primitive 数值转换规则当作引用数组协变。现有 helper 已拒绝 `int[][] -> long[][]`，同时允许 `int[][] -> Object[]` 的 Java 数组形状关系（`build.rs:33590–33613`）。initializer composition 需保留同一结果；wrong primitive leaf/rank 不得通过 snapshot class hierarchy。
- **extra reader**：给 child retained value 增加第二个 reader 时，`array_initializer_reader` 的 sole-use 条件应拒绝 parent chain。两层 candidate/Sites 不得局部提交为成功 initializer。
- **reorder/interleaved effect**：更改 index/store 次序，或在 child store 与唯一 parent consumer 之间放置不属于表达式的 effect，必须被现有顺序、`interval_is_expression` 和 effect closure 拒绝。不能为了协变而重排元素或吞掉 effect。
- **实际不兼容的 verifier-valid store**：保留 `aastore` 执行语义和可能的 `ArrayStoreException`。即便 child ownership 结构闭合，不能因结构 rewrite 移除运行时类型检查或把不兼容 store 呈现成成功 Java initializer。

这些负边界验证现有唯一 reader、type proof 与完整正文 fallback 的组合；它们不要求新增 walker、通用类型体系或复制本 change 的 plan。后续 change 先独立实现这一最小复用方向，再用根日志确认候选路径、全部 family legs 和各拒绝控制。
