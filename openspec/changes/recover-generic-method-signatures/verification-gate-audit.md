# Generic 3.3 整仓门禁审计（2026-09-24；未关闭）

Root 对已准入的静态返回子切片完成[三方整类及五个负例重放](verification-root-subset.md)、[预算与 CLI 停止复核](verification-root-3.2.md)。reader 冻结签名测试 3/3、query 方法签名回归 1/1、泛型投影 6/6、预算 4/4、类源码 47/47、类源码 CLI 16/16、`jarde-java --lib` 139/139 通过。`cargo fmt --all -- --check`、`git diff --check`、`openspec validate recover-generic-method-signatures --strict` 通过。

严格整仓 Clippy 在 `enumswitch.rs` 两个 `useless_conversion`、`region.rs` 的 `too_many_arguments` 和 `type_complexity`、`report.rs` 三个 `needless_option_as_deref` 共七处既存告警停止。`p5_corpus_fingerprint` 除文件核对外通过；共享工作树当前另有 61 个未登记的在途夹具，不能为泛型变更盲目重录。相邻 `p3_numeric_comparison` 的旧 BCI 期待和 `p3_required_conversions` 的旧固定 IR 计数也未通过，已在[移位邻接复核](../recover-shift-expressions/verification-root-3.2.md)独立记录。由于 2.3/3.1 的更广泛正文与调用绑定仍未证明，且全仓门禁未消除，3.3 保持未勾选。
