# Design

## Context

动机见 proposal。公共 API 复现保存在 `results/public-repro-v1/`：唯一成员和 foreign-owner 两项控制通过；重复矩阵测试在首个 abstract+Code 组合、首个 identity 请求真实失败，返回 `Performed/NotDeclared(Abstract)`，此前对同一成员表的 Name 请求已通过 Ambiguous 断言。后续八个组合及第二个 identity 未执行，不能计为失败或覆盖。

该实验仅新增私有测试，运行在包含四个工程候选及 P1/test overlay 的私有 checkout；不是 primary 644 的完整运行。相关 `src/facade.rs` 与 644 的源码相同，实际执行的完整源码与命令由独立 pins 绑定。当前生产尚未修复。

`resolve_body_ref` 的 Name 分支已收集全部匹配项并构造既有候选；Method 分支检查 owner 后调用首取的 `declared_method`。`body_result` 对首取的无 Code 成员直接返回，绕开准备类和 reader 的重复签名门。该路径与新 driver/callee 修复互不覆盖。

## Goals / Non-Goals

**Goals:** 只收敛 class-view 同次成员表上的目标解析；两种引用在 owner 约束相同后得到相同唯一性结果，歧义在任何 body 工作之前返回。

**Non-Goals:** 不增加 `PhysicalMethodId` 的 ordinal，不改变成员项的 index 或原始记录，不改变 reader/JVM API、完整类格式验证、方法恢复、缓存或 CLI 协议；部分成员表及 shared-budget 的既有行为保持，不把未实证的其它边界纳入本片。

## Decisions

### 1. 共用现有候选收集，保留 owner 校验在前

Method 分支先执行原 foreign-owner 判定，之后将其 raw name 与精确 descriptor 交给和 Name 相同的本地匹配路径。零项沿用 `not_reached_or_missing`，一项沿用原 ordinal/member，多个项沿用 `BodyResolution::Ambiguous` 与原候选构造。若首取 helper 因此没有调用方，可直接删除；不在另一个分支复制第二套唯一性算法。

这比新增跨 crate selector、公开 ordinal 或拒绝整个成员列举更小，也保留现有有效 identity 的数据形态。按具体 member index 展示 class-source 无 Code 声明是另一种操作，不能把它误改成名称选择。

### 2. 在 body admission 之前结束歧义选择

共用分支仍在 `requested_bodies`、类准备及 `body_result` 之前运行。返回全部候选，保留相同 identity 对应的不同原 index/flags；候选不因 identity 相等而去重。此时不消费 method_bodies/code_bytes 或 IR。既有 class/member 读取和结果项收费按本次实际工作保留，不要求回滚合法用量。

### 3. 使用公共矩阵与既有逐 body 门禁

复用现有 selection-class 字节构造，测试 abstract、native、Code 的九种有序重复组合；每组先比较 Name 候选，再分别请求两个列举项提供的 identity。实现后必须实际到达全部组合，不以修复前首行失败证明全矩阵。

唯一 abstract/native/Code、不同 descriptor、foreign-owner、缺失成员、停止的成员表和逐 body 预算控制使用现有公共测试及必要的小增补。测试直接调用 `Engine::class_view`，不通过私有解析 helper 掩盖入口差异。候选和原 `task_operations`、prepared handoff、artifact binding/demand-count 测试共同验证只读一次、只按需解码及身份优先性。

### 4. 沿用现有依赖和报告

已有 `BodyRef`、`ClassContentItem`、`TargetCandidates` 与 `OperationOutcome` 能完整表达结果；外部包或新框架没有缺失能力可补，不新增依赖、许可负担或宿主耦合。此片验证目标选择，不把 classfile 结构可读等同于 JVM verification 或 Java 源码恢复。

## Risks / Trade-offs

- foreign identity 被同名本类记录替换 → owner 检查保持第一分支，并保留原错误码和无 body 用量控制。
- 候选按相等 identity 去重 → 对每个原 index/flags 精确断言，保持声明顺序。
- 无 Code 的轻量入口被迫准备类 → 对唯一声明及歧义分别断言零 body/code 增量，复跑 demand counts。
- partial member-table 或逐 body 停止被重新解释 → 复跑既有未到达、共享预算和 stopped-body 控制，不扩选中前缀的既有契约。
- 新回归早失败却被记作全矩阵 → 记录实际抵达范围，修复后验证九组及每组两种 identity 均执行。

## Migration Plan

coding root 在其负责的集成 checkout 采用规划及最小复现，保留旧失败，再完成本片源码和公共回归。各次命令绑定确切源码、原始输出与实际测试范围；执行当前适用的 fmt、Clippy、OpenSpec、workspace 和最终修复产品 CI 后，才能关闭交付任务。规划及私有复现不代替这些门禁；规范同步和归档在验收后进行。
