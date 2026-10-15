# Design

## Context

动机见 proposal.md。现有 `Draft.protected` 从 create 的空 Vec 经融合并集、装配副本发布，表明所需载体与传递路径已经存在。`CanonicalHandlerRow.protected` 来自实际异常边；有 throw-site 的记录不会包含所有范围内非 throwing block，不能拿它填充 block 的范围事实。

融合可沿前向 goto 合并非连续块；`[id.bci, end_bci)` 是包络而非成员清单。raw CFG 已验证半开区间和指令边界，create 仍持有每个真实 RawBlock，适合在此补齐事实。该范围事实属于有证据的 normalization，不构成输入 verifier 通过、runtime 选择或 Java source recovery 能力扩展。既有 `correct-proved-legacy-clone-frame-coverage` 修装配 clone 闭包，回归应保留但本变更不重复其能力。

## Goals / Non-Goals

**Goals:** 在现有 Draft 上补齐真实成员的范围事实，澄清公开 accessor，预算内沿融合与装配发布；测试要直接要求应有的非空值并证明没有包络误判。

**Non-Goals:** 不建立第二份异常分析表、新 public carrier、solver/pass 或 crate；不扩充 handler rows、Frame/SSA 异常输入，也不据此改变 Java Try/List 准入。

## Decisions

1. 在 create 取得 RawBlock 后按其真实半开 span 扫 raw handler ranges，相交即加入该 ordinal。raw 边界完整验证保证相交对应实际指令，不能按融合后包络或 handler-row throw owners 反推。按原始异常表顺序收集，融合沿现有 union 排序/去重，文档明确“任一真实成员受保护”。
2. 每次扫描先 poll/计 AnalysisSteps，每个新增 ordinal 在 push 前计 IrItems。现有 fusion 对 protected 的复制、去重比较、扩容以及 assemble 的保留副本须逐处审计；可用 `mem::take`/消费 Draft 移动不再需要的 Vec，避免无意义克隆及重复存储计费。若副本确需保留则先收费，避免仅统计最终 block 数。只处理本字段的新增成本，不顺带重写整个 CFG 管线。
3. 沿既有标准库 Vec 与集合机制实现；项目内部 CFG 已提供所需范围、边界与身份，外部库无法代替该事实补齐。无新增依赖或许可成本，不增设抽象缓存/索引。
4. 内部 canonical 测试用真实 raw CFG 构造端点、重叠、非 throwing、部分融合、前向 goto 空隙、live/dead clone；raw CFG 为 crate-private，公开 `p3_method_ir` 对同次 SSA 实际发布块，以真实 instruction BCI 与物理 Code 异常表匹配并精确比较 accessor，不新增公开 API。没有 SSA 载荷的 dead/unreachable 节点由内部 raw span 回归承担，不能因缺 SSA 推断范围为空。移除当前“所有 protected 都有 handler row”的过强假设，改以物理异常表匹配；handler row 测试继续按实际边判断，避免用一种事实的语义替代另一种。

## Risks / Trade-offs

- [此前空字段导致既有测试与外部消费者依赖空值] → 按公开文档修正，明确部分融合含义，用正反控制验证实际 ordinal；不承诺错误空值兼容。
- [有异常记录的图新增计费改变精确预算门] → 更新真实费用证据，保留无 handler 零增量控制；低限额/cancel 测试覆盖新增扫描及存储，不能直接放宽停止断言。
- [非连续融合误用包络扩大范围] → 只从 create 的真实 RawBlock 收集，并用被跳过保护 gap 作拒绝控制。
- [block 范围和 handler-row 边事实混淆] → 分别检查不同非空集合，保留 throw-site 与 clone path 身份，Java 消费者不迁移到本字段。

## Migration Plan

先在私有目录准备候选和基线摘要，经独立 review 后交 coding root 串行应用；实际通过 canonical unit、公开 P3、legacy clone/预算门，再执行 fmt/clippy、OpenSpec strict 与 workspace/CI。日志绑定最终实现版本并独立提交，验收后同步主规范与归档。若回退，按独立实现提交撤销，保留发现记录。
