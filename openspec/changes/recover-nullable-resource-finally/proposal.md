## Why

固定 JADX `TestTryCatchFinally9` 把可空 `InputStream` 保存在同一个局部槽，返回字符串前或异常传播前仅在非空时关闭一次。Jarde 当前安全拒绝，且固定 JADX 的 Java-input 输出虽然可编译，却在资源存在时漏掉 `close()`；恢复必须以物理 class 的行为而非单一反编译文本为准。

## What Changes

- 对固定双行 catch-all、自保护 handler、正常/异常两份可空清理以及保存返回值建立有界物理证明。
- 在既有 Guard、Region、Build 中输出保留相同输入流局部变量的 `try/finally`，保持资源取得、Scanner 操作、非空关闭和异常覆盖的执行顺序；无法证明时安全拒绝。
- 用固定原 class、原 Java 8 转写、JADX DX 与 Jarde 完整源码做编译和运行对照，并把已证的 JADX Java-input 漏关作为负向参照。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`: 增加可空资源的双行 finally 与保存返回值恢复契约。

## Impact

影响 `jarde-java` 的 finally 证书、局部变量词法绑定、区域所有权和来源映射；复用现有 IR/AST 与预算/取消机制，无新增 crate 或对外协议。固定证据位于 `cf16-test9-catch-finally`。不覆盖 Test2 的多循环 void 清理、Test5 的双返回、DEX 输入解析或其他 Java-input 失真问题。
