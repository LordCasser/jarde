# CF-16 修复后验收

`replay.sh` 用固定 SHA 的 `TestTryCatchFinally12$TestCls.class`、同方法 BCI/opcode/异常表的最小完整类和 pinned JADX 运行三方比较。两套完整类的原 class、JADX、Jarde 均用 `javac --release 8` 重编；`java -Xverify:all` 对 `runTest` 的三个 case、default 和异常组合各跑九条路径。两套 Jarde 输出逐字等于对应原 class，详见 `results.txt` 与六份 `*-run.txt`。运行轨迹同时记录各次调用的追加效果、返回值和异常 catch 顺序。

产品测试 `cf16_named_catch_owns_complete_switch_and_range_end_transfer` 对固定方法检查 `[11,61)→64` 原异常行、全部 canonical 正常/异常边、`Try→Switch→Catch` 文本、各 case 调用一次、全部物理指令的来源和根 Region owner `[0,40,48,56,64] / [61] / [79]`。BCI 61 只有独立 `Straight` owner，在 Try 源码节点上是 derived 来源；case 末尾被消去的 goto BCI 45/53 也各有 derived 来源。

`variants/` 是 `make-variants.py` 生成的 Java 8 源以 `javac --release 8 -g` 编译的 verifier 有效 class。三个部分 case 保护样本不被提升为整 switch 外层 catch；真实 TWR 样本仍保持 `jre_guard_resource_init` 拒绝。额外前驱、竞争异常入口、额外/重复出口和非正常出口无法在本固定布局中直接得到 verifier 有效变体，因此 `region::tests::range_end_transfer_rejects_competing_entries_and_exits` 用受控 edge 集验证证书拒绝。预算、取消的产品测试检查无部分正文或来源发布。完整 `InnerClasses` family 与 Test13 的独立债务不在本固定方法验收范围内。

本轮通过 `cargo test -p jarde-java --tests --locked`、`cargo check --workspace --locked`、`cargo fmt --all -- --check`、OpenSpec strict 与 `git diff --check`。使用专用 Cargo target；验收后清理。
