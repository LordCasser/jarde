# Candidate full-family verifier v6

本脚本尚未执行。它继续绑定 collector-v3、build-v5 和原有 frozen baselines；不启动 collector，也不运行工具链。结果写入 `candidate-full-family-root-acceptance-v6.json`，已存在时拒绝覆盖。

v5 的实际独立验收因 `KeyError: fields` 失败：已固定的 EM23 helper 在验证物理成员、owner、flags/descriptor 和 source-map BCI 后返回紧凑方法记录，不包含 presentation `fields` 与 `quality`。v6 不修改或替换 helper；窄范围包装器先调用该固定 helper 完成原有身份和来源核验，再按已核验文档中的物理方法身份逐项连接原始 report 的 `fields`、`quality`，并要求两侧方法集合完全一致。其余候选、raw、source-map、default/all、命令与运行时检查不变。

该修复只解决 verifier 的结果结构适配，不改变 collector 或证据，也不表示验收已通过。
