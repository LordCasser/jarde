# Shift 3.3 整仓门禁审计（2026-09-24；未关闭）

Root 在当前共享工作树复跑 `jarde-java --lib` 139/139、移位表达式 5/5、JVM 合法移位负边界 2/2；[相邻回归](verification-root-3.2.md)另记录两项非移位的失败。`cargo fmt --all -- --check`、`git diff --check` 与 `openspec validate recover-shift-expressions --strict` 通过。

`cargo clippy --workspace --all-targets --locked -- -D warnings` 在 `jarde-java` 的七处旧告警停止：`enumswitch.rs` 两个 `useless_conversion`、`region.rs` 的 `too_many_arguments` 与 `type_complexity`、`report.rs` 三个 `needless_option_as_deref`。`p5_corpus_fingerprint` 的其它断言通过，但文件核对发现共享工作树中 61 个新增夹具未列入 manifest（此前记录为 60，随在途夹具增加）；不能为让本 change 变绿而盲目重录语料。未修复这些跨变更门禁，因此 3.3 保持未勾选。
