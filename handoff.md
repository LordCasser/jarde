# Jarde 接续入口

持续目标保持 active，用户要求继续在当前聊天推进。按固定 JADX **71 语法单元 / 612 测试文件**的账本完成大颗粒 MVP，再探索双方尚未覆盖的情况。组件测试通过不能直接增加整体完成数。

## 当前唯一交付项

OpenSpec [recover-structured-list-generic-bodies](openspec/changes/recover-structured-list-generic-bodies/tasks.md) 当前 **7/9**。结构化 void 正文的物理 census、List 参数真实用途证明和泛型声明/调用正文事务已经实现；复用现有 AST/精确 argument edits，不新增 solver、pass 或公共 carrier。永久测试覆盖 String/Integer、inner option、null add、控制反例、三 profile、来源和停止。详见 [verification-root.md](openspec/changes/recover-structured-list-generic-bodies/verification-root.md)。

**本轮本地验收起点是 e9e3a0ec27e5bf7698bdf4817ff08d38bcb63374，交付提交以当前 Git 与下述自身 CI 记录为准。** 用户授权提交推送；3.3 尚未闭合，当前产品自身 CI、最终主线审计仍待完成。旧产品锚点 5f5fb2c6、旧 CI 38197592542 和 f12668ed0 的最终审计仅证明前一项，不能用于关闭当前项。

## 最新有效源码与本地证据

最终 Clippy 曾发现 report.rs 两处 iter_cloned_collect 和 class_source.rs 一个 unneeded_wildcard_pattern，root 已作等价写法修正。当前 report.rs SHA ecf8c4cb10199555ae756cc3754a8a7dfe0cbc4255e7acd6280f94aea51903fb；class_source.rs SHA 16e7a07f5f133f948a5d88ea467e397f9685a18e56cd8a4734792d6efbfaa1bb。失败与后续 Clippy 通过分别保留于当前 change 的 final-clippy-failure-root-v1 / clippy-preflight-root-v2。

最新 focused-root-v12 已实际通过八命令：API 7 passed/1 ignored、jarde-java 375 passed、reader census 1 passed、P5 fingerprint 5 passed/1 ignored、fmt/build/finally clean 0；4822 输入闭包前后相同。receipt SHA 74350dfc154c2fd77aa8a9636dd8d29b301eb119e167be56a0f18af011996a73。冻结 CLI `/private/tmp/jarde-cf11-structured-list-candidate-cli-root-v4`，61920032 bytes、0555、SHA d3d15d3dd875bc79410298e3ed1e56010fc39c1849e97dc893cadb49d8c65e6f。归档见 [focused-census-and-registration-root-v12](openspec/changes/recover-structured-list-generic-bodies/results/focused-census-and-registration-root-v12/README.md)。旧 CLI/root-v11 证据对应旧来源，不覆盖当前源码。

## 运行中的门禁与下一步

全工作区 **session18940 已 exit0**。fresh workspace-root-v3 正常 passed，195 命令全部 0，无 guard stop；两组 seed 各 96 批/364 targets/3502 passed/0 failed/98 ignored，4822 来源一致、finally clean 0、target absent。execution SHA 3c2b8d52b4f1927091ea273b2503e371f066efcf64e62c944c5cdfa4cd08a7ba；独立 acceptance SHA 56d4bf06a166e2074da8951ad37f66fc2df165c045983b17ad6c21657373a461。root 两个解析路径实际通过，归档见 [full-workspace-accepted-root-v3](openspec/changes/recover-structured-list-generic-bodies/results/full-workspace-accepted-root-v3/README.md)。

最终本地门禁 **session49761 已 exit0**：fmt/当前 CI Clippy/global strict357-0/CLI rebuild/exact clean 五命令0，新CLI字节恒同、4822来源一致、target absent，root独立 accept-final.py exit0。final receipt SHA 3cf03577eea64a7d19f1164cbc2da3d322686c7eb5d5baca3dd50f1e22286b04，acceptance SHA f91f3ab30fea420d4d39ba1662d1cb9ef229b64aaef4af9359ef3ae29d43ad0c；见 [最终门禁](openspec/changes/recover-structured-list-generic-bodies/results/final-validation-root-v2/README.md)。3.2 已关闭。

新 CLI v4 的整类对照 **session6599 已 exit0**，root 独立 checker 接受12份/352命令/704raw/80成功运行对/24控制编译失败无runtime/41 oracle labels。新 CLI 精确停止两调用实际4/0；collector因方法name读取层级错误failed，root独立接受其完整raw且保留原失败状态。见 [整类及停止归档](openspec/changes/recover-structured-list-generic-bodies/results/final-comparison-cli-v4-root-v1/README.md)。3.1/3.2均已关闭，当前没有 Jarde 运行工具链，target absent。下一步提交推送确切产品并验收该HEAD自身CI（3.3）；最后文档推送、only main/14aux detached/全部干净无target/主线祖先的实际finalHEAD外置审计（3.4）。旧矩阵空间失败没有拼接，71/612和整CF11完成数不增。

## 工具和资源约束

root 独占共享 Git/Cargo/rustfmt/JDK/JADX/CLI；Luna 仅只读或私有准备，不改共享源码，不运行工具链，不创建分支/worktree。外部命令最低 5 GiB 可用空间、target 上限 1 GiB、1 秒进程组守卫、default Rust stack、最终 exact cargo clean。现有构建仍运行时不得清 target 或并行运行其他工具链。/tmp 清理只移除可再生成缓存或做保留字节的历史去重，当前依赖与独有样本保留。

## 后续大颗粒方向

循环体 try/catch 是下一项，当前仅架构侦察，**没有新基线或实现验收**。见 [架构范围](openspec/changes/recover-loop-body-try-catch/architecture-scout-root-2026-10-12.md) 和 [JADX 算法差异](openspec/changes/recover-loop-body-try-catch/jadx-algorithm-review-root-2026-10-12.md)。优先核现有 proved_loop_catch_joins 的具体拒绝条件，不删除 CFG 边强制归约；DV.plainFor/while 与 foreach 的 protected-local SSA 拒绝必须分别归因。私有基线包 `/private/tmp/jarde-loop-body-catch-baseline-prep-luna-v3` 已经 root 阅读，仍待串行运行 debug/none 新整类对照与 Q-i/Q-ii/Q-iii 同次事实。

历史验收与架构债务分别保留在各 change 的 results 和 [completion-review.md](openspec/completion-review.md)，不能混入当前产品改动扩大范围。
