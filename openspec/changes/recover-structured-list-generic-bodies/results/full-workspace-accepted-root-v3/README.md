# 当前来源全工作区验收

root 从零执行正常 passed 规则的两组 seed 全目标验证，session18940 已真实 exit0。195 命令全部 exit0，无 guard stop；每组 96 批、364 targets、3502 passed/0 failed/98 ignored，各目标 exact-once。728 份目标摘要、7200 个具名结果、390 份 raw streams 完整保存。4822 输入闭包前后相同，default Rust stack、5 GiB/target1 GiB/1 秒守卫和 exact cargo clean 满足，target 不存在。

原 execution SHA `3c2b8d52b4f1927091ea273b2503e371f066efcf64e62c944c5cdfa4cd08a7ba`；独立 workspace acceptor 的 summary SHA `56d4bf06a166e2074da8951ad37f66fc2df165c045983b17ad6c21657373a461`。root 自己执行 ownCI verifier v8 的 verify_focus/verify_workspace，第二解析路径也通过，逐目标原始流、忽略语义和具名结果相同。精确新 CLI root-v4 与 focused v12 绑定保留。

完整原始目录、两解析器与 root 调用 argv/stdout/stderr/退出码、冻结工具和原始 root 检查脚本在 capture-and-tooling.tar.gz；摘要包含 archive SHA。旧 workspace-root-v2 的 failed 状态没有改写，也没有拼接其结果。本条只证明当前来源本地全工作区；最终 lint/strict/CLI 重建、自身 CI 与干净主线仍独立待验。
