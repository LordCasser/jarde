## Context

固定 JADX HEAD `2fb1b16386941660fda07e9017285aec40fcb37f`，`TestTryCatchFinally17.java` SHA-256 `471221838e2f6fa2240c75a68a1fc217880f22dd165d595a3f4e847eab37098c`。固定 Java 8 目标 `test()I` 的四条真实异常行是 `[0,3)→9 UnsupportedOperationException`、`[0,3)→16 NullPointerException`、`[0,3)→24 any`、`[16,19)→24 any`。正常清理在 BCI 3、第一 catch 清理在 10、提前返回前清理在 19、catch-all 清理在 25，四者均为同一 `invokestatic doFinally()V`，且都不在任何保护区。BCI 17–18 将常量 `1` 保存于局部，22–23 加载并返回；30–31 返回常量 `0`；28–29 加载并重抛原 Throwable。全部 18 个物理指令 BCI 必须有来源。

已固定的[八路径回放](../../evidence/java-syntax-2026-09-28/cf16-test17-two-catches/README.md)表明原/JADX 的完整 Java 8 类均可重编并经 `-Xverify:all` 运行，结果相同。JADX `MarkFinallyVisitor` 借 handler scope 的各完成路径比对重复指令；本任务借鉴“每条路径逐一证明副本”的原则，但以 Jarde 现有物理异常行、Canonical CFG、SSA 与预算为证据，不移植其删指令标记或创建通用图同构机制。当前 Jarde 的两行 Test16 证书和三/五行共享证书均不接受这四行事实，不能靠虚构行或放宽数组长度授权。

## Decisions

1. **单独的有界四行证书，复用 FINALLY pass。** 在现有 `shared_finally_candidate` 和 `guarded` 的四行分派上识别精确的两具名行、正文 catch-all 行及仅覆盖第二 catch 保存返回值前缀的 catch-all 行。逐指令核真实覆盖、四份相同且无参数/栈结果的解析调用、两个具名参数入口、catch-all 保存/重抛；不把第二 catch 的清理纳入其 catch-all 保护区。未知四行几何继续拒绝。
2. **先闭合所有完成路径。** 核 Canonical block 分区、正常边与异常边、每份清理没有外部入口或额外出口。正常与第一具名 catch 汇到同一返回 `0`；第二具名 catch 的常量 `1` 必须由保存槽经同值 SSA 传到唯一返回；catch-all 必须重抛进入它的原 Throwable。验证具名 catch 参数无不被源码表达的使用，清理抛错不能重新进入本方法的 handler。任何值、目标或覆盖竞争不明，拒绝整个证书。
3. **沿用 Try AST 与一次输出 checkpoint。** Region 在既有 Guard 所有权下构造一个 `try`、两个真实 catch 及 finally。第一个 catch 是经证明的空正文；第二个 catch 表达源级 `return 1`，其物理清理调用只作为 finally 来源，不重复输出。Builder 沿用 catch header、return、finally 的现有 AST/来源/回滚路径；若现有 `SharedFinallyBuild` 无法表达“一个共同返回 + 一个提前返回”，仅增加此证书的有界完成状态，不更改通用异常语义或公共 AST。
4. **以真实语义和负例验收。** 扩展固定 replay 到 Jarde 完整源码，不用手工补目标方法。原/JADX/Jarde 在正常、两个具名异常、Error 逃逸及四条清理自抛路径逐字一致。构造并冻结 verifier 有效的调用目标不一致、异常行范围/顺序变化、第二 catch 返回值改写、原 Throwable 改写、清理外部入口与清理落入自身保护区等近邻；全部拒绝唯一 finally。核 18 个 BCI 来源、预算/取消原子性，并回归 Test12–16、普通 catch、TWR/monitor。

## Risks / Trade-offs

- 固定 JADX 交换两个互不相交 catch 的源码次序仍保持行为；Jarde 以物理行顺序投影，不能用源码次序猜证明。
- 第二 catch 的 `return 1` 若只按常量文本匹配，错误的局部值别名可能被折叠；必须证明保存、加载和返回的 SSA 同值关系。
- 固定测试关闭编译检查；完整类三方重编与八路径仅支持 Java 8 形态，不推出 DX/D8 等价。
- 实现若发现通用图层对真实异常根有误判，应把该架构债单列，不以本窄证书修改所有方法的 CFG 规则。
