## ADDED Requirements

### Requirement: Bulk class execution spans the whole class task

bulk恢复操作 SHALL 从一个类任务开始执行到该任务正常返回、早返回或异常退出为止连续计入正在执行的类任务；公开并发峰值 MUST 使用这个统一区间，并保持在配置的类任务窗口上限内。任务准备前的可观察等待 MUST 属于该任务的执行区间，不能因调度顺序而漏记已经实际重叠的任务。

#### Scenario: Two tasks overlap before preparation
- **WHEN** 两个已启动的类任务在准备前的同一观测gate内同时驻留
- **THEN** 操作最终报告的并发类任务峰值为2，且不超过配置窗口；不能在gate已观察到2时报告1

#### Scenario: All exits release the same task interval
- **WHEN** 类任务正常结束、读取失败、预算或取消停止，或在任务入口触发panic
- **THEN** 任务执行计数随同一任务调用离开释放，操作保持既有失败/停止结果且返回时没有遗留worker；单worker配置的峰值不超过1
