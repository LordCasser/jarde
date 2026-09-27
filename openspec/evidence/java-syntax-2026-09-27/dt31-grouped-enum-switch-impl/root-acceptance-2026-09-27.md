# DT-31 主线独立验收

主线合入实现后，以重新构建的 CLI（SHA-256 `8f1f0012324350e4fc65c7fef4b3e3835102e6d4fda00df280b466270c98e727`）执行本目录 `replay.py`。除 CLI 自身 SHA 外，重放摘要的每个字段都与 `acceptance/summary.json` 相同。原始、固定 JADX 与 Jarde 的完整源码均通过 `javac --release 8` 和 `java -Xverify:all`；全部六个 enum 组合与两个 null 路径的结果及可见副作用顺序相同。Jarde 的两个站点均标记 `projected=true`，损坏其中一张表后均为 `false`，冻结单站点源码逐字节相同。

另以同一 CLI 重放 DT-07 基线；除 CLI SHA 外，它与冻结摘要逐字段相同，说明此次枚举修改未改变匿名类拒绝边界。主线 `class_source` 75/75、`jarde-java` enum 定向测试、格式检查、两个 OpenSpec 严格校验通过。联合 `<clinit>` 证明还补齐了同一次 BCI 认领列表内部的重复检查。不同 helper 的多站点形态未由本次正例证明，继续留在 DT-31 扩验范围。
