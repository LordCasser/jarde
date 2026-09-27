## 1. 已有 AST 的拼写

- [ ] 1.1 在 `emit.rs` 的现有 `StmtKind::If` 分支仅对单子 `If` else 输出 `else if` 链；多句/非 If/fallback else 保持原格式，运行 emitter 目标测试。
- [ ] 1.2 保留子 `Stmt` 的 `OriginSet` source-map span、语句数和预算/停止行为；用嵌套链来源及低预算/取消目标测试验证。

## 2. 固定三方验收

- [ ] 2.1 重放 [CF-03 脚本](../../evidence/java-syntax-2026-09-27/cf03-branches/replay.py) 的 `ChainOnly` 切片：原 class、JADX、Jarde **完整类源码**均 Java 8 重编、`java -Xverify:all` 五行相同，Jarde 三个 `else if`；保留 `BranchShapes` 共享尾的独立差距事实。
- [ ] 2.2 运行相关 emitter/source-map 测试、`cargo fmt --check`、`cargo check --workspace` 及 `openspec validate spell-proved-else-if-chain --strict`，更新验收报告并提交；清理 Cargo 产物。
