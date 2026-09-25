## 1. 证据与调用归属

- [x] 1.1 固化 verifier-valid 反例、原/JADX/Jarde Java 8 重编运行和 `new@1` 误报；验收证据目录可用记录的命令复现 `CST/SCT/SCT`、BCI 与 class 哈希。
- [x] 1.2 从构造器物理实参沿现有 SSA 定义/读取建立本块的值依赖 BCI 集合，只让集合内的区间 `Invoke` 通过普通 `new@1`；验收定向单元测试对 BCI 4 的独立 `()V` 调用给出 `StatementFree` 拒绝，对真实调用实参仍通过。

## 2. 产物与契约验收

- [x] 2.1 验证拒绝的 `new`、独立调用、`<init>` 保留 bytecode/origin 且不宣称 Structured/已呈现；验收受控 fixture 的 class-source 报告及 `jarde-java` 定向测试。
- [x] 2.2 验证 essential、all、BCI range 的正文及站点判定一致，预算/取消保持既有停止语义；验收定向与相邻构造回归、`cargo fmt --check`、适用 Clippy 和 `openspec validate refuse-unconsumed-construction-invokes --strict`，记录私有 Cargo target 的清理。
