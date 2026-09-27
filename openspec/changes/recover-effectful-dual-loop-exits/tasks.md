## 1. 先行可行性门槛

- [ ] 1.1 以冻结 `EffectfulExits` class 重建 Region/Frame 首访、回边和两臂走访路径；从 Jarde 实际 SSA 核 BCI 35 的 slot 1 值/φ 及唯一消费。若无法在现有 walker 中有界地只放行首次 header、回边停止并将头比较保留为体内 `If`，记录具体阻碍和安全拒绝测试，停止本 change 的投影实施。

## 2. 窄证书与源码投影（仅在 1.1 可行后）

- [ ] 2.1 有界证明单入口/单回边、头部独占效果出口、体内独占纯转移、唯一共同后继、完整正常/异常边及 SSA 定义/消费；额外入口、分叉后继、handler、额外 latch/效果、错 SSA 与预算/取消拒绝。
- [ ] 2.2 必要时仅增加 Region 的 Endless form，使 body 首访头部、回边停止，两条退出都成为 LoopBreak；builder 用现有 AST 输出 `while (true)`，检查循环/header/效果块/后继各拥有一次且 source map 覆盖固定 BCI。

## 3. 三方验收与回归（仅在 1.1 可行后）

- [ ] 3.1 原 class、固定 JADX 和 Jarde 完整 Java 8 源码经 `javac --release 8 -g:none` 与 `java -Xverify:all` 三组输出同为 `8:1 / 3:0 / 8:1`；重放纯双网关及单出口循环控制，并保留 `NotIndexedLoop` 外层形态为独立差距。
- [ ] 3.2 运行相关 Rust 定向测试、`cargo fmt --all -- --check`、`cargo check --workspace --locked`、`git diff --check` 与 `openspec validate recover-effectful-dual-loop-exits --strict`；清理专用 Cargo target，提交可供 root 独立验收的证据。
