## Context

见 proposal.md。root 实际基线为 24 命令/48 raw；原始 debug/no-debug 目标方法 Code 同为 42 字节、21 指令、表行 `[29,33)→36 Exception`。完整源码重编矩阵为 37 命令/74 raw。临时 stderr 诊断 CLI 已实际重放八个公开请求，递归去除 usage 后完整 JSON 同基线，随后恢复源码并 exact clean。

同次事实：normal IDs 顺序为 `[0,6,14,17,40,26,29,36]`，边为 `0→6,6→14/17,14→40,17→26/29,26→40,29→0,36→6`。normal view 排除唯一异常边 `29→36`。外层 natural set 实际为 `{0,6,17,29,36}`，包含逆前驱收集的 handler36；旧四块假设已被真实 trace 否定。catch join `36→6` 已证，irreducible 集为空；外层 header0 单 successor、唯一 latch29 单 successor，latch-chain/effectful-dual-exit/header-tested/latch-tested 全部返回 None，未进入 body 或 Try reader。

已有 retry 只支持保护区起点等于重试头、handler 回到同一头的顶层形。不能把原 RetryLoopFacts 的 header 随意改为6再假装 protected29等于它。现有 prefix+protected-loop 路径要求 loop 位于保护区内，也不满足本形。

## Goals / Non-Goals

**Goals:** 在同次 canonical/code/SSA/effects 下证明一个局部复合循环，复用现有区域与正文生成；完整保留原类六方法、四边界与所有来源，并通过原样整类编译运行。

**Non-Goals:** 不改变 normal-flow 的异常边语义，不新增 CFG、公共 AST、pass、solver 或依赖；不放宽声明/def-use、异常归属或所有权检查。只覆盖单具名行、单 protected canonical block、单直线 handler 的有界 MVP；多 handler、重叠/共享 row、jsr clone、其他 finally 与任意异常 SCC 分别留债。

## Decisions

### 1. 在外层 loop 分类处先证明整个局部组合

从当前 header、唯一普通 latch、物理表行及 handler 回跳推导外层入口、内层入口、保护区入口、正常续接和外层出口，禁止按 BCI、局部槽、方法名、常量池编号或源码文本匹配。入口前缀是一个已有可呈现的直线 canonical block；内层前缀是无环、单入口的有界普通子图，路径只到该 protected block 或经纯 Transfer gateway 到同一外层出口。

先核唯一具名物理 row 与 canonical row ordinal/范围/handler/throw sites，一条完整 protected block（允许 range-end goto 在保护区外），现有 CatchSite 的真实异常参数绑定，以及 protected 正常 successor 等于外层入口。handler 无普通前驱，只有一个真实 goto 回到内层入口；handler 自身未被本 row 保护，可能异常继续逃逸。

逐边核外来入口、出口、call/return、异常、clone 与 owner 闭合。内层入口只接受外层前缀及本 handler，前缀内部只接受本子图；所有外层退出 gateway 为无额外效果、无异常边的纯 transfer。只接受同一个真实外层退出。自然 set 中的 handler 仍只由 CatchClause 拥有，gateway 纳入组合 owner 集；最终块集与 visited 增量必须精确一致且无重复。多余节点/边/表行或无法闭合路径拒绝。

考虑过通用 endless loop 加全局异常 SCC：会改变其他循环和所有权分析，当前证据不需要。只放行 header 的单 successor 也无法恢复内层异常重试，故不能仅删 classifier 条件。

### 2. 复用 Loop/Try/Frame，显式区分 protected 与 retry header

输出为外层 Endless(header0) 的直线前缀后接内层 Endless(header6)。内层包含原分支、Try 和单 catch。到40的分支为外层 break；handler37→6为内层 continue；try33→0为内层 break，内层正常完成后外层尾部继续迭代。最后 return40 在外层循环后，只拥有一次。

上述 BCI 仅解释冻结输入；产品从实际边与证书派生。保留 goto14/26/33/37 的真实来源和同一物理方法身份，隐式外层回跳不能吞掉33的归属。

复用 guard::catches 与 try_level 的保护正文/参数/handler 构建，私有 retry seam 如需调整，只做必要的角色拆分：protected entry 与 retry header 不能混用，正常 exit 与外层 break target 不能混用。一个局部证书只存消费所需的边界和 owner，不复制 CFG/SSA。不得伪造一个违反原 RetryLoopFacts 不变量的值来绕过证明，也不得把本形插进现有顶层 retry 早拒中后整体放宽条件。

完整证明后才创建两个 loop frame。组合构建保留完整 checkpoint；任一 child fallback、owner不全、预算/取消/期限失败都恢复所有实际修改状态，原 StopReason 向上传递。最终整方法 coverage、overlapping-owner、效果和异常边门照常执行。

### 3. 预算与验收沿用原闭环

新增块/边/row/SSA 扫描逐项计费并 poll，集合受本次 view/预算限制，保持递归与默认栈。已有库和 Runtime 足以完成，不新增包；JADX 仅参考局部 loop/catch 归属算法，不复制其类或引入 synthetic splitter 全局改图。

Luna 准备私有实现与测试，root 审查后串行应用、构建。补同构改名、不同前缀效果及异常路径的完整 Java 类；Oracle 来自真实原 JVM，不猜常量。反例必须真正触达候选证明：错 handler 回跳、错正常续接、普通 handler 入口、额外/共享/交叉 row、clone/物理身份错配、带效果 gateway、额外入边及晚停止。不靠不合法字节码或更早的无关拒绝凑负例。

完整原始类与补充类均保持全部方法，debug/no-debug、child/bundle-child、default/all；最终 CLI 核全物理方法/BCI/UTF8 map，原样用 javac8 和23重编，再在两 JVM `-Xverify:all`、CodeSource及仅 rebuilt/driver/AssertJ 的隔离路径运行。原始 check、边界参数、同实例累计和预置计数均保留。JADX major55 的外层编译边界单列，不当错码；失败无 runtime 计数。

## Risks / Trade-offs

- handler 已在 natural set 被重复认领 → 分离普通前缀与 CatchClause owner，完整 owner multiset 和 visited 增量双核。
- 0/6/29/40 混同 → 证书独立角色，正常 break、异常 continue 与外层标签分别验证及运行。
- 候选扫描扩大费用或吞 Stop → 真正内部停止门、旧预算表与 full workspace 检查，禁止调高旧表掩盖重复扫描。
- 结构恢复后暴露独立 Builder 缺口 → 先取同次证据，不放宽声明门；若无法靠正确结构解决，记录债务并修订当前计划再继续。

## Migration Plan

root 串行集成，无新分支/worktree。focused正反/停止/map、完整双JDK矩阵、fmt/CI同款Clippy/MSRV/reader/指纹/P5与双fresh workspace后推送产品并核自身CI。构建每次重新核5GiB余量、target1GiB、每秒进程组守卫、默认栈、finally exact cargo clean。任务按实际完成勾选，最后审计真实HEAD与干净主线；本片不增加71/612整体完成数。
