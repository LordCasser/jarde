# Candidate full-family verifier v4

该独立验收脚本尚未执行。它读取完整类采集结果、两个已接受的原始基线、冻结 CLI 和构建记录；不运行 Java、Rust、JADX、CLI 或 Git 命令。根任务负责实际执行并审核结果。

采集器仍固定为 `prepare-candidate-full-family-luna-v2.py`，因此本版准确读取其输出目录 `candidate-root-luna-v2`，不改采集器及其 SHA/schema 预期。构建记录绑定 `run-validation-build-luna-v5.py`（SHA-256 `93f22e0bccf4459563a0bd4d8274fbf28d85d483ba63a2ac609aae95ad2bdb56`）、输出目录 `validation-build-root-v5` 和 schema `int-field-multiply-validation-build-root-v5`。CLI 与 metadata SHA 由执行时参数传入，metadata 固定为 `candidate-cli-v2.json`。接受结果固定写入 `candidate-full-family-root-acceptance-v4.json`，若该结果已存在则拒绝覆盖。

相对 v3，本版只纠正采集输出目录与验收结果/schema 版本。v3 尚未执行。v4 只绑定现有 v2 collector 的真实输出和已提供的 v5 runner；不得将缺失的采集/build 产物视为成功验收。
