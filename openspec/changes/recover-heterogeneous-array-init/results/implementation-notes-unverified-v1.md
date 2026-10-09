# EM18 产品实现记录（待 root 编译与复核）

按本轮授权只编辑了：

- `crates/jarde-java/src/build.rs`
- `src/facade.rs`
- `crates/jarde-java/src/report.rs`

没有改 OpenSpec 规划/checkbox、fixture、Git 状态或运行 Cargo/rustfmt。

`Builder::array_initializer_element` 在现有 null、精确类型、Object 与 primitive 路径之后调用 `initializer_reference_widens`。该判据复用 `array_reference_widens`、`platform_array_argument_widens`、`release_reference_argument_widens`、`java_lang_throwable_widens`、`platform_interface_argument_widens`，并仅在同秩、reference leaf 的前提下把既有 scalar 表向 `T[]...` 提升。snapshot 记录只接受 `store_bci + full source spelling + full component spelling` 全匹配。成功原样返回 `Expr`，没有加入 element cast；`reference_overload_calls` 没有被查询。新增内部测试覆盖 Number/CharSequence/Collection/Throwable scalar 与等秩数组、方向/Java release、rank/primitive 边界、数组形状给出的 Object[] 路径，以及 snapshot 证明的精确站点和完整类型匹配。

facade 的 `prove_snapshot_hierarchy_widenings` 原调用参数扫描保留；新增 opcode `aastore` 分支，通过 `snapshot_aastore_reference_pairs` 找同 BCI SSA 指令，按绝对 `Slot::Stack(depth)` 排序并取最高三项，要求深度连续、类型为 named arrayref / int index / named reference value，且 source 与 arrayref loader anchor 相同。reader `descriptor_facts` 验证 array descriptor；移去一维后得到 component。仅等秩 scalar 类或引用数组进入已有 `snapshot_header_chain_widens`；结果记录 store BCI 与完整 source/target Java spellings，walk 输入仍是 deepest class internal names，仍复用当前 selected Runtime/header resolver、depth 8、cache 和预算。unknown/null/primitive array、坏 descriptor、rank mismatch、loader disagreement 不产生证据。新单测覆盖 scalar/rank-two reference shape 及 primitive/malformed descriptor 拒绝。

修正 `ProvedSnapshotHierarchyWidening`、RecoveryRequest 字段/setter 与 facade 注释：target 可由可信 snapshot header 直接点名，即使 target 本身不是物理 snapshot class；缺失中间层级仍无证明。method-only 入口仍空，调用参数行为未变。

尚未执行编译、focused tests 或格式化检查，避免与 root 的统一 Cargo/验收流程冲突。root 请优先核对 facade SSA operand 解析、reader descriptor API、Rust 类型与 borrow 后串行编译；若编译反馈需要修复，请解冻授权文件后再通知我。

## Root diff-review follow-up (still uncompiled)

- 修正了 scalar source shape：`I` 可作为合法 source internal class name；数组分量一侧先用 reader `DescriptorKind::Field` 解析并要求最深 descriptor leaf 为 `L...;`，所以 `I`/`[I`/`[[I` primitive component 不会被当成 snapshot class proof。
- AASTORE pair 判据拆为可单测的小函数，新增 primitive/multidimensional-primitive/malformed array、非 int index、loader 不同的拒绝测试，以及合法 `Child`→`Base` pair。
- 按 SSA contract 检查真实读取数：`frame.rs` 的 `aastore` row 是 `POP_RIR`（`Ref,Int,Ref`），frame replay 为每个 pop 记录一次 slot access；`ssa.rs::run_block` 对每个 read access 追加一个 `SsaInstruction.reads` 项。因此消费集必须正好三个 read。实现现在要求 `site.reads().len()==3`，stack slots 过滤后也必须正好三个，再按绝对 depth 验证连续且按 arrayref/index/value 取值；prefix 体现为 depth 起点非零，不是额外 read。
- Root 的第二轮 diff follow-up：descriptor 侧加了独立 `snapshot_reference_descriptor_shape`，先 reader-parse 完整 Field descriptor，再要求 leaf descriptor 明确为 `L...;`；source 侧仍将裸 internal `I` 解释为类名。pair helper 另要求 array/value 两个 SSA loader anchors 均等于请求 runtime 的 selected loader。`SsaInstruction.reads()` 现在必须总数恰为 3，再要求 stack access 正好三个；这与 frame `POP_RIR`/SSA access→read 的实现合同一致。单测增加 `[I`、`[[I`、malformed array、错误 index 和不匹配 loader。仍未运行 Cargo。
