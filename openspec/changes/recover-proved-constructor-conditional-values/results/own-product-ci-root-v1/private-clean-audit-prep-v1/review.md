# 私有 clean-main audit 准备审阅

此目录只包含后续 audit 所需的静态脚本、pins 与本审阅。没有执行 audit，也没有运行 Git/GH/Cargo、联网或修改共享文件。

## 候选与构建绑定

- Audit 目标是 `/private/tmp/jarde-constructor-conditional-final-cli-root-v1`，预期字节数 `61286928`、SHA-256 `a71c31e80da7c6550aa572e5368f0ecad12ac862066c5d40535f5818c2263241`、mode `0555`。该身份目前来自已冻结的 v6 candidate `/private/tmp/jarde-constructor-conditional-candidate-cli-root-v6`；其实际大小、哈希与 mode 已读并记录在 `pins.json`。
- 最终 CLI 路径和 `/private/tmp/jarde-constructor-conditional-final-lint-build-root-v1/execution.json` 在准备时均不存在。脚本只在将来运行时验证它们，不预写 final build 成功。
- 最终 lint/build runner 的当前 SHA-256 为 `7bf3a61aec009aaa629efad18fe8e177b25d0e3b828de135d52eba498ea5c2c3`，schema 为 `constructor-conditional-final-lint-build-root-v1`。脚本还要求最终 receipt 与 frozen candidate byte-identity、focus receipt v6 一致、target 清理、4727 source pins before/after 完全相同、fmt/clippy/OpenSpec/build/clean 五条命令全部退出 0 且无 guard stop。
- final receipt 的 SHA-256 与预期 HEAD 都是运行时必填参数，因为 final receipt 尚未产生且目标 commit 由 root 的后续状态决定。

## Receipt 与 live source 验证

脚本继续沿用 clean-main v3 模板的 live source SHA → 指定 HEAD Git blob OID 核验、单一 local/main 与 origin/main、15 个 worktree、14 个 detached 辅助 checkout、main ancestry、clean 状态、无 `target`/`fuzz/target`、磁盘至少 5 GiB 等审计门。

本版补充验证 final build 及 focused build receipts 所引用的每条 stdout/stderr raw stream：路径必须落在各自输出目录，文件名与 command index/channel 一致，不能是 symlink，长度和 SHA 必须匹配；目录里 `.raw` 文件集合也必须与 receipt 引用集合完全一致。审计自身运行时产生的 Git stdout/stderr 在写最终 JSON 前也会重新核对长度和 SHA。此处只是准备校验逻辑，没有读取或伪造不存在的 final build receipt。

## 范围差异与静态检查

相对模板，仅适配 constructor-conditional 的 CLI/build 路径、schema、候选 identity、source closure 数 4727，以及其真实 focused receipt 与依赖 guard/scan pins；保留 HEAD/build-receipt SHA 的运行时输入。为证明所有外部 raw receipt 均可追溯，新增了 focused/final/audit 三组 raw stream 校验，没有引入通用新框架。

已对最终 audit 文件使用 `ast.parse` 做 Python AST 语法解析；结果通过，顶层 45 个节点。没有 import/执行 audit 文件，因此没有触碰脚本中的 Git、远端、磁盘状态或其他命令。
