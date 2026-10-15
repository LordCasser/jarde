# Tasks

## 1. 唯一选择与局部回归

- [x] 1.1 修复 direct/prepared driver，在完整成员表唯一性确认后才判定 Code；加入无 Code+Code 双顺序、两条无 Code、两条有 Code、唯一 abstract/native/有 Code、缺失、不同 descriptor 及其他重复方法控制。实际回归验证 Failed/RawFacts Failed/后续 NotPerformed/body NotInspected、Header 保留和 preparation 后 body/Code/IR 零增量；详见 [verification-root.md](verification-root.md)。
- [x] 1.2 修复 direct/prepared callee 的私有 locate 和错误传播，复用现有歧义输入错误；加入同等物理成员矩阵与 binding/完整性控制。回归验证不返回伪 no-body、错误码一致且不消费 body/Code/IR 额度；保留 reader 解码歧义门，详见 [verification-root.md](verification-root.md)。

## 2. 集成与交付验证

- [x] 2.1 串行运行 `cargo test --test p1_prepared_class --test p2_cfg --test p3_prepared_input --test d3_artifact_binding --all-features --locked`，核验相邻 prepared/binding 路径未回退；记录实际命令、日志、源码摘要与独立 review，详见 [verification-root.md](verification-root.md)。
- [x] 2.2 通过当前 fmt/clippy 门、OpenSpec strict 校验及 workspace 集成；保存对应最终实现提交和实际 CI 结果，独立交付本变更，verification 文档准确区分私有候选、已运行测试和最终验收。

## Workflow follow-up

- 验收完成后同步主规范并按项目流程归档，核验归档内容与最终实现一致。

2026-10-15 最终联合验收：以上早期“待验”描述保留为历史。产品d3e2b2457338b61660b842f5705b225bba08a73a自身CI38673548915成功，workspace-v3两seed各3587/0/98，focused-v3为462/0/2；独立审计接受，4/4关闭。见verification-root.md新增联合验收节。
