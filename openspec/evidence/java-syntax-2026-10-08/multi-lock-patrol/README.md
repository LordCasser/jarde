# 多锁/可中断锁巡查（2026-10-08 root，lock-guard 域登记边界的实测）

## 探针

[fixture/ML.java](fixture/ML.java)（`--release 8`）：嵌套双锁（`a.lock(); b.lock(); try{count++} finally{b.unlock(); a.unlock();}`）、`lockInterruptibly()` 形（直体+单解锁）、同锁双等待点（`multiAwait`：两处 `await`）。jadx 对照见 [results/jadx-ML.java](results/jadx-ML.java)。

## 结果

| 形 | jarde 现状 | jadx | 判定 |
| --- | --- | --- | --- |
| **nestedLocks** | 整方法拒——`prove_finally_copy` 四件套（"exceptional path repeats code… lacks the complete straight-body, copy, range, and ownership proof"，BCI 41）+ uncovered [58,41] | **完整解**（双锁+try/finally+双解锁原序） | **真缺口 A**：finally 体含**多语句**（两 unlock）——行集证书的 finally 体只证单调用 |
| **interruptibly** | 整方法拒——同四件套（BCI 27） | **完整解** | **真缺口 B**：行前调用 `lockInterruptibly()` **可抛**（`lock()` 不可抛故 LK 过）——但异常行的自有范围 `[7,24)` 已把该调用**排除在保护区外**（抛出时 unlock 不该跑=字节码自身已证），四件套的"行前可抛调用即拒"在此形上过保守 |
| **multiAwait** | **完整恢复（0 引注）** | 同构 | **登记边界关闭（实测）**——多等待点不是缺口 |

原类行为：main 输出 `2`（[results/original-main.out](results/original-main.out)）。

## 归因与处置

两缺口同属 lock-guard/resource-guard **证书族的扩展**（非新机制）：
- A：finally 体从"单调用"扩到"语句序列"——安全来源=序列中每一 unlock 的接收者与某一行前 lock 调用 SSA 同一（嵌套序=获取逆序释放）；
- B：行前可抛调用——当异常行的范围**不含**该调用（字节码自证顺序：调用完成才进入保护区）时，"行前可抛即拒"可放宽到"行前可抛 ∧ 在行范围外"；`lockInterruptibly` 的 InterruptedException 声明同时解释 throws 子句。

立窄片 `recover-nested-lock-finally-bodies`（A+B 双形状，MVP：≤2 锁/≤2 解锁语句）。多等待点边界以本巡查实测关闭。
