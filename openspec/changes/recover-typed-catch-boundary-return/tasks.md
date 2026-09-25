## 1. 冻结证明与拒绝边界

- [x] 1.1 将冻结的独立 `PlainMultiCatch`/`PlainRunner` 纳入定向回归，先钉 BCI 30/32 引用、同处理器双异常行与原 class 四行；组合探针的 `choosePlain` 只作同形状交叉验证，再验收独立类的完整返回。
- [x] 1.2 构造 Java 8 verifier 有效的独立负例，令命名行结束后同块仍有可能抛错或有副作用的指令，并钉其不得进入 `try` 正文；对 catch-all/`chooseFinally` 保留既有拒绝，逐一检查报告原因和 BCI 来源。

## 2. 复用现有 Region 异常边结算

- [x] 2.1 从同次恢复请求只读透传方法 `ACC_SYNCHRONIZED` 判定；仅对非同步方法、命名行匹配的规范块证明范围末端后缀是唯一无效果终止 `Return`，返回值 SSA 来源及半开范围位置精确，且无显式 monitor 或额外出口。以 1.1 正例和 1.2 负例验证，不修改所有权校验或异常图。
- [x] 2.2 将已证明的返回作为 `try` 的正常出口发射，保留多重 catch 合并、值求值位置和每个物理 BCI 只认领一次；断言 `PlainMultiCatch.choose` 无 BCI 30/32 引用，异常分支和普通分支均有返回。
- [x] 2.3 对证明失败、预算/取消和竞争 handler 保持原子引用与真实失败原因；复跑现有 typed-catch、nested-try、TWR、monitor 和 finally 定向回归，不允许 `chooseFinally` 被本规则收为错误的源码 `finally`。旧 catch-all 测试的唯一失败经同工作树受控 A/B 证明与本变更无关，见验证记录。

## 3. 独立整类验收

- [x] 3.1 root 用重建 CLI 导出未手改的 `PlainMultiCatch` 完整类，经 `javac --release 8` 编译后以 `java -Xverify:all` 跑四行，逐行与原 class 比较返回值；组合探针只检查 `chooseFinally` 仍拒绝，JADX 的正常路径 2/4 行重复清理另记为 finally 专项证据。
- [x] 3.2 root 审核异常表半开范围、SSA 返回绑定、来源和负例；运行适用 Rust 测试、格式/Clippy、`openspec validate recover-typed-catch-boundary-return --strict` 并清理私有 Cargo target，记录未解决的 finally 专项而不混入本变更。
