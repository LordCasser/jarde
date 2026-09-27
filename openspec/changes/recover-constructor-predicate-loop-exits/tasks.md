## 1. 固定样本和实际值流

- [x] 1.1 在主线重放固定 `NotIndexedLoop` 的原/JADX/Jarde 完整类，核对原 class SHA、四条运行输出、当前拒绝 BCI 与正常 CFG，并记录 [物理阻碍](../../evidence/java-syntax-2026-09-27/cf08-endless-loops/not-indexed-fixed-triage-2026-09-28.md)。
- [x] 1.2 用同轮 SSA/Code 实查 BCI 25 构造对象的栈身份、BCI 38 谓词链及 local2 在 64→69 的 phi 链；在测试或验收记录中列出每个定义/消费者，确认与设计一致后实施。

## 2. 固定二层循环闭合

- [x] 2.1 在既有双出口 Region 候选中证明 `new; dup; ldc; invokespecial; astore; goto` 的初始化对象与唯一保存，保留旧调用返回值出口；用固定正例及 verifier 有效对象身份/额外消费负例核验。
- [x] 2.2 在同一候选证明循环体 `array load/store → File.getName → String.equals → branch` 的有序单次效果与接收者/结果消费；用固定正例及 verifier 有效调用/效果/分支变异核验。
- [x] 2.3 证明内层三来源 local2 在 64 的唯一一跳转送、外层 69 的 null 输入及判空/返回使用，复用既有尾续接；用额外入边、未定义值或消费者变异核验拒绝，并保证固定方法不再含 `@bytecode`。
- [x] 2.4 若 Builder 局部范围阻碍完整类编译，仅在该受证候选内修正范围/来源归属；核固定源码唯一 `while (true)`、没有索引 `for`，所有消费 BCI 及双层 join 均可定位且没有伪造或漏掉调用。

## 3. 三方与回归验收

- [x] 3.1 将原/JADX/Jarde 完整 Java 8 源码分别重编并以 `java -Xverify:all` 跑 null、空数组、含 `f`、含 `h` 四路，确认三方逐项输出 `null / null / f / h`，保存可重放证据。
- [x] 3.2 复跑已验收双出口循环、二层三来源样本和 verifier 有效负例；核预算/取消原子性，并运行 `cargo test -p jarde-java --tests --locked`、`cargo fmt --all -- --check`、`cargo check --workspace --locked`、`openspec validate recover-constructor-predicate-loop-exits --strict` 和 `git diff --check`，清理专用 Cargo target。
- [ ] 3.3 root 独立审阅 CFG/SSA/效果/来源证书，重放完整类三方运行及关键负例并记录验收；通过后更新 CF-08 清单，仅把已证固定形态标为恢复。
