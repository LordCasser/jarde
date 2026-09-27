## Context

[固定 Test11 证据](../../evidence/java-syntax-2026-09-28/cf16-test11-loop-finally/README.md)包含原 nested class、同 BCI/异常表 Java 8 可观察 probe、完整三方源码及重放。目标 `test(List)` 的 33 个 BCI 跨 0–79，真实异常行仅 `[0,4)→38 any` 与 `[38,40)→38 any`；正常清理循环在 BCI 4–35，异常清理循环在 40–76，原 Throwable 于 76–78 重抛。第二行只保护 handler 的 `astore`，不保护迭代清理。原/JADX 行为一致，Jarde 当前在方法级安全拒绝。

[第一处追踪](../../evidence/java-syntax-2026-09-28/cf16-test11-loop-finally/architecture-trace.md)证明 `region::recover` 在 Walker/FINALLY 之前把 handler-only SCC `{48,58}` 错判不可约；该 SCC 有唯一正常外部入口 48，但现有 `NormalFlowView::dominates` 只从方法 BCI 0 计算。[隔离实验](../../evidence/java-syntax-2026-09-28/cf16-test11-loop-finally/architecture-trace-followup.md)仅放行该 SCC 后，下一处是 BCI 0 的 `jre_region_exception_edge`：FINALLY 检查已运行，却没有两份循环的证书；自然循环表仍未收录 handler 的 BCI 48。两处都要处理，不能把预检消失当作语法恢复。

## Goals / Non-Goals

**Goals:** 固定双行 Java 8 lowering 的完整、可验证、可追溯 `try/finally` 循环；normal-flow 的 handler 组件循环事实仍保守且预算有界；现有普通循环和 finally 证书不回归。

**Non-Goals:** 任意重复 CFG 的通用归并、DEX/D8、多个异常清理副本、try-with-resources、将纯物理 handler loop 直接猜成原始源码写法。

## Decisions

1. **保留方法入口 dominance，另证异常组件的局部循环。** `NormalFlowView::dominates` 的调用者依赖 BCI 0 语义，不全局改成虚拟入口。对正常边形成且与方法入口不可达的组件，仅在有唯一可达根、没有第二个正常或异常入口进入循环内部、所有节点可从该根到达时，按同一现有图计算局部 idom，供 SCC 可约性和 natural-loop 发现使用；没有唯一根或预算不足就拒绝。多入口真不可约测试继续拒绝。选择这个私有事实而非把异常边并入 normal graph，是因为异常边的控制语义不同，直接合并会改变 branch/join 证明。组件的计费按已访问节点与边，停止不发布半成品。
2. **在现有 FINALLY 通道作有界双循环副本证明。** 两行必须精确匹配物理范围；正文唯一可抛调用受第一行保护，第二行只含不抛的 handler binding。以现有 `Facts`、canonical CFG、SSA 和 `Operation` 核两份 `List.iterator` → `Iterator.hasNext` → `Iterator.next` → 同一逐项调用：比较同一列表入口值与 `this`、同一符号调用目标、参数/返回及迭代分支，允许编译器选不同的 iterator/item local 槽，但绝不按文本相似删除副作用。核循环各只有一个入口、回边和退出，正常到 return、异常到原 Throwable rethrow；每个可能抛错的清理指令都在两行保护范围之外。原 Throwable 的 store/load/throw 必须保持 SSA 身份，额外边/行/副作用直接拒绝。既有简单调用清理证书不放宽。
3. **复用 Region 与 AST 的结构输出和一次提交。** 先取得完整物理所有权，再由已证两份循环建立一个 try 的 finally body，使用现有 iterable foreach 或等价有界循环表达；Builder 只在两个分支与来源均可呈现时省略异常副本，失败回滚并保留完整 quote。全部 33 个 BCI 须留在 source map。JADX 的 `MarkFinallyVisitor.processTryBlock`/`findCommonInsns` 提供沿各出口寻找重复清理的顺序参考；本实现增加异常范围、SSA、外部入口和全覆盖门，不照搬其通用 `DONT_GENERATE` 标记策略。

## Risks / Trade-offs

- **组件入口误判** → 只有一个正常根且无异常边进入循环内部时才发布局部循环；保留多入口、零入口、跨组件合流近邻的拒绝。普通方法入口 dominance 不改。
- **两份循环看似相同但副作用不同** → 同一符号调用目标、接收者和逐项值由 SSA 核对；迭代器/调用抛错路径分别执行对照。不同 local 槽只在明确的生产消费对应下重命名。
- **独立前置修正只把失败移到后面** → 任务与验收以完整 class-source、Java 8 重编、原/JADX/Jarde 运行一致为闭环；预检通过而 FINALLY 不成立不算完成。
- **输出/取消预算** → 组件分析、路径匹配和源码构建沿用现有计费/轮询，失败不发布半个 try、半个循环或未锚定的派生文本。
