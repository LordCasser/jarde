## Why

调用方已明确需要多线程全量导出。当前逐方法 sweep 把同一容器和 class 的准备工作反复计入请求；共享 retaining store 的用户实测显示较大收益，但每请求 p50 不能替代整包总时间，也不能据此确认已追平 jadx。本 change 将这项需求落为一个按类复用、并行执行、持续交付的完整操作，目标是在相同工作集合及明确产出质量下接近或超过 jadx 的端到端性能。

## What Changes

- 增加同步、可取消的库级批量恢复入口和薄 CLI `export`：显式选择一个 snapshot 的物理范围，遍历其中的类与方法声明，交付现有方法恢复产物、来源映射及结果平面。完整 Java 类/工程生成不属于本 change。
- 同一类任务共享已验证字节、结构、方法定位及按需同类 callee 事实；定位保留 raw name/descriptor 的全部候选，类名不是复用身份。单方法入口消费同一底层实现，不维护另一套恢复流水线。
- 类任务进入有界 worker 窗口；库显式配置 worker 数，CLI 默认解析 `--jobs auto`，`--jobs 1` 为串行路径。导出操作拥有共享事实生命周期，现有单请求入口不隐式启动线程或预读全包。
- 将准备、worker、编码/交付接入同一总预算，各方法保留局部限制与取消状态；共享工作只计一次。单项结果容量不随 jobs 改变，窗口不足时只缩减并公布有效并发。输出顺序稳定，背压和取消能唤醒等待任务，返回前回收所有 worker。
- 完整执行保持逐方法语义与来源一致；新并行操作在紧预算、取消时允许完成子集随调度变化，但必须区分执行覆盖与交付覆盖，不能伪造相同部分结果或丢掉已付工作量。
- 建立串行直接、共享 store、类内复用、1/2/4/6 worker 的消融和冷端到端导出对照；将性能主张与能力交付分开验收，不把 p50、计数或更早拒绝当作加速。
- 用户追加需求：CLI 增加单类源码快捷视图（`class-source`），把一个类的声明、字段与逐成员恢复文本装配成可读 Java 形态。它是既有按需恢复结果的装配，不是第二套恢复实现，也不声称可编译工程；未产出/仅解释/无 Body 的成员必须显式标记。
- **BREAKING（限定相关共享事实 API）**：允许将 owned facts 返回改为不可变共享句柄/借用，让 method locator 绑定可信读取；不保留并行的旧解析实现。具体受影响签名在实施首步列出。既有单请求语义不因新批量操作改变。

## Capabilities

### New Capabilities

- `bulk-recovery`：显式范围的批量恢复、worker 上限、总量与局部限制、确定性发布、部分交付及 CLI 导出。
- `class-source-view`：单类源码快捷视图（库操作 + CLI 子命令），装配既有恢复结果为可读类文本，并诚实报告未产出成员。

### Modified Capabilities

- `classfile-inspection`：同一可信 class 读取可支持多个方法的直接定位/解码，不重复扫描成员表，保留损坏、重复声明和按需边界。
- `facts-cache`：不可变事实的共享消费及导出作用域的保留策略；保持原有身份、完整发布、容量和失效契约。
- `analysis-contracts`：批量组合的共享准备证据、执行/交付覆盖和并行中止适用范围；保留既有单请求承诺。
- `measured-execution`：完整并行结果与部分执行的确定性边界、全量导出成本和跨工具对账。

## Impact

实施涉及 `jarde-reader` 的 class 读取、facts ownership 与预算；`jarde-jvm` 的已准备 class 交接、driver/callee 读取；根 `jarde` 的操作调度与类型化事件；`jarde-cli` 的流式导出、单类源码视图和退出状态；P5 harness 与 A07/A08/A13–A18 回归。`jarde-java` 的恢复规则继续由现有流水线负责。首版复用 Rust 标准库并发原语和现有依赖，不新增 crate、持久数据库、全局线程池、async runtime 或通用任务框架。

本 change 承接 [optimize-demand-workloads](../optimize-demand-workloads/proposal.md) 的 O2/O4/O7，以及本路径必需的 O5 共享 payload、O8 流式输出；父专项继续负责测量和其它候选处置，不重复拥有实现。O1 使用已归档容器复用能力。

前置条件是冻结真实构建与反例集合。[整数比较](../archive/2026-09-20-decide-comparison-contexts/verification.md)、[局部类型](../archive/2026-09-21-unify-local-type-decisions/verification.md) 和 [concat 表达修正](../archive/2026-09-21-re-express-string-concatenation/verification.md) 已独立归档；[当前完成复核](../../completion-review.md) 确认 T1–T4 原判据关闭，另登记 T5（append 的 int 参数消费 char）和 CLI 文档预算等边界。本 change 不代修这些独立缺口。并行发布仍须在实际普通 worker 栈上复跑深表达式进程门禁，不能用当前串行/测试线程证据替代；性能比较固定同一正确性基线并保留已知失败，不以删样本或更多拒绝计收益。

明确非目标：完整类级 Java 工程装配（单类 `class-source` 快捷视图是用户追加范围，属装配视图而不是可编译工程生成）、resources 重打包、自动 layout/依赖发现、全局 XRef、方法内并行、跨独立调用的 single-flight、后台预热、持久缓存、断点续跑协议，以及预设必达的加速倍数。
