# CF-16 Test7 固定证据审计

固定 JADX checkout 为 2fb1b16386941660fda07e9017285aec40fcb37f；固定测试源码 SHA-256 为 c29cc1bbecbd910299da5c30ad94b92e3f534910f72d8517e1ce9107fb3c5d08。IntegrationTest 的默认 TEST_INPUT_PLUGIN 是 dx，只有环境变量取值 java 时才选 Java classfile input。Test7 的 test() 使用默认带调试信息配置，testNoDebug() 先调用 noDebugInfo()，两者都没有显式选择 input plugin。故这个固定测试默认覆盖 Java source → DEX → JADX；下文的 Java 8 classfile 是独立转写的 Java-input 对照，不能当作默认 DX 测试的物理输入。

测试是正例，但两个层面的约束不同。test() 要求输出有 try、exc(obj) 和具名 catch (Exception e)，并排除 throw th;；testNoDebug() 仅排除 throw th;。IntegrationTest 还会编译反编译结果并调用嵌套类的 check()，它验证 test(null) 返回 true 且 f 增加一次，也验证 "r" 触发的 AssertionError 逃出 typed catch 后 finally 仍让 f 增加一次。因此 Test7 的组合断言是行为正例，而单独的 no-debug 文本断言较弱。

Java-input 对照逐句保留固定源码中 TestCls.test(Object)、exc(Object) 与字段 f，只省略使用 AssertJ 的测试辅助 check()，以便把独立夹具用 javac --release 8 编译。fixture 源码为 [TestTryCatchFinally7.java](TestTryCatchFinally7.java)，测试运行器为 [Runner.java](Runner.java)。debug 与 no-debug 的目标 class 分别固定在 [baseline/debug/physical-class/](baseline/debug/physical-class/) 和 [baseline/nodebug/physical-class/](baseline/nodebug/physical-class/)；它们的 SHA-256 分别是 679d3163c709f59f0f9cf5db91f82ce142f1f18fa0f64cdabd95da2798e3d604 和 76ebe374cd152a2a836442605e2a6e651e7507989fae8d2a0f874c55a4e7690f。完整 javap -p -c -v 输出记录了对应的物理方法和调试表。

两个 class 的 test(Object) 是相同 Java 8 lowering。代码依 BCI 分为：0–5 调 exc 并将结果写入 slot 2；6–16 正常出口递增字段；19–32 捕获 Exception、把 false 写入 slot 2 后递增字段；35–49 catch-all 保存 Throwable 到 slot 4、递增字段并重抛；50–51 汇合后读取 slot 2 返回。异常表为 [0,6)→19 Exception、[0,6)→35 any、[19,22)→35 any、[35,37)→35 any。debug class 的 LocalVariableTable 将 slot 2 命名为 res，分布在 [6,19)、[22,35)、[50,52)；no-debug class 去掉局部变量表，但 BCI、指令和异常表完全相同。

JADX Java-input 对照输出一份可重编的嵌套类源码。原 fixture 和 JADX 源码都通过 javac --release 8；在 java -Xverify:all 下，正常的 null、逃出 typed catch 的 AssertionError 和另一正常值路径都逐字得到 true/f=1、AssertionError/f=1、true/f=1。运行明细在各模式的 original 与 jadx-java-input 日志中。

Jarde 的完整 class-source 只输出 constructor 与 exc，test(Object) 是 explanation-only，说明 local 2 crosses a quoted fallback region，边界为 0、19、35、50。完整类因此无法通过 Java 8 编译；javac 的实质诊断是缺少返回语句。这个目标有三份可观察的 finally 副本（正常、typed catch、catch-all），并且返回局部 res 跨 catch/fallback 区域使用；它不是已有“透明空 finally”形态：f++ 是可观察写入，且 typed catch 和 catch-all 共存。现有空 finally 机制不能据此计为已覆盖。

可重放脚本每次先在唯一临时 Cargo target 构建当前 checkout 的 CLI，然后构建两种 Java 8 输入、运行固定 JADX CLI、重编并执行 original/JADX 源码。输出目录必须为空。运行 openspec/evidence/java-syntax-2026-09-28/cf16-test7-audit/replay.sh /tmp/cf16-test7-replay 即可。默认固定路径是本地 pinned JADX checkout 和它的 jadx 安装；可用 JADX_CHECKOUT、JADX 覆盖。脚本核 pinned HEAD、固定测试 SHA、目标方法指令/异常表相同以及 original/JADX 行为，退出时清掉 Cargo target。baseline/ 保存本次 SHA、完整 javap、JADX 与 Jarde 源码和 javac/java 日志。范围只证明固定测试的输入 profile 与上述独立 Java classfile 转写；不证明 JADX 默认 DX 输出与 Java-input 相同，也不把 CF-16 其它 finally lowering 纳入结论。

## 实施验收

`acceptance/` 是本次恢复后的 fresh CLI 记录。debug 和 no-debug 两份固定 class 各有 32 条 `test(Object)` 指令与同一四行异常表；Jarde 完整类经 `javac --release 8` 重编，方法均输出一个具名 catch、一个 finally、一份 `this.f += 1` 和一个合流后的布尔返回，32 个 BCI 均有来源。`acceptance/summary.json` 记录固定原 class SHA、JADX revision 和 CLI SHA，`acceptance/*-jarde-class-source/runtime.log` 与原源码/JADX 日志记录正常值和逃出 typed catch 的 AssertionError 均使 `f` 恰增一次。

`probe/TestTryCatchFinally7.java` 只改 `exc(Object)`，加入抛 `Exception` 的路径，并将传入的 `AssertionError` 原对象重抛。`probe/replay.py` 逐字节比较 probe 与固定 debug/no-debug 目标方法的 52 字节代码及 32 字节异常表；`acceptance/probe/summary.json` 记录二者完全相同的 SHA。`CatchRunner` 以 `null`、`"e"`、同一 `AssertionError` 对象走正常、具名 catch、catch-all 三路，逐项核返回值、`f=1` 和 Throwable 身份。原 class、原 probe 源、JADX Java-input、Jarde 完整类均经 Java 8 编译与 `java -Xverify:all`，四方输出均为 `true/false/AssertionError`，每路字段增量一次；日志在 `acceptance/probe/`。

`neighbors.py` 从固定 debug class 冻结七份 verifier 有效近邻：错字段、错接收者、错增量、返回值改写、Throwable 改写、自保护行扩围、清理后重进受保护正文。SHA 与 `java -Xverify:all` 成功记录在 `neighbors/sha256-and-verify.txt`。Jarde 定向测试逐份拒绝唯一 finally 并核关键物理 BCI 来源；预算耗尽和取消均不交付半成品源码或来源映射。默认 JADX 集成测试仍是 DX profile；这里只将固定 Java 8 Java-input classfile 切片计为恢复。
