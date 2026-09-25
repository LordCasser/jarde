## Context

`Walker::region_at_inner` 在 `visited.insert` 失败时构造局部 `FallbackReason::Loop`。这能阻止某些重入无限递归，却不能撤销此前已提交的 arm：额外入口样例的 Region 树多次持有相同 `CanonicalBlockId` BCI 25，后继 BCI 30 被留成无栈来源的 Straight。方法输出因此是一组彼此冲突的局部所有权决定。现有 `quoted_whole(live, reason, canonical)` 已用于不可约图、交叉异常区及无法交代的指令，可作为拒绝载体。

## Goals / Non-Goals

**Goals:** 在呈现 Java 前发现同一 canonical 身份被多个 Region 认领，原子丢弃整个部分结构；输出一个准确的所有权冲突理由，保持所有已解码 live 指令及未入 canonical 的指令可追；预算或取消拒绝时不提交半个结果。

**Non-Goals:** 将 `(gate ? extra : left) || other || rhs()` 恢复成 Java 表达式，改变 `IfRegionMaker` 式条件合并策略，修改 CFG/SSA 或异常图，修正字段 rule-detail 的 `presented` 含义。字段报告问题另列在 roadmap，不能因正文 fallback 就声称解决。

## Decisions

1. **查 Region 树，不从 BCI 猜重入。** 方法级 walk 完成后，以 `Region::blocks()` 枚举 canonical 身份，按 `CanonicalBlockId`（含路径）去重；同一 BCI 的不同 `jsr` 路径不是重叠。先审计 switch fallthrough、loop header/test、try handler 对这一定义的既有合法复用。校验扫描前按 Region/块数收费并逐步轮询取消。
2. **覆盖已有部分结构，而非尝试局部补洞。** 发现真实重复身份时，弃掉已建 Region 树，复用 `quoted_whole` 的所有 live block；诊断使用明确的所有权冲突代码（而非 `jre_region_loop`，因为该样例无循环）。已有 region 起点、consumer 和被访问集合不得继续让报告显示结构化方法。全局拒绝是保守且可检验的边界；日后若局部闭合证据足够，可缩小拒绝范围而不改变本契约。
3. **来源与其它缺口保持正交。** Whole-body fallback 的逐指令 BCI 来自 `preserve-region-fallback-instruction-origins` 的已验证采集路径。若 canonical 还遗漏 decoded 指令，继续发布 `UnaccountedInstructions` 来源，不能因替换 Region 树而吞掉 P3-R7 的独立事实。精确 quote/source map、去重及预算是依赖 change 与本 change 的联合验收。
4. **只凭实际正文判定语义。** 该负例引用正文允许可编译却不等价；`quality=fallback` 与 `representation=mixed` 必须明确，不能把 JVM 对照用作 fallback 语义等价证明。字段 rule-detail 目前可能标 `presented=true`，需在独立报告层任务中用最终 AST/发射记录纠正，不能靠本次改名或删字段事实掩盖。

## Risks / Trade-offs

- [合法共享 Region 被误判] → 以 canonical 身份而非 BCI 判重，跑 loop/switch/try/jsr 与既有结构化语料，发现既有合法重用则先明确该 Region 的所有权语义。
- [整段引用遗漏方法尾或无 canonical 指令] → `live` 覆盖所有可达块，P3-R7 的独立未覆盖指令追加引用；逐条来源由依赖 change 验证。
- [全方法拒绝过宽] → 当前没有安全的局部闭合证据，整段 quote 比伪 Java 更诚实；限制触发条件为重复**同一** canonical 身份。
- [额外扫描超预算] → 先收费再扫描；预算/取消按现有 StopReason 结束，无部分产物。
