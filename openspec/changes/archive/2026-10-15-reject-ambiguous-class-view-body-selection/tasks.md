# Tasks

## 1. 选择修复与公共回归

- [x] 1.1 在保留 foreign-owner 优先校验后，让两种 body 引用复用现有候选与歧义路径，移除不再使用的首取 helper；实际运行公共 class-view 回归，核 abstract/native/Code 九种有序重复组合、每组 Name 及两个列举 identity 均返回完整有序 Ambiguous 候选，原失败不可被跳过或改期待。
- [x] 1.2 实际通过唯一 abstract/native/Code、不同 descriptor、foreign-owner、缺失成员、部分成员表及逐 body 停止控制；证明歧义与唯一无 Code 不触发类准备或 body/code/IR 准入，保留原 index/flags、错误码、coverage 与合法已用预算，并记录对应源码和测试结果。

## 2. 集成与交付

- [x] 2.1 在确切修复源码上运行受影响的 `task_operations`、`d2_prepared_handoff`、`d3_artifact_binding` 和 demand-count 门（需要 test-support 的目标按现有 feature 启用，并核非零具名执行），以及 task CLI 的 class-view/列举 identity 链/stopped-body 对照；核一次类读取、按需 body 解码、既有完整/部分结果及库/CLI 同报告，保存原始日志并经独立 review。
- [x] 2.2 coding root 完成当前 fmt、CI Clippy、OpenSpec strict 与适用 workspace 集成检查；独立提交修复，核最终源码、修复产品自身 CI 和交付记录后关闭。准确区分私有失败复现、修复后实际矩阵和最终验收，不以规划、旧 driver/callee 或历史 CI 代替本片证据。

## Workflow follow-up

- 验收后按项目流程同步 delta 并归档，复核最终实现与归档一致；相关 Stop 传播问题继续由其独立 change 跟进。

## 终态验收

- 产品e8c自身CI38679565191双seed7178/0/196、79具名门接受；root802/independent1462checks，完整证据见results/root-own-ci-v1。4/4已完成，primary4900编译来源逐项恒同；后续同步归档只修改规格与交付文档。
