## Context

固定基线与七条可观察路径见 `openspec/evidence/java-syntax-2026-09-28/cf16-test13-multisegment/README.md`；四个 verifier 有效近邻见同目录 `neighbor-root-acceptance.md`。固定 `test(I)V` 的五条异常行按物理顺序为 `[0,10)→44 Exception`、`[15,37)→44 Exception`、`[0,10)→56 any`、`[15,37)→56 any`、`[44,49)→56 any`。四份清理位于 `[10,14)`、`[37,41)`、`[49,53)`、`[57,61)`，分别后接 `return14`、`goto41→63`、`goto53→63`、保存的 Throwable 在 61/62 重抛。此处数字只是固定验收输入，产品判定不得依赖这些具体 BCI 或成员名。

现有 `guard.rs::shared_finally_candidate` 只接收两/三行；`FinallyCompletion`/`SharedFinallyCompletion` 只能记录两份正常完成。`region.rs::shared_finally_body` 和 `Plan.body()` 使用一段连续正文；`build.rs::SharedFinallyBuild` 也只记录一个 protected span。因此把两个三行候选叠加会重复认领第五行、handler 和 catch 清理，且无法证明四个出口。第一拒绝点是 `prove_finally_copy` 在 handler 56 的 `jre_guard_finally_copy`，后续不会产生可交给 Region 的完整计划。

## Goals / Non-Goals

**Goals:** 对同形状五行表输出一个具名 catch 和一个 finally；四种完成路径副作用与原 class 一致；物理 BCI、异常行、正常/异常边、唯一 owner 与停止原子性闭合。

**Non-Goals:** 任意数量的保护段/清理副本、任意 cleanup 表达式或 CFG、`FinallyOnce` 的其它保护形态、TWR 与 monitor。固定 JADX 的扩围反例会少执行一次清理，不能作为语义证明规则。

## Decisions

1. **只建立一个封闭的私有证书。** 在既有 Guard 的 finally 分派中识别五行的两个同目标具名行、两个同目标 catch-all 行与一个 catch-body catch-all 行。证书记录两个不相交正文段、具名 handler、四份 cleanup、四种 typed completion、五个 ordinal、全部 owned block 与事实 BCI；不增加公共 pass、IR 或依赖。替代的“两份 SharedFinally 组合”会共享 row5/handler/catch 副本而双 owner，不能采用。
2. **证明全局路径而不是相似文本。** 每段的异常范围只包含各自正文，三份正常清理均在自己的保护范围之外；catch-body 的保护范围止于 catch 清理。证明两个具名行和两个 any 行覆盖同一对正文段且次序匹配，最后 any 行只保护具名 catch 正文。借助 canonical CFG/NormalFlow 枚举所有正常入口、出口及异常边，验证早退和两个汇合路径必经且仅经一份清理；阻断额外入口、绕过边、自保护范围或竞争 handler。以同次 SSA 证明四次调用的精确目标、接收者来源、实参、唯一消费与效果次序，并证明 handler 存储的 Throwable 正是 `athrow` 的值。不可从方法名推断“清理”。
3. **Region 只扩展此证书的有界遍历。** `Plan.body()` 的单连续范围不能假装涵盖两段，需在私有 shape 中携带两个 segment 和完成记录，按证书拥有的节点集合构造一个 `Region::Try`，把具名 catch 正文以及后续 join 分开；每个物理块只被认领一次。失败时回滚整段候选并留下原拒绝。无需通用的多段异常区重写。
4. **Builder 复用源级 `try/catch/finally`。** 在既有构建 checkpoint 内保留 try 中的早退和 catch 正文，选一份受证 cleanup 作为唯一 `finally` 正文；其它三份物理副本、两个正常 transfer 和原异常保存/重抛成为来源事实。四种完成记录决定值/异常/后续汇合位置，不让后续块进入 finally。预算、取消、无法陈述成员或来源时不发布半份结构。
5. **对照以原 class 为基准。** 固定 class、pinned JADX 和修后 Jarde 完整 Java 8 源码运行七条路径；固定原与 JADX 在这七条路径一致。四个有效负例分别改变调用目标、分支、异常范围和重抛值，都必须拒绝错误归并，其中扩围样本以原 class 的重复清理为唯一行为 oracle。继续回归 Test12、共享 catch-all 与普通 finally，避免新候选抢占旧证书。

## Risks / Trade-offs

- [五行顺序相同但语义边不同] → 检查 canonical 边、保护覆盖、所有入口/出口和 Throwable SSA 身份；形状文本相似不足以通过。
- [四份清理调用中有隐式别名或额外消费] → 借现有方法解析与 SSA 完整性；无法唯一证明时原子拒绝。
- [有界 Region 将汇合块或 handler 双重认领] → 逐块 owner、每个物理 BCI 与五条行来源断言，负例和固定全类验收。
- [扩大既有 finally 候选优先级造成回归] → 只在精确五行候选上尝试私有证书，保留其它 Guard 路径与停止传播。
