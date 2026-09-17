## Why

P1 已归档，但复核发现 query fuzz 用文件魔数选择请求，合法 CLASS 固定落入一种请求；CI 的 cargo-deny 也未审计独立 fuzz workspace。修正验证工具才能让后续 P2 的执行依赖可靠门禁。

## What Changes

- 同一 artifact 执行全部五个固定 query 请求，target 与 corpus test 共用驱动，并验证还原旧路由会触发回归。
- CI 明确审计根与 fuzz 两个 workspace，使用同一个 deny.toml；NCSA 从全局允许收窄为 libfuzzer-sys 0.4.13 的例外。
- 将 CLI 对根 jarde 的已有 path 依赖补为精确当前版本，保持 wildcard deny；不新增依赖或改变解析结果。
- 保持 512 MB smoke 限额，记录修改后的本地 60 秒实测及 CI 20 秒配置，不混用平台与 sanitizer 口径。

## Capabilities

### New Capabilities

无。仅修复验证工具，使用 skip_specs，不新增产品能力。

### Modified Capabilities

无。P1 生产行为及已归档规格保持不变。

## Impact

限于 fuzz harness/tests/README、CI、deny.toml、CLI 既有 path 依赖的版本约束及本 change 的验证记录。复用 libFuzzer/cargo-deny，不新增依赖或修改 lockfile，不实现 P2 代码。P2 的架构设计和任务修订单独保留在 p2-jvm-ir。
