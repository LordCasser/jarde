## Why

CF08 的双入口前缀循环已有真实三方反例：同一 verifier-valid Java8 class 在原始双 JVM 和 JADX 完整重编的48个观察中一致，Jarde 四个 API 配置均因不可约 SCC 整方法拒绝，八次完整源码重编失败。差距在“首次可跳过、随后每轮执行”的共享前缀，不能靠放宽自然循环判定修复。

## What Changes

- 为有完整拓扑、SSA 和严格 int 局部证明的双入口前缀循环增加窄区域证书，呈现为条件初始更新、普通 while、循环尾更新和原返回。
- 每个物理块仍只有一个区域所有者；同一已证明前缀在两处源码展示并保留同一物理来源。缺证继续整方法不可约拒绝，Stop 不发布半成品。
- 修订主规范对不可约图的一律拒绝描述，仅允许这一明确受证形状；增加永久正反边界、真实完整源码与原/JADX/Jarde 对照。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`: 为严格受证的条件初始 int 前缀循环规定结构化恢复、唯一物理所有权、重复展示来源、失败与停止契约；其余不可约图保持拒绝。

## Impact

落点是 jarde-java 的 region 与 Builder、必要的既有区域遍历及回归语料；公共查询/CLI协议和依赖不变。沿用已交付 canonical CFG/SSA/局部类型/预算/SourceMap，不改 jsr clone 身份、不建通用 CFG 复制 pass、state machine、solver 或 DEX 前端。阶段前提是当前主线及冻结 CLI96e、已接受的 [26命令基线](../../evidence/java-syntax-2026-10-19/cf08-multientry-prefix-baseline-root-v1/README.md)。不扩到原 Smali 的数组/String、副作用或异常形，不关闭整个 CF08/71/612。
