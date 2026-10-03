## 1. 取证与基线

- [x] 1.1 重放 sif 配对（WCallI/WCallC，SHA 核对）：定位匹配器 CP 种类枚举位与 agent 最小补丁判据复核；记录 4 个新折叠 corpus 类形态。判据位 = `src/facade.rs::project_static_fold_owner_texts` 内直接覆盖段匹配器（`matching` 的 `match &entry.kind`）；最小补丁的「+ descriptor 提及」后半句**不采纳**（`InvokeDynamic`/`Dynamic` 返回描述符分量已在 `af37db70` 的生产者锚落地，且直接匹配器加 descriptor 会放宽健全性）——**只加 owner 位置**。取证 `sif2/README.md` §1、`sif2/results/anchor-probe.txt`。
- [x] 1.2 冻结至少两个变体/负例：多接口方法调用族（`WCMulti`/`WCSibling`/`WCDefault`/`WCallI1`）、覆盖段不含 CP 索引负例（`NAnchor`，覆盖段仅 `astore` 无 CP 索引；`NLocal`，类级 no-capture 未证）；`java -Xverify:all` 前后记录。两个负例 before/after 文本**逐字节相同**。取证 `sif2/README.md` §3、`sif2/fold-outputs/`、`sif2/results/fold-output-sha256.txt`。

## 2. 判据位扩展

- [x] 2.1 InterfaceMethodRef owner 入匹配（design 决策 1）；WCallI 折叠、`F1` 产物重编行为一致；既有家族 diff 逐字不变。单 `match` 臂同权扩展；`M1`/`M2`/`FV1…FV5`/inner-class-folding/lambda-inline/nested-spelling/fcp 8 类两腿 SHA 全同。取证 `sif2/README.md` §2/§5、回归测试 `tests/member_class_static_folding.rs::interface_call_owner_anchors_the_fold_and_an_unnamed_segment_still_refuses`（回退 `src/facade.rs` 即 FAILED，可证伪）。
- [x] 2.2 负例保持拒绝；预算/取消不变。健全性要求（覆盖段含 CP 索引 / 异常处理器 / 生产者链）与预算/取消路径未改动。

## 3. 回归与验收

- [x] 3.1 全仓测试全绿、fmt、CI 完整 `-A` clippy、`openspec validate --all --strict`、diff check；磁盘纪律同前。`cargo test --workspace --tests --locked --no-fail-fast` **2903 passed / 0 failed / 46 ignored**（基线 2902，本片 +1 回归测试）；`cargo fmt --all -- --check` 通过；CI 实有 29 项 `-A` 清单 clippy 零警告；`openspec validate --all --strict` **261 / 0**（基线同为 261）。照例见 `sif2/results/{tests,clippy}.log`、`sif2/README.md` §6。
- [x] 3.2 WCallI 与变体三方对照：原 class/固定 JADX Java-input/Jarde 重编 `java -Xverify:all` 逐路径一致；记录输出 SHA。6 个新具名用例 + corpus 5 类的 Jarde 折叠重编全部 `javac --release 8` 0 错误、`-Xverify:all` 逐字等于原 class；JADX dev 文本在本片 6 个新具名 fixture 上输出不符、在 `Y1`/`F1` 形上不可编，**行为腿不可用**（口径同 `handoff.md`/`sif/README.md` §6/`fcp/README.md` §6）。取证 `sif2/threeway/`、`sif2/results/verify_behavior.sh`。
- [ ] 3.3 root 独立复核判据位、corpus 归类与三方行为，更新账本与巡查记录。**归类实测为 5 类**（非本条目原写的 4 类）：fcp 生产者锚已进基线，`SDDiamond` 是 owner 扩展 ∧ fcp 生产者锚的合取（强制 `static_fold_stored_type_anchor` 返回 `None` 后本变更对 `SDDiamond` 仍 `refused`，对 `SDAbstract`/`SDIndirect`/`pkg.SDPacked` 仍 `projected`）；本片 5 类与 fcp 8 类无交集。取证 `sif2/README.md` §5。