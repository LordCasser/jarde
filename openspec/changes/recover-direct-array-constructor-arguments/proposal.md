## Why

按 JADX 71/612 固定账本新编译原始 TestFinally3 后，debug/noDebug 的 finally 均已恢复，但完整类被 `new ByteArrayInputStream(new byte[0])` 阻断。实际 SSA 表明数组唯一消费者就是构造器，现有参数判据仅接受依赖中的中间调用，漏掉了直接消费；不需要增加 finally 机制。

## What Changes

- 将现有数组证明与普通构造参数组合，按精确物理实参值识别直接消费，保持原参数依赖集合不变。
- 支持裸 primitive/reference 数组、动态长度、多维分配，以及现有数组证明完整接受的初始化和子数组组合；不以零长度或 byte 类型人为限缩。
- 统一普通构造的完整数组准入；若通用证明覆盖旧 String(char[]) 窄证书，则替换该历史特判，不因 owner、常量元素或直接 return 白名单拒绝完整证据。
- 保持单次求值、顺序、异常覆盖、所有权、共享预算和停止传播，保留完整物理来源。
- 用未改写的原始 TestFinally3 debug/noDebug 完整类和独立数组家族，对照原程序、JADX 与 Jarde；失败完整源码和真实编译错误保留。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：已证明的直接数组构造参数可以组合为完整 Java 表达式。

## Impact

现有 init.rs 构造验证和 receiver 调用选择、build.rs 数组事实访问及后置实参 reader 闭合、report.rs 必要调用点和相关测试；复用当前 ArrayInitializers、Sites 与 Builder。无新 crate、依赖、公开 carrier、solver 或 pass。

## Prerequisites and Non-Goals

前置 finThrow、nestedFin、exception-only finally 已分别验收。先完成真实数组/构造 SSA、所有权探针及旧产品整类编译基线，再实现；原输入和工具链冻结。一般数组 alias/phi、未被现有 array 证明接受的 store 文法、DEX/smali、显式 finally return/throw 及通用 fallback 来源债务不混入本改动。局部完成不增加 71/612 整体单元数。
