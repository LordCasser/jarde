## Context

见 `proposal.md`。冻结 `SharedFinallyCall.handled` 的异常行依 ordinal 为 0 `[4,21)→26 IAE`、1 `[4,21)→35 any`、2 `[26,30)→35 any`。正常路径 BCI 20 保存返回值、21 调用清理、24/25 读取并返回；具名 catch 路径 BCI 26 保存异常、29 保存另一返回值、30 调用清理、33/34 读取并返回；共用 handler BCI 35 保存原异常、36 调用清理、39/40 读取并重抛。slot 1 的正常返回与 catch 参数是不同定义。前缀 0–3 重置计数器，保护范围从 4 开始。

现有 `prove_finally_copy` 只有一条异常行和一个正常返回，`Frame::own_finally` 与 `finally_edges_accounted` 也只记一条，`region_at` 先尝试具名 catch，Builder 的 finally 分支硬编码空 catches。此前可行性审计已确认不能通过放松单出口证书、扩张局部词法范围或仅修改输出修复。固定 JADX 在正常路径调用两次清理，因此只作语法参考，原 class 是行为 oracle。

## Goals / Non-Goals

**Goals:** 为冻结调用形态建立一个私有原子证书，分别恢复 try `[4,21)` 和 catch `[26,30)` 的有界正文，复用现有 `Try` AST 输出一份 finally；两份返回值与原异常身份、每条路径清理一次以及物理来源完整闭合。

**Non-Goals:** `SharedFinally` 的 `cleanupCount++` 三副本效果、`FinallyOnce.escaping()` 的只有异常出口、多个具名 catch、其他清理操作、循环/switch 中 finally、一般局部作用域修复、通用异常 IR 或新 AST。原 `FinallyOnce` 的其他方法继续独立记录。

## Decisions

1. **整体证明三行，保留单出口证书原状。** 新私有证书要求 ordinal、半开范围、handler、catch 类型和行优先级同时匹配。三个清理副本各为一条相同目标的无参 `invokestatic cleanup()V`，按 SSA 使用关系证明两个 saved-return 在清理后原值返回，以及 handler 重抛最初收到的异常。每个副本只能有已证明入口；清理不能被对应 catch-all 再覆盖。这样不把字段读-加-写的可交换性误当成调用等价性。JADX 的副本查找顺序可参考，但其抑制重复语句和正常路径重复清理不能成为 Jarde 的所有权依据。
2. **两个受限子 Region 共同拥有异常表。** 证书先形成完整的 try/catch/finally 物理集合及三条异常行，随后从 `[4,21)` 和 `[26,30)` 各恢复一份不交叠正文。Region 在候选证书通过后才选择该形态，避免既有具名 catch 优先分支截走入口。两个正文、两个返回副本、异常副本和计数器前缀均必须一次认领并有直接或派生来源；无法闭合就撤回整个候选。
3. **复用 `Try`，在 Builder 中保留两份返回快照。** 在同一 checkpoint 内构造已有 `StmtKind::Try { body, catches, finally_body }`，而非新增节点。try 和 catch 正文分别保留原保存值与返回的定义/使用关系，catch 参数不得与正常返回的 slot 1 合并。唯一清理语句放入 finally。任何局部声明、source map、预算或取消失败都原子回滚。
4. **用完整类和原 class 验收。** 冻结 `SharedFinallyCall`、Runner、字节码与三方基线；分别重编原/JADX/Jarde 完整 Java 8 类并 `java -Xverify:all`。Jarde 正常/catch 返回与计数须等于原 class 的 `normal:1 / caught:1`，且 `handled` 不含引用。JADX 的 `normal:2` 单列。加入扩围 catch-all、row/handler 改动、副本额外入口/目标差异、返回或重抛关系不闭合、预算/取消的拒绝回归；不把未覆盖的 `FinallyOnce` 方法记为已恢复。

## Risks / Trade-offs

- [具名 catch 优先级或共享 handler 被错误折叠] → 三行 ordinal、两个半开范围及每条异常边共同证明；失败保留整体引用。
- [slot 1 复用导致错误返回或局部穿越] → 两条返回分别按物理定义/使用追溯，catch 参数另设作用域。
- [清理调用在正常路径重复或被丢弃] → 三份副本分别归属，运行计数以原 class 为准，并核 source map。
- [多子 Region 部分认领] → 在一个 checkpoint 暂存所有权和输出，任一环节失败撤回整个候选。
