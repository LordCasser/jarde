## Context

见 proposal.md。root 的冻结 CLI 回溯显示 24 个既存 `render_value` 调用之后，下一调用在函数栈探测指令处崩溃；单帧 ARM64 prologue 分配约 47 KiB。Linux CI 只提供 stack overflow/SIGABRT，不把本地回溯冒充 Linux 精确帧测量。原 `MAX_VALUE_DEPTH = 24` 在函数体内检查；深度有界不等于每个递归帧足够小。

当前 `render_value` 先处理条件缓存、已绑定值、postfix snapshot、binding refusal、array alias 和 this alias，再按 Definition/Operation 构造表达式。算术分支只需要两栈操作数，依次渲染左右值，沿用消费位置 `at`、`depth + 1`、`arithmetic_op` 和生产 BCI。庞大的其他分支仍占用该递归函数的栈帧。

## Goals / Non-Goals

**Goals:** 相同输入在 2 MiB 栈返回原报告；普通算术递归的帧不包含无关 Operation 分派局部量，原值嵌套界实际能生效。保持重命名、物理报告、text/maps/usage 和预算停止行为。

**Non-Goals:** 不扩大可恢复值形状、不降低或提高深度界、不新建线程执行产品工作、不更改 AST/Emitter/Clone、不解决所有 Operation 的栈占用、不改 lambda 256-node inline 门、不改 JVM verifier 或公开 verification 状态。其他深度消费者只记录债务，不在本片猜测修复。

## Decisions

1. **隔离递归帧。** 保留 `render_value` 作为共享优先级与深度入口，在这些前置判断之后直接执行现有 Arithmetic 分支。把剩余 Definition match 原样移到一个私有 `render_value_definition` 方法，移除其中原 Arithmetic 分支；现有 `other` 拒绝分支保留。无新的证明器、缓存或计划。算术只在该 BCI 无 construction site 时走轻量分支，保持原 site 优先级。
2. **不改变算法语义。** 迁移算术臂的原验证/错误文本、左后右求值顺序、同一个 at、depth 加一、Expr/来源构造；无新增或删除 budget charge/poll。所有递归调用仍回到共享 `render_value`，不能绕过原缓存/绑定/snapshot/alias 拒绝。
3. **真实资源边界回归。** 既有 overdeep lambda 测试的实际 reader/class-source 调用用标准库显式 2 MiB thread 执行，编译仍生成完整真实 class；保留原测试名与 helper rename 断言。这个固定栈等于 Linux Rust 常见测试边界，禁止通过大栈或环境变量使测试通过。不增加镜像算法测试。
4. **复用与许可证。** 只复用 Rust 标准库、现有 Budget、SSA、AST 和拒绝通道；无第三方代码移植或依赖。本片不需参考 JADX 的新机制。

## Risks / Trade-offs

- 优先级移动造成绑定或 construction site 被算术绕过 → 入口前置判断保持原序；site 存在时仍交给原定义分派。
- debug 编译器重新合并函数栈帧 → 实测 2 MiB 回归与新 CLI；必要时只依据生成 prologue 证据调整，不预先添加 inline 属性。
- 拆分后语义/费用改变 → 对同 class/default/all 与冻结 CLI 的完整 text/maps/物理身份/usage 比较；相关整数、浮点、消费位置与 lambda 回归；父片所有来源对照依同一新 CLI 核验。
- 2 MiB 本地 main 与 Linux test 并非同一调用链 → 同时保留实际本地复现、永久 thread 回归与精确新产品 CI，不能以其中一条代替另一条。

## Migration Plan

Luna 仅准备私有补丁，root 审查后串行应用；在 5 GiB free/target 1 GiB 一秒守卫下构建、检查、冻结 CLI 并 clean。新产品提交和自身 CI 通过前，三个父片交付任务继续未完成；常量名候选继续私有。
