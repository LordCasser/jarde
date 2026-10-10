# Root verification — preserve-proved-return-arm-loop-latch-origins

当前规划4/4、实现1/7，无本片生产改动或新CLI。先等待前片If来源产品1f386686c6a31813254a85fed48d2af0af84a61c自身CI38068627541独立接受与clean交付；不能用前片工具构建结果当本片验收。

root真实临时诊断v1因私有owner_count少解一层引用发生E0277，原patch/raw/source/argv均保留；root v2仅修**owner，实际cargo focused test1/0/0。独立核raw哈希/名称/shape：Loop5最后If16 joinNone，thenStraight19 return21，elseStraight22 SSA22,25，natural仅latch22/fullcanonical outgoing22Normal5、唯一owner/无continue leaf，诊断不是public扁平RegionRecord。全部52产品/测试/输入pins恢复，target峰值153324998，cargo clean341files/144.8MiB后target不存在。诊断接受记录results/diagnostic-acceptance-root-v1.json。

5GiB最低free/本仓target1GiB、一秒进程组guard按用户明确批准继续。规划strict实际340/0，见results/planning-strict-root-v1；后续文档更新需再strict。Luna只private准备最小patch/完整类collector和verifier，root未应用或运行；后续验收只能从新CLI实际命令/raw构造。

独立computed-init for债务见results/computed-init-for-debt-root-v1.md；全来源覆盖不能计为TestLoopCondition5 for断言或整个71单元闭合。后续整单元候选只读triage在results/next-ledger-triage-luna-v1.md，CF12重放计划尚待root对JADX harness自动check调用的审查更正，不用未显式调用check()推断没执行。
