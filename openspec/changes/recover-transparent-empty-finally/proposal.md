## Why

固定 JADX `TestEmptyFinally.TestCls.test(FileInputStream)` 的 Java 8 class 有同范围的 `IOException` 与 catch-all 两行；后者仅保存并原样重抛 Throwable，对应源码中无效果的空 `finally`。JADX 输出可重编并在三条异常路径与原 class 一致，Jarde 当前把透明 handler 留成未覆盖块，完整源码的具名 catch 因缺少可抛操作而不能编译。

## What Changes

- 在既有 catch/Region/Build 路径证明固定两行布局中的 catch-all 是无副作用、原异常身份不变的编译器脚手架；只输出具名 `catch(IOException)`，保留受保护的 `close()`，不凭空输出空 `finally`。
- 把被吸收 handler 的物理指令、异常边和来源纳入同一次闭合所有权检查；变异形态、证明不足、预算停止或取消时原子拒绝。
- 以固定原 class、原 Java 8 转写及 JADX 源码的 close 成功、受检异常吞掉、运行时异常传播路径作为行为基准，另核 Jarde 完整源码 Java 8 重编、`-Xverify:all`、BCI 来源和 verifier 有效近邻。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：受证的透明 catch-all 重抛行可在具名 catch 后作为空 finally lowering 吸收，输出保留语义的普通 try/catch。

## Impact

先决条件是现有 JVM 异常表、SSA、普通 catch 及来源映射。改动限于 `crates/jarde-java` 的 Guard/Region/Build 和固定证据/测试；无新 crate、AST 类别、公共 API、依赖或 CLI 选项。不覆盖非透明 handler、任意空 finally 模式、DEX/D8，也不把无证的 catch-all 当作可省略代码。
