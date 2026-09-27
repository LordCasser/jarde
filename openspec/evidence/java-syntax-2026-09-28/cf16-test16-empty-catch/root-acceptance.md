# CF-16 固定 Test16：空 catch 与调用型 finally 的 root 验收

固定 JADX `TestTryCatchFinally16` 的目标 `test()V` 保留同一方法的两个真实异常行：`[0,3) → 9 Exception`、`[0,3) → 16 any`。三份物理 `doFinally()V` 调用分别从正常、具名 catch、异常清理路径执行。实现只在这一精确两行、十一条指令、四个 Canonical block 的形态下签发 FINALLY 证书；具名 handler 的 store 只绑定未使用的 catch 参数，catch-all 仍加载并重抛同一个 Throwable。与已有三行 shared-finally 证书分开，Region/Builder 沿原有 Guard 和 Try 通道一次性投影空 catch 与唯一 finally。

我审阅了两行范围、三份调用目标及 SSA/CFG 边闭合，使用新的专用 Cargo target 构建 CLI，独立运行 [`accept.sh`](accept.sh)。脚本重新编译固定源与可观察辅助方法，核目标 class 字节、原/JADX/Jarde 三份完整 Java 8 源码，并在同一 Runner 上以 `java -Xverify:all` 验证六条路径逐字相同：正常、具名异常、Error 逸出及三种清理自身抛错覆盖。Jarde 仅输出一次 `finally` 调用和一个真实空 `catch (Exception ...)`；全部 11 个目标 BCI 均可从 source map 查询。五个 verifier 有效的近邻（错误调用目标、清理被保护、异常行顺序颠倒、Throwable 改写、外部入口）都保持字节码引文，不发布错误结构。

独立 `p3_shared_join_finally` 17/17 通过，覆盖新形态、旧 Test12–14、源图与预算/取消原子停止。实现代理另外完成 `cargo test -p jarde-java --tests --locked`、workspace check、fmt；OpenSpec strict 与 diff check 均通过。独立运行输出位于 `/private/tmp/jarde-cf16-test16-root-accept`，验收专用 Cargo target 已清理。

验收范围仅为固定 Test16 的 Java 8 classfile 形态；Test17 的四行双具名 catch、Test11 的循环清理、原始 `FinallyOnce.main` 等仍独立待处理，CF-16 整单元未标为追平。
