## Why

P0/P1 及 P1 验证维护已归档。本轮算法复核固定在 `eac3759`，并纳入随后 `955d7f3`/`823173b` 的 5.2 验收增量（2026-09-19）。`layer-jarde-crates` 已完成 7/7，尚未归档；P2 已完成至 5.2，共 25 项。3.4b 返回地址证明、3.5 规范化、Frame、初始化、SSA 与 `analyze_method` CLI 均已交付。本轮新增异常状态传播修正 4.2b 和派生存储计费修正 4.3b 后为 **25/29**；5.3/5.4 仍未完成。P3–P5 未实施。当前工作是关闭复核反例并完成阶段验收，见 [最新复核](verification.md#review-2026-09-19-ir)。

## What Changes

- 在共享 reader 内补齐类型化操作数和目标边界校验，为依赖闭包、IR 存储、工作列表和子程序克隆增加可取消的预算契约。
- 增加显式绑定 snapshot、RuntimeView、loader domains 和平台 Header providers 的 Demand Resolver，区分 Resolved/Missing/Ambiguous 等状态。
- 增加带运行环境的独立解析入口及声明引用查询，覆盖 A11 的 Base/Sub 关系，避免先按 CP owner 精确过滤导致漏项。
- 在已有 raw CFG 上补齐 wide/数组操作数、returnAddress 值流和异常上下文的可靠性，再交付有界规范化、CanonicalCFG、Frame、stack/local SSA 与指令级异常模型；Frame 区分 Top、初始化别名和每个 throw-site 输入。
- SSA 在私有实现中分开 JVM 语义驱动和名字分配，借鉴 droidsaw 的职责边界与独立小图对照；前驱尚未就绪不能被当成无定义，内部块存储顺序不能改变值来源。当前不引入 droidsaw-common 或跨项目共享 crate，具体取舍见 design 的 6.1。
- 增加固定 Phase/Pass 契约、origin/diagnostic 传播及 Bytecode 输出，分别报告 quality、syntax_status、compile_status、semantic_validation、verification、coverage 和 execution。
- 修正 Frame 的异常状态固定点与 Frame/SSA 派生存储计费缺口，再完成入口计数、golden/fuzz 和整体出口门禁；不把历史测试通过当成新反例已关闭。
- 保持结构 XRef 可独立运行；P2 不承诺 Java 8 高阶源码恢复。

## Capabilities

### New Capabilities

- `demand-resolver`: 在显式运行环境中按需解析声明、查询声明引用，并报告有界 dispatch 候选。
- `jvm-ir`: 从共享 reader 的保真 facts 建立有预算的 raw/canonical CFG、frames、SSA、effects 和异常模型。
- `conservative-output`: 以 Bytecode 表示交付方法分析，独立记录 Conservative/Fallback 质量、未执行验证和未完成范围。

### Modified Capabilities

- `analysis-contracts`：预算契约扩展——P2 1.3 增加六个计费维度与第二个非累加高水位维度 `dependency_depth`，并把这套维度同时写进 `Limits`/`UsageSnapshot`/终止维度枚举与库、CLI 两个请求 schema（显式、缺失即协议错误）。该 delta 只收紧"新增维度必须同时出现在两处并保持显式"这一条，不改变既有维度的语义。

新增能力通过显式运行环境的解析入口和方法分析入口提供；现有 `Engine::query` 保持 physical X0/X1 契约，不因有 resolver 就隐式执行解析。共享 reader 的 IR facts 先作为内部接口，沿用已有物理身份、BCI 和执行语义。若实现需要改变已有能力的可观察行为，先增补对应 spec delta，不以本计划授权未声明的主规格变更，也不为旧接口额外维护兼容层。

## Impact

影响 `jarde-reader` 的共享 facts/budget、`jarde-jvm` 的 resolver/IR/结果模型，以及 `jarde` 门面与薄 JSON CLI；`jarde-query` 保持独立 X0/X1。复用 noak；petgraph 0.8.3 已通过 3.1 准入并用于 raw CFG，沿用 std-only、确定性排序、规模/取消约束，不重新选型或增加推测性依赖。保持 Rust 2024、MSRV 1.88、同步可取消和纯 Rust 生产链，不新增推测性 crate、异步框架或数据库。验收重点为 A09、A10、A11、A13、A14、A16、A17。

可靠处理基线为 Java 8 Runtime Profile 下的 45–52 历史输入；现代语义、Region/Java AST 恢复与缓存分别留给 P3/P4/P5。复核发现的 fuzz 请求路由与独立 workspace 审计缺口作为单独验证维护改动处理，不混入 resolver/IR 实现。
