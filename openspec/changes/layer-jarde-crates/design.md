## Context

动机和前置条件见 [proposal](proposal.md)。截至 2026-09-18，生产包仍为 `jarde` 与 `jarde-cli`；下述是待实施边界，不是现状描述。

源码依赖支持拆分，但不支持按每个术语各造一个 crate：

- `classfile` 依赖 budget/error/model；artifact 与 classfile 共用身份。budget/error 互相引用类型，留在同一底层包即可，不需要依赖注入解环。
- `query` 与 `xref` 互相引用，是同一查询组件。`resolver` 通过 `xref::scan_candidates` 复用候选扫描；查询侧不依赖 resolver/IR。
- `passes` 使用 `ir` 的阶段类型，CFG、call-context 和调度由 `engine::run_method_analysis` 串联。只移动目录而把 driver 留在门面，会迫使整个中端内部结构公开。
- `view` 是声明式 profile/domain/identity 数据；`multi_release` 依赖 artifact/classfile/view，没有 Header resolver 依赖。视图选择不等于符号解析，更不等于 verifier 或源码恢复。

证据入口：`src/lib.rs`、`src/engine.rs`、`src/classfile.rs`、`src/query.rs`、`src/xref/mod.rs`、`src/resolver.rs`、`src/passes.rs`。实施前按最新基线重核文件及私有访问，不能把这份清单当作完整搬迁脚本。

## Goals / Non-Goals

**Goals:**

- 用单向生产依赖约束 reader、纯查询、JVM 语义和门面；只做查询的调用方可以直接依赖查询包，不编译 CFG/SSA/Java 恢复。
- 共用原始 facts、identity、预算与结果基础类型，按生产者所有权组织接口；底层无法回调门面以取得能力。
- 让未实现的 Frame/SSA 和 P3 恢复落在明确归属里；每次搬迁可单独验证行为不变。

**Non-Goals:**

- 不把目录拆分当成算法简化，不保证总代码量下降或编译更快；不在本 change 修正语义缺陷。
- 不新增 `common/core/traits` 杂物包，不把 resolver/CFG/SSA/pass 各拆一包，不建插件系统、通用 ISA backend、全新 session/Engine 框架。
- 不引入 droidsaw、不替换 petgraph/noak，不实现 Java recovery、缓存或并行运行。

## Decisions

### 1. 首轮三个新包，按可独立消费的能力拆

箭头表示生产依赖：

```text
jarde-cli → jarde（统一门面）
              ├─→ jarde-reader
              ├─→ jarde-query → jarde-reader
              └─→ jarde-jvm   → jarde-reader + jarde-query

P3 才增加：jarde → jarde-java → jarde-jvm + jarde-reader
```

| 包 | 归属 | 禁止承担 |
| --- | --- | --- |
| `jarde-reader` | artifact/classfile/model/budget/error/view/multi_release；Header/bytecode inspection 和 source materialization | consumer 查询、Header 闭包解析、CFG、源码恢复 |
| `jarde-query` | query/xref；X0/X1 的编译、候选过滤、consumer 扫描、分页/报告 | resolver、CFG/SSA、Region/Java AST |
| `jarde-jvm` | environment/providers/members/dispatch/resolver；cfg/call_context/passes/ir；方法分析 driver；后续 normalization/Frame/SSA | Java 命名/排版、宿主协议、全局 XRef 预建 |
| `jarde` | 统一 `Engine` 入口和有意设计的类型再导出；委托三包 | pass 循环、Header 闭包实现、指令语义、reader/查询的第二套实现 |
| `jarde-cli` | 参数与 JSON 适配，仅依赖门面取得项目能力 | 另造算法或结果状态 |
| `jarde-java`（P3） | Region、Java recovery AST、命名、source map、输出 | 改写原始 X1 facts 或反向被 JVM 底座依赖 |

`jarde-jvm → jarde-query` 是声明引用查询复用结构 scanner 的真实依赖，并不允许 `analyze_method` 自动进行全局扫描。reader 携带 `RuntimeProfile` 等数据不表示运行环境已经被解析。纯查询调用方选择 `jarde-query`；统一门面有意聚合能力，本轮不为它再建复杂 feature 矩阵。

