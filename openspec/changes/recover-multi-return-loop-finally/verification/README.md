# CF-16 固定 Test5 验收

基线为 `a97ecc1a928e1aacca05fcecc53f788ac611b699`。输入 class SHA-256 是 `4d5a64965763bf8c0a9b81cd0881a429bf715aa5d91110de8039fd992bc91827`；最终 fresh CLI 的完整 class-source SHA-256 是 `e3edb8d1f1db62014e74bfa27b7463250d1304309e8fbdec756ce5072f6ef8a3`，见 [输入输出摘要](input-output-sha256.txt)与 [Java 源](TestTryCatchFinally5$TestCls.java)。CLI 的报告为 `structured`、`fallbacks = []`。原始证据与固定 JADX 均来自 `openspec/evidence/java-syntax-2026-09-28/cf16-test5-multi-return/`。

Guard 只接受 `[21,34)→93`、`[44,83)→93`、`[93,95)→93` 三行，以及固定的 45 个指令起点和全部 canonical 边。第一处 `null→local 5→return`、第二处 `list→local 6→return`、handler 的原 `Throwable→local 7→athrow` 均按 SSA 验证；三份 `close()` 的目标和接收者一致。正常循环回边为 `76→53`。Region 走普通 do-while 构造，Builder 提交一个 `try/finally`；测试逐个检查目标方法的 45 个物理 BCI 均有来源。

运行 `sh verification/run-behavior.sh`：固定原 class、原 Java 8 源、固定 JADX 与 Jarde 完整 class-source 全部以 `javac --release 8`（固定 class 原样使用）和 `java -Xverify:all` 运行，四份输出逐字一致。九条路径覆盖空 `c`、`first=false`、循环一/两次、`first`/`load`/`toNext` 抛错、`close` 覆盖返回与正文异常；[固定结果](fixed.run.txt)记录操作序列、清理次数、返回值与异常对象身份，另三份结果在同目录。Jarde 完整类源码有一个 `finally`、一次 `close()`，并可独立编译；CLI 模板中的“不保证整个项目可编译”说明不影响此处实际编译结果。

`neighbors/generate.py` 冻结六个近邻。`sh neighbors/verify.sh` 逐个通过 `java -Xverify:all`，[verify.txt](neighbors/verify.txt)逐一记载 SHA-256。接收者不同、目标不同、保存值改写、自保护范围扩大、Throwable 改写、循环额外出口分别覆盖证书各边界。基线 Jarde 和当前 Jarde 的 `recover` 输出分别存于 `neighbors/baseline/`、`neighbors/recovery/`，六项均保留 `@bytecode` 而不发布 `finally`。`sh neighbors/run-recovery.sh /path/to/jarde-cli` 可重放当前结果。定向测试另证明预算耗尽及取消时正文和源映射同时为空。

验证通过：`cargo test -p jarde-java --tests --locked`（包括 Test3、Test4、Test11、Test13、TestEmptyFinally 的既有切片）、`cargo check --workspace --all-targets --all-features --locked`、`cargo fmt --all -- --check`、`openspec validate recover-multi-return-loop-finally --strict`、`git diff --check`。日志位于本目录。CI 同款 Clippy 精确允许列表加 `-D warnings` 的唯一报错是基线已有 `region.rs:3867` 的 `clippy::manual_contains`，不属于本改动；额外仅暂免该项后，完整 workspace Clippy 通过，见 [两份日志](clippy.log)及 [暂免日志](clippy-baseline-waived.log)。该基线 lint 由独立任务处理。

本验收仅声明固定 Test5 及这六个有效近邻。Test2、Test9 和其他编译 profile 不在此证书内。
