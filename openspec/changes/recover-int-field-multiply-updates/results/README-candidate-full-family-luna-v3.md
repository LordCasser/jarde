# 乘法候选全类重放 collector v3

v3 保留 v2 的 CLI 与 metadata 参数、基线 pins、原始证据复核、源码映射、物理成员 census、预算、8 个候选全类编译/运行腿及 closed inventory。输出改到 `candidate-root-luna-v3`，manifest schema 升为 v3，避免覆盖已记录 v2。

只调整 `multiplyDivide` 的可选 read-details 证据检查：default profile 要求 `read_details` 状态为 `not_requested` 且字段访问列表为空；all profile 要求状态为 `complete`，并逐项核对五条精确 owner/name/descriptor/access/BCI、非 static、无 refusal 且全部 presented。source map、呈现文本、原始字节、物理 census、预算和运行结果检查保持不变。

v2 已有 40 条命令的结果及失败标志保持原样。v3 仅准备，未运行 CLI、JDK、Cargo 或 Git；也不代表所有全类重放已通过。
