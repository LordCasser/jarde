# Candidate full-family verifier v5

本脚本尚未执行。它独立检查候选完整类采集输出、两个已接受的原始基线、冻结 CLI 和构建记录；不会启动 collector，也不会调用 JDK、Rust、JADX、CLI 或 Git。根任务负责实际验收。

本版绑定 collector-v3：`prepare-candidate-full-family-luna-v3.py`，SHA-256 `d6e74e8bc09c163e23bed18d823f575514ae19da57b968e0d3a26245e6c8be69`，输出目录 `candidate-root-luna-v3`，collector schema `em23-int-field-multiply-candidate-full-family-luna-v3`。构建绑定 validation runner-v5：SHA-256 `93f22e0bccf4459563a0bd4d8274fbf28d85d483ba63a2ac609aae95ad2bdb56`，输出目录 `validation-build-root-v5`。CLI 与 metadata SHA 由参数传入，metadata 固定为 `candidate-cli-v2.json`。验收结果固定写入 `candidate-full-family-root-acceptance-v5.json`，已存在时拒绝覆盖。

default 模式逐个物理方法核对 `read_details` 为 `not_requested`，且 `report.fields` 为空；all 模式要求 `complete`，并在 all 模式检查精确 field-access / BCI / presented 信息。两种模式的完整源、方法正文和 source map 仍要求逐项相同，因此 default 的呈现文本仍受 all 模式证据约束。构建及 collector 的成功与否必须由实际记录和原始输出决定；本说明不表示它们已通过验收。
