# 构造条件值关闭证据的主线审计

关闭证据提交 `026de6ae4e2ab671b7e660a1c6d32aefaeab842a` 推送后，root 于 2026-10-11T19:21:12Z–19:21:18Z 实际执行 outside-repo audit，exit0/status accepted_clean_main。HEAD/main/origin/main/远端 main 一致且仅 main；15物理 checkout、14aux detached/干净/main祖先、无 target。4727完整源码与已验 final CLI build/产品 Git/live 相同。机器可用27345055744B。

actual-audit 和 invocation 保存原始字节；本审计没有删除 protected worktree，目录仍存在。它接受的是上述真实提交，不冒充后续关闭任务文档的最终 HEAD。最终文档提交推送后，以真实新 HEAD 再运行同一脚本，结果保存在 `/private/tmp/jarde-constructor-conditional-final-clean-main-root-v1/execution.json`，该 outside-repo receipt 不会弄脏交付树。

最终build receipt SHA256 `90ecba1b52177d7dfae590679603e4624c4e56a42ae2b58d0616d7078c1f46e2`；CLI `a71c31e80da7c6550aa572e5368f0ecad12ac862066c5d40535f5818c2263241`。
