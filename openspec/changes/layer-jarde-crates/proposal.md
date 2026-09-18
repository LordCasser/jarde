## Why

P2 已有 resolver、CFG、pass 调度和 legacy analysis，后续还要加入 Frame/SSA。继续只靠单个 crate 的模块约定，无法用 Cargo 约束“纯查询不依赖反编译”，`engine.rs` 也同时承担公共入口和方法分析实现。现在已有真实依赖可据，不必等全部中端写完再拆。

## What Changes

- 首轮新增 `jarde-reader`、`jarde-query`、`jarde-jvm` 三个 workspace crates；现有 `jarde` 成为统一门面，`jarde-cli` 仍是薄适配器。
- 将共享物理身份、预算/错误、声明式 view 和唯一 reader 放在读取层；纯 X0/X1 留在查询层；运行环境、声明解析和 JVM 分析放在语义层。保留 `jarde-jvm → jarde-query` 的真实候选扫描依赖，禁止反向依赖。
- 把跨包 `pub(crate)` 访问改成少量有不变量的类型化接口，不整体公开内部可变结构、不复制 decoder、不增加通用 backend/pass 框架。
- 同步测试归属、fuzz workspace、依赖门禁和 A17 路径；证明拆分没有改变身份、原始引用、阶段状态或预算计费。
- P3 首个实际恢复闭环落在 `jarde-java`；本 change 不创建空 crate，也不实现 Region/Java 输出。

实施状态：P2 `0.3/0.3b/3.4` 的历史绿色基线已用于拆包，reader/query/jvm 及门面主体已完成，目前收尾 3.1–3.3。新发现的返回地址证明缺口由 P2 3.4b 独立修复，结构验收后先交接该修正，再进入 `3.5/4.x`。本 change 是结构重组，不兼带语义修复、功能增加、算法替换或 droidsaw 接入；不把主体搬迁等同于全部验收完成，也不预估编译提速。

## Capabilities

### New Capabilities

无新的分析行为。本 change 设置 `skip_specs: true`，以 design 的依赖图和 tasks 的编译/回归门槛验收，不为纯重构另造行为 spec。

### Modified Capabilities

无。沿用 `analysis-contracts`、`query-api`、reader/XRef 以及 P2 的既有行为契约。公共导入路径若需收敛，直接更新仓库内调用方，不为旧模块布局保留双实现或兼容层。

## Impact

涉及根及子包 Cargo manifests、现有 `src/` 的归属、库/CLI/examples、tests/fuzz、CI 和文档。第三方生产依赖及 MSRV 保持现有准入结果，按实际使用下放到所属 crate；多个 crate 使用同一物理身份和预算类型，不经 JSON 或复制模型传递。拆分后仍整体维护，不增加独立发布流程。
