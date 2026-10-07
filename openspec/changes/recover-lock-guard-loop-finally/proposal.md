# 锁卫循环 finally 证书（recover-lock-guard-loop-finally）

## Why

[explicit-lock 巡查](../../evidence/java-syntax-2026-10-05/explicit-lock-patrol/README.md)（第 12 锚）+ [local-scope 1.2/1.3 取证](../preserve-local-scope-across-exception-regions/results/planning-1.2-1.3/03-boundary.md)：j.u.c 最高频骨架 `lock(); try { while-await/scan … } finally { unlock(); }`（`LK.take/put/tryLockQuick`）整方法拒。门控实验证明拒绝链在 guard/region 层（声明规划是下游）：

1. 单行保护区（`[7,50)→59 any`）走 `ExceptionEdge` fallback（`region.rs:6663`），`guard::examine` 因 `own_try==node && handlers==1 && catch_type==none` 被**跳过**（`region.rs:2732-2755`）；
2. `shared_finally_candidate` 的 1 行分支要求 `row.start_bci == 0`（`guard.rs:9835-9843`）——锁形行从 lock 调用后开始；
3. `prove_finally_copy` 明文拒绝"行起点前的会抛调用"（`guard.rs:2849-2852`，即 `lock.lock()` 本身）且只证直体后缀；
4. `finally_body_supported` 不接受 `Region::Loop`（`region.rs:2400`）——保护体含循环；`SavedReturn` 保存值写在体内（非 lead）。

jadx 全解。锚 13（io-wrapping）同因加 family-6/`jre_new_shape` 位点（其整类验收另需那两片——本片只关 guard 证书）。

## What Changes

新增**锁卫证书**（与 `prove_conditional_finally` 等并列的已认证形状，非放宽）：

- 形状判据：方法序 = `lock()`（行前唯一会抛调用）→ 保护体（含循环，body 由循环自己的 region reader 呈现——对照 `LoopFinally` 的 `*_regions` 构造）→ 正常路径保存返回值 → `unlock()` → return；异常路径 handler = `unlock()` → 重抛（**无自保护行**，单 any 行）；
- 安全性来源：两份 unlock 各自的接收者是同一 lock 对象（SSA 同一性），异常路径无其它效果（重抛原异常），await 形（`Condition.await` 的监视器释放语义是 JVM 事实，文本呈现不动它）；
- `tryLockQuick`（if 守卫形）：`Unproven::FinallyCopy` 的四件套（straight-body/copy/range/ownership）在该形状下的证明；
- `SavedReturn` 体内写形的提升呈现（对照既有固定形状证书族 `flag_saved_return` 等）；
- MVP 不含：多锁/嵌套锁、`lockInterruptibly`、Condition 多等待点——如实登记。

## 硬不变量

1. TWR `resources()` 的 `prove_finally_copy` 生产路径零触碰；
2. 真两份清理副本形（非锁形）拒绝不变；
3. 不得产出"可编译且行为不同"文本（unlock 顺序=控制流事实；`LK.put` 的 void 形已在 soundness 片验收 fixture）。

## 验收

- `LK.take/put/tryLockQuick` 恢复（0 该族拒绝），整类剥离编译 exit 0、`-Xverify:all` 行为与原一致（巡查记录值）；local-scope 1.2/1.3 的测试面（`preserve_local_scope_plan`）零回退且负例按 re-slice 落地显式更新；
- 门控实验先行；全门禁 + oracle ignored 腿 + corpus 指纹。

## Capabilities

### Modified Capabilities

- `java8-recovery`：锁卫（lock/unlock try-finally 含循环保护体）按源码形态呈现，方法行为完整。
