# Candidate full-family verifier v3

该独立验收脚本尚未执行。它读取完整类采集结果、两个已接受的原始基线、冻结 CLI 和构建记录；不运行 Java、Rust、JADX、CLI 或 Git 命令。根任务负责实际执行并审核结果。

脚本绑定构建脚本 `run-validation-build-luna-v5.py`（SHA-256 `93f22e0bccf4459563a0bd4d8274fbf28d85d483ba63a2ac609aae95ad2bdb56`）、输出目录 `validation-build-root-v5` 和 schema `int-field-multiply-validation-build-root-v5`。CLI 与 metadata SHA 由执行时参数传入，metadata 固定为 `candidate-cli-v2.json`。候选采集输出固定为 `candidate-root-luna-v3`；接受结果固定写入 `candidate-full-family-root-acceptance-v3.json`，若该结果已存在则拒绝覆盖。

相对 v2，本版只更正候选验收版本绑定、`multiplyDivide` 访问数变量引用，以及 test1 访问统计的含义：分别报告接收对象 `a` 的两次读取、数值字段 `f` 的一次读取和一次写入。原基线 census 的读取、保存和返回已由 helper-only 实测确认，保持原样。v4 构建预检因磁盘余量不足而未启动；本版只绑定新的 v5 记录格式，不把未执行的构建或候选结果称为已验收。
