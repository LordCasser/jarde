## Context

`StmtKind::If` 已含 `then_body` 与 `else_body`，每个 `Stmt` 有自己的 `OriginSet`。[CF-03 chain 冻结源码](../../evidence/java-syntax-2026-09-27/cf03-branches/baseline/chain/source/jarde/cf03/ChainOnly.java)中每一层 else 都只有一个子 `If`；JADX 写出三个 `else if`。`emit.rs::stmt` 当前对所有非空 else 都写 ` else {\n`，然后在更深缩进输出子 `if`，导致仅样式上的差距。`Emitter::stmts` 用 `node(&stmt.origin)` 记录 span，且 `stmt` 计数；直接省掉对子节点的调用会丢来源和预算约束。

## Goals / Non-Goals

**Goals:** 只对直接单子 `If` 链输出 Java 的 `else if`；保持各条件的求值顺序、原 AST 和每个子节点的来源区间；完整 Java 8 类源码重编及验证运行与原 class 一致。

**Non-Goals:** 修改分支结构、合并条件、压平有额外语句或 fallback 的 else 块、修复共享尾区域、对 `if` 条件做代数简化。

## Decisions

1. **在 emitter 内复用现有 AST。** 仅匹配 `else_body.as_slice() == [Stmt { kind: StmtKind::If { .. }, .. }]`，向同一行写 ` else if`；否则沿原 else 块输出。链的每一层都只读已有子节点，不改 `Stmt`、Region 或其锚点。
2. **为子 `If` 保留完整记录。** 子节点的文本依旧由其 `OriginSet` 包裹 `node`/source-map replay，计入一条语句并按既有预算停止。`else` 与 `if` 交界的来源采用原父、子锚点，不允许无来源文本或把子节点映射成父节点。
3. **用副作用和拒绝边界验收。** `ChainOnly` 五行输出检查条件调用次数和默认分支效果，JADX/Jarde 均须完整重编。构造含多句 else、非直接 `If`、fallback 的目标 emitter 测试，确认不折叠；source-map 重放、低预算/取消不得发布半段结果。

## Risks / Trade-offs

- [拼写优化吞掉子节点来源或少计预算] → 为每个子 `If` 保持原 `node` 包裹并核对来源/计数；测试比对重放区间。
- [多句 else 被错折叠改变作用域] → 只匹配恰好一个直接子 `If`，其余走原分支。
- [把 CF-03 共享尾编译差距误算为修复] → 保留 `BranchShapes` 基线和单独账本状态；本任务只结 `ChainOnly` 的已证样式片。
