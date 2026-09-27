## 1. 冻结多出口基线与可行性

- [x] 1.1 提取仅含 `handled`、计数器及外部 Runner 的最小 Java 8 类族；catch 返回可用字面量以隔离本项。固定原 class/JADX/Jarde 完整源码与 `javac --release 8 -g:none`、`java -Xverify:all` 结果，核真实异常行 ordinal、BCI/SSA 与三份副本。原 class 为运行 oracle，JADX 的实际结果单独记录；冻结 `FinallyOnce` 的已证 `normal:2` 差异不得被当成等价目标。
- [x] 1.2 在现有 Guard/Region/Builder 逐点验证三行、两返回、共用 handler 的有界子正文所有权和局部声明可行性；若无法完整表达，保存拒绝与具体阻碍，不放宽旧证书或输出部分 try。

## 2. 私有多出口证书与投影（仅在 1.2 可行后）

- [ ] 2.1 按 row ordinal、半开 BCI、正常/异常边和 SSA 证明唯一具名 catch、共享 catch-all、两个 saved return、同异常重抛及三份等价清理；不同目标/实参、额外入口、扩围保护、预算/取消均拒绝。
- [ ] 2.2 复用现有 `Plan`、受限子 Region 与 `Try` catches/finally 输出，确保全部 owned BCI 一次认领、三条异常行由证书记账，source map 显示指令的直接/派生来源，失败回滚为整段引用。

## 3. 三方行为与回归（仅在 1.2 可行后）

- [ ] 3.1 最小类族的原/JADX/Jarde 完整 Java 8 源码各自重编和验证运行；断言 Jarde 与原 class 正常/catch/异常路径语义一致，固定 JADX 正常多清理一次单列差异。原 `FinallyOnce` 的其他方法不据此结项。
- [ ] 3.2 复放已受证单出口 finally、具名 catch 和扩围异常行负例；运行定向 Rust 测试、`cargo fmt --all -- --check`、`cargo check --workspace --locked`、`git diff --check`、`openspec validate recover-shared-catchall-finally --strict`，清理专用 Cargo target，供 root 独立验收。
