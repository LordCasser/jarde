# 固定 Test14 条件 finally：root 验收

验收对象是固定 JADX `2fb1b16386941660fda07e9017285aec40fcb37f` 的 `TestTryCatchFinally14$TestCls.test()V` 同布局切片。原 class SHA-256 为 `8857b84944f1a0c8ec0d8805d2ba0e7430dfadfda7629b4a14ea4064eb1d4ece`；消除无关 synthetic accessor 的最小完整类目标 class 为 `356108017a0d24be26105ea227e93e3089a8832411ba938d3e3ece4193ed35e0`。两者目标方法的 BCI、opcode、符号操作数及唯一异常行 `[0,14)→31 any` 逐项一致。

root 在合入 `e44c2caf` 前以独立 fresh CLI（SHA-256 `091936901a10511b6c2cfb2de513a12cd7b109f259ac3119882f9e8acf33e160`）运行 `tests/fixtures/p3-conditional-finally/replay.sh`。原、固定 JADX、Jarde 三份完整 Java 8 类均重编并经 `java -Xverify:all`，七路径逐字一致：空字段、正常清理、正文置空、正文抛错、置空后抛错、清理抛错、正文和清理均抛错；最后一种传播清理异常。Jarde 输出唯一 `try/finally`、正文和清理各一次 `if (this.t != null)`、各一次相应调用，无 `@bytecode`；23 个目标 BCI 均有来源。

root 审阅了单 catch-all 的 CFG 边与全块所有权、两副本各自独立的字段读取与调用 SSA、handler 原 Throwable 的 store/load/rethrow，以及 Region 两段有界 `If` 和 Builder 同 checkpoint 的原子输出。六个冻结近邻与新增 slot 0 改写、第二次字段读取改指其他字段、handler 调用改指其他方法，共九个 verifier 有效负例均拒绝结构化 `finally`，保留字节码引文；预算和取消不发布半成品。独立 `p3_shared_join_finally` 16/16 通过，`cargo fmt --all -- --check`、`git diff --check`、OpenSpec strict 通过。实施代理的全量 Java 测试及 workspace check 也通过。

结论只覆盖固定 Test14 的上述 `test()V` 形态；CF-16 的其它 lowering 与 `FinallyOnce.main` 剩余差距继续单独追踪。重放输出位于本机 `/private/tmp/jarde-cf16-test14-root-replay`，命令与固定输入保存在本目录及 `tests/fixtures/p3-conditional-finally`。
