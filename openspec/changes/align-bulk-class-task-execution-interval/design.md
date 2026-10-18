## Context

见proposal。失败收据为/private/tmp/jarde-typed-foreach-workspace-seed1-root-v1-20261018/execution.json：前三批中第三批command4退出101，bulk_recovery_lifecycle中gate.peak>=2/worker high_water==2通过，而公开concurrent_classes_high_water==1；来源未变，finally exact clean通过。源码run_class_task先执行gate/delay后才Registry.enter。enter已用同一mutex更新executing与high_water，Entered::drop饱和减计数，不是原子max竞争。Atlas局部incoming q_1a14e1f6e30_ade2定位worker_loop和coordinate，当前源码逐项确认worker/serial共用此入口。

## Goals / Non-Goals

**Goals:** 让已有计数guard覆盖整个类任务调用，重用旧锁和RAII退出，消除真实重叠观测与计数区间错位。

**Non-Goals:** 不改ClassGate算法/超时、调度/窗口/池、恢复算法、输出重量表、阈值、公开数据模型或依赖；不新增独立counter、observer或sleep来修测试。typed foreach与后续原名折叠保持独立。

## Decisions

把现有_entered移至run_class_task函数体入口、panics_at/gate/delay之前；不复制guard。开始执行任务即可计入，因此真实准备前等待、入口注入panic也有同一生命周期，unwind/早返回由既有drop减计数。调用者未持registry锁；enter自身返回前释放锁，gate阻塞不持registry mutex。保持原ClassGate及high_water==2门可作为确定的后置证明：两个任务都已enter之后才可能同时进入gate。提高timeout/加sleep/降低assert都不能修区间；改worker_loop只覆盖并行路径且重复serial逻辑，不采用。

标准库Mutex/RAII和现有Runtime已足够，无第三方库/新许可/新机制。生产不含test-support时仍复用同一任务入口；不影响parse/verify/source recovery结果。

## Risks / Trade-offs

- panic退出下重复或缺少减计数 → 只移动唯一guard，核Entered::drop与TaskGuard职责，现有worker panic/结束门验证返回无遗留worker。
- gate等待持registry锁造成死锁 → enter返回前state guard已drop；保留真实双任务gate门。
- 只靠重试掩盖失败 → 保存原native失败，保留所有断言，根因修正后focused与两fresh native重新完整开始；独立接受最终提交自身CI。

## Migration Plan

Luna私有交最小可审patch，root应用/对抗审查；root串行focused bulk门、fmt/CI Clippy/MSRV/相邻及P5、最终CLI输出重绑、两fresh native/自身CI。旧b78候选及其native/CI按真实状态保留；修复单独提交，typed foreach沿最终代码重新验收。默认stack/5GiB/target1GiB每秒守卫，Rust后exact clean。不得因为叶子通过关闭全局71/612目标。
