# 固定 Test12 三方法独立验收

root 在主线 `73f6ae0b` 的实现上独立构建 Jarde CLI，执行 `p3_shared_join_finally` 定向测试（10/10 通过）和 `replay-test12.sh`。固定同布局最小类的原 class、pinned JADX、Jarde 完整 Java 8 源码均重编并以 `java -Xverify:all` 跑完九路径，三份输出逐字相同：`test1` 正常出口为 `call-out-finally`、NPE 为 `call-npe-catch-out-finally`、IAE 为 `call-iae-finally`；`test2/3` 分别是 `call-finally`、`call-npe-catch-finally`、`call-iae-finally`。runner 还断言三方法的 IAE 路径实际抛出 `IllegalArgumentException`。固定 class 与八个两副本变体均通过 JVM 验证；定向测试确认八变体没有错误发布合并的 finally。

审阅证书与构建路径：两份清理用现有 SSA 值流核对当前实例字段、String 常量、`append(String)` 调用和唯一 `pop`；两条异常行和正常/异常出口闭合后才领有块。新 `FinallyCompletion::Joined` 与旧保存返回形态互斥，`test1` 的 BCI 55 留给独立后续块，`test2` 融合正常块的 BCI 45 由 Builder 在 finally 后发射一次真实 void return。内层具名 catch 只在该证书限定的 Frame 内开放，单个获证 catch `pop` 只在正文构建作用域中消去。定向测试断言物理 BCI 来源、唯一 block owner、预算/取消回滚；实现中候选块查找失败按 `None` 拒绝而不 panic。

`cargo fmt --all -- --check`、OpenSpec strict、证据 SHA 清单和 `git diff --check` 通过。代理已执行全量 `jarde-java --tests` 与 workspace check；root 的独立运行覆盖定向 10 测试及完整九路径复放。固定完整类的 `runTest` 仍有独立 Region 缺口，InnerClasses family 仍需单独验收，所以此处仅判固定 `test1/2/3` 与同布局最小完整类通过，不判整份固定类追平。
