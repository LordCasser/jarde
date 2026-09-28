## Why

固定 JADX `TestTryCatchFinally3.TestCls.test(ClassNode,List)` 的 Java 8 方法在正文循环、具名 `catch(Exception)` 和三个 `unload()` 副本之外还有一条指向自身的 handler 绑定行。原类与固定 JADX 在已测路径上执行一次清理；当前 Jarde 对整段方法安全拒绝。Test4 的四行证书针对清理内部的 `IOException` catch，不能证明这里的正文 catch 与循环。

## What Changes

- 在既有 FINALLY Guard/Region/Build 通道，为固定四行 shared-join lowering 证明正文循环、具名 catch、三份同一接收者的 `unload()`、handler 自保护绑定及原 Throwable 重抛，输出一次 `try/catch/finally`。
- 扩展固定证据的可注入路径：正文正常、`load`/visitor/logger 抛错、清理抛错及异常覆盖；核方法级 Java 8 重编/验证运行、全部 BCI 来源与 verifier 有效近邻，证明不足或停止时保留拒绝。
- 把 `LOG` 的 `<clinit>` 源码投影作为独立的类初始化缺口记录。本 change 只验收 `test` 的方法级恢复，不以手工补齐静态字段的整类源码冒充完整 class-source 通过。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：受证的具名 catch、正文循环与共享 catch-all 清理可以在四行自保护布局下恢复为唯一 finally，同时保持异常覆盖和物理来源。

## Impact

限定在 `crates/jarde-java` 现有异常证书、区域与源码投影及对应证据，不新增公共 API、CLI 选项或依赖。先决条件是已验收的 Java 8 循环、catch、FINALLY 基础；不扩展到 DEX/D8、任意四行异常表、资源关闭模式或独立的静态字段初始化。
