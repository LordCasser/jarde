## Why

P1 已归档，P0 reader 与 P1 query/view 已有实现和验证证据；P2 尚未开始。2026-09-17 复核发现，现有 reader 未向 IR 提供类型化操作数，预算未覆盖语义闭包与 IR 膨胀，physical query 也没有解析所需的运行环境输入。先补齐这些契约，再建立 Demand Resolver 和 Java 8/历史 classfile 的 JVM IR 管线。复核依据及独立维护项见 design.md。

## What Changes

- 在共享 reader 内补齐类型化操作数和目标边界校验，为依赖闭包、IR 存储、工作列表和子程序克隆增加可取消的预算契约。
- 增加显式绑定 snapshot、RuntimeView、loader domains 和平台 Header providers 的 Demand Resolver，区分 Resolved/Missing/Ambiguous 等状态。
- 增加带运行环境的独立解析入口及声明引用查询，覆盖 A11 的 Base/Sub 关系，避免先按 CP owner 精确过滤导致漏项。
- 增加 raw CFG、jsr/ret 分析与有界规范化、CanonicalCFG、Frame、stack/local SSA、effects 和指令级异常模型。
- 增加固定 Phase/Pass 契约、origin/diagnostic 传播及 Bytecode 输出，分别报告 quality、syntax_status、compile_status、semantic_validation、verification、coverage 和 execution。
- 保持结构 XRef 可独立运行；P2 不承诺 Java 8 高阶源码恢复。

## Capabilities

### New Capabilities

- `demand-resolver`: 在显式运行环境中按需解析声明、查询声明引用，并报告有界 dispatch 候选。
- `jvm-ir`: 从共享 reader 的保真 facts 建立有预算的 raw/canonical CFG、frames、SSA、effects 和异常模型。
- `conservative-output`: 以 Bytecode 表示交付方法分析，独立记录 Conservative/Fallback 质量、未执行验证和未完成范围。

### Modified Capabilities

无。新增能力通过显式运行环境的解析入口和方法分析入口提供；现有 `Engine::query` 保持 physical X0/X1 契约，不因有 resolver 就隐式执行解析。共享 reader 的 IR facts 先作为内部接口，沿用已有物理身份、BCI 和执行语义。若实现需要改变已有能力的可观察行为，先增补对应 spec delta，不以本计划授权未声明的主规格变更，也不为旧接口额外维护兼容层。

## Impact

影响 `jarde` 的 reader 适配、budget、resolver、IR 和结果模型，以及调用相同库入口的薄 JSON CLI。复用 noak；通用图算法优先对既有候选 petgraph 做准入验证，通过后引入。保持 Rust 2024、MSRV 1.88、同步可取消和纯 Rust 生产链，不新增推测性 crate、异步框架或数据库。验收重点为 A09、A10、A11、A13、A14、A16、A17。

可靠处理基线为 Java 8 Runtime Profile 下的 45–52 历史输入；现代语义、Region/Java AST 恢复与缓存分别留给 P3/P4/P5。复核发现的 fuzz 请求路由与独立 workspace 审计缺口作为单独验证维护改动处理，不混入 resolver/IR 实现。
