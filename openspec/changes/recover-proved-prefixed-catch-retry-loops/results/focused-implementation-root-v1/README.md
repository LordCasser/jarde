# 最后四原件：定向实现与失败历史

本包保留两项 Luna 实现、真实 Reader/Builder/API 测试、预算与取消负例、失败修订、诊断探针、fixture/指纹普查及最终 Clippy/fmt/CLI 构建。248 payload 已逐成员重开，未收录 candidate-cli 二进制。完整三方、native workspace 和本产品自身 CI 另行验收，不由此包代替。

最终 functional receipt 的 Java 核心552、CF12 9、Do2 public1、Loop3 public2、Loop4/5 public4通过；该 aggregate 因未使用导入的 Clippy 失败仍保留 FAILED。修正导入后 final-static-build-root-v2 的 Clippy、fmt、build 和 clean 全通过；实际 CLI SHA256 为 `ed74e005c1bc97bcdf4a15f6a27da9e9d8a1ecdf7fe576836820ed2ce323222d`。旧预算门365@29/130@1未改。

archive SHA256 `a94ac7838d8b3a1408edfcbf60e6842c68aeaba12aa235d2a5c43a06648294f1`。原临时路径只是来源标签，跨机器复查以inventory和归档成员为准。
