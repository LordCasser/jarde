## Context

见 [proposal.md](proposal.md) 和 [行为合同](specs/java8-recovery/spec.md)。[既有 finally 证书](../recover-proved-finally-cleanup/design.md)已证明 `ImplicitCleanup.run()` 的同一 catch-all 行、两份等价 `cleanup()`、返回值 BCI 19→23/24 的保存与读取、handler BCI 25→29/30 的同对象重抛，以及两份清理都在 `[0,20)` 外。当前唯一阻断是 `guard::resources` 的 `statement_free(protected)`；直接删门槛会使 `Builder::body_range` 静默略过 BCI 3 的分支。正常臂的 BCI 15–19 与清理 20–23、返回 23–25 被规范 CFG 合为一块，因此只按块取子 Region 也会重复清理。

本地 JADX 1.5.6 的 `MarkFinallyVisitor` 先在各出口寻找重复指令并标 `DONT_GENERATE`，`ProcessTryCatchRegions` 后包裹区域。它提供先证明副本再包裹结构的顺序参考；[冻结三方执行](../../evidence/java-syntax-2026-09-22/finally-completion/implicit-cleanup/analysis.md)表明其这一个输出在清理抛错覆盖返回时 trace 为 `299`，而原 JVM 为 `29`。Jarde 必须以异常范围和真实完成流为依据，不移植该启发式或依赖 JADX 运行时。

## Goals / Non-Goals

**Goals:** 将已证的单正常出口 `finally` 与受保护范围内的可证 `if`/`throw` 组合，确保半开 BCI 边界、局部作用域、块归属、异常边和来源完整；针对冻结 `ImplicitCleanup` 四条完成路径得到可重编的完整类，且保留已有直线与 lead 正例。

**Non-Goals:** 不改变 class 解析、Java 8 方言/descriptor 验证、runtime 选择或 JVM verifier；不改 canonical CFG/SSA，也不新建通用异常 IR。多正常出口、循环、switch、嵌套 guard、显式替代完成的清理及不能证明的类型/生产者另案处理。证明不足仍引用，不为追求可编译文本放宽语义门槛。

## Decisions

1. **沿已有证书扩展，不根据 catch-all 猜语法。** `FinallyCopyProof` 继续先证明异常行优先级、两份清理效果与实参、正常唯一出口、保存值、原异常身份和覆盖范围；只向私有 `Plan` 传递子正文需要的异常行与唯一正常终结位置。旧 `statement_free` 路径不变。备选是以相似 handler/call 指令自动构造 `finally`，会把扩围 `[0,23)` 的二次清理错并成一次。
2. **先在受限 Frame 中恢复子 Region，完成后才认领所有权。** 对非直线候选，在 `region_at_inner` 标记入口 visited 之前开启有界子遍历；Frame 只允许证书的受保护块与指定正常终结，并只把该行、该范围内的 catch-all 边交给外层 guard。其它异常/Call 边照旧拒绝，递归不再对同一入口重试 guard。子树必须全为已支持结构、无 fallback/重叠/逃逸，块集合与证书的受保护块精确相等。先暂存 visited 和结果，全部核对通过再一次提交子树及 cleanup/handler 的所有权；失败恢复原状态并引用整个候选。备选是先认领 `Plan::owned` 再递归，入口立即被视为重入，且失败会留下半个所有权。
3. **只在 guard 内对融合块按 BCI 分段。** `Region::Guard` 可携带一个内部结构化正文；对外仍由 `Plan::owned` 一次报告物理块，内部子树只供呈现和声明路径，不作为第二 owner。Builder 在子树内将指令发射限制于证书的受保护半开范围，在唯一正常终结块的受保护尾部写入已证明的保存值 `return`，使其留在实际分支及局部作用域；随后在独立范围写一份正常清理作为 `finally`。不虚构新的 canonical block 或改变全局 `block()` 语义。备选是在整个 `if` 后追加 return：正常臂声明的 local 可能不可见；直接写完整终结块会把 cleanup/return 也写在 `try` 内。
4. **子正文与最终 Try 一起提交或整体回退。** 预检正常臂唯一性、局部声明可见性、值与调用目标、可呈现指令以及来源闭包；暂存子正文、返回和清理 AST 与 Builder 可变状态。任一阶段 fallback/停止/来源不完整时恢复暂存状态，仅输出含 lead、保护区、两份清理、保存/读取/重抛与延迟生产者的完整引用。默认/all 正文相同，映射包含 BCI 0/3、6–19、20/23/24、25/26/29/30 的真实方法来源。备选是把子正文的局部 fallback 放进已投影 `try`，它既不能编译也误称结构完成。
5. **只用仓内算法和现有 AST。** JADX 是 Apache-2.0 算法参考，但其 visitor 绑定可变 Insn/Region 和 `DONT_GENERATE`，且有已复现的重复效果；不复制代码或引入依赖。Jarde 复用自己的不可变 SSA/异常表、预算与 `Try`/`If`/`Throw`/`Return` 发射。实现若发现现有结构无法保守表达某个边界，应明确拒绝并在独立任务记录，而不是将规则扩成一般异常重写器。

## Risks / Trade-offs

- [同一物理块的子正文与清理被发射两次或漏发] → 对所有已解码 BCI 做半开区间唯一归属和 source map 核对，正常终结块的返回只插入一次。
- [局部变量在分支外失效或返回值在清理后重算] → 在唯一正常臂内生成旧值 return，整类 Java 8 重编并运行字段快照与四种异常完成路径。
- [catch-all 边被子遍历当成新 guard 或吞掉其它 handler] → 仅接受证书指定 ordinal/保护范围内的真实异常边；其它边完整拒绝。
- [预算、取消或发射失败留下半个 `finally`] → 事务式暂存 ownership/Builder 状态，失败后引用整段；按子 Region、返回、来源三个阶段定向验证。
- [相邻 guard 规则回归] → 直线 `finally`、融合块 lead、资源、monitor、命名 catch 与覆盖型负例独立回归。

## Migration Plan

无公开格式或依赖迁移。可按恢复规则回退到现有 `statement_free` 拒绝；冻结证据、正反例和来源验证保留。
