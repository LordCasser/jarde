## Context

P5 以被选目标阶段（P1、P2、P3 或 P4）的结果契约和真实基线稳定为进入条件；不要求所有后续阶段完成。架构明确要求“先正确，再按瓶颈优化”，并拒绝预设索引格式、性能数字和新依赖。

## Goals / Non-Goals

**Goals:**

- 建立可复现、可审计的 baseline/optimized measurements 和语料矩阵。
- 只选择被数据支持的 query 合并、cache/index、并行和退化优化。
- 以完整 cache key、失效规则、直接路径回退和 A14–A18 回归保证语义不变。

**Non-Goals:**

- 不在规划阶段承诺固定 P95、吞吐、内存或加速倍数。
- 不预设持久数据库、索引布局、并发 runtime 或额外 crate；依赖选择由测量和审计决定。
- 不用优化掩盖 parser、resolver、recovery 的正确性缺陷，也不改变任何已有 result contract。

## Decisions

1. **先测量再选择。** benchmark 先固定 fixture fingerprint、配置、缓存状态和资源代理，再比较候选；没有收益或回归证据的优化保持 disabled。
2. **cache key 覆盖所有语义输入。** snapshot/view/platform/registry/query/IR/recovery/pass/budget 版本均参与 key；相比只按文件 hash，能避免 profile、dialect 和缺失依赖变化造成错误命中。
3. **直接扫描作为可比较的参考路径。** cache/index 失败、损坏或禁用时，在剩余预算内回退同语义直接路径；请求已取消或预算耗尽则返回终止状态，不通过重跑重置预算。完整执行用于冷/热等价对照，中断时允许已完成子集受调度影响。
4. **并行只改变调度，不改变发布顺序。** worker 结果按稳定 identity/evidence 顺序合并，single-flight 的取消与订阅独立；不为吞吐牺牲 coverage/partial 语义。
5. **发布门槛使用对照而非口号。** 对每项优化比较 correctness、资源和失败语料；固定数字只有在真实语料和重复测量足够后才由后续决策确定。

## Risks / Trade-offs

- [Risk] benchmark 样本偏向导致错误优化方向 → 覆盖版本、打包、身份、字节码、恢复、退化和对抗矩阵。
- [Risk] cache 键遗漏语义维度 → 集中构造 key，变更版本时强制失效并做差分测试。
- [Risk] 并发放大内存或破坏取消 → 按字节/内存权重调度，协作取消和压力测试。
- [Risk] 优化路径与直接路径细节不同 → A01–A18 全量回归，默认路径阻断式 gate。

## Migration Plan

先加入只读 benchmark/metrics 和 disabled-by-default 实验开关；在对照通过后逐项启用。缓存/索引格式若最终需要落盘，必须另有版本和迁移策略；若目标阶段契约需要改变，必须通过 OpenSpec 明确修订，不作旧接口持续有效的承诺，也不预设最终持久化形态。
