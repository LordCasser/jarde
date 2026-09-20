## Context

见 [proposal](proposal.md)。`LoadRoot` 当前有 Snapshot、ArtifactTree、External；前两种在 ZIP 上都只从 container 根匹配。`LayoutTarget::ClassLayer` 已保留 entry prefix，但没有合法的 resolver root 可表达它。`read_own_definition` 的 binding 检查本身应保留，问题是 root 描述力不足。

## Goals / Non-Goals

**Goals:** 用一种统一加载位置表达普通 JAR 根和 WAR classes 前缀，并让 CLI 可取得真实身份完成最小使用闭环。

**Non-Goals:** 不把 layout 识别当作加载策略，不自动排序 WEB-INF/lib，不迁移 MR selection 进入 resolver，不引入另一套 logical class identity。

## Decisions

### 1. 收敛 LoadRoot，物理地址和逻辑名字各自保留

新形状为 `StandaloneClass { snapshot }`、`Container { origin, prefix }`、`External { id }`；prefix 复用 `ArchiveNameBytes`。原 ZIP Snapshot root 迁移为 root origin + 空 prefix，原 ArtifactTree root 迁移为对应 container + 空 prefix，原 standalone Snapshot 迁移为 StandaloneClass。不并存旧变体，不增加 WarRoot、BootRoot 或 LayoutLoader。

prefix 为空或以 `/` 结尾，非空却缺边界分隔符时拒绝为无效环境声明；不依赖 ZIP 存在目录 entry。组合采用 checked length 的 raw bytes 拼接：`prefix + internal_name + b".class"`。不 trim、不 URL decode、不大小写折叠、不折叠 `.`/`..` 或反斜杠。它是归档内精确字节前缀，不是宿主文件系统路径，也不触发解包。

例如 `WEB-INF/classes/com/demo/A.class` 的物理名字完全不变；声明 root prefix 为 `WEB-INF/classes/` 时，对 `com/demo/A` 查名才找到它。空 prefix 不会自动识别布局。相同 prefix 下的重复 ordinal 仍构成同一选择位置的多个候选。

### 2. runtime binding 继续走同一 resolver

环境验证检查 snapshot/content、container origin、prefix 形状及既有 loader/domain 条件。符号查名、driver/caller binding、已知事实复用、root snapshot 提取及 report/view fingerprint 使用同一 root 定义；不能只修 `tree_candidates` 而让 binding 仍按旧 root 比较。

候选绑定必须校验 Header 的 `this_class` 与请求内部名一致；现有分支缺少这项检查时在统一 binding 路径补齐。路径不一致返回可定位诊断，不尝试下一个 root 掩盖坏候选。名称一致也只证明该候选绑定成立，不代表完整类加载或 verification。声明顺序决定跨 root 优先级，容器扫描顺序不能替代；同名同内容不同 origin 不合并。prefix 进入运行环境身份，物理目录 cache 不因此复制目录；跨环境不缓存 Missing 或唯一选中结果。

### 3. CLI 只补枚举闭环

增加 JSON operation `enumerate_artifact_tree`，调用 facade 现有同名入口并直接返回库 report；继续使用显式 limits。通过 report 的 snapshot/container/entry 身份构造现有 recover_method 请求，由调用方声明 root prefix。输入不匹配、Partial、Cancelled、nested 错误保留库语义，不自动修正 snapshot，不自动构造 classpath，不增加全 artifact recovery。

同一个 outer WAR 内的 classes 和 nested libs 足以通过当前 CLI 单 input_path 完成 fixture；CLI 多外部 artifact 输入是另一项产品需求，不混入本次。

### 4. 与定向容器优化衔接

prefix 只影响目录内的查找 key；`bound-container-lookup` 只改变取得目录/backing 的方式，两者的语义可以独立验证。建议先稳定定向访问再迁移 root enum；cache off/on 都必须得到同一候选和 binding。库和 CLI 的普通 enumerate 不自动展开树；仅显式 tree 操作或显式 container origin 访问对应范围。

### 5. 复用与依赖

复用现有 ContainerOrigin、ArchiveNameBytes、layout evidence、环境 validator、providers 和 serde；不需要容器框架或 Java loader 库，也不升级 rawzip/flate2。生产依赖的许可和维护边界沿用既有准入。物理布局 parse、运行时选择、class dialect、verification 和 source recovery 继续分阶段，泛化 prefix 能力不宣称 Boot/MR 专项已通过。

## Risks / Trade-offs

- enum 迁移漏掉非恢复消费者 → 编译穷举所有 match，核对 query/runtime matrix/provider identity、CLI 与现有 tests。
- prefix 改写损失物理身份 → 测试源 map 与拒绝诊断仍指向带 prefix 的真实 entry；raw bytes 测试覆盖非 UTF-8、大小写和非规范分隔符。
- 便利功能变成自动选择 → 测试“只有 layout 检出、没有显式 prefix”仍 unbound，不能由 layout 自行补 root。
- runtime 能力被过度宣称 → 初始验收限定 Java 8、ClassPath、MR disabled，Boot classes prefix 仅做受控同机制样本。

## Migration Plan

一次迁移仓库内所有 LoadRoot 构造与 JSON fixtures，再加入 prefix binding 和 CLI 枚举闭环。更新 API 示例，明确 breaking 形状，不保留旧 schema 适配层；无持久数据迁移。若回滚，整体回滚该 root 迁移，不通过禁用 binding 作为降级办法。
