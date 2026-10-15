# Tasks

## 1. Repair and regression

- [x] 1.1 统一三个层的新项总 entry 计数及容量拒绝归因；增加插入排列、零/满 entry、同 key 重写控制，核三类总库存不超限且 entry/byte 拒绝原因准确；root reader 单元与对应 P5 回归通过，见 [verification-root.md](verification-root.md)。
- [x] 1.2 CP/Header 不兼容移除同步扣减实际项权重；增加 format/registry/product 失效后重新保留、混合 store 及活动 payload 引用控制，核不需 clear 可重新保留且其他项权重不变；root 新增回归通过，见 [verification-root.md](verification-root.md)。

## 2. Integration and delivery

- [x] 2.1 在确切修复源码上实际运行 reader 单元、P5 facts-cache/definition-read/shared-payload/container 门禁及生产边界回归，核 direct/cache-on 完整语义与取消/预算回退；命令、源码 SHA、独立 review 与失败历史说明见 [verification-root.md](verification-root.md)。
- [x] 2.2 coding root 串行完成 fmt、当前 CI Clippy、OpenSpec strict 和与主线改动共同适用的工作区门禁；独立提交修复并核最终代码、真实测试与 CI 证据，更新验证记录后关闭。规划或私有未测试 patch 不作为完成证据。

## Workflow follow-up

- 验收通过后按项目流程同步 delta 并归档；复核归档结果及最终真实 HEAD，不以旧产品证据替代新修复验证。

2026-10-15 最终联合验收：以上早期“待验”描述保留为历史。产品d3e2b2457338b61660b842f5705b225bba08a73a自身CI38673548915成功，workspace-v3两seed各3587/0/98，focused-v3为462/0/2；独立审计接受，4/4关闭。见verification-root.md新增联合验收节。
