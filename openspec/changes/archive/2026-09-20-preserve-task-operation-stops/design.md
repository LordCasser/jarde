## Context

动机与两条可复现反例见 [proposal](proposal.md) 和 [完成复核](../../completion-review.md)。`search_named_classes` 已能返回搜索 coverage、execution 和诊断，但 `bind_method` 只检查候选数量；`bind_class` 的零候选分支也直接报未找到，单候选分支仍可在搜索未完整时执行。成员表局部停止还需在共享搜索处传播。

`class_view` 保留 `ClassViewBody` 的独立执行平面，却没有把成功返回的 body 值内部的停止合并到顶层；CLI 用 `class_view_plane` 再次遍历 body 弥补这一差异。修正必须落在库边界，不能要求每个宿主复制此逻辑。

## Goals / Non-Goals

**Goals:** 完整搜索才决定唯一/缺失；未完整选择可交付可靠候选而不执行；组合操作的顶层停止与子结果一致，局部失败与共享预算终止分开处理。

**Non-Goals:** 不改变名称匹配、物理身份或 runtime root 规则；不推断 classpath，不扩展恢复能力；不重构 reader、facade 分层或性能生命周期。

## Decisions

### 1. 复用现有选择证据，增加最小结果分支

在 `OperationOutcome<T>` 增加 `Incomplete(Box<TargetCandidates>)`，复用现有 query、候选、limits、coverage、execution 和 diagnostics。名称搜索的状态先于候选数量判断：未完整进入 Incomplete；完整后才区分零、一、多候选。内部 `ClassBinding`/`MethodBinding` 同步传递；不增加 session、全局状态或第二套错误报告。

拒绝将 Incomplete 编成 Ambiguous：零或一个已确认候选并没有证明歧义；也拒绝只在后续 analysis 合并停止，因为那已经分析了尚未唯一绑定的目标。显式物理身份路径不触发名称搜索，仍在现有身份和运行环境校验下执行。

共享名称搜索必须吸收 `ClassMemberFacts::stopped_at`，避免“Header 可读、成员后缀不可读”仍回答完整的成员选择。处理顺序不得为了发布候选重新初始化 Budget；已经拒付或取消时只交付此前真实取得的前缀。

### 2. 在库内汇总 body execution，保留各自 coverage

复用 `merge_execution` 与现有停止优先级，将 `Read.execution`、`Refused.execution` 合入类视图；`NotDeclared` 本身不是失败。顶层 usage 在结束时从同一个 Budget 更新。局部解码损坏可以隔离后继续其他显式请求的 body；共享预算耗尽/取消则停止后续工作。

类/成员结构 coverage 与每个 body 的 coverage 分别计算。不能简单把新的顶层 `complete` 传给现有 `class_view_coverage` 并将完整成员表改成未扫描；需要保留类/成员阶段自身的完成依据。诊断与停止位置保留在原来源，顶层状态不得要求调用方先遍历文本或子报告才知道是否完整。

### 3. CLI 只适配新的库结果

新 Incomplete 序列化库原值并退出 4；完整歧义仍退出 3，真实输入错误仍退出 2。类视图的顶层 execution 在库内可靠后，CLI 不再承担库缺失的汇总责任。更新枚举匹配、示例与库/CLI 对照，不保留旧输出兼容分支。

### 4. 复用与依赖

现有 `TargetCandidates`、`ExecutionReport`、`Budget`、serde 与 CLI 渲染足以表达该修正，没有外部库能力缺口；依赖、许可和供应链面均不变。fixture 复用提交的 `NestedEval.class`，损坏变体在测试中构造；不添加目标执行或网络依赖。本 change 只修选择/执行证据，不改变 dialect、runtime selection、verification 或源码质量的含义。

## Risks / Trade-offs

- 公共 enum 增加分支影响穷尽匹配 → 一次性迁移库测试、示例与 CLI；不维持旧分支语义。
- 部分候选可读但不能自动选择 → 返回候选供调用方显式选物理身份，保留可用证据，不扩大唯一性声明。
- 顶层汇总误伤成员结构 coverage 或局部失败隔离 → 同时断言坏 body、好 body、完整成员表和共享预算停止。
- 历史全绿造成再次漏验 → 新反例先在 `85828c4` 行为基线上验证失败，再在修正提交验证通过。

## Migration Plan

先补选择与类视图反例及正向对照，修共享搜索/选择，再修组合状态并迁移 CLI。通过固定提交门禁后归档并同步 delta；重新冻结 benchmark 候选，旧 `85828c4` 仅作为历史比较臂。回退按本 change 的独立提交进行；回退后必须恢复“停止语义未关闭”的公开状态，不能保留完成声明。
