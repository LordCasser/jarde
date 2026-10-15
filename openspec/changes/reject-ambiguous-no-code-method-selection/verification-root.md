# Root verification

Root 于 2026-10-15 完成本变更的本地 focused 验收。独立协议审查 [protocol-review.json](/home/lordcasser/.cache/jarde-validation/core-integrity-20261015/method-focused-v1/protocol-review.json) 状态为 `accepted`、无审查错误，核对 4,938 项；6 条命令、12 条原始输出流；49 通过、0 失败、0 忽略。对应执行回执见 [execution.json](/home/lordcasser/.cache/jarde-validation/core-integrity-20261015/method-focused-v1/execution.json)。

## Source changes reviewed

本变更基于 `8a56a863a90f25555b7e5754a2ff6ea49e07830c`。当前实现差异限于以下三处；SHA-256 来自 method-focused-v1 起始源码清单，执行前后 4,887 项源码清单逐项一致。

| 文件 | 变更内容 | SHA-256 |
| --- | --- | --- |
| `crates/jarde-jvm/src/engine.rs` | Direct driver 在完整 Header method list 中筛出 raw name/descriptor 精确匹配，找到第二个即返回既有歧义输入错误；prepared driver 对 locator 结果作同样唯一性检查。增加内部 `ambiguous_method()`，复用 `classfile_method_ambiguous` 和既有错误消息。 | `3de4fb79ccab307afad95bcc59331257adc67a1ce4b3987e4ea43279da2e7998` |
| `crates/jarde-jvm/src/callee.rs` | 私有 `locate` 改为 `Result<Option<_>>`；Read/Prepared 两种来源在判定 no-Code 之前检查唯一性，并将歧义错误传播给调用方。 | `1f22ef28276c1a310a1e7c4aae1a5fdbe6414e178836b0c03440f05d246ff51f` |
| `tests/p3_prepared_input.rs` | 增加固定物理 classfile 构造和回归矩阵，覆盖无 Code+有 Code 双顺序、两条无 Code、两条有 Code、abstract/native 混合、唯一与缺失控制，以及 direct/prepared driver 和 callee。另核对 prepared binding、未完成成员表、loader binding 优先级和零 method-body/Code/IR 用量增量。 | `b34a2b990f368b3f2f397bb36a40ee00a075f9c482c9963f6f788945250ec238` |

Design 中的早期私有候选基线摘要为 `engine.rs` SHA-256 `a1f7f9a8e0fd8d9faf9a8474da576bf5a95c8a32636f07e4d69f86ee37f9f6b2` 与 `callee.rs` SHA-256 `32d8d8f3c76d790663c8098ae9773993da1cd3fbc05a4bf017d5b279f3d070d5`；两者仅描述候选准备时的基线，不作为最终运行绑定。

私有候选 `validated-candidate.patch` 位于 `/home/lordcasser/.cache/jarde-review/20261014/isolated-core-fixes/method-v1/validated-candidate.patch`，SHA-256 为 `6023dde15262d16dcccc13070e18bab3342ce13227d838157a750dcc3d63fa20`。该 patch 仅包含上列三个源码路径，其 driver/callee 唯一性判断和测试矩阵与已验收 worktree 修改相符；它不单独充当运行证据。

## Validation record

`method-focused-v1` 于 `2026-10-15T01:39:35.647475+00:00` 完成。回执记录默认 Rust stack、Rust stable toolchain（预期版本 1.99.0）、OpenSpec 1.11.0，以及单构建 job、关闭增量编译和串行测试线程。Node 本地运行时为 24.19；GitHub CI 使用 Node 22，因此 OpenSpec 命令的本地 Node 版本与 CI 不同，不声称工具链完全相同。

执行期间构建命令采用 5 GiB 最低可用空间、1 GiB `target` 上限、1 秒轮询和超限进程组终止。最终 `cargo clean` 成功退出，回执记录 `target_absent: true`。

以下命令均以 exit code 0 完成：

1. `cargo fmt --all -- --check`
2. 当前 CI exact Clippy 命令：`cargo clippy --workspace --all-targets --all-features --locked`，使用仓库现有 lint allowances 并以 `-D warnings` 收尾。
3. `openspec validate --all --strict --no-interactive`
4. `cargo test -p jarde --test p1_prepared_class --test p2_cfg --test p3_prepared_input --test d3_artifact_binding --all-features --locked --message-format json`
5. `cargo build -p jarde-cli --locked`；冻结 CLI 为 59,595,832 bytes，SHA-256 `82ee117bb8858ebb97f96170d7e7ea0cd9d9b283f21d44f5434919dd88cbb1a9`。
6. `cargo clean`，作为最后清理步骤完成。

定向 integration 命令共 49 项：`d3_artifact_binding` 13 项、`p1_prepared_class` 12 项、`p2_cfg` 12 项、`p3_prepared_input` 12 项；归属按原始输出中的 Cargo `Running` 字段与各测试结果核对，不按 `--test` 参数顺序分配。测试覆盖完整物理成员表中的顺序无关歧义拒绝，并核对拒绝发生于方法体准入前：driver 报告 RawFacts 阶段失败、后续阶段 `NotPerformed`、body `NotInspected`、Header read 保留；callee 返回同一 `classfile_method_ambiguous` 输入错误。两条有 Code、两条均无 Code、abstract/native 组合都不消费 method-body、Code 或 IR 额度。唯一有 Code、唯一 abstract/native、缺失及其他方法名/descriptor 的重复不受影响；prepared definition mismatch、loader binding 与不完整成员表等先行错误保持优先。

本次记录证明的是这些定向 integration tests、格式、CI Clippy 门、OpenSpec strict 和 CLI build 的本地结果；没有运行完整 workspace 测试，也没有完成该提交自身的在线 CI。最终文档应结合 root 的实际归档检查确认提交和 CI 状态；在这些交付条件完成前，不把本变更扩展描述成整个 workspace 或线上 CI 已通过。

本轮归档见 [root-focused-v1/README.md](results/root-focused-v1/README.md)，`capture.tar.gz` SHA-256 `a249499081d5f0eb6ad34122ae6baa5aaad155bbeed28cd5c22450d16f6beb99`。旧私有实验的原始两项失败（driver 错报 Complete、callee 错返无 Code 首项）与源码来源单独保存在 `private-history.tar.gz`，SHA-256 `051e1c08b37b139388ddc150527bb2e729e25b974d2c01d7e5a92445ff67548b`，详见 [private-history-review.json](results/root-focused-v1/private-history-review.json)。它们不计作本轮根测试，也不将私有 requested 命令中零测试的 d3 目标算作验证通过。
