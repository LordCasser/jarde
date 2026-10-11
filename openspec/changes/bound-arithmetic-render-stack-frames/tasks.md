## 1. 诊断与最小修复

- [x] 1.1 固定实际 e041 CI 失败、同 CLI 的 2 MiB 复现与回溯，完成调用链审查。验收：真实 raw、class/source/CLI pins 和 frame 分配记录可追溯，明确本地 main 与 Linux test 证据区别。
- [x] 1.2 Luna 私有实现，root 审查应用轻量算术入口与一个非算术定义分派方法。验收：原优先级/site/深度/at/顺序/错误/来源和所有 budget charge/poll 不变，没有复制算术实现。
- [x] 1.3 既有 overdeep lambda 回归显式使用 2 MiB 线程执行真实 class-source。验收：root 实际精确测试通过，报告保留原 helper rename；相关 lambda、数值和消费位置回归通过。

## 2. 产品验收与交付

- [x] 2.1 在 5 GiB free/target 1 GiB 守卫下冻结新 CLI，重放原 2 MiB 输入并比较冻结原报告的完整 text/maps/身份/usage；核父片最终对照无变化。验收：实际完整报告而非 signal，逐字段差异原样保存，相关原/JADX/Jarde 行为证据有效绑定。
- [x] 2.2 完成格式化、适用测试/lint、完整 workspace 与 OpenSpec strict。验收：精确命令/raw/source pins 均通过，target 已清理，不借此前源码的执行结果。
- [ ] 2.3 提交推送实际产品并验收自身全部 CI job/step/两 seed 测试，更新父片 verification/handoff 后完成干净交付。验收：精确新 head、实际 CI 日志和 main/分支/target 状态；e041 失败不覆盖或冒充已接受。
