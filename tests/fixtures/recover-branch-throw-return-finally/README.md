# 分支 throw / return 穿 finally

`BranchThrowFinally.java` 与封存 EX 的 finThrow 同体。目标 class 只含构造器和该方法；driver 单独编译，通过临时替换 System.out 注入清理异常，不改目标代码。双编译器：Corretto javac 1.8.0_432 的 `-source 8 -target 8 -g:none -proc:none`，OpenJDK javac 23.0.1 的 `--release 8 -g:none -proc:none`。实际工具、命令和原始运行见对应 OpenSpec 的 baseline-root-v1。

两份 class 均为 492B、Java8，finThrow Code38B，唯一 catch-all `[0,17)→27`，20个物理指令。九个 driver case 覆盖零/正值/负分支，以及清理 RuntimeException、Error、null receiver 对正常返回与主异常的覆盖；已知清理异常核对象身份。原始主异常在目标内部创建，其身份由 SSA 证明，不冒称动态直接观察。

`boundaries/EXExtraExit.java` 是 javac23 的真实额外正常出口邻居，保留完整 class。它有两条保护范围及三份 cleanup，单完成副本证书必须拒绝；不要求其它已有完整证书也拒绝。三份目标 class 共新增7个 Code、4条 handler、4个直接 branch target，无 jsr。driver class 不放入目标目录。

- `BranchThrowFinally.java`: 175B / SHA256 `d789e5f16a0b2f2e45bc17e5141fa26c308971bf3291d3cd969134db872f500e`
- `BranchThrowFinallyDriver.java`: 4000B / SHA256 `31abc061a81114dda2b707cb31bbd2185200e8f3927b210f51ebaf7b5ce8f929`
- `boundaries/EXExtraExit.class`: 443B / SHA256 `262d3de06d949ba254713566bc32adec9d9e2d361c2c16ae51cc214aac669839`
- `boundaries/EXExtraExit.java`: 378B / SHA256 `e70d1250a4f5b0ec51e865022f9cfd69b0e63870588355329f2f96ca5f84e7f8`
- `v8/BranchThrowFinally.class`: 492B / SHA256 `fbe9f7e2194d961a5a8908d850f0dccfbcbf732872873126d5d74a2552ca3632`
- `v8-javac8/BranchThrowFinally.class`: 492B / SHA256 `16dcc6ecf5af4ba2b572934b8208c774352eb96bdfc8adf3fd405b1d245ec01d`
