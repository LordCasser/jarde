## 1. 取证与基线

- [x] 1.1 重放固定 A10/A11/A12（SHA 核对）：定位类字面量准入判据与 nested-spelling 可拼性事实的复用接口；记录 A12 两形与 A11 三形基线（引注文本）。（判据即 `decode.rs::source_internal_name`，全仓仅 `class_literal_type` 两处调用；A12 前 6 / A11 前 15 条 `@bytecode` 引注，转录见 evidence `results-values/*.{before,after}.txt`。）
- [x] 1.2 冻结至少四个变体/负例：多段嵌套 `A$B$C` 字面量、折叠域内字面量（jar 口径）、本地类字面量（拒绝）、匿名类字面量（拒绝）；各自 `java -Xverify:all` 通过并记录实现前后行为。（N2/fn2.jar 多段+jar 折叠、LC 四类本地/匿名负例均为既有冻结；新增冻结 A10.class、A9 家族+a9.jar、a11.jar、`fixture-variants/WC1`（字面 `$` 顶层类）、`fixture-variants/WV1`（数组形），SHA 与前后行为见 `results-values/fixture-sha256.txt` 与 README。）

## 2. 准入扩展

- [x] 2.1 判据复用可拼性事实准入嵌套名（design 决策 1–2）；A12/A11/A9.main 恢复、整类重编运行与基线逐字一致；A10 五形 diff 逐字不变。（A12 6→0、A11 15→0、A9 17→0 引注；A12/A11/A9/A10 重编运行逐字一致（`results-values/*-jarde-recompile.out`）；`A10.after.txt` 与巡查主线转录字节一致。N2 三形方法级恢复，重编工程编译通过；结构反射两线偏离属折叠深度域遗留 1。）
- [x] 2.2 负例保持拒绝；呈现两口径（折叠/分离）按 nested-spelling 既有规则锚定；预算/取消原子性不变。（LC 2→2 保持拒绝；折叠域 A12/N2/A11/A9 简单拼写、分离域 N2/WV1/WC1 池形；预算/取消契约测试无改动全绿。）

## 3. 回归与验收

- [x] 3.1 `cargo test --workspace --tests --locked --no-fail-fast` 全绿（含 nested-spelling、类字面量、反射相关全部既有测试）、fmt、CI 完整 `-A` clippy、`openspec validate --all --strict`、diff check；磁盘纪律：每轮构建前 `df -h /`，低于 12Gi 先 `cargo clean`，报告前必 clean。（实测 296 目标 / 2931 passed / 0 failed、fmt 干净、CI clippy `--all-features` exit 0、openspec 268/268、`git diff --check` 干净；全程磁盘 ≥25Gi。）
- [x] 3.2 A10/A11/A12 与变体三方对照：原 class/固定 JADX（dev）/Jarde 重编 `java -Xverify:all` 逐路径一致；记录输出 SHA。（`results-values/`：jadx-*.out、*-jarde-recompile.out、fixture-sha256.txt；JADX dev 的 defpackage 重打包使 getName 路径自身偏离原类，已按路径记录口径。）
- [ ] 3.3 root 独立复核准入判据、负例边界与三方行为，更新 EM 账本（反射/类字面量域）与巡查记录。
