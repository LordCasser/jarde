# 乘法验证 build runner v3

v3 从 guarded runner v2 派生，保留 source base 参数 `977f761d9f68c6cb4de02f42b060de5290a1a947`、9 条验证命令、20 GiB 可用空间与 1 GiB `target` 守卫、source pin 前后核对及冻结 CLI 流程。新记录写入 `validation-build-root-v3`，schema 为 `int-field-multiply-validation-build-root-v3`；CLI 与 metadata 仍使用候选 v2 路径，执行前会拒绝覆盖已有文件或目录。

runner 在每份 execution JSON 中记录自身绝对路径和 SHA-256。配套 CI verifier 固定核对脚本字节哈希及 execution 中的记录，避免 build 证据只依赖自述的 runner 版本。

预算测试已按 root 的修正验证 `full_budget.output_bytes > 0` 和限制在 `complete_output - 1` 的行为。runner 只执行现有测试，不修改源文件。此脚本仅准备，尚未执行；v2 的失败 raw 与记录保持原样。
