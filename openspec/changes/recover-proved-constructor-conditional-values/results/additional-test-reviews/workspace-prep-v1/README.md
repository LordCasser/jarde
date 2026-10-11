# Constructor conditional full-workspace runner candidate

此目录只保存私有静态准备件，供 root 审核后决定是否执行。runner 不在本次准备中调用 Cargo、Rust、JDK、JADX 或 Git，也没有写共享 workspace 或 target。它不会携带 v4 的运行结果作为本轮结果。

## 文件

- `workspace-root-v1.py`：从 for-condition workspace runner v4 派生的 guarded workspace runner，计划输出到 `/private/tmp/jarde-constructor-conditional-workspace-root-v1`。
- `pins.json`：runner、guard/scanner 和参考 runner 的 SHA-256，以及静态推导记录。
- `review.md`：目标数、批次数、输入闭包和边界审阅。

runner 在执行开始时重新扫描当前 manifests、Rust 源、fixture、frozen evidence 和 `include_bytes!/include_str!` 输入；每个命令前后比较完整 SHA pins。corpus fingerprint 只被动态纳入该闭包，不预写旧值。新增 fixture 与 API 测试路径是闭包必须包含的显式前置条件。

执行计划由当前 manifests/文件扫描推导为 361 targets、6 个 unit-target 包批次和 89 个每批最多四个 integration-test target 批次，共 95 个测试批次。连同 fmt、metadata 和最终 clean，命令数在运行时由 inventory 计算。具体清单见 `pins.json`。

本候选尚未执行；其中没有 workspace test 通过声明。
