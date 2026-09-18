## Why

P1 及其验证维护已归档。P2 已交付基础契约、Header resolver/声明引用/已知候选，以及 raw CFG；3.4 有未提交的调用上下文候选，规范化与 Frame/SSA 尚未完成。2026-09-18 review 在 `4beb6b9` 加工作区上确认 loader 传播、returnAddress 值流、异常 locals 与派生存储预算缺口。先修正这些基础行为，再继续 canonical IR；不以既有测试通过代替反例验收。当前状态及证据见 design.md、verification.md。

## What Changes

- 在共享 reader 内补齐类型化操作数和目标边界校验，为依赖闭包、IR 存储、工作列表和子程序克隆增加可取消的预算契约。
- 增加显式绑定 snapshot、RuntimeView、loader domains 和平台 Header providers 的 Demand Resolver，区分 Resolved/Missing/Ambiguous 等状态。
- 增加带运行环境的独立解析入口及声明引用查询，覆盖 A11 的 Base/Sub 关系，避免先按 CP owner 精确过滤导致漏项。
- 在已有 raw CFG 上补齐 wide/数组操作数、returnAddress 值流和异常上下文的可靠性，再交付有界规范化、CanonicalCFG、Frame、stack/local SSA 与指令级异常模型；Frame 区分 Top、初始化别名和每个 throw-site 输入。
- 增加固定 Phase/Pass 契约、origin/diagnostic 传播及 Bytecode 输出，分别报告 quality、syntax_status、compile_status、semantic_validation、verification、coverage 和 execution。
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

影响 `jarde` 的 reader 适配、budget、resolver、IR 和结果模型，以及调用相同库入口的薄 JSON CLI。复用 noak；petgraph 0.8.3 已通过 3.1 准入并用于 raw CFG，沿用 std-only、确定性排序、规模/取消约束，不重新选型或增加推测性依赖。保持 Rust 2024、MSRV 1.88、同步可取消和纯 Rust 生产链，不新增推测性 crate、异步框架或数据库。验收重点为 A09、A10、A11、A13、A14、A16、A17。

可靠处理基线为 Java 8 Runtime Profile 下的 45–52 历史输入；现代语义、Region/Java AST 恢复与缓存分别留给 P3/P4/P5。复核发现的 fuzz 请求路由与独立 workspace 审计缺口作为单独验证维护改动处理，不混入 resolver/IR 实现。
