## Context

固定 checkout `2fb1b16386941660fda07e9017285aec40fcb37f` 的 `TestTryCatchFinally12.java` 有九条 `check()` 运行断言。Java 8 原 class 的三方法物理布局如下；本次基线的固定内部类 SHA-256 为 `d9b9cb676203d943ee3cf97d66e62dda2a637125d7f737be1e22ca275c209185`。

| 方法 | 具名/清理异常行 | 正常清理 | 异常清理 | 后续块 |
| --- | --- | --- | --- | --- |
| `test1` | `[0,5)→8 NPE`；`[0,29)→42 any` | BCI 29–38，前有受保护 `-out` | BCI 43–52，BCI 53–54 重抛 | 55 |
| `test2` | `[0,5)→8 NPE`；`[0,19)→32 any` | BCI 19–28 | BCI 33–42，BCI 43–44 重抛 | 45 |
| `test3` | `[0,5)→18 NPE`；`[0,5)→42 any`；`[18,29)→42 any` | BCI 5–14、29–38 | BCI 43–52，BCI 53–54 重抛 | 55 |

每份清理都是 `aload_0; getfield sb; ldc "-finally"; invokevirtual StringBuilder.append(String); pop`。固定 JADX 默认输出三个 `finally`，完整源码 Java 8 重编后九条结果与原 class 一致。当前 Jarde 三方法分别在 BCI 42/32/42 报 `jre_guard_finally_copy` 并整体引用。顶级类克隆和只保留字段、`call`、三方法的最小类具有逐项相同的三方法 BCI/opcode 与异常表；最小类移除 InnerClasses family 和 `runTest` 的独立 Region 多 owner 问题后，拒绝仍在同一 BCI。详细重放放在 `openspec/evidence/java-syntax-2026-09-28/cf16-nested-finally/`。

当前 `guard::prove_shared_join_finally` 的三副本/同一 join 证明只收当前实例 `Z` 字段写入；`prove_finally_copy` 的两副本证明只收保存返回并拒绝竞争异常行。`region::finally_body` 已能用受限 Frame 递归恢复 `If`/`Sequence`，但尚不允许内层 `Try`；`Region::Try`、`StmtKind::Try`、Builder checkpoint、SSA 来源与预算均已有。固定类 `runTest` 的 BCI 40 多 owner、内类 family 组装是其它边界，不能用本次 finally 证明顺手改动。

## Goals / Non-Goals

**Goals:** 从固定三方法恢复每条路径恰好一次的 `-finally`，让同字节码布局的最小完整类九组原/JADX/Jarde Java 8 行为一致；物理来源和失败原子性完整。

**Non-Goals:** 任意 `append` 链、可变接收者别名、不同参数的清理、清理中的分支或显式 return/throw、任意异常表图、固定类 `runTest` 与 InnerClasses family 的独立缺口。

## Decisions

1. **复用既有 Guard/Region/Builder，不另立恢复 pass。** `test3` 是现有 `SharedFinallyCompletion::Joined` 的三行/双 goto 形态，仅缺清理效果证明；在该候选中加入受限实例追加副本分支。`test1/2` 是单一外层 catch-all 的两副本和正常 goto，可扩展私有 `Shape::Finally` 的完成合同，在原 `SavedReturn` 路径之外表示 `Joined`，继续用 `Plan::join`；原保存返回证明不放宽。若实现能在现有形态内更少改动且表达同一互斥不变量，应选择更小改动。不要把两种物理布局误认成同一条固定三行证书。
2. **先证明调用的值流，再比较副本。** 每份只接受上述五步：`this` 来自本方法入口 `Local(0)`，`getfield` 的 owner/name/descriptor 一致，字符串常量相同，`append` 的 invoke kind/owner/name/descriptor 一致，调用结果仅被紧随的 `pop` 消费。每个 stack producer/consumer 关系按 SSA 和真实 BCI 核对，并证明没有额外使用；外部字段读取在每份清理执行时重新求值，不能假设 `sb` 在 try 前后不变。这个受限比较辅助函数可被两副本/三副本证书复用，不构造一般指令等价框架。
3. **异常表和完成路径闭合后才认领。** `test1/2` 的具名 NPE 行只覆盖内层调用，外层 catch-all 完整覆盖内层 handler（及 `test1` 的 `-out`），两份清理均在外层保护区外。`test3` 的两条正常路径各经一次清理后唯一跳向后续 return，共用异常 handler 必须保存、清理并重抛同一个 Throwable。证明全部普通/异常边没有中途入口、绕过清理、重复清理或竞争 handler；行范围扩至清理内部时必须拒绝。`Plan::owned` 不含后续 return 块，`Plan::join` 由 Region 继续扫描。
4. **已有有界正文表达嵌套 catch。** 对 `test1/2` 在提交 visited 前，以既有 Frame/`region_at` 在外层保护范围内恢复内层 `Region::Try` 和随后的 `-out`，核子 Region 的块、BCI 和 row 消费闭包与证书一致。当前 `region_at` 的两个 `own_finally.is_none()` 门会阻止 finally frame 内的具名 catch，`Frame::protected` 又会清空 `own_finally`；因此不能只把 `Region::Try` 加进正文白名单。仅对新证书的内层具名行开有界入口，同时保留外层 catch-all 对内层正文和 handler 的覆盖责任，逐条异常边/行核对归属。`guard::catches` 不能因为外层 plan 的 facts 含内层 handler BCI，就把真实 NPE 行当作合成清理行过滤；需要在这次有界读取中限定外层证书的可见性，不改变普通 catch 读取的全局规则。Builder 用已证明的正常清理产生唯一 finally 语句，异常副本、goto、handler 存储/重抛和异常行作派生来源；任何声明、正文、表达式、来源或预算失败都通过现有 checkpoint 整体回滚。
5. **JADX 只作算法参照。** pinned `MarkFinallyVisitor` 先分出口 scope，再比对清理副本并抑制重复输出，这个顺序可借鉴；Jarde 的等价条件仍由 JVM 异常表、CFG 和 SSA 证明。pinned `SameInstructionsStrategyImpl.java:34` 的常量参数比较把一个表达式与自身比较；现成常量不等价变体未触发错误输出，但此代码不能作为参数等价依据。无需新依赖；现有 class reader/SSA/Region/Java 8 发射器已覆盖解析、方言选择和运行选择。测试中的 JVM 验证/执行只检验产物，不进入产品路径。

## Risks / Trade-offs

- [实例 `append` 的结果或 receiver 被其它指令使用] → 核每份局部 SSA 消费和入口 `this`；不能证明就整候选拒绝。
- [异常范围含正常清理导致其抛错后再次清理] → 半开行覆盖检查先于副本折叠，并用 verifier 有效扩围负例验证。
- [内层 catch 被外层 Guard 吞掉或物理块双 owner] → 子 Region 完整闭包、row 归属和 Builder 来源集合相等后才提交 visited。
- [固定完整类另有 `runTest` 或 family 缺口] → 对固定三方法直接核 BCI/异常表；用同布局的独立最小完整类运行九路径；其它债务另记，验收不声称整份固定类追平。
