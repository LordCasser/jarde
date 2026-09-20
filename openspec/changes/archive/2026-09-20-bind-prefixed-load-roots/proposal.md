## Why

benchmark 中 158 次 `resolution_definition_unbound` 集中在 `WEB-INF/classes/...`：物理 entry 可读，但当前 LoadRoot 只能按 `<internal_name>.class` 从 container 根匹配，无法表达布局类路径前缀。Layout 已有物理证据，缺口位于调用方声明的加载位置；绕过 driver binding 会破坏身份契约。

## What Changes

- **BREAKING**：将 ZIP 加载位置统一表达为 container origin 与显式 raw entry prefix；standalone CLASS 保持独立含义，不增加 WAR/Boot 专用 loader。
- 用 `prefix + internal_name + .class` 精确查名，物理 raw name、ordinal、snapshot 和 origin chain 保持原样。
- 保留 loader 委派/root 顺序、重复定义歧义、损坏候选停止和 driver binding；不自动激活扫描到的库。
- CLI 增加薄的 artifact-tree 枚举操作，使调用方取得真实 container/entry 身份，再显式构造前缀 roots 和恢复请求。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `demand-resolver`：允许显式前缀加载位置及其身份绑定规则。
- `artifact-views`：CLI 提供与库一致的物理树枚举及 root 声明所需证据。

## Impact

前提为 reader 物理布局、origin replay、环境校验和 root 顺序已交付。影响 `LoadRoot`、environment/providers、相关 view identity/序列化、CLI 和集成测试；使用现有 raw bytes/serde，不新增依赖。可在独立提交实现，建议在 `bound-container-lookup` 定向容器入口稳定后集成以减少重叠。

不实现 Servlet/Boot 自动加载规则、classpath.idx、MR overlay 与 resolver 的整合、module path、外部依赖下载或全 artifact 入口。前缀支持不等于完整 Boot 支持。关联分析见 [benchmark review](../../benchmark-review.md)。
