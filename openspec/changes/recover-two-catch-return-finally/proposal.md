## Why

固定 JADX `TestTryCatchFinally17` 是 Java 8/DX/D8 的正向 finally 断言，但原测试关闭了源码重编。已冻结的 Java 8 同布局样本证明原 class 与固定 JADX 的完整源码在八条验证路径中一致；Jarde 目前仅在目标 `test()I` 安全拒绝，不能表达两个具名 catch、其中一个提前返回以及四份清理副本。

## What Changes

- 在现有 FINALLY Guard/Region/Builder 通道内，证明固定 Java 8 四行异常表、两个具名 catch、正常与两个 catch 的三种正常完成、catch-all 异常完成以及四份相同的静态零参 void 清理；输出唯一 `try/catch/finally`。
- 对提前返回的 `1`、共同返回的 `0`、原 Throwable 重抛和清理自身抛错做 CFG/SSA 与异常覆盖证明；证明失败或预算停止时保留完整拒绝，不发布部分语句。
- 在[固定 Test17 基线](../../evidence/java-syntax-2026-09-28/cf16-test17-two-catches/README.md)上重编原/JADX/Jarde 完整 Java 8 类并运行八路径；增加 verifier 有效的错误近邻与来源断言。仅覆盖该 Java 8 lowering，DX/D8 另验。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：受证的双具名 catch 与一条提前返回路径可恢复为唯一 finally，同时保持正常/异常返回和物理来源。

## Impact

修改 `crates/jarde-java` 已有私有 FINALLY 证明和结构输出，不新增字节码 IR、公共 API、CLI 开关或依赖。Test16 的两行空 catch、Test12–14 的既有证书保持独立。固定 Test15 是别名负向断言，其 Java 8 等价场景与本四行形态不同。
