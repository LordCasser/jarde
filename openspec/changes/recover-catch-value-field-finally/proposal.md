## Why

固定 JADX `TestTryCatchFinally7` 在具名 catch、两个正常完成与 catch-all 重抛中各执行一次可观察的 `f++`，随后返回跨 catch 保存的布尔值。Jarde 当前安全拒绝并生成不可重编的完整类；现有 Test3 四行 joined finally 只覆盖无返回值的 `unload()` 副本，不能证明这里的字段增量与保存返回值。

## What Changes

- 对固定四行异常表、三份字段增量、具名 catch 和 shared boolean 返回值建立有界证书，检查异常覆盖、字段目标、接收者与 SSA。
- 复用现有 Guard、Region、Build 的 try/catch/finally 与返回值构建，输出完整可重编的 Java 8 类；未证的形态保持安全拒绝。
- 以原 class、原源码、JADX Java-input 和 Jarde 对正常、具名 catch、逃出 typed catch 的错误做 verifier 运行对照，并冻结清理/异常表变异的 verifier 有效反例。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`: 增加具名 catch 共享字段清理与跨 catch 返回值的四行 finally 恢复契约。

## Impact

影响 `jarde-java` 的共享 finally 证书、局部值词法绑定、区域所有权和来源映射；无新增 crate、pass、AST 语法节点或对外 API。固定证据位于 `cf16-test7-audit`。默认 JADX 测试为 DX profile，实施目标是独立固定 Java 8 classfile；no-debug 文本断言本身较弱，不外推到其他 lowering。
