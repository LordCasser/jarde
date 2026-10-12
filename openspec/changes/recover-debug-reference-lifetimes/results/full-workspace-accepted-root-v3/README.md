# 完整工作区独立验收

workspace-v3（session 91131）195 条命令全部退出 0。两组固定 seed 各 96 批、365 targets、3547 passed / 0 failed / 98 ignored，合计 7094 / 0 / 196。4860 个来源前后及当前文件一致。默认 Rust stack、5 GiB 余量 / target 1 GiB 守卫和 finally exact cargo clean 已验收，target 不存在。

[capture.tar.gz](capture.tar.gz) 保留 receipt、metadata、390 个原始流、15 个精确源码快照、4860 来源闭包、工具及独立接受结果。root 另逐成员复开 419 项；archive SHA256 `3b3f45e9b0daade6ee1b5d47e660abec73605db38dc2f562d8079efc21357ef5`。receipt SHA256 `f3ef8761467bca849881ddcea9a8369583689652c7341907c61fe3126239e3b8`。

首轮独立 verifier（session 90716，exit 1）沿用旧 p3_patterns=91 断言。当前源码及两 seed 原始输出均为同一组 92 个通过用例；root v5 只将四处数量/描述改为 92，保留 v4 runner 和其余门，session 82680 实际接受。该失败与最小修正见 [acceptor-review-root.json](acceptor-review-root.json)。

本记录只接受本地 workspace；产品提交、自身 CI 与真实 main 审计须分别完成。旧 workspace-v1/v2 失败记录保留，未拼接为成功。
