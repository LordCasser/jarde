# 最终 native 与自身 CI 验收

产品 `cc1bf5fa37ef51c4d11a78ea1e9dfcabfe169d2c`，最终 CLI SHA256 `9ea9177b8992c8444c9f9af0158ca7bc99b3e7dfee4696b342aea7da2e295167`。[自身 CI 39186343472](https://github.com/LordCasser/jarde/actions/runs/39186343472) 全部四个任务、52 个独立官方 ZIP 步骤通过；双 seed 各 389 targets、3798 passed / 0 failed / 98 ignored，98 个精确具名门均通过，全部 3896 个具名结果完全一致。13 个继承补充检查保留，未以汇总日志替代步骤证据。

fresh native 一次执行 389 targets / 102 batches，3798 / 0 / 98，53 个精确门和 CF07/CF11/finThrow/nestedFin 九个周边回归各一次。默认栈、每秒守卫、target 峰值 847800437 B，finally 精确清理后 target absent。构建与 native 共 5216 个源码路径逐 Git blob、实际 bytes/mode 绑定产品；CLI 与完整对照同一字节。没有重复运行指定门冒充覆盖。

`capture.tar.gz` 保留 18 个明确目录的 524 个 regular payload 及 inventory，包含完整收据、raw stdout/stderr、官方 ZIP、静态准备版本、网络等待状态、root 首次汇总的错误形状断言和修正说明。归档 SHA256 `1b69842821ac1f006bb746ea80d64e74e0369f33a626719978c981bcbad8522e`；root 另逐载荷复开 SHA、mode 与原件 bytes。CLI、JDK、target 与缓存不复制。pack 工具只保存 regular 文件，原始目录名通过成员路径保留；不声称保留空目录。

完整源码与运行对照见[独立入口](../focused-and-full-comparison-root-v1/README.md)，原件/JADX 的接受基线按其独立归档精确复用。后续文档提交不冒称又运行一次产品 CI。主 spec 已同步，strict 与最终 clean-main 外置收据另核。
