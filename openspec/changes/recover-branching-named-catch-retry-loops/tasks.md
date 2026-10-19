## 1. 同轮定位

- [x] 1.1 用完整原件 debug/no-debug 运行同轮 Reader/IR/Region 探针，确认实际首拒绝与重复 owner 路径，归档原流及输入 pin，并验证临时源码精确恢复和 target absent；证据见 results/same-run-ir-root-v1。

## 2. 最小实现与定向验证

- [ ] 2.1 Luna 在私有目录扩展现有 RetryLoopFacts 与 Try 组装，root 审查后集成；验证保护区、三块 handler 边闭合、分支内 continue/throw 以及实际 owner/transfer 次数，旧单块 retry 回归通过，不增加 CFG/Region/Builder 机制。
- [ ] 2.2 保留完整原件及可达控制例 fixture，增加公共入口、错边/外部入口/共享行/错误范围/错回跳/终结拒绝和预算 used-minus-one/预取消测试；串行定向门、Reader 指纹、fmt/Clippy 实际通过并留原流，任何拒绝不泄漏候选 owner。

## 3. 完整对照与交付

- [ ] 3.1 用最终冻结 CLI 完成原件 default/all、debug/no-debug、JDK8/23 整类重编尝试及隔离 VM 对照，保留原 framework 边界；独立可达控制例覆盖正常、重试、重抛、retry 效果自身抛异常及条件反转，root 核验完整源码、事件顺序、异常范围、metadata/来源与真实可达边界。
- [ ] 3.2 同一候选通过一次 fresh workspace、既有门与最终产品自身 CI，源码/构建及 Git 绑定、官方原日志/固定双 seed 结果接受；默认栈、5 GiB/target 1 GiB/每秒 guard/finally clean，失败原流保留。
- [ ] 3.3 root 最终架构与对抗性验收后同步主规格、更新账本/handoff，提交推送并核对主线干净、target absent；只按实际证据勾选，保持 CF09 与 71 单元整体 partial。
