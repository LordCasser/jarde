## Why

固定 `FinallyOnce.escaping()` 只有异常完成路径：原 class 和固定 JADX 均执行清理一次，JADX 用普通 `catch (Throwable)` 表达，Jarde 仍在唯一 catch-all handler 安全拒绝。现有 finally-copy 证书要求正常清理副本而该字节码没有；[边界证据](../../evidence/jadx-feature-inventory-2026-09-27/finally-exception-only-debt.md)表明应先复用普通 try/catch 路径，而非为源码关键词另建 finally 机制。

## What Changes

- 为单一 catch-all 行、仅异常完成的受保护体与保存原 Throwable 后重抛的 handler 建立有界证明。
- 使用现有 Try/Catch Region 和 Builder 输出等价的 `catch (Throwable)`，把无 CP class index 的 catch-all 类型明确表示为 `java.lang.Throwable`。
- 检查所有异常/正常边、handler 入口、原值身份和清理块归属；证据不足继续拒绝，不改动其它 finally 证书。
- 冻结原 class、pinned JADX、Jarde 的完整 Java 8 重编与验证运行，以及可验证的竞争行、正常出口、外部入口和异常改写近邻。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：对已证明仅异常完成的单行 catch-all 清理输出行为等价的普通 catch，并保持无法证明形态的保守拒绝。

## Impact

影响 `jarde-java` 的 Guard catch 识别、异常行类型表达、Region Try/Catch 所有权与 Builder catch header；不新增 pass、finally 形态或通用异常重写机制。已完成的 Test12/Test13 和普通具名 catch 必须保持原状。`FinallyOnce.handled()` 有具名行、竞争 catch-all 与正常清理副本，不属于本 change；其独立重复清理差距仍待解决。
