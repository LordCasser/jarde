## Why

普通 Java 8 `try { return value(); } finally { cleanup(); }` 在 bytecode 中把清理逻辑复制到正常返回和异常重抛路径。现有 Jarde 正确拒绝未经证明的复制形状，却使完整生成类缺少返回而无法编译；完整审计还表明 JADX 对相邻的覆盖返回/抛错形状会产生可编译的错副作用，不能把它当语义 oracle。证据见 `../../evidence/java-syntax-2026-09-22/finally-completion/`。

## What Changes

- 对**不覆盖原返回/异常**的窄形状，只有在正常与异常清理副本等价、每条出口恰好执行一次且返回旧值/重抛原异常均已证明时，生成 Java `try/finally`。
- 保留所有清理、返回、异常表与物理 BCI 来源；证据不足时保持引用，不把 `catch_type == 0` 本身解释为源码 `finally`。
- 以自写完整类的正常返回、try 中抛错、清理中抛错及副作用顺序做原 class/JADX/Jarde 执行对照，并保留覆盖返回/抛错形状的独立负边界。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`: 对已证明的非覆盖型 `finally` 清理恢复语义等价的 Java 结构，未证明的 catch-all 继续保守拒绝。

## Impact

影响 `jarde-java` 的 guarded-region 证明、区域归属及现有 Try 语句发射；无新 crate、运行时依赖或公共 API。产品不执行目标代码；JVM 运行仅为受控测试。本 change 排在当前错值修复之后串行实施。`finally` 的 return/throw 覆盖、break/continue、嵌套清理、多出口共享、旧 `jsr/ret` 与任意 catch-all handler 都不是本最小切片的已支持行为，另行分析，不能由此 change 宣称覆盖。