不另建 model/core 包：目前基础数据、字符串构造、预算和 reader 是共同消费的最小输入层，把它们再拆开暂时只会增加公开 API。reader 的边界固定为输入事实与执行基础契约，不能把后续 semantic/recovery 类型倒灌进去。若将来出现完全不需输入能力的真实独立消费者，再凭依赖证据评估。

### 2. 跨包接缝按生产者收敛，不一键改成 pub

Rust 的 `pub(crate)` 不能跨 workspace 包，不能靠重导出绕过。这是拆分的主要设计工作：

| 接缝 | 处理 |
| --- | --- |
| `ArtifactSnapshot::read_entry_internal` | 改为 reader 拥有的最小有界物化接口，保留快照/entry 校验、读取类别与计费语义；不能把 internal 直接公开成跳过预算/完整性检查的入口 |
| classfile facts、类型化 operands、CP/attribute 读取 | reader 提供最小的类型化 facts API；已验证事实由 reader 产生，内部存储私有或只读访问，不开放伪造“已验证”状态的构造路径。不同 parse/dialect/verification 状态继续区分 |
| `JvmString::from_parts` 等校验相关构造 | 留在 reader 内部；上层消费同一类型，不能再造 lossy 字符串或平行 identity |
| `query::execute`、`xref::scan_candidates` | query 暴露有界执行入口和 resolver 真正需要的 candidate filter/result；不开放 scanner 的全部内部状态，不新增可插拔扫描框架 |
| `xref::with_usage`、artifact 的预算诊断码等共享小工具 | 与基础执行类型相关的放回 reader 的 budget/error/model 所有者；不为几个函数建立工具 crate，也不让 reader 反向依赖 query |
| `ClassTarget/ClassSource`、inspect 报告与 materialize | 从 engine 放到 reader 的检查入口；门面可以再导出，底层不 import engine |
| `analyze_method`、pass ledger/driver 与解析入口 | 将验证、调度和报告装配一起放入 jvm；facade 只委托，不能为维持门面原实现而公开全部可变 CFG/ledger |

原来的模块路径不是架构约束。只保留有产品意义的再导出，仓库内调用方一次迁移；不为后向兼容保留旧算法副本。跨层传递同一个 `&mut Budget` 与现有取消状态，禁止 facade、query、jvm 各开一份新预算，禁止为了跨包多做解析、复制 body 或重新计算身份。

### 3. P3 使用独立恢复包，但现在不造空壳

P2 继续产出 JVM 分析结果，不让 Frame/SSA 依赖 Java 表达能力。P3 `1.3` 随第一个真实 Region→AST→文本闭环创建 `jarde-java`。恢复侧需要的 typed method/CFG/value/effect/origin 视图由 jvm 所有；P3 `1.1` 明确只读访问契约，不能仅凭现在的摘要型 report 假定已能恢复，也不能让 java 读中端私有字段或反向调用统一门面。

不预建“跨 DEX/JVM 的通用 IR 包”。droidsaw 的 SSA 名字分配与指令语义分离、普通控制流与异常事实分离、Region 与 emitter 分离可在各自所属包内实现。具体复用判断见 [P2 design §6.1](../p2-jvm-ir/design.md) 和 [P3 design](../p3-java8-recovery/design.md)；保持自研边界不等于证明自研质量更高。

### 3.5 接缝归属的三个定案（2026-09-18，盘点后）

盘点发现三处 design 未定、实现者会各自猜测的地方，现定案：

- **`blake3` 的使用收敛进 reader**（不按「直接使用」散到四个包）：摘要就是身份的一种，身份的所有者是 reader。reader 提供 `Digest::of(&[u8])` 一类的构造，query/jvm/门面改为调用它，`blake3` 只作为 reader 的直接依赖。理由：否则 `query` 的游标摘要、`providers` 的字节身份、门面的报告摘要会各自实现一遍，正是「平行身份」要避免的。若将来某处确有非身份的散列需求，凭证据再议。
- **`CandidateFilter` 不整体公开**：query 对外只暴露 resolver 真正需要的两种候选形状（成员形状、signature-polymorphic），`Exact` 是 query 的内部语义。用一个只含那两种变体的公开枚举包住内部枚举，而不是给内部枚举加 `#[non_exhaustive]` 后整体导出——后者等于把 scanner 的语义面放开。
- **`read_entry_internal` 随搬迁改名为 `read_entry_for_analysis`**：公开面里不该出现 `_internal` 这样的名字，且它确实是「为分析而读」的入口。改名与升公开面同步进行，并在其文档里写明它保留的三件事（快照/entry 校验、读取类别与计费）。

