# Jarde reference-prefix 联合证据归档说明

此说明对应共享结果目录中的 `capture.tar.gz`。该归档保存已完成的验收证据、运行收据与原始输出、源码快照及其 pins，供后续逐成员复核。它是证据包，不是新 CLI 的最终交付，也不表示实现任务已全部完成。

## 已接受的整类运行证据

`candidate-v5` 和 `boundary-root` 是两个独立的 accepted 状态，不能把各自的场景数或命令数合并解释：

- Candidate v5 收据为 `passed`。它覆盖 debug 与 none 两种候选；每种 25 条实际命令，4 份 Jarde 报告均通过，每份报告包含 11 个方法，并包含 4 个 JADX negative controls。收据按 17 场景原始输出 oracle 校验，oracle SHA-256 为 `50ea25f5e9f74bc8d69b3dc945cd9ba006ef9a615a9e149c73e64c33f70f6231`。
- Boundary root 收据为 `passed`。32 条实际命令、64 份 raw 输出；覆盖 4 个 original runtime、8 个 Jarde full-class runtime，以及 8 个 JADX compile-failed/no-runtime 场景，每个 runtime 10 个场景。收据确认各 runtime 输出字节相同、目标路径不存在；oracle SHA-256 为 `6d8cc61bcb7a81467d4ee3ed20ce5269cffe7c9107dadf3af56dce9166732b48`。

以上是归档中的 accepted 结果；它们不代表后续 focused 验证、工作区验证或产品交付已经结束。

## 后续 focused runs 的状态

- Focused v6 的执行状态原样保留为 `commands_passed_pending_independent_acceptance`。这表示所记录命令完成，但尚无独立验收结论；不得称为 accepted 或最终通过。
- Focused v7 整体状态为 `failed`。失败点是 Clippy `int_plus_one`；其余记录为 409 个 library tests、71 个相邻测试和 8 个 structured tests 通过。最终 exact clean 命令退出码为 0，且 target 不存在。失败收据与 raw 输出均留档。
- 归档不含 v8/v9。v8 实际整体 failed：新增测试的临时引用触发 E0716，Java library test 编译退出 101，finally exact clean 退出 0、target absent。root 已保存 exact source，修正为显式局部绑定；v9已完成并独立接受，见[最终局部门](../final-focused-root-v9/README.md)；原v8失败状态不变。

## 归档范围与复核

归档含通过的 candidate/boundary 验收收据、v4 verifier failure observation、focused v6/v7 相关记录、命令规格、必要工具脚本文本、capture 输入和 `.class`/原始 input JAR 等小型来源证据，以及按收据匹配的源码快照。未将 CLI、JDK、JADX 等大型工具二进制打入包内。失败 v3/v4/v5 的收据/raw 与 exact-source 备份按失败证据保存，不把失败材料混作新成功结果；v7 的 exact test source 来自与其 receipt pins 匹配的快照，另有 v7 audit prep 文件单独标注为 prep，不冒充 exact source。

独立逐成员复开记录为 `independent-archive-review-root-v1.json`：状态 `passed`，归档 SHA-256 为 `b958133beba3dd9a52b5f2934736ad8801c1df32eb0abee6c51e02ef85cfc40a`，共复开 390 个成员（含 manifest），所有 exact snapshots 均已收录。归档大小为 4,683,949 bytes。`archive-pins.json` 与独立复核 JSON 提供归档及成员校验信息；原始收据保持原样。

本归档不关闭 OpenSpec 剩余工作。当前Clippy/strict/fresh CLI及必要focused已按最终局部门接受；fresh workspace 正在运行，产品自身CI、clean-main以及2.3仍待完成。请依据任务记录和后续独立验证推进，不能由本证据包推定这些检查已通过。

联合结果使用 v6 冻结 CLI（62,562,992 bytes / 0555，SHA-256 `fe00fa7f0edc587a4e85bcc1136bd4397738c2a9df632432a2e79a8221c8bdd6`）。后续源码新增仅为测试和 fixture；最终新构建 CLI 必须重新核对上述字节身份，若不同需重跑完整类矩阵，不能将旧运行冒充新产品。
