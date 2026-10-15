# Design

## Context

问题见 proposal.md。当前 `Shared` 以同一个 mutex 持有 `entries`、`containers`、`definition_reads` 和 `retained_bytes`；definition-read 准入已经合计三张表，container 与 CP/Header 准入仍只合计两张表。CP/Header 的 `take` 失效分支只 remove，其他两层失效分支会扣减移除项权重。

## Goals / Non-Goals

**Goals:** 所有新项准入和 entry 拒绝统计使用同一库存定义；失效移除后 store 权重准确；保留当前解析、读取、停止和回退行为。

**Non-Goals:** 改变 key/schema、发布公共计数 API、引入淘汰或重新测量堆驻留；调整 JVM 方法选择或 Java 源码恢复。

## Decisions

1. 在现有 `Shared` 内增加一个私有总条目计数 helper，统一三个层的准入及拒绝原因判断。继续在同一临界区内查询和更新，不引入第二个可漂移的条目计数器。直接重复三表求和虽能修错，但已导致两处遗漏；现有标准容器即可完成，不需第三方库、许可或维护负担。
2. CP/Header 失效时使用实际 remove 返回的 entry 扣减其 weight，沿用其他层的扣减策略。保留旧 payload 的 Arc 引用仍有效；store 代理权重下降不代表进程或活动请求的全部内存已释放。
3. 保留同 key 重写语义。本变更不重构缓存模型，也不把内容解析、registry 语义验证、运行时选择、JVM verification 和 Java 恢复混成一项事实。
4. 回归同时核实际结果与统计：三类插入排列、entry 零/满边界、capacity refusal 归因、失效再保留、混合 store 和活动 payload。测试需在原实现暴露缺陷，并保持 cache-off/direct 足额结果等价；不只测试 helper 本身。

## Risks / Trade-offs

- 严格计入 definition-read 后部分请求会少保留事实 → 核完整结果相同、当前请求不重复解析，拒绝统计如实报告。
- 重复扣减或扣错项 → 由实际移除对象提供 weight，并核多层驻留及再保留场景。
- 并行工作使来源漂移 → 私有候选记录来源 SHA，由 coding root 串行应用、构建和最终验收；测试结果绑定确切源码快照。

## Migration Plan

无需数据或公开 API 迁移。作为独立修复提交，在 focused/P5 门禁通过后合入；若出现结果或停止语义回退，撤回该独立提交并保留失败证据。规划与私有 patch 不关闭实施任务。
