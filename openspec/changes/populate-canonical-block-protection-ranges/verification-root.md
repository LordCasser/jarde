# Root verification

## 验收范围与结论

本记录只覆盖 `populate-canonical-block-protection-ranges` 的本地 focused 验证，以及经根验收器合并的明确测试 scope。它不代表整个 workspace 已通过，也不代表实现提交自身的在线 CI 已通过；实现提交自身 CI 尚缺。`canonical-combined-root-v1.json` 的状态是 `accepted_local_combined_scopes_only`。

实现范围是完整 canonical 图中每个 block 的物理保护范围 ordinal：按物理异常表声明顺序去重，基于真实 raw member 与半开保护范围相交计算。融合节点表示至少一个真实成员受保护，不把非连续成员间的包络空隙视为成员。该事实与 handler-row/throw-site 的异常边事实分开，不扩大异常分析输入、Frame/SSA 输入或 Java source recovery 能力。

## 代码与计费变更

生产与范围回归改动位于 `crates/jarde-jvm/src/canonical.rs` 与 `tests/p3_method_ir.rs`；另有实测后对 `tests/p5_bulk_corpus.rs` 的六项费用常量及说明校准。最终 focused v2 源码闭包为 4,887 个文件；两文件摘要如下：

| 文件 | 字节数 | SHA-256 |
| --- | ---: | --- |
| `crates/jarde-jvm/src/canonical.rs` | 152,162 | `41dc3c015ef1845ddf04ad548859eccf8676c42813c03b2ad45cd7cf77eccc70` |
| `tests/p3_method_ir.rs` | 38,863 | `f0face82e17b6733920ddc955a308ad35cff8c76a399b388ec3dcf2c9e2d77dd` |

首次 focused 验证（v1）保留为失败记录：canonical 单元 32/0/0、公开 P3 5/0/0、legacy 组合命令中的四个测试目标分别为 8/0/0、16/0/0、11/0/0、5/0/0；Frame 单测 48/0/0、SSA 单测 19/0/0 均通过；但 P5 bulk 为 3 passed、2 failed、1 ignored，失败原因是新计费使 `many-method-class` 的旧精确 pin 过低。v1 整体状态仍为 failed，不能用后续校准改写其历史结果。finally `cargo clean` 成功，`target` 不存在。

之后单独运行既有 ignored recorder，结果 1 pass。六条 P5 计费记录中，只有 `many-method-class` 变化：`ir_items` 从 21,390 到 21,458（+68），`analysis_steps` 从 12,104 到 12,439（+335）；其余五行所有字段不变。direct 与 shared 两种 arm 的 187 次请求均出现相同的 IR +68、Steps +335，其他计费维度不变。根验收确认只对 `tests/p5_bulk_corpus.rs` 六个精确常量及其说明作了校准；反向 delta 可逐字恢复旧测试文件，未改断言或 fixture。计费证据见 `canonical-billing-delta-v1.json`。

## focused v2 执行

v2 在同一最终实现源码上执行以下六项：

1. `cargo fmt --all -- --check`
2. `cargo clippy --workspace --all-targets --all-features --locked -- ... -D warnings`（使用项目现行 Clippy allow 列表）
3. `openspec validate --all --strict --no-interactive`
4. `cargo test -p jarde --test p5_bulk_corpus --all-features --locked --message-format json`
5. `cargo build -p jarde-cli --locked`
6. `cargo clean`

六项命令均退出码 0；P5 bulk 结果为 5 passed、0 failed、1 ignored（既有 recorder 仍在常规 suite 中 ignored）。独立协议审查 `canonical-focused-v2/protocol-review.json` 接受此 scope：4,934 checks、4,887 个源码文件、6 commands、12 raw streams，5 passed、0 failed、1 ignored。清理后 `target` 不存在。

本地收据记录 Rust stable 预期版本 `1.99.0`、OpenSpec `1.11.0`，并使用 Codex bundled Node runtime 路径（执行收据未记录 Node 版本号）；这不是对在线 CI 工具链等价性的声明。构建阶段守护阈值为可用磁盘至少 5 GiB、`target` 不超过 1 GiB、每秒检查。finally `cargo clean` 阶段关闭这些资源阈值以确保清理完成。

## 合并 scope 与证据归档

根验收 `canonical-combined-root-v1.json` 接受的仅是本地合并测试 scope：v1 已通过的 144 项 canonical/public/legacy/frame/SSA 功能测试，加上 v2 的 5 项 P5 计费测试，共 149 项不同测试；ignored recorder 额外执行的 1 pass 单独计数。合并验收明确保留 v1 整体失败状态，且 v1→v2 之间唯一的源码文件变化是 `tests/p5_bulk_corpus.rs`。这项合并不等价于重新运行全部 149 项，也不扩展为 workspace 或 CI 通过。

证据归档由根维护于 [`results/root-focused-v1/`](results/root-focused-v1/)；本地执行收据和根验收文件位于私有验证缓存 `core-integrity-20261015` 中。归档 `capture.tar.gz` 已逐项复开核验，SHA-256 `85aec3874a1334af6b7c2648e8c7e940bb6d298272687533e88a74ef47b5d128`，75 payload，18 命令/36 原始流；范围与数据清单见 [archive-review.json](results/root-focused-v1/archive-review.json)。

## 尚未完成

本地 focused 与合并 scope 不替代 workspace 全量验证、实现提交自身 CI、独立提交及后续同步/归档流程。相应集成门仍未完成，见 `tasks.md` 的 2.2。
