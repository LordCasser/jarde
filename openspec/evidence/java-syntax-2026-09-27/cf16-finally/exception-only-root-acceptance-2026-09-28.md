# CF-16 单行仅异常完成 catch-all：root 验收

主线合入 `824b850c` 后，root 独立审阅 `guard::exception_only_catch`、`CatchTypes`、普通 Try/Catch Region 与 Builder header。证书只接纳一条 catch-all 行；固定 `FinallyOnce.escaping()V` 行 `[4,15)→14 any` 的 handler 入口 `astore_0` 是范围内唯一非抛出指令，清理 BCI 15–20 在范围外，BCI 23/24 重抛从入口保存的同一 Throwable。受保护体在直线块以 `athrow` 终结，无正常完成边；静态 int 字段增量要求同一字段/常量、SSA 单一消费与完整边所有权。具名 catch、多 catch 继续由 CP 类型表示；此证书产生的 `ProvenThrowable` 才拼写 `java.lang.Throwable`。没有新增 finally 形态或通用异常重写。

原固定 class SHA-256 `3ad6857285368c95c3176a520300c4abba09084443fe7fc08e8972330e9cedd2`。root 用专用 fresh CLI 重新执行 [`replay.sh`](../../../../tests/fixtures/p3-exception-only-catchall/replay.sh)；只替换无关 `handled()` 的类与最小运行类逐 BCI/opcode/异常行均等于固定 `escaping()`。原、固定 JADX、最小类的 fresh JADX 和 Jarde 完整 Java 8 源码均能重编并 `java -Xverify:all`，目标结果为 `java.lang.IllegalStateException:state:1`；身份对照原/Jarde 同为 `true:java.lang.IllegalStateException:same:1`。Jarde 输出普通 `catch (java.lang.Throwable)`，清理一次，无 `finally` 或目标方法引用；14 个物理 BCI 均有来源。

七个 verifier 有效近邻的原 class 路径已记录在 [`README.md`](../../../../tests/fixtures/p3-exception-only-catchall/README.md)：正常出口、异常范围缩/扩、竞争行、外部入口、改写重抛值和分支清理。root 重放后均未错误生成 `catch(Throwable)` 或 `finally`。独立定向 Rust 测试 4/4 通过，包括块 owner 唯一、来源、异常身份及分析/输出预算和取消的原子停止；代理执行的 `cargo test -p jarde-java --tests --locked`、workspace check、格式、OpenSpec strict 与 diff check 全通过，root 另行确认 OpenSpec strict 与分支 diff check。root 专用 Cargo target 随验收清理。

原完整类还含独立的 `handled()` 和 `main()` 拒绝；这里的三方可运行完整类是目标方法布局不变、无关方法缩减的验收载体。该结果只勾销 `escaping()` 的单行仅异常完成切片。`FinallyOnce.handled()` 的拼接保存返回及 Test14 条件清理仍按各自 OpenSpec 处理，CF-16 整单元未追平。
