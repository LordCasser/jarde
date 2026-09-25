## Why

合法的 `PostfixHandlerBoundary.update()` 在异常表跨越后置更新链时，Jarde 在 Region 阶段保守拒绝是正确的，但拒绝正文只映射 BCI 0、13，丢失已解码的 BCI 3–12。[短路额外入口样例](../../evidence/java-syntax-2026-09-25/short-circuit-chain-extra-entry/analysis.md)同样漏已解码 BCI 8/26/29。`Builder::covered_bcis` 把 `CanonicalBlock::blocks()` 当成逐指令起点；该 API 实际只返回原始块起点。拒绝范围的来源因此比它实际覆盖的字节码窄，妨碍定位反编译缺口。

## What Changes

- 在 Region fallback 的既有来源路径中，以可核对的已解码指令事实列出被拒 canonical 块内的物理指令起点，并保留未覆盖指令的原有来源。
- 对普通块、融合块、克隆与部分解码分别验证范围、去重、顺序、预算和停止语义；不改变 Region 是否拒绝或 Java 语法投影规则。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`: Region 拒绝报告的来源覆盖到已证明属于拒绝范围的物理指令，而非仅覆盖块起点。

## Impact

主要涉及 `jarde-java::build` 的 fallback 来源采集及对应测试。无需新增 IR、公共 API 或语法机制；异常图根、try 词法归属和后置递增证明不属于本变更。额外入口样例的重复 Region 所有权、误报循环与字段 `presented=true` 也不由来源枚举修复，必须单列。证据见 `../recover-postfix-lvalue-values/verification-handler-boundary.md`。
