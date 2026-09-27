# CF-16：仅异常完成的清理边界

`openspec/evidence/java-syntax-2026-09-27/cf16-finally/input/FinallyOnce.java` 的 `escaping()V` 是与 Test13 多段清理独立的形态。输入源码 SHA-256 为 `5b712ee4532579ae16fefb8f378b1be452dccb128f1687f380f1ee109e7396a1`，原 class 为 `3ad6857285368c95c3176a520300c4abba09084443fe7fc08e8972330e9cedd2`。异常表仅有 `[4,15)→14 any`：受保护体在 BCI 13 主动抛出 `IllegalStateException`，handler BCI 14 保存原异常，BCI 15–20 更新计数，BCI 23–24 加载同一异常并重抛。原 class 经 `java -Xverify:all` 输出 `state:1`。

主线基线在 BCI 14 报 `jre_guard_finally_copy`，Region 还将异常专有块 14 记为 `jre_region_uncovered_blocks`。`guard.rs::finally_copy` 能看见“保存异常、清理、重抛”，但 `prove_finally_copy` 假定存在正常路径的清理副本/保存返回；本例正常切片在 BCI 13 以 `athrow` 结束，因没有正常副本无法配对。现有拒绝文案声称“normal path also runs”与本例不符，诊断质量可单独修正。

固定 JADX `2fb1b16386941660fda07e9017285aec40fcb37f` 输出等价的 `catch (Throwable th) { cleanupCount++; throw th; }`，并未提取 `finally`。对 Jarde 有两个候选方向：在精准证明 catch-all handler 和 Throwable 来源后输出同样的普通 catch，或在现有 Guard/Region/Builder 中增加 exception-only completion，输出更贴近原源码的 `finally`。选择前需核现有普通 catch 证书是否可安全接纳 catch-all；不能仅因文本目标是 finally 就新增一个机制。无论哪条路径，都必须证明保护体的完成路径、清理不自保护、handler 无分支/竞争入口、原 Throwable 的 SSA 身份及清理异常优先级。

只读邻近审计已用 `javac --release 8` 与 `java -Xverify:all` 验证两类不能折叠的形态：handler 清理按分支产生不同效果，以及嵌套两个 finally 且内层清理抛错时外层仍执行。临时源码已清理，后续实施前要冻结可重放的正反例。该项不属于 Test13 五行证书或当前 Test12 `runTest` 的实现范围；CF-16 仍未追平。

2026-09-28 更新：此处原为实施前审计，最终选用普通 `catch(Throwable)` 路径，见[独立 root 验收](../java-syntax-2026-09-27/cf16-finally/exception-only-root-acceptance-2026-09-28.md)。它只闭合固定 `escaping()` 的一条异常行；`handled()` 和 Test14 仍是独立未完成切片，CF-16 整单元未追平。