### 4. 依赖和测试随所有者迁移

- 新包路径使用 `crates/jarde-reader`、`crates/jarde-query`、`crates/jarde-jvm`；根包继续 `jarde`，CLI 维持原位置。锁文件中的第三方版本和 features 不顺手升级。
- noak/rawzip/flate2/blake3 随实际读取用途归入 reader，petgraph 归入 jvm；serde/thiserror 等按直接使用声明，必要的共同版本用 workspace 配置。不以拆包为由降低纯 Rust、MSRV 1.88 和 supply-chain 门槛。
- 单元测试跟随实现，真实门面/CLI 集成测试留在外层。reader/query 测试不能通过 dev-dependency 引入 facade/jvm，使独立验证名存实亡。
- 现有 `classfile::test_class` 之类跨模块测试辅助按消费者盘点；优先复用仓库内测试资源/显式测试支持模块，不把测试 builder 暴露进生产 API，也不默认新增 test-utils crate。
- 同步 fuzz 的 path dependency、独立 lock、fixtures 路径、CI 扫描目录与包选择。A17 既保留运行时“不启动/不读 body”证据，也加入 Cargo 依赖闭包证据；不能因 `src/` 变空而让旧字符串守卫假绿。

## Risks / Trade-offs

- [跨包 API 比原模块可见性更宽] → 明确 facts 生产者与只读消费者，逐项审核必要公开面，不暴露所有字段。
- [拆包导致重复读取、预算重置或报告变化] → 对同一绿色基线重跑 golden、精确预算边界、取消和身份对照；只允许沿用既定 `elapsed_millis` 归一策略。
- [正在修复的 P2 代码与目录搬迁冲突] → 先验收当前修正，再独立迁移；不并行改同一批源文件。
- [包数增加却边界不成立] → 独立 query consumer 的编译/测试和生产依赖检查是退出门槛，不以“workspace 能编译”代替。
- [统一门面仍编译全部 P2 能力] → 这是门面的选择；轻量调用方直接选 query/reader。不声称所有用户都会获得编译收益。
- [公共 crate 发布涉及多个包] → 本 change 不做发布；保持同仓统一维护，未来若发布门面，关联包采用协调发布而非不可发布的 path-only 假设。

## Migration Plan

1. 验收 P2 `0.3/0.3b/3.4` 及其依赖，记录精确基线 commit、实际测试和依赖树；未绿不开始文件搬迁。
2. 搬 reader 及所需检查入口，收敛 facts API；原 query/jvm 暂在根包、改用新 reader。运行 reader 与外层基线回归。
3. 搬 query+xref，暴露最小候选扫描接缝；证明 query 可独立使用，再接回声明引用路径。
4. 搬 jvm 和方法 driver，把根 engine 收窄为委托；同步 examples/CLI/fuzz/CI，不重写算法。
5. 完成下述门槛及只读复核，更新源码路径和验证文档，才继续 P2 `3.5/4.x`。迁移失败时回退本次独立重构提交，不能撤销已验收的语义修复，也不维护两套并行实现。

退出门槛：

- `cargo check/test -p jarde-reader`、`cargo check/test -p jarde-query` 能独立执行；一个只声明 `jarde-query` 的最小 consumer 完成真实查询。
- `cargo tree -p jarde-query --edges normal` 不包含 `jarde-jvm`、`jarde-java`、petgraph 或任何宿主包；reader 无 query/jvm/facade 反向依赖。reader/query 的测试依赖也不能拉入这些项目上层包。
- workspace fmt/clippy（warnings deny）、全部测试、P0 oracle/P1 golden、当前 P2 回归、MSRV、两套 workspace supply-chain 和既有 fuzz replay 门槛通过；执行与跳过项如实记录。
- X0/X1 不启动 resolver/CFG/SSA；单方法仍不读无关 Body；相同输入的身份、BCI、引用数、阶段/coverage/execution、预算使用与库/CLI 报告保持一致。以既有反例为准，不借拆包改 golden 掩盖差异。
