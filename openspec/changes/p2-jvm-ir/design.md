## Context

当前复核基线为 `4beb6b9ce5322a94ff0a0c571ae532d687096523`（2026-09-18）及未提交的 3.4 工作区。1.1–1.3、2.1–2.5、3.1–3.3 有交付记录；3.4 已有候选实现，尚未验收；3.5、4.x、5.x 未完成。原 11/20 项勾选保留为历史交付，本轮新增三项前置修正后为 **11/23**；这不表示已发现的问题被修复。P1 与其验证维护均已归档。

本轮确认四个反例，涉及定义 loader 的层级解析、returnAddress 值来源、异常路径 locals 和调用上下文存储预算；同时修订 wide facts、Pass requires 与 Frame/SSA 设计。当前执行顺序以「Migration Plan」和 tasks 的 0.x 为准，3.4 不得直接交接 3.5。详细输入、实际结果及验证边界见 [verification.md](verification.md) 的「2026-09-18 当前工作区复核」。

以下 P1 复核及各片交付说明保留历史时点；标为契约的段落以本次修订为实施目标，不表示代码已符合新要求。

### P1 复核证据

未发现要求撤销 P1 归档的证据。复核覆盖最终 query 身份、损坏候选、Code/metadata/bootstrap consumers、共享 reader 与 P2 接口边界、CI 和 fuzz harness；不是对全部代码的无缺陷证明。

| 核验 | 本轮结果 |
| --- | --- |
| 本地与远端基线 | HEAD、origin/main、远端 HEAD 同为 `8fcdd66`；复核开始时工作树干净 |
| 三次 CI | [35233298026](https://github.com/LordCasser/jarde/actions/runs/35233298026)、[35233830192](https://github.com/LordCasser/jarde/actions/runs/35233830192)、[35234243136](https://github.com/LordCasser/jarde/actions/runs/35234243136) 均 completed/success，stable、MSRV、supply chain、fuzz smoke 四 job 成功；head SHA 分别匹配 a6bcccb、4f9e48a、8fcdd66 |
| OpenSpec | 修改规划前 `openspec validate --all --strict --no-interactive` 为 10 passed / 0 failed |
| 本轮定向回归 | `cargo test --locked --test p1_query_api --test p1_query_bounds --test p1_xref_code --test p1_xref_metadata --test p1_xref_bootstrap`，分别 23、4、28、33、28 passed，总计 116 passed / 0 failed / 0 ignored，exit 0 |
| 验证边界 | 完整 299 passed / 1 ignored 双种子测试与本地 60 秒 fuzz 沿用 P1 归档记录，本轮未重跑。CI 每 target 为 20 秒，不能写成 60 秒 |

### 单独处理的验证维护项

以下是验证缺口，不是已证明的运行时错误。它们由独立的 [harden-p1-validation](../archive/2026-09-17-harden-p1-validation/tasks.md) 承接，验证通过后再进入本 tasks 的实现；P2 设计先完成。不把维护代码混进 IR 任务，也不重新打开历史归档。

本轮状态：V1/V2 修复和本地正反例验证已完成，见[验证记录](../archive/2026-09-17-harden-p1-validation/verification.md)。维护提交 `555c785`、`acbba49` 已推送，CI run `35238994798` 四个 job success（含双 workspace supply-chain 与修复后的 fuzz smoke）；该 change 已归档，P2 实现前置条件满足。

| 项 | 证据与影响 | 最小修正与退出条件 |
| --- | --- | --- |
| V1：query fuzz 请求与魔数耦合 | `fuzz/fuzz_targets/query.rs:21` 用首字节选择 `query_request`，后者对 5 取模。合法 standalone CLASS 的 `0xCA % 5 = 2`，固定走 LiteralValue；普通 PK ZIP 的 `0x50 % 5 = 0`，固定走 Invocation/Type。种子 minimal-class/minimal-jar 已核对。不能声称其他 shape 对所有特殊 ZIP 都不可达，但合法 CLASS 的变异没有覆盖其余请求形态 | 同一成功 open 的 artifact 顺序执行 5 个固定请求，每次释放前一结果，明确每次预算及最多 5 次的总工作界限。target 和 corpus test 共用驱动；通过真实 CLASS/JAR 断言五种形态均被调用，且还原旧选择器会失败。受损输入继续断言 execution/coverage；重跑 corpus 与 smoke |
| V2：CI 审计不含 fuzz workspace | CI cargo-deny action 只在根执行，根 workspace 不含 `fuzz/`；后者有独立 manifest/lock。本地两个 workspace 均通过不等于 CI 持续覆盖 | CI 显式审计根与 fuzz/Cargo.toml 两个依赖图，共用根 deny.toml；日志标明 manifest/工具版本，四段检查均通过。以临时拒绝规则证明只在 fuzz 图中的依赖也会使门禁失败，不改提交的 lockfile |

NCSA 决策：允许测试专用 `libfuzzer-sys 0.4.13`（本地发布包声明 `(MIT OR Apache-2.0) AND NCSA`），在 V2 中将全局 NCSA allow 收窄为该 crate/版本的例外，保持一个 policy 文件。升级重新审核，未来生产依赖不能自动继承许可例外。机制见 [cargo-deny 官方配置](https://embarkstudios.github.io/cargo-deny/checks/licenses/cfg.html)，实施时核对实际工具版本的语法。无需为此替换 fuzz 引擎。

RSS 决策：暂保留 CI 512 MB 限额。P1 本地 artifact_tree 的 482 MB 峰值接近上限，但没有该候选 CI OOM 的证据。V1 改变单输入工作量后，分别记录 CI 20 秒、本地 60 秒的 target、平台、工具链、sanitizer、语料和 peak RSS，不能直接互换环境的峰值。若出现 OOM，先保留复现输入及分配/计费证据，区分引擎膨胀与 sanitizer 开销，再调整实现或限额，不用缩短运行掩盖问题。

维护后本地 60 秒实测：query 216 MB、artifact_tree 500 MB，均 exit 0；后者距限制仅 12 MB，已记入维护验证记录，不宣称余量充足。

### P2 进入时的接口缺口（历史基线；1.x/2.x 已交付，剩余缺口见 0.x）

| 代码现状 | 处理 |
| --- | --- |
| `src/classfile.rs::InstructionFact` 只有 opcode、BCI、width、spans、CP index，decode_code 已消费 noak 指令事件 | 在同一薄适配保留 local/immediate/branch/switch 等类型化操作数，不能再写一套 decoder |
| `src/budget.rs::Limits` 无语义闭包或 IR 分配/迭代/克隆限额 | 扩展现有 Budget 的维度与计费表，先计费再分配 |
| `src/view.rs::LoadDomain` 只有 parent LoaderId、roots、policy 等声明 | 请求显式绑定 domain 表与不可变 snapshot/Header providers；ID 本身不等于可读取定义 |
| `src/query.rs::QueryRequest` 只有 physical view，resolution 未实现 | 新增显式运行环境的解析入口，不隐式升级 physical query |

## Goals / Non-Goals

**Goals:**

- 先交付有界 Header-only resolver，再逐片交付方法 IR；每片有明确退出条件。
- 在 Java 8 profile 下正确处理 45–52 输入或明确 fallback，保留物理身份及原始 facts。
- 分开报告解析限制、资源终止、质量降级和未执行 verifier。

**Non-Goals:**

- 不实现 Region recovery、Java AST、Java/Mixed 文本生成或 P3 高阶恢复。
- 不实现现代 module/nest/condy 深度语义、闭世界 points-to 或自定义 classloader 执行。
- 不建设缓存、全局索引、动态插件框架或推测性 workspace crate。

## Decisions

### 1. 显式运行环境入口与共享读取

解析请求携带 RuntimeView、平台/provider 绑定、调用方身份、原始符号、访问/指令种类和 budget；方法分析请求携带物理方法身份、同一运行环境与请求阶段。CLI 只翻译参数并调用库。

声明引用查询属于新的 demand-resolver 入口：输入 declaration identity 和范围，复用结构 consumer 读取 use-site，逐个解析并比较声明。`Sub.foo` 不能在解析前因为 owner 不是 Base 被丢弃，raw CP 候选也不能当实际引用。扫描和解析覆盖分别返回，未解析 use-site 不能被当成已排除。初版使用有界结果与显式 Partial，不借用 P1 游标作为运行视图游标，也不预建全局反向继承索引。

`Engine::query` 的 physical X0/X1 保持原契约，已有解析 relation 仍明确 UnsupportedAnalysis；不会隐式采用机器 classpath。P2 只提供基础声明解析与已知范围 dispatch，不提前宣称 P4 深度 X2/X3 完成。

### 2. 先补共享 facts，再构图

复用 noak 0.7.0 指令事件和现有 checked-width 适配，内部 facts 保留 immediate、local、CP 引用、相对 branch、switch key/default/target，以及 raw opcode、字节范围、BCI。不能从展示文本恢复操作数，也不能重新启用已记录存在上界风险的 TablePairs 迭代。

所有 branch/switch/handler target 用 checked 运算并核对指令起点；异常保护区间为半开区间，end 可为 code_length。溢出、跳入操作数或非法目标不得进入 CanonicalCFG。解码成功、版本允许和完整 verification 分开；51+ 禁止 jsr/jsr_w/ret 的约束见 [JVMS 8 §4.9.1](https://docs.oracle.com/javase/specs/jvms/se8/html/jvms-4.html#jvms-4.9.1)。取证展示仍保留原始事实。

### 3. 预算先于闭包与 IR 分配

复用现有 Budget、取消 token、deadline 和 execution，新增计费表覆盖类/Header 数、Body/方法数、依赖深度、IR 存储项、IR 边、分析步骤、规范化克隆。依赖深度独立于容器 nested_depth。IR 存储项包含 frame/locals 槽、SSA 值/phi 输入和 origin 成员，不能只计 block 数而遗漏 dense state 膨胀。

扩展、排队、克隆、分配前 checked 累计计费，重复 worklist 访问消耗步骤预算。单请求对同一物理定义/loader 绑定去重，不合并同内容不同 origin。实际重复读取仍按实际字节计费，不借此引入 P5 缓存。

阶段共用一个请求预算生命周期；fallback 使用已保留的可靠 facts，不 reset budget 重读完整 Body。结果还受已有 ResultItems/OutputBytes 限制。provider 遵守同步协作取消/预算协议；第三方不可中断调用只能承诺前后检查和输入规模界限，不能声称硬实时中止。

### 4. 可复核的 Header providers 与 loader 选择

首片支持显式提供的不可变 CLASS/JAR snapshots、平台 Header 集、Java 8 classpath 和有顺序的 ParentFirst/ChildFirst domains。使用最小 provider 接口与请求内映射，不新增服务容器；不根据宿主 JDK 猜平台，不自动下载依赖。provider 身份绑定输入内容，结果携带 loader、物理定义和选择依据。

有序 roots 的同名定义可以按策略确定选择，不能一律 Ambiguous；同一选择位置有无法区分的定义或顺序不确定时保留歧义。缺少 parent/provider、循环 parent graph、Custom/Unknown policy 或未支持 module mode 不得静默变成扁平 classpath。确定的 Missing、策略不支持、输入损坏和预算/取消分别报告。

按 [JVMS 8 §5.4.3](https://docs.oracle.com/javase/specs/jvms/se8/html/jvms-5.html#jvms-5.4.3) 分字段、class/interface method 和指令种类处理，覆盖访问控制、构造器、invokespecial、default conflict；signature-polymorphic/数组方法建立明确支持或 unsupported 分支。声明解析与 KnownCandidates dispatch 分开，缺失依赖、外部子类和 transformer 保持 open-world。

闭包直接消费 Header，不通过分页 X1 或 Type consumer 猜测全部定义。单方法只读本类/必要依赖 Header 和目标 Body；显式范围 CHA 可枚举该范围全部 Header，仍不读取实现 Body。

### 5. 分层 IR 与指令级异常语义

顺序为 raw facts → raw CFG/returnAddress → bounded legacy normalization → CanonicalCFG → Frame → stack/local SSA 与 type/effect facts。raw CFG 起就保留 throw-site、handler ordinal、保护区间及异常路径状态；采用指令级 throw site 或足够细的 block，不能仅从最后一条指令连异常边。

规范化按调用上下文处理 jsr/ret，保存新节点到原 BCI 的一对多 origin；共享/嵌套子程序、异常覆盖或膨胀超界时明确 fallback。Frame/SSA 处理 category-2、dup/swap、未初始化对象、handler entry、数组/null/未知引用合流和正常/异常 predecessor 的 phi；矛盾输入不能猜测栈形状。

OriginSet 锚定物理 class/method 与 class offset/BCI，不把有口径债务的通用 Entry.span 当 Code 坐标。规范化不覆盖原始 facts，也不改变 P1 XRef 次数。PassDescriptor 用固定 phases 和最小静态描述表检查依赖/环、required/produced facts、dialect/scope/budget 与 invalidation；不建立任意调度框架。Region 是后续消费方，P2 不构建 RegionIR。

### 6. 通用库准入先行

延续 `../../dependencies.md` 的 noak 与 petgraph 候选。petgraph 0.8.3 的容器、SCC/支配算法先验证 Rust 1.88、许可/纯 Rust feature tree、确定性输出、平行 normal/exception edges、不可达节点、自环、多出口和预算。稳定输出按物理/BCI 身份排序，不依赖 hash 顺序；算法适用条件核对[官方 dominators 文档](https://docs.rs/petgraph/0.8.3/petgraph/algo/dominators/fn.simple_fast.html)。

通过后引入依赖，并在同一提交中只移除 CI normal-tree 对 petgraph 的阶段性禁令，保留 JVM、网络、数据库和异步依赖边界。依赖进入生产图不意味着 X1 可以构图。缺少协作取消接口的算法要记录规模界限和取消延迟；确有不满足契约的证据再评估其他库或局部实现。JVM frame、returnAddress、异常/effect 属于项目语义适配，不能用通用图算法代替，也不因此自写整套图算法。

### 7. Bytecode 是 P2 的输出基线

P2 实际只产生 `representation=Bytecode`、`syntax_status=NotJava`、`compile_status=NotAttempted`；Java/Mixed、Structured 和源码语法检查留给 P3。按证据记录 Conservative/Fallback、LocalInvariants/FixtureDifferential/Unproven，完整 verifier 未实现时始终 `verification=NotPerformed`。

复用已有 `CoverageState::CompleteWithinSchema` 并绑定请求阶段/范围/schema，移除旧规划未定义的 CompleteWithinScope。原始 bytecode 完整但规范化不支持时，bytecode 结构覆盖可完整，IR 阶段明确未完成；预算、取消、损坏造成的未读范围仍 Partial。不能把质量当 execution，也不能把 bytecode 完整当 SSA 完整。abstract/native 的无 Body 状态按声明返回，不伪造空 Code。

## 公共 schema 骨架（1.1，P2 新公共类型的唯一契约来源）

以下类型是 P2 第一片交付的公共 API 契约。字段、variant 与 serde tag 由本节固定；新增 variant 或改字段必须先改本节与 `specs/demand-resolver`、`specs/jvm-ir`、`specs/conservative-output`。1.1 只交付可编译的最小 API、示例与反例测试，不实现解析、闭包、CFG 或 SSA；1.3 负责把预算维度真正加入 `Limits`/`UsageSnapshot` 与计费表；2.x–5.x 填充报告内容；实现后用只读复核确认本节与代码一致。

### 模块与依赖方向

| 文件 | 内容 | 依赖（实际 import；禁止项见下） |
| --- | --- | --- |
| `src/model.rs` | `OriginSet`/`OriginMember`（共享身份层，与 P1 把 `PhysicalDefinitionId` 加进 model 同一先例） | 既有 `model` 内部依赖，无新增边 |
| `src/environment.rs`（新） | 运行环境绑定、provider 声明、环境身份与环境问题码 | `artifact`、`error`、`model`、`view` |
| `src/resolver.rs`（新） | 解析请求/报告、声明引用查询请求/报告 | 上述 + `budget` + `query::{ConsumerKind, ConsumerSchema, XrefOperation}` 词汇表 + `xref::{scan_candidates, with_usage}`（2.4 起） |
| `src/providers.rs`（2.1/2.2 起，crate-private） | 有效搜索序、位置展开、候选匹配与读取、按需 Header 闭包与 `WalkGaps` | `artifact`、`budget`、`classfile`、`environment`、`error`、`model`、`view`（不引用 `resolver`，报告装配留在 resolver） |
| `src/members.rs`（2.3 起，crate-private） | JVMS 5.4.3 三条搜索路径、maximally-specific 集合、访问与调用种类规则 | `providers`、`classfile`、`error`、`model`（用 crate-private 的 `MemberUse` 镜像避免依赖 `resolver`） |
| `src/dispatch.rs`（2.5 起，crate-private） | 范围枚举（P1 scope 词汇）、CHA-lite 候选发现、open-world 事实分类 | `providers`、`artifact`、`environment`、`error`、`model`、`view` |
| `src/passes.rs`（3.2 已交付，crate-private） | pass 描述表与启动校验（前置/invalidation/顺序） | `ir`（阶段与事实词汇）、`error` |
| `src/cfg.rs`（3.3 已交付，crate-private） | raw CFG、指令级 throw site、handler 顺序、effect facts；0.2 将修正 wide 分类 | `classfile`（1.2 的操作数事实）、`passes`、`budget`、`error`、`model`、`petgraph`（准入见 3.1；**不得**被 `query`/`xref` 引用） |
| `src/call_context.rs`（3.4 工作区候选，crate-private） | returnAddress/调用上下文；当前候选须先完成 0.3/3.4 修正 | `cfg`、`classfile`、`budget`、`error` |
| `src/frames.rs`、`src/ssa.rs`（4.x 起计划，crate-private） | descriptor 驱动的 Frame、未初始化值合流、stack/local SSA 与 phi | `cfg`、`passes`、`budget`、`error`、`model` |
| `src/ir.rs`（新） | 方法分析请求/报告、阶段与产物状态 | `artifact`、`budget`、`environment`、`classfile`（`VerificationStatus`）、`error`、`model`、`resolver`（`HeaderRead`/`ReadReason` 词汇，2.2 起）、`view` |
| `src/engine.rs` | 三个薄委托入口 | 上述 |
| `src/query.rs`、`src/xref/**` | **不得**引用 `environment`/`resolver`/`ir`，也不得经 crate 根 re-export 的路径引用 P2 类型（A17：physical X0/X1 不启动 resolver/IR） | — |

1.1 允许的 additive 公共项（不是新类型，故不再单列契约）：`validate_environment`、`EnvironmentProblemCode::{ALL, as_str}`、`ReferenceUse::matches_symbol`、`AnalysisStage::ALL`、`OriginSet::{insert, is_empty}`、`Coverage::not_requested`、`CountedBudgetDimension::ALL`、`Limits::counted_limit`、`UsageSnapshot::counted_usage`。任何 `ALL` 列表必须配一个测试期穷尽 `match`，使“新增 variant 漏加 ALL”编译失败（1.3 的零计费断言依赖它）。

类型级依赖 `resolver → query` 只借用词汇表；反向依赖（`query`/`xref` 引用 P2 模块）被禁止，且由源码级守卫测试检查。

### 环境模型（1.1 固定语义，2.1 实现 provider 读取）

- **搜索顺序归 domain**：`LoadDomain.roots`（P1 已有、顺序参与身份）是该 loader 唯一的顺序来源；provider 不携带顺序、优先权或 delegation。
- **可读内容由入口提供的不可变 snapshot 给出**：`LoadRoot::Snapshot{snapshot}` / `ArtifactTree` 的字节按 `SnapshotId` 在入口参数 `content` 中匹配；匹配不到 → `ContentNotProvided`，不猜测、不联网、不用宿主 classpath。
- **provider 是命名声明**：`HeaderProvider.roots` 的每个 root 必须等于某个参与 domain 的 `roots` 条目（等值校验），否则 `ProviderRootUnbound`；选择依据以 loader + root 序号 + provider id 记录。
- **`LoadRoot::External{id}` 只表示"声明但不可读"**：使相关解析为 `Missing` 或未决，绝不当作可读定义或扁平 root 列表。
- **调用方 domain 唯一**：`runtime.load_domain` 必须在 `domains` 中有且仅有一个 loader 相等的条目，且两者全等；缺失或重复都记 `DuplicateLoader`（message 区分两种），不全等记 `CallerDomainMismatch`。
- **调用方 loader 一致**：`CallerContext.loader` 必须等于 `runtime.load_domain.loader`（两者都声称调用方身份，不允许静默分叉），否则记 `CallerLoaderMismatch` 且不产生任何解析结果。
- **校验范围就是 domain 表**：`runtime.load_domain` 自身没有全等条目时只报上述问题，不再校验它自己的 roots/policy（1.1 有意如此；2.1 若把它当作唯一拒绝入口须扩展）。`RuntimeProfile` 的 profile 能力（release/multi-release/layout）与 `LoadDomain` 的 `external_override`/`runtime_transformation` 判定归 2.x，1.1 不报 `UnsupportedPolicy`。`ProviderId` 允许重复（1.1 无唯一性规则）。
- **不去重合并同内容不同绑定**：去重键是 (物理定义, loader)；同一 snapshot 被多个 loader 引用时分别处理。

### 类型骨架

```rust
// src/environment.rs
pub struct ProviderId(pub String);
/// 可读内容的命名声明；顺序仍由 domain 决定，provider 不携带 delegation 或优先权。
pub struct HeaderProvider { pub id: ProviderId, pub roots: Vec<LoadRoot> }
pub struct ResolutionEnvironment {
    pub runtime: RuntimeView,            // 身份：physical view + profile + 调用方 LoadDomain
    pub domains: Vec<LoadDomain>,        // loader 唯一；含且仅含一个与 runtime.load_domain 全等的条目
    pub providers: Vec<HeaderProvider>,  // 可空；空 = 没有额外内容来源，不代表可以猜测
}
pub struct CallerContext {
    pub loader: LoaderId,
    pub enclosing: Option<PhysicalMethodId>,   // 调用点所在方法；声明查询可以为 None
}
#[derive(...)] pub enum EnvironmentProblemCode {   // 闭集，序列化为 snake_case
    DuplicateLoader, CallerDomainMismatch, CallerLoaderMismatch, MissingParent, ParentCycle,
    UnsupportedPolicy, UnreadableRoot, ContentNotProvided, ProviderRootUnbound,
}
pub enum EnvironmentSubject { Loader(LoaderId), Provider(ProviderId), Root { loader: LoaderId, index: u32 }, Symbol(SymbolRef) }
pub struct EnvironmentProblem { pub code: EnvironmentProblemCode, pub subject: EnvironmentSubject, pub message: String }
pub struct EnvironmentIdentity {         // 报告回指的环境身份（摘要/hash 留到确有消费者时再加）
    pub runtime: RuntimeView,
    pub domain_loaders: Vec<LoaderId>,   // 声明顺序
    pub providers: Vec<ProviderId>,      // 声明顺序
    pub content: Vec<SnapshotId>,        // 入口实际提供的 snapshot
}

// src/resolver.rs
pub enum ReferenceUse {                  // 解析规则的输入：访问/调用种类（不复用 XrefOperation 输出词表）
    ClassReference,
    FieldRead, FieldWrite,
    InvokeStatic, InvokeSpecial, InvokeVirtual, InvokeInterface, InvokeDynamic,
}
pub struct DispatchScope { pub scope: PhysicalScope, pub consumers: ConsumerSchema }
pub struct ResolutionRequest {
    pub environment: ResolutionEnvironment,
    pub target: SymbolRef,               // 原始符号字节，不归一化
    pub use_kind: ReferenceUse,
    pub caller: CallerContext,
    pub dispatch: Option<DispatchScope>, // 显式范围时同时请求 KnownCandidates
}
pub enum ResolutionAnalysis { Performed, NotPerformed }
pub enum ResolutionState {               // 语义判定；不含 NotPerformed/Cancelled（见不变量 3）
    Resolved, Missing, Ambiguous, Inaccessible, IncompatibleClassChange,
    UnsupportedPolicy, BudgetExceeded,
}
pub struct ResolvedMemberRef {           // 2.3/2.5 扩展必须先改本节
    pub loader: LoaderId,
    pub definition: PhysicalDefinitionId,
    /// `SymbolRef` 的 owner 是声明类内部名（可与引用 owner 不同），name/descriptor 为原始字节。
    pub member: SymbolRef,
}
pub enum OpenWorldEvidence { ExternalSubclass, UnknownLoader, RuntimeTransformation, MissingDependency, OrderedRoot { index: u32 } }
pub struct DispatchCandidate { pub member: ResolvedMemberRef, pub evidence: Option<OpenWorldEvidence> }  // None = 无开放性事实（例如来自第 0 个 root）
pub struct DispatchReport { pub scope: PhysicalScope, pub candidates: Vec<DispatchCandidate>, pub open_world: bool }
pub struct ResolutionReport {
    pub environment_identity: EnvironmentIdentity,
    pub environment_problems: Vec<EnvironmentProblem>,
    pub target: SymbolRef,
    pub use_kind: ReferenceUse,
    pub caller: CallerContext,
    pub analysis: ResolutionAnalysis,
    pub state: Option<ResolutionState>,        // None ⇔ analysis == NotPerformed
    pub resolved: Option<ResolvedMemberRef>,
    pub candidates: Vec<ResolvedMemberRef>,    // 仅"同一选择位置无法区分"的 Ambiguous
    pub dispatch: Option<DispatchReport>,
    pub coverage: Coverage,
    pub execution: ExecutionReport,
    pub diagnostics: Vec<Diagnostic>,          // 与环境问题同 code 的稳定诊断
}
pub struct DeclarationRefQuery {         // 2.4 实现
    pub environment: ResolutionEnvironment,
    pub declaration: ResolvedMemberRef,
    pub scope: PhysicalScope,
    pub consumers: ConsumerSchema,
    pub max_items: u64,                  // 0 = 不设条目上限；不使用 P1 游标做运行视图游标
}
pub struct DeclarationRefItem {          // 逐项必有解析尝试
    pub referenced: SymbolRef,           // use-site 上的原始符号
    pub consumer: ConsumerKind,
    pub operation: XrefOperation,
    pub origin: OriginSet,               // 物理 use-site
    pub resolved: Option<ResolvedMemberRef>,
    pub state: ResolutionState,
}
pub struct DeclarationRefReport {
    pub environment_identity: EnvironmentIdentity,
    pub environment_problems: Vec<EnvironmentProblem>,
    pub declaration: ResolvedMemberRef,
    pub scope: PhysicalScope,
    pub consumers: ConsumerSchema,
    pub unsupported_categories: Vec<ConsumerKind>,
    pub analysis: ResolutionAnalysis,
    pub items: Vec<DeclarationRefItem>,
    pub unresolved_candidates: u64,      // 依赖缺失或预算停止造成的未决候选，不得当已排除
    pub has_more: bool,
    pub returned_items: u64,
    pub coverage: Coverage,
    pub execution: ExecutionReport,
    pub diagnostics: Vec<Diagnostic>,
}

// src/ir.rs
pub enum AnalysisStage {                 // 声明顺序即固定 phase 顺序
    RawFacts, RawCfg, LegacyNormalization, CanonicalCfg, Frame, Ssa,
}
pub enum StageState { NotRequested, NotPerformed, Completed, Partial, Failed { code: String } }
pub struct StageResult { pub stage: AnalysisStage, pub state: StageState }
pub enum Representation { Bytecode }     // P2 只产生 Bytecode；Java/Mixed 属 P3
pub enum Quality { Conservative, Fallback }              // 判定规则在 3.5/5.1 落地前必须在此钉死；
                                        // analysis=NotPerformed 时无产物，取 Fallback 只表示非 Conservative，
                                        // 不得据此推断做过降级恢复
pub enum SyntaxStatus { NotJava }
pub enum CompileStatus { NotAttempted }
pub enum SemanticValidation { LocalInvariants, FixtureDifferential, Unproven }  // 报告记录最强适用证据
pub enum NoBodyKind { Abstract, Native }
pub enum MethodBodyState {
    NotInspected,                                  // 尚未定位/读取 Body（本切片即此状态）
    Present,
    DeclaredWithoutBody { no_body_kind: NoBodyKind },   // 内部 tag 与字段不得同名，见不变量 7
}
pub struct MethodAnalysisRequest {
    pub environment: ResolutionEnvironment,
    pub method: PhysicalMethodId,
    pub stages: Vec<AnalysisStage>,      // 期望集合；引擎按固定 phase 顺序规范化、去重并补齐前置
}
pub struct MethodAnalysisReport {
    pub environment_identity: EnvironmentIdentity,
    pub environment_problems: Vec<EnvironmentProblem>,
    pub method: PhysicalMethodId,
    pub loader: LoaderId,
    pub body: MethodBodyState,
    pub representation: Representation,
    pub quality: Quality,
    pub syntax_status: SyntaxStatus,
    pub compile_status: CompileStatus,
    pub semantic_validation: SemanticValidation,
    pub verification: VerificationStatus,       // 复用 classfile::VerificationStatus::NotPerformed
    pub requested_stages: Vec<AnalysisStage>,   // 规范化后的请求集合
    pub stages: Vec<StageResult>,               // 实际执行的阶段（含补齐的前置），phase 顺序
    pub origin: OriginSet,
    pub coverage: Coverage,
    pub execution: ExecutionReport,
    pub diagnostics: Vec<Diagnostic>,
}

// src/model.rs —— 共享 origin 身份
pub struct OriginSet { pub members: Vec<OriginMember> }     // 生成顺序即声明顺序，按等值去重
pub enum OriginMember {
    ClassFile { definition: PhysicalDefinitionId },
    ClassRange { definition: PhysicalDefinitionId, span: ByteSpan },   // span 一律是 class 文件坐标
    MethodPoint { method: PhysicalMethodId, bci: u32 },                // 与 Location::Code 同义
}
```

### 必须保持的不变量

1. 解析与方法分析只能在显式绑定 `ResolutionEnvironment` 且入口提供 `content` 时启动；P1 physical X0/X1 没有这些字段，因此永不启动 resolver/IR，也不隐式采用宿主 classpath（A17）。
2. 环境校验失败不产生唯一解析结果：domain 唯一性、父可解析、parent 无环、caller domain 全等、provider root 归属于某 domain、内容可提供、未支持 policy 与不可读 `External` 各给 `EnvironmentProblem` + 同 code 诊断（环境诊断按问题顺序排在能力码诊断之前、severity 为 `Error`），并保留原始符号与未完成范围。
3. 平面分离：`analysis`（能力是否运行）、`state`（语义判定）、`coverage`（范围）、`execution`（终止）、产物状态（representation/quality/syntax_status/compile_status/semantic_validation/verification）互不推断。`state = Some(v)` **当且仅当**本次运行到达了一个语义判定；`state = None` 覆盖两种情形：能力根本没运行（`analysis = NotPerformed`），或运行了但未及判定就停止（`analysis = Performed` + `execution` 为 `Cancelled`/`Failed`/`Partial`）。因此取消是 `state = None` + `execution = Cancelled`，输入损坏是 `state = None` + `execution = Failed{Error{code}}` + 带 origin 的诊断（与 `specs/demand-resolver` 的"另行记录输入损坏、取消和实际 execution"一致，不占用语义状态）；预算停止有判定，用 `state = BudgetExceeded` 并同时进 execution。`Inaccessible`/`IncompatibleClassChange` **保留给访问与链接规则**（2.3/2.5），不得用来表示读取失败。
4. `OriginSet` 只锚定物理 class/method 与 class offset/BCI；`MethodPoint` 与 `Location::Code` 同义但独立类型，不使用有口径债务的 `Location::Entry.span` 作为 Code 坐标；规范化产生一对多 origin 时保留全部原始 BCI。
5. 一个请求共享一个 `Budget` 生命周期；计费先于分配/排队/加边/克隆；fallback 不 reset、不重读完整 Body；`DependencyDepth` 与容器 `NestedDepth` 相互独立。
6. 未实现、不支持、缺失依赖、预算停止、取消与输入损坏分别用 `analysis`/`state`/`environment_problems`/`diagnostics`/`execution` 表达；不得 panic、返回空结果或伪造唯一解析。
7. serde：unit-only 枚举 → snake_case 字符串；带载荷枚举 → `tag = "kind"`；请求/身份/成员类型加 `deny_unknown_fields`（与 P1 一致）。两处已核实的例外：`MethodBodyState::DeclaredWithoutBody` 的内部 tag 与载荷字段不得同名，线格式固定为 `{"kind":"declared_without_body","no_body_kind":"abstract|native"}`；`EnvironmentSubject` 用外部标记（`{"loader":…}` / `{"provider":…}` / `{"root":{"loader":…,"index":…}}` / `{"symbol":{…}}`），因为 `tag = "kind"` 无法序列化字符串载荷的 newtype variant。带载荷枚举一律**不加** `deny_unknown_fields`（与 `model` 的既有枚举一致）；`{"kind":"declared_without_body","kind":"native"}` 被拒是 serde 的重复字段错误，与此无关。
8. `MethodBodyState` 只描述 Body 是否被定位/读取：未读（含预算/取消/未到该阶段）一律 `NotInspected`；Body 已读取则为 `Present` 或 `DeclaredWithoutBody{..}`，此后阶段未请求或未完成只由 `stages`/`analysis` 表达，不改变 `body`。`representation`/`syntax_status`/`compile_status`/`verification` 是本阶段的能力基线，不是"已完成"的声明：是否运行由 `analysis`/`stages` 表达，因此 `body = NotInspected` 与 `representation = Bytecode` 可以并存。
9. `MethodAnalysisReport.loader` 是**调用方运行 domain 的 loader**（方法定义所在 loader 属解析结果，不由该字段声称）；`MethodAnalysisReport` 没有 `analysis` 字段，能力是否运行只由 `stages` 表达；`stages` 只列**已调度**阶段，未调度阶段不产生 `NotRequested` 条目（1.1 中 `StageState::NotRequested` 不可达）。
10. `max_items` 截断与 P1 页限同一映射（coverage `Partial`、execution 仍 `Complete`、`has_more = true`），但不复用 P1 游标、不承诺续页等价。
11. P2 的公共 IR 面是状态、覆盖与诊断；IR 载荷（raw facts/CFG/Frame/SSA 值）在 P2 保持 crate-private，5.1 若需要公开计数会先在本节加类型。
12. `Engine::query` 的关系语义在 P2 不变：`references_definition`/`may_dispatch_to` 仍返回 `UnsupportedAnalysis`。把它们接到 resolver 需要先改 `query-api` 主规格与 `QueryResolution`，不属 P2 默认范围。
13. P2 的 fuzz/性质不变量不得照抄 P1 的蕴含式（P1 要求 `CompleteWithinSchema ⇒ execution Complete ∧ skipped 空`）；P2 允许"bytecode 覆盖完整 + 某 IR 阶段 `NotPerformed`/`Failed`"，5.3 必须按各平面分别断言。

### 1.1 的入口与诚实不可用状态

三个薄委托入口（命名固定）：

```rust
impl Engine {
    pub fn resolve_symbol(&self, content: &[ArtifactSnapshot], request: &ResolutionRequest, budget: &mut Budget) -> Result<ResolutionReport>;
    pub fn declaration_references(&self, content: &[ArtifactSnapshot], query: &DeclarationRefQuery, budget: &mut Budget) -> Result<DeclarationRefReport>;
    pub fn analyze_method(&self, content: &[ArtifactSnapshot], request: &MethodAnalysisRequest, budget: &mut Budget) -> Result<MethodAnalysisReport>;
}
```

- 请求级失配返回 `Err(Error::invalid_input(..))`：`resolution_snapshot_mismatch`（`runtime.physical.snapshot` 不在 `content` 中）、`resolution_target_use_mismatch`（`SymbolRef` 与 `ReferenceUse` 不自洽，例如 `ClassReference` 配方法符号）、`analysis_no_stages`（空 `stages`）。**环境问题不是 `Err`**，它们进 `environment_problems` 与报告。
- 1.1 对合法请求的诚实状态：`analysis = NotPerformed`、`state = None`（解析）或 `stages` 全 `NotPerformed`（分析）、`execution = Failed { reason: Unsupported { code: "resolution_not_implemented" } }`（或分析路径上的 `"method_analysis_not_implemented"`）+ 同 code 诊断、三维 `coverage = NotRequested`、所有 counted 维度 usage 为 0（`elapsed_millis` 除外）。**该语义在 3.3 起收窄**：解析侧的 `resolution_not_implemented` 仍用于被拒环境与 class 符号的声明查询；分析侧的 `method_analysis_not_implemented` **只在环境被拒时出现**（合法请求已按 pass 表真跑方法分析），它随 5.1 的完整接通彻底消失。code 用能力名，2.x/3.x 落地后消失，相关测试随之退役。
- 1.1 的三个入口**不轮询 Budget**（不启动任何工作），因此预先取消的 token 也会返回 `Failed{Unsupported{…}}` 而不是 `Cancelled`；`Cancelled` 语义从 2.x 起才有承载者，5.3 的性质不得对 1.1 写“取消 ⇒ Cancelled”的蕴含式。
- A17 的可证伪检查：源码级守卫测试断言 `src/query.rs` 与 `src/xref/**`（目录枚举，含新增文件）不含 `crate::environment`/`crate::resolver`/`crate::ir` 记号，也不含经 crate 根 re-export 的 P2 类型名；匹配前归一化 `::` 两侧空白；P2 类型名单必须由 `src/environment.rs`/`resolver.rs`/`ir.rs` 的公开类型**推导并自证完整**（漏一个就失败），而不是手写清单。该守卫是 token 级近似（注释里复述不变量也会命中；别名、raw identifier、`#[path]` 包含等拼写层面不保证覆盖），A17 的构造级证据由 5.2 的构造计数补强。
- 1.1 的正例只证明各平面**可以分别取值**（同一报告里各取不同值、互不推断），不证明跨取值组合的语义；跨取值由 5.x 的阶段结果补强。

### 1.3 负责的预算维度（名字固定，字段由 1.3 加入）

`CountedBudgetDimension` 增 `ClassHeaders`、`MethodBodies`、`IrItems`、`IrEdges`、`AnalysisSteps`、`NormalizationClones`；`BudgetDimension` 另增 `DependencyDepth`（高水位，不累加，不得用 `NestedDepth` 代替）。1.1 先补 `Limits`/`UsageSnapshot` 的构造器或分维访问器，使后续加维是增量；1.3 在同一变更内更新 CLI 请求 schema、goldens 与 fuzz 的 usage 断言，并说明这是 P1 证据的 additive 变化。计数单位（1.3 的实现契约）：

| 维度 | 计数单位 | 计费时机 |
| --- | --- | --- |
| `ClassHeaders` | 一次 Header 读取**尝试**（同一 (definition, loader) 绑定在同一请求内去重后不再计；失败尝试计一次） | 读取前 |
| `MethodBodies` | 一次 Body（`Code`）读取尝试（同上；无 Body 的成员不尝试、不计） | 读取前 |
| `IrItems` | 一个派生存储项：frame/local 槽、SSA 值、phi 输入、origin 成员各计 1 | 分配/入队前 |
| `IrEdges` | 一条派生边：CFG 边（含异常边）、SSA def-use 边各计 1 | 加边前 |
| `AnalysisSteps` | 工作列表的一次 pop/处理（重复访问计数） | 处理前 |
| `NormalizationClones` | `jsr/ret` 规范化产生的一个克隆节点 | 克隆前 |
| `DependencyDepth` | 依赖闭包深度的**高水位**（非累加，与容器 `NestedDepth` 独立），进 limits/usage/终止维度 | 每次扩展前比较 |

去重后的 (definition, loader) 绑定由结果身份表达，不影响计费口径；预算在一个请求内共享一个生命周期，fallback 不 reset、不重读完整 Body。

1.3 的 churn 已量化，必须在同一变更内一起更新（漏掉任何一项都算未完成）：

| 位置 | 规模（2026-09-18 实测） | 要求的改法 |
| --- | --- | --- |
| `src/budget.rs` | `CountedBudgetDimension` 9 → 15、`BudgetDimension` 11 → 18、`try_from`、`Limits::counted_limit`/`get`、`UsageSnapshot::counted_usage`/`add`、`Budget::check_nested_depth` 旁新增高水位入口 | 生产改动即计费契约本身 |
| `Limits { .. }` 字面量 | **73 处 / 24 个文件**（`src` 4、`tests` 14、`crates` 3、`fuzz` 1、`examples` 2），其中需改字段的结构体字面量 **42 处**（41 个 `Limits {` + CLI `From` 里的 1 个 `Self { }`） | 新增 `impl Default for Limits`（**全零，fail-closed**，文档写明"这是测试/工具的基底，不是隐式生产限额"）：P1 测试不回填新维度，各处以 `..Limits::default()` 收尾（一行改动），将来再加维只需改 `Default`；只有真正使用新维度的调用方显式赋值 |
| 高水位维度入口 | `Budget::check_nested_depth` | 同形新增 `Budget::observe_dependency_depth(depth)`（比较后取 max、超限报 `BudgetDimension::DependencyDepth` 并保留 `consumed = depth-1`），两个高水位维度共用同一模式但互不影响 |
| CLI 请求 schema | `crates/jarde-cli/src/main.rs::RequestLimits`（`deny_unknown_fields`、全字段必填） | 新维度**同样必填**（不引入静默默认值；CLI 会先接受、到 5.1 才使用），并同步 `From<RequestLimits> for Limits`；这是 CLI JSON 契约的**有意变更**，要写进 verification |
| P1 golden | `tests/fixtures/p1-golden/*.json` 共 **24 个 usage 对象**（4/4/5/10/1） | 手工补齐新字段（golden 是 checked-in 期望，不得自动生成） |
| fuzz harness | `fuzz/src/lib.rs::assert_usage` 逐维手写断言（加维不会编译失败 → 静默漏检） | 改为按 `CountedBudgetDimension::ALL` 遍历，使新增维度自动纳入上限断言 |
| 其余逐维断言 | `tests/p1_query_bounds.rs::assert_within` 同类手写清单；`src/artifact.rs::budget_dimension_code` 的 7 个新诊断码字符串无锚定 | 两处都改为 `ALL` 驱动或加 serde 名对照断言（同一「加维不得静默漏检」纪律） |
| 文档 | `docs/support-matrix.md` 的"十一项 limit"、README 的 limit 列表 | 更新为 18 项并说明哪些由 P2 使用 |

`ALL` 的测试期穷尽 `match`（1.1 已为 `CountedBudgetDimension`/`EnvironmentProblemCode`/`AnalysisStage` 建立）必须扩到全部新增维度；`DependencyDepth` 与容器 `NestedDepth` 同为高水位，二者独立。

### 1.1 的验收义务

- 新增可编译的最小 API 与至少一个 `examples/` 入口，构造并打印解析请求与方法分析请求的诚实结果；
- 反例：缺少 parent domain、parent 环、caller domain 不一致、provider root 未归属、内容未提供、未支持 module mode/loader policy → `EnvironmentProblem` + 同 code 诊断、无唯一解析结果、usage 的 counted 维度全零；
- 反例（A17）：`Engine::query` 的 physical X0/X1 不启动 resolver/IR（源码级守卫 + 未接线）；
- 正例：`representation=Bytecode`、`syntax_status=NotJava`、`compile_status=NotAttempted`、`verification=NotPerformed`、`quality`、`coverage`、`execution` 在同一报告里分别取值且互不推断。

## 1.2 契约：类型化操作数与目标校验（reader 层，保持 crate-private）

1.1 的类型化操作数只在 reader 层，**不进入公共 schema**：`classfile::InstructionFact` 与公共 `inspect_method_bytecode` 的输出保持不变（P0/P1 契约不动，golden/CLI 无需改），新增事实只在 `pub(crate)` 范围内。

- **同一薄适配、不另建 decoder**：在现有 `method_code_facts` 的 noak 指令事件 walk 里同时产出 `pub(crate) struct InstructionOperands`，并与 `instructions` 同序同长地放进 `MethodCodeFacts.operands`：

  ```rust
  pub(crate) struct InstructionOperands {
      pub(crate) immediate: Option<ImmediateValue>,        // bipush/sipush/ldc 族/iconst… 的常量
      pub(crate) local: Option<LocalOperand>,              // { index: u16, wide: bool }
      pub(crate) increment: Option<i32>,                   // iinc 的有符号增量
      pub(crate) constant_pool_index: Option<u16>,         // 与 InstructionFact 同值
      pub(crate) branch_offset: Option<i32>,               // 相对分支偏移（编码值）
      pub(crate) switch: Option<SwitchOperands>,           // tableswitch/lookupswitch
      pub(crate) effective_opcode: u8,                     // wide 时是被包裹的 opcode，否则等于原始 opcode（0.2）
      pub(crate) atype: Option<u8>,                        // newarray 的元素类型码（0.2）
      pub(crate) dimensions: Option<u8>,                   // multianewarray 的维数（0.2）
      pub(crate) interface_count: Option<u8>,              // invokeinterface 编码的 count（0.2）
  }
  pub(crate) enum ImmediateValue { Int(i32), Long(i64), Float(u32), Double(u64) }   // 浮点用位模式
  pub(crate) enum SwitchOperands {
      Table { default_offset: i32, low: i32, high: i32, offsets: Vec<i32> },
      Lookup { default_offset: i32, pairs: Vec<(i32, i32)> },   // (match, offset)
  }
  ```

  不得从展示文本恢复操作数，也不得靠再次遍历字节流重建语义；宽度与既有 `instruction_width` 口径保持一致。
- **保留原始宽度**：操作数按 JVMS 宽度解码（wide 前缀、iinc、ldc/ldc_w/ldc2_w、bipush/sipush、invokeinterface 的 count、multianewarray 的 dimensions 等），所有 `u16/u32` 转换用 checked 运算；越界或形状不符返回带 BCI 的结构化错误，不产生部分操作数。
- **目标校验独立于 CFG**：`MethodCodeFacts::control_flow_targets(&self) -> Result<Vec<ControlFlowTarget>>`，以同一方法的指令起点集合为准，把相对 branch/switch offset 换算成绝对 BCI（checked），逐条产出：

  ```rust
  pub(crate) struct ControlFlowTarget {
      pub(crate) instruction_bci: u32,   // 发出该目标的指令
      pub(crate) kind: ControlFlowTargetKind,
      pub(crate) target_bci: u32,
  }
  pub(crate) enum ControlFlowTargetKind {
      Branch { offset: i32 },
      SwitchDefault,
      SwitchCase { index: u32, key: i32 },   // tableswitch 的 key 由 low+index 给出，lookupswitch 直接给 key
      Handler { ordinal: u32 },
  }
  ```

  校验规则（JVMS 4.7.3 与 `specs/jvm-ir` 的"保护区间核对有效指令边界"）：
  - branch/switch default/switch case/`handler_pc`：目标必须落在某个指令起点（跳入操作数即非起点 → `classfile_instruction_invalid_target`）、不得越出 `code_length`（含负向换算落到 BCI 0 之前 → `classfile_instruction_target_out_of_bounds`）；
  - 保护区间：`start_pc` 必须是指令起点、`end_pc` 必须是指令起点或恰为 `code_length`、`start_pc < end_pc`（空区间不是合法保护区间）、`end_pc <= code_length`；违反者用 `classfile_exception_range_invalid`（区间关系）或 `classfile_instruction_invalid_target`（端点不在起点）报告。
  - `Handler` 行的 `instruction_bci` 取 `handler_bci`（异常表记录没有唯一的"发出指令"；保护区间仍由 `exception_handlers[ordinal]` 给出）。
  - Body 未完整解码（预算/取消/decode 停止）时 `control_flow_targets` 是**可靠但不完备**的视图：不会放过非法目标，但会把"落在未读后缀里的合法目标"报成非法。调用方 MUST 先查 `execution`/`stopped_at`，不得把该 `Err` 直接当成方法损坏；3.x 消费前若需要更强的类型级保护再收紧。
- **switch 条目**：语义取自已解码事件（default/low/high/npairs 来自指令事件），条目本身允许从该指令**已记录字节区间**按 checked 偏移读取，且区域长度必须等于解码形状；这不算"再次遍历字节流重建语义"，也不得迭代 noak 的 `TablePairs`/`LookupPairs`（其 `high == i32::MAX` 会在 overflow-checks 下 panic）。
- **保留的指令操作数（0.2 补齐，D03/D27）**：`newarray` 的 atype、`multianewarray` 的 dimensions、`invokeinterface` 的 count 与 **wide 的 effective opcode** 都在 crate-private 的 `InstructionOperands` 上保留（见上面的字段），由共享 noak 事件适配直接给出，**不另写 decoder**。
  - `effective_opcode` 恒存在且等于原始 opcode，**只有** `wide` 包裹形态不同（`wide iload/istore/fload/dload/aload/astore/…/ret/iinc` 取被包裹的那个 opcode）。公共的 `InstructionFact`（raw opcode/width/span/BCI）**保持原样不变**，这两点是 `cfg`/`call_context` 决定分块、终结指令、`may_throw`、局部读写与方言违规的**唯一依据**——它们必须改读 `effective_opcode`，不得再看原始 opcode 猜 wide。
  - atype/dimensions/count 是**语义不同**于 `immediate` 的事实：`immediate` 仍只表示「指令本身压入的常量或 CP 条目」，`newarray` 的类型码不是压栈常量，两者不得混用（4.x 的 Frame 要按 atype 决定元素类型、按 dimensions 决定弹栈槽数）。
  - 合法/非法样本都要有：atype 全取值区间与越界、dimensions 为 0、count 与描述符推导出的参数槽数不一致；`InvokeInterface` 的 count 是**校验事实**，不是信任来源（5.1 若要用它，先与本项目自己的描述符推导对账）。
  - `immediate` 不承担 ldc 变体与 CP tag 的配对合法性（那属 4.x verifier 领域）。
- **存储与计费**：操作数 facts 只按既有 `CodeBytes` 1:1 计入，不新增维度；实测约 80 B/指令的派生放大（有绝对上界）在 1.3 由 `IrItems` 记账或在本节写明上界。
- **1.2 不构建 CFG**（3.x 才做），只交付可复核的校验事实与错误。
- **错误语义**：无效目标/溢出/形状不符各给稳定 code + 原 BCI 的定位诊断，不 panic、不静默跳过该指令；已有错误码（`classfile_instruction_*`）优先复用，必要时才新增。
- **1.2 的验收**：wide/iinc、正负相对分支、tableswitch 的 default/key/target、lookupswitch 的 default/pair、handler 边界（含 `end == code_length` 与 `start > end` 反例）、非法目标（跳入操作数/越界/溢出）反例、以及 P0 既有指令边界 oracle 与全部既有测试不回归；不要求也不允许 1.2 引入 CFG/SSA/AST 或改动公共输出。

## 2.1 契约：Header providers、搜索顺序与内容身份（crate-private 事实 + 报告层状态）

2.1 只实现"名字 → 物理定义（含最小 Header 事实）"的**查找**，不实现闭包（2.2）、成员解析（2.3）或 dispatch（2.5）。

### 搜索模型

- **搜索顺序完全由 domain 表决定**，按递归式定义（`R(parent)` 为空链时即中止）：`ChildFirst` 的 loader 序列为 `roots(l) ++ R(parent)`，`ParentFirst` 为 `R(parent) ++ roots(l)`；每个 loader 内部按 `LoadDomain.roots` 声明顺序。混合链即各 loader 按**自己的** delegation 递归展开（等价于"ChildFirst 组按调用方→顶层在前、ParentFirst 组按顶层→调用方在后"），不是整条链统一方向。`Custom`/`Unknown` 与 `ModuleMode != ClassPath` 已在 1.1 判为 `UnsupportedPolicy`，2.1 不猜顺序。
- **一个 root 内的候选**：ZIP/ArtifactTree root 的候选是 raw name 恰为 `internal_name + b".class"`（字节精确、大小写敏感，沿用 P1 的候选纪律）的非目录 entry，按 container 顺序 + entry ordinal 升序；standalone CLASS root 的候选是它自身（读 root 字节并以 `this_class` 原始字节比对名字）。
- **首个匹配即胜出，不回退**：按有效序列遇到的第一个有候选的 root 决定结果；该候选损坏时报告该失败，**不得**继续到后面的 root（JVM 语义：搜索顺序先到的定义就是那个定义）。
- **Ambiguous 仅指同一选择位置无法区分**：同一個 root 内同名且字节不同的多个 entry（重复路径或嵌套容器同路径）→ `Ambiguous` 并列出各自 origin；同一 root 内同名同字节的重复 entry 也视为无法区分（不按 ordinal 猜）。
- **Missing 是事实**：全部参与位置都没有该名字 → `Missing`（不是错误）。
- **任一 1.1 环境问题都拒绝整次查找**：返回 `NotPerformed` + `state = None` + `Failed{Unsupported{resolution_not_implemented}}` + 该问题的同 code 诊断，且**零读取**（`LoadRoot::External` 与未提供内容的 root 因此不是"该位置不可判定"而是整次拒绝）；这条比"只跳过该位置"更强，2.2 的按位置展开必须先与它对齐。

### 类型（crate-private，报告层复用 1.1 的公共状态）

```rust
pub(crate) struct HeaderLocation {
    pub(crate) loader: LoaderId,
    pub(crate) root_index: u32,             // 该 loader 的 roots 声明下标
    pub(crate) definition: PhysicalDefinitionId,
    pub(crate) entry: Option<PhysicalEntryId>,   // standalone CLASS root 为 None
}
pub(crate) enum HeaderLookupState { Found, Missing, Ambiguous }
pub(crate) struct HeaderLookup {
    pub(crate) state: HeaderLookupState,
    pub(crate) location: Option<HeaderLocation>,
    pub(crate) candidates: Vec<HeaderLocation>,  // Ambiguous 时非空
    pub(crate) header: Option<ClassHeaderFacts>, // Found 时来自既有 classfile reader
}
```

`ClassHeaderFacts` 是对既有 `classfile` 事实的最小封装（`major/minor/access_flags/this_class/super_class/interfaces/member headers/constant_pool`），不新增公共类型。

### 计费与身份

- 每个 **Header 读取尝试**记一次 `ClassHeaders`（与契约的计数单位一致）；ZIP 名字匹配走既有枚举路径（其 `archive_entries`/`result_items` 计费不变，2.1 不新建索引，索引属 P5）。
- `definition` 由真实 origin + `class_bytes` 摘要/长度 + 路径变体派生（沿用 P1 的 `PhysicalDefinitionId` 语义）；同字节不同 origin 不合并。
- 失败的尝试与损坏候选保留 origin 证据；预算/取消按既有 `Err(BudgetExceeded)`/`Err(Cancelled)` 传播，由报告层映射为相应平面。

### 2.1 的验收

- fixtures：`ParentFirst`/`ChildFirst` 的顺序差异（同名类在两个 loader 各自 root 中的选择）、同一 loader 内同名有序 root 的优先、同 root 同名不同字节 → `Ambiguous`、缺失 parent / 循环 parent（走 1.1 的校验）、`UnsupportedPolicy`（module mode 与 Custom/Unknown）、`External` root 与未提供内容、standalone CLASS root 按 `this_class` 命中、同名同字节不同 origin 不合并、损坏候选不回退到后续 root（并给出该 root/entry 的 origin 诊断）。
- 每个 fixture 断言选择结果（loader + root 下标 + definition 身份）、`ClassHeaders` 计数、以及失败时的状态与诊断码；报告层只映射 1.1 已定型的 `ResolutionState`，不新增公共 variant。

## 2.2 契约：按需 Header 闭包、读取 reason 与去重

2.2 把 2.1 的单次查找组织成**按需闭包**：只扩展解析与目标方法真正需要的 Header，并把每一次读取的身份与理由记进报告（A14/A16 的证据面）。

### 读取记录（additive 公共字段；1.1 已交付的三个报告各增一个 `reads`）

```rust
// src/resolver.rs
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReadReason {
    RequestedDefinition,   // 请求目标自身
    ParentChain,           // 沿类型的 super_class 链解析出的定义（不是 loader 链）
    HierarchyClosure,      // 父类/接口闭包（2.3 的成员解析需要）
    DispatchScope,         // 显式 CHA 范围枚举（2.5）
    MemberOwner,           // 成员解析命中的 owner
    DriverMethodBody,      // 目标方法 Body —— 唯一的 Body 升级理由
}
pub struct HeaderRead {
    pub loader: LoaderId,
    pub definition: PhysicalDefinitionId,   // 成功读取并获得身份才入记录
    pub reason: ReadReason,
}
```

`ResolutionReport`/`DeclarationRefReport`/`MethodAnalysisReport` 各增 `pub reads: Vec<HeaderRead>`：按**实际发生顺序**、同 **(definition, loader)** 只记一次（reason 取首次读到该绑定的需求）；只有**成功读取且取得定义身份**才入记录（被拒绝的尝试、损坏与读取层失败不入记录），Ambiguous 的每个候选各记一条，standalone 声明别名同样记录。**变更口径**：类型面是 additive；wire 面是 `deny_unknown_fields` 下的**必填新字段**（缺 `reads` 的旧 JSON 会被拒），按 1.3 的先例记为有意的 schema 变更，不是兼容性承诺。（重复使用同一个已读 Header 不重复记录）。失败尝试不产生记录（其证据是 `environment_problems`/诊断）；`ClassHeaders` 仍按**尝试**计数（1.3 口径），因此 `reads.len() <= usage.class_headers` 恒成立，该不等式本身是 2.2 的一条断言。`elapsed`/`coverage` 语义不变；本字段是**additive 公共变更**，1.1 的既有测试需同步（记进 verification）。

### 闭包算法与去重

- **起点与后继**：首个符号需求使用调用方的 initiating loader；选中 Header 后，父类/接口符号由该 Header 的 **defining loader** 发起查找。不能把整次请求固定为 `runtime.load_domain.loader`。查找 memo 用 `(initiating_loader, internal_name)`，层级节点与已展开/祖先集合用解析后的 `(defining_loader, definition)`；记录与声明比较沿用既有物理身份，不因同名或同 bytes 合并。ParentFirst/ChildFirst 逐 loader 生效。
- **0.1 修正边界**：沿用 `HeaderClosure`/`HierarchyWalk` 与现有身份，给需求及待展开层携带搜索起点；同步 members、声明引用和 dispatch 的层级比较。dispatch 的祖先必须匹配目标声明的 loader/definition，只有 owner 字符串相等不构成继承证据。不新增 resolver 框架或缓存。
- **driver/caller 绑定**：读取物理 Header 后，在声明的 loader 环境下核对其名称解析结果是否为该 `(loader, definition)`；不能给任意 content 中的定义直接贴调用方 loader。不同 snapshot 可以是合法依赖 root，不能以 snapshot 必须相等替代绑定校验；不在绑定内或同名被遮蔽的定义须明确诊断、停止运行时语义阶段，原物理 facts 可保留。该规则一并关闭 D15/D25 的身份口径，进入 Frame 前完成。
- **去重键是 (物理定义, loader)**：同一请求内同一绑定只读一次，同一 `definition` 在不同 loader 下是两条独立记录（同 bytes 不同 origin/loader **不得**合并）。
- **深度**：每向上一层（parent 链或接口闭包）调用 `Budget::observe_dependency_depth`（1.3 已交付），超限即停并保留可信前缀；`DependencyDepth` 与 `nested_depth` 独立。
- **计费**：Header 读取尝试记 `ClassHeaders`；方法 Body 读取尝试记 `MethodBodies`（2.2 只允许 `DriverMethodBody` 一个理由，其他理由出现在 3.x/4.x 的分析阶段）；工作列表迭代记 `AnalysisSteps`。**不读无关 Body**：闭包只读 Header，Body 读取必须带显式 reason 且只有目标方法。
- **停止语义**：预算耗尽/取消/缺失依赖都在**下一次扩展前**停止，`execution` 为 `Partial`/`Cancelled`（`TerminationReason::BudgetExceeded { dimension }` 或 `Cancelled`），`coverage` 保留已扫描范围并把未完成部分记 skipped；不得把停止报告成 `Missing` 或空闭包。

### 2.1 实现要点（供复核与后续切片对齐）

- **位置**：查找机器在新增的 crate-private `src/providers.rs`（`resolver → providers → {view, environment, artifact, classfile, budget, model, error}`，无反向边），报告装配留在 `src/resolver.rs`；不新增公共类型。
- **有效序列**：从 `runtime.load_domain` 沿 `parent_loader` 收集链，再**按每个 loader 自己的 delegation 定序**（ParentFirst 父序列在前、ChildFirst 本 loader 在前），最后按各 domain 的 `roots` 声明顺序展开位置。混合链按各自声明（不是整条链统一方向）。
- **计费与覆盖**：读取尝试记 `ClassHeaders`；枚举/读取沿用既有 `archive_entries`/`read_bytes`/`class_bytes`/`attribute_bytes`/`entry_bytes`/`output_bytes`/`result_items`；performed 查找把已检查位置记入 **`runtime_resolution`** 维度的 `provider_search_position` 区间（未检查部分 skipped → `Partial`），`artifact_structural`/`dynamic_analysis` 保持 `NotRequested`。
- **不应用运行 profile**：2.1 的 `Resolved` 只表示"按 raw name 选中的物理定义"；`RuntimeProfile` 的 release/multi-release 与 `external_override`/`runtime_transformation` 判定不在 2.1 执行（MR 选择由 `Engine::select_multi_release` 单独提供，uncertain-runtime 诊断归 2.5）。
- **覆盖语义**：结论成立的查找 = 按规则在**首个命中处停下**，因此 `runtime_resolution` 为 `CompleteWithinSchema`、`scanned = [0, examined)` 且**无 skipped**；只有预算/取消/损坏导致的停止才是 `Partial` + `skipped = [examined, positions)`。skipped 的单位语义是"未达判定的 position"（含被拒绝搜索的那一个，故 union 覆盖全部 position 而无空洞）。
- **能力码**：环境被拒时仍是 1.1 的 `resolution_not_implemented`（能力未运行）；类查找执行时若请求带 `dispatch`，追加 **`dispatch_not_implemented`（Warning）** 并在 2.5 消失；成员符号仍是 `resolution_not_implemented`（2.3 落地后消失）。
- **共享身份规则**：`PhysicalVariant` 路径派生（`META-INF/versions/<N>/`）从 `src/xref/mod.rs` **原样搬**到 `src/model.rs`（`pub(crate) physical_variant_for_path`），使 P1 与 P2 对同一 entry 得到同一身份；行为不变（P1 golden 全绿）。
- **crate-private facts 的 dead_code allow**：`HeaderLookup.header`/`ClassHeaderFacts`/`HeaderLocation.entry` 由 2.2 消费，沿用 `classfile` crate-private facts 的既有约定。

### 2.2 实现记录与已知边界

- **范围的发布方式**：单次查找的 extent 由 2.1 的 `HeaderSearch` 提供；请求级的覆盖由 `HeaderClosure::searched_extent()` 对**一次请求内各次查找求和**发布（2.3 起沿用、2.5 的 scope 枚举同样如此）。memo 命中的需求不再返回 extent，所以覆盖平面必须走这个求和入口，而不是读某次 demand 的返回值。

- **历史边界已撤回**：2.2 当时把 loader 分量视为公开路径不可证伪；本轮通过 ChildFirst → parent 定义 Owner → parent Base 的公开成员查询证明，后继查找必须切换起点。当前代码错误选择 child Base，见复核 R1；由 0.1 修正并重新验证 2.2–2.5，不再作为可接受边界。
- 层级展开（`ParentChain`/`HierarchyClosure`、`WalkGaps`、环诊断）在本切片无公开入口，语义由 `providers` 的 lib 单测固定，消费者是 2.3/2.5（相关项带 `#[allow(dead_code)]`，移除即产生 9 条 warning）；复核指出的四类未固定语义（`parent_chain` 不跟接口、损坏/读取失败不入记录、Ambiguous 每候选一条、停止的需求不被记忆）已由补测固定。
- 公开层只能构造预取消与预算停止；「两个 demand 之间被取消」由 lib 两层之间的用例证明。

### 2.2 的验收

- **深链**：父类链深度超过 `dependency_depth` → 终止维度为 `DependencyDepth`、保留前缀、`reads` 只含已读深度。
- **高扇出**：一个类实现多个接口且接口再继承 → 每条 Header 只读一次、`reads` 无重复 (definition, loader)。
- **缺失依赖**：闭包中某一层在全部 root 都 `Missing` → 记录该层状态、不伪造定义、`coverage` 标明未完成范围。
- **循环引用**：A→B→A 的继承环（非法 class）→ 不无限扩展（去重键终止）、给出可定位诊断。
- **预算/取消**：预取消与中途取消分别得到 `Cancelled`，`usage` 与 `reads` 一致（`reads.len() <= class_headers`）。
- **无关 Body 读取为零**：闭包请求后断言 `usage.method_bodies == 0`（除非显式请求目标方法 Body），且 `reads` 的 reason 集合不超过本次请求允许的理由。
- **同 bytes 不同 origin/loader 不合并**：同一 class 字节放在两个 loader 的 roots 下时，分别以各自环境请求会得到两次各自的选择与两条记录；0.1 必须增加单次请求跨 defining loader 的对照，断言实际物理定义而非仅比较返回名字。
- **深度 0 的边界**：闭包目标自身是依赖深度 0、即使 `dependency_depth = 0` 也允许读取；停止时 `reads` 与 `coverage` 一起发布可信前缀。
- **不读无关 Body 的证据是 `code_bytes == 0`**（配一条真实 body 读取路径的对照，例如同一 fixture 经 `inspect_method_bytecode` 得到非零 `code_bytes`）；`method_bodies` 在 3.x 接通计费前没有计费点，因此不能单独作为该证据。

## 2.3 契约：成员解析（JVMS 5.4.3）、访问与调用种类规则

2.3 在 2.1 的 Header 查找之上实现**成员声明解析**：给定 `SymbolRef`（Field/Method）、`ReferenceUse` 与调用方身份，按 JVMS 5.4.3 找出声明所在类与成员；不实现 dispatch（2.5）。

### 解析过程（结构性近似 + 明确状态，不猜）

1. **owner 解析**：用 2.1 的查找解析 `SymbolRef` 的 owner 类（必须 `Resolved`；否则继承该状态）。
2. **按种类搜索**（JVMS 5.4.3.2/5.4.3.3/5.4.3.4）：
   - 字段：声明类自身 → 其超接口（递归）→ 其超类（递归）；
   - 方法（class method resolution）：声明类 → 超类链 → 超接口的 **maximally-specific** 集合；
   - 方法（interface method resolution，owner 是接口）：该接口 → 其超接口的全部 maximally-specific 集合。
   每步都经 2.2 的闭包机器读 Header（同 (definition, loader) 去重），reason 按第 3 条：`super_class` 边派 `ParentChain`、`interfaces` 边派 `HierarchyClosure`、按身份直接命名的定义（成员 owner 与 use-site 所在类）派 `MemberOwner`。
   **maximally-specific 必须按 JVMS 5.4.3.3/5.4.3.4 的原定义**：候选是"名字与描述符都匹配、且**既非 `ACC_PRIVATE` 也非 `ACC_STATIC`**"的声明，再在其中剔除被严格子接口声明覆盖者；
   因此 `static`/`private` 的接口声明**不参与**该集合（JVMS 注："Superinterface methods that are private and static are ignored by resolution"）。集合为空即回退为 lookup 失败（→ `Missing`），不得因此误报 `resolution_default_conflict`。owner 自身（直接点名）的声明不受此过滤影响：`invokestatic` 点名 static 接口方法必须解析成功，`invokeinterface` 点名 static 则按调用种类规则报 ICCE。
3. **reason 语义（2.3 起生效）**：`ParentChain` 表示"沿声明/owner 的 `super_class` 链向上"读到的 Header，`HierarchyClosure` 表示"沿接口图（`interfaces`/超接口）"读到的 Header，`MemberOwner` 表示**按身份直接命名**而读取的定义（请求成员的 owner 定义，以及访问规则所需的 use-site 所在类），`RequestedDefinition` 表示请求目标自身，`DispatchScope` 归 2.5，`DriverMethodBody` 归 3.x。因此两个变体都有真实生产者，不得留未产出的公共 variant。
4. **判定**（与 `specs/demand-resolver` 的 7 状态对齐）：
   - 唯一命中（字段；或方法在 class/超类链上唯一）→ `Resolved`；
   - 方法只有接口候选时，取 maximally-specific 集合：**恰好一个非抽象** → `Resolved`（该 default 方法）；**多个非抽象**（Java 8 default conflict）→ `IncompatibleClassChange` + `resolution_default_conflict` 诊断；**全部抽象** → `Resolved`（那个抽象声明）+ Warning `resolution_method_is_abstract`（JVMS：解析成功，AME 发生在调用时，不由解析阶段伪造）；
   - 什么都没找到 → `Missing`（保留原始 `SymbolRef`/descriptor/origin，不伪造空成员）；
   - 同一步骤内多个无法区分的候选（同一类里同名同描述符重复声明）→ `Ambiguous` + 各自 origin。
5. **调用种类规则**（`ReferenceUse`；违反即 `IncompatibleClassChange` + 具名诊断）：
   - `InvokeStatic` 要求静态方法，`InvokeVirtual`/`InvokeInterface`/`InvokeSpecial` 要求实例方法（`<init>` 仅 `InvokeSpecial`）；
   - `FieldRead`/`FieldWrite` 中 static 指令（`GetStatic`/`PutStatic`）要求静态字段，实例指令要求实例字段——由 2.4/2.5 的 use-site 提供指令级种类时使用；
   - class method resolution 命中接口方法或 interface method resolution 命中类方法 → `IncompatibleClassChange` + `resolution_kind_mismatch`；
   - `InvokeSpecial` 命中抽象方法 → 同上（JVMS 5.4.3.3 对 invokespecial 的额外约束）；
   - `InvokeStatic`/`InvokeSpecial` 对 **class 与 interface owner 都合法**（Java 8 起的 static interface method 与 `I.super.m()`）；`InvokeDynamic` 的 `owner` 只是搜索起点——`CONSTANT_InvokeDynamic` 没有方法引用可约束，故不施加调用种类规则。
6. **访问规则**（JVMS 5.4.4，与解析分开）：调用方类名由 `CallerContext.enclosing`（其 `owner` 定义读 `this_class`）得到；运行时包 = (loader, 包名)。`public` 通过；`private` 要求同类；`protected` 要求同类/同包/子类；包私有要求同包。**判定不通过 → `Inaccessible` + `resolution_access_denied` 诊断**；调用方类未知（无 `enclosing`）时**不做访问判定**，返回 `Resolved` + Warning `resolution_access_not_checked`（不谎称已检查）。
7. **明确的 unsupported 分支**：
   - **signature-polymorphic**（`java/lang/invoke/MethodHandle` 的 `invoke`/`invokeExact`，JVMS 2.9）：调用点 descriptor 与声明不同，故按 **name 匹配**解析到声明并返回 `Resolved`，同时给 Warning `resolution_signature_polymorphic`（`target` 保留调用点描述符、`resolved.member` 是声明描述符，两者都在报告里可见）；
   - **数组 owner**（`[` 开头，如 `[I.clone()`）：P2 不实现数组类型方法解析 → `UnsupportedPolicy` + `resolution_array_owner` 诊断（不伪造 Object.clone）。

### 2.3 的诊断码与语义边界（实现后固定）

- 诊断码：`resolution_default_conflict`（多个非抽象 default）、`resolution_kind_mismatch`（调用种类/owner 种类/静态性/`<init>`/abstract+`InvokeSpecial`）、`resolution_access_denied`、`resolution_access_not_checked`、`resolution_signature_polymorphic`、`resolution_array_owner`、`resolution_hierarchy_missing`（层级某层 Missing）、`resolution_hierarchy_ambiguous`（层级某层 Ambiguous）、`resolution_hierarchy_cycle`（层级环，与 2.2 共享）、以及既有的 `resolution_method_is_abstract`。
- **语义近似（有意，记入 spec 边界，不得被当作 JVMS 完全实现）**：解析期报 default conflict 而 JVMS 8 把它放在 invocation selection；interface owner 不隐式继承 `java/lang/Object` 的方法（未命中即 `Missing`）；只检查成员自身的访问标志，不检查声明类的可访问性（JVMS 5.4.3.1）；调用方定义不一致或内容未提供属 stop（`state = None` + `Failed`），不是 `NotChecked`；字段的 static/instance 指令级规则留给 2.4/2.5（`MemberUse` 不含指令级种类）。
- **schema 限制**：`ResolvedMemberRef` 没有类内坐标，同一 owner 内同名同描述符的重复声明只能表达为相同的 refs（由用例固定）；`Ambiguous` 与 `resolved` 互斥（只在 `Resolved` 时发布 `resolved`）。
- **scope 校验**：`validate_declaration_reference_query` 必须校验 `PhysicalScope::ArtifactTree { root_container }` 的 root（与 P1 的 `query_artifact_tree_root_mismatch` 同一规则），不得接受会被静默忽略的容器名。
- **计费纪律（对所有解析报告统一）**：每发布 1 条**随扫描结果增长的列表条目**（`DeclarationRefReport.items`、`DispatchReport.candidates`，以及 4.x/5.x 方法分析报告的同类列表）与**每发布 1 条进入该报告的诊断**（未决诊断、2.3 的规则诊断、闭包自报的诊断如 `resolution_hierarchy_cycle`）前，各计一次 `ResultItems`——与 P1 的「每个返回 item = 1、每条域诊断 = 1」同口径；装配中途预算耗尽时保留已发布前缀并把 execution 标为 `Partial{BudgetExceeded{ResultItems}}`。
  **判定证据字段不单独计费**：`ResolutionReport.resolved` 与 `ResolutionReport.candidates`（Ambiguous 位置那组无法区分的定义）是**判定本身**的证据，至多对应该选择位置上已经付费过的枚举与 Header 读取（每个候选至少一次已计费的读取尝试）——把它们也计费等于为同一份工作收两次钱。
- **停止归属**：解析停止（如 `ClassHeaders`）与装配停止（`ResultItems`）同时发生时，`execution` 报**先发生的解析停止**，装配停止由 `has_more` 与覆盖平面的 `Partial` 表达。
- **不收费的三类元数据**（与 P1 的 `query_relation_unsupported`／terminal diagnostic 同纪律）：环境平面诊断（`environment_problems` 及其镜像诊断，被拒环境必须零字节零计费）、**停止解释**诊断（`budget_exceeded_*`/`cancelled`/结构错误码——它们解释请求或中断，不能自付，否则报告会失去停止原因）、以及**范围未运行**的说明性诊断（如 `resolution_dispatch_no_declaration`，它解释某平面为何没有运行）。
- **计数与诊断成对**：`unresolved_candidates` 与进入报告的未决诊断严格一一对应；装配被拒的候选既不进 `items` 也不计数，其 use-site 由停止诊断保留。
- **Class 符号的声明查询**：候选规则只对成员声明定义，因此 Class 符号的 `DeclarationRefQuery` 与被拒环境一样返回 1.1 的诚实不可用状态（`resolution_not_implemented` + `NotRequested`），不读字节。
- **计费**：成员解析使 `analysis_steps` 成为真实输入，因此成员请求必须给非零值（否则第一步即 `BudgetExceeded`）；`dependency_depth = 0` 仍允许读取成员 owner 自身（深度 0 不观察深度）。`reads` 的 reason 集合按上一条语义产生。
- **coverage 求和语义**：成员请求的 `runtime_resolution` 区间是**该次请求内各次查找已检查位置之和**（每次查找自身的 `[0, examined)` 与未决 `[examined, positions)` 拼接），不是单次查找的区间；任一查找有未决分支即为 `Partial`。停止发生在**推导出有效搜索序之前**（例如预取消、或 owner 直接是数组）时，没有任何 position 可声明：区间为空、状态仍为 `Partial`（不编造区间，也不因空区间而报 `CompleteWithinSchema`）。

### 2.3 的验收

- 合法/非法对照：静态 vs 实例（各一条 ICCE 反例）、private/protected/包私有/跨包与跨 loader 的访问对照（含"无 `enclosing` 时给 `Resolved` + `resolution_access_not_checked`"）、`<init>` 只经 `InvokeSpecial`、抽象方法解析成功但带 Warning；
- **Java 8 default conflict**：一个接口提供 default、两个接口各提供 default（conflict → `IncompatibleClassChange`）、抽象-only（`Resolved` + Warning）、子接口覆盖父接口 default（唯一非抽象 → `Resolved`）；
- signature-polymorphic 与数组 owner 各一条（含 `target` 与 `resolved.member` 描述符不同的断言）；
- 跨 loader：同一 owner 名在两个 loader 命中不同定义时，解析结果跟随 2.1 的选择（不合并）；
- 每条正例同时断言 items/状态/诊断/`reads` 的 reason 集合（`HierarchyClosure` 出现）、`usage.class_headers` 与"不读无关 Body"（`usage.method_bodies == 0`）。

## 2.4 契约：显式运行环境的声明引用查询（复用结构 consumer）

2.4 交付"找出**解析到该声明**的全部真实 use-site"：不能按 `SymbolRef` 的 owner 精确筛选（`Sub.foo` 的 CP owner 是 `Sub`，却解析到 `Base.foo`），也不能把未使用的 CP 条目当引用。

### 复用方式：给扫描机器加一个 crate-private 候选形态过滤器

- 在 `src/xref/mod.rs` 增 crate-private `CandidateFilter`：
  - `Exact(QueryTarget)`（现有行为，`Engine::query` 用它）；
  - `MemberShape { name, descriptor }`：按原始 name/descriptor 字节匹配 `SymbolRef::Method`/`Field`，**owner 不参与**（A11 的核心）；
  - `SignaturePolymorphic { owner, name }`：**仅**用于签名多态声明（`java/lang/invoke/MethodHandle` 的 `invoke`/`invokeExact`）。JVMS 2.9 下调用点描述符由站点决定、必然可能与声明不同，若仍按 descriptor 匹配，真实 use-site 会变成「连候选都不是」的静默漏报。该形态按 owner + name 精确字节匹配（owner 在此**是**身份的一部分，与 `MemberShape` 的「owner 不参与」不冲突：签名多态要求 owner 恰为 `MethodHandle`），解析侧的 `same_declaration` 仍比较声明符号。`ScanContext` 持有当前过滤器，`code.rs`/`metadata.rs`/`bootstrap.rs` 的 `use_site_answers`/`asks_symbol`/`target_matches`/`answers` 改为调用同一 `ctx.candidate_matches(..)`（`resource.rs` 只做 literal，不参与）。
- 新增 crate-private 入口 `xref::scan_candidates(snapshot, scope, consumers, filter, max_items, budget) -> CandidateScan { items, has_more, coverage, execution, diagnostics }`：复用同一 `ProviderScan` 与各 consumer 子扫描、同一计费与 ordering，但**不带游标**；`max_items` 直接约束扫描、`has_more` 上报截断，规则与 P1 页限一致（`artifact_structural` 与 `runtime_resolution` **两维**均为 `Partial`、execution 仍 `Complete`）。`scan` 与 `scan_candidates` 共享同一 unit 循环（容器顺序、子扫描顺序、`ResultItems` 计费点与停止语义只写一份）。
- **A17 方向不变**：`CandidateFilter` 与 `scan_candidates` 都不引用 resolver/ir；`src/query.rs` 与 `src/xref/**` 仍不含 P2 记号（守卫测试继续成立）。`Engine::query` 的行为与证据逐字段不变（P1 golden 全绿即证据）。
- 每个候选的 `origin`/`consumer`/`operation`/`evidence` 直接沿用 P1 的 item 证据形状；resolver 侧只做"把候选的 owner 解析成声明并与目标比较"这一步。

### 查询语义

1. 候选来自结构 consumer（未使用的 CP 条目不是引用：`MemberShape` 只匹配**被 consumer 消费**的 use-site，因为过滤器作用在 use-site 判定上，不在池枚举上）。
2. 对每个候选：解析其 owner（2.1 的查找 + 2.3 的成员规则），把解析出的声明与请求的 declaration 比较（loader + 定义身份 + 成员符号）。**只有解析到请求声明的候选进入 `items`**（`resolved` 即该声明、`state = Resolved`）；解析到别的声明的候选计入 `scanned` 但不进 `items`，因此 `items` 的每个条目都满足"该 use-site 确实指向请求声明"这一可复核断言。
3. **未决候选不得当已排除**：候选的 owner 解析为 `Missing`/`Ambiguous`/`IncompatibleClassChange`/`Inaccessible`/`UnsupportedPolicy`/`BudgetExceeded`/取消/环境问题、**请求级**环境问题时计入 `unresolved_candidates` 并保留 `origin`（经 `resolution_candidate_unresolved`（Warning）诊断携带 P1 的 use-site 位置），`coverage` 标 `Partial`，`has_more` 视截断而定；不得把它们算作"已排除"。
   **证据不含指令级成员种类的候选**（`metadata` 的成员事实如 `EnclosingMethod`、bootstrap 的成员参数、`ldc` 的 `MethodHandle`）同样按未决处理：没有指令种类就无法按 JVMS 施加 static/instance 规则，因此不臆造种类（这类候选当前不会成为 `items`）。若将来需要它们成为引用证据，必须先给 2.3 的 crate-private 词表加"无指令种类"的可能并明确跳过哪些规则。
   损坏的 class 候选条目不是候选级未决，而是**扫描级停止**（`execution = Failed { Error { code } }` + `has_more`），不进 `unresolved_candidates`。
   "解析到别的声明"的候选既不是 `items` 也不是未决：可由 `reads`/usage 与"不在 items 也不在 unresolved"观察（报告不设第三个桶）。
4. `max_items` 截断：按 P1 页限规则（`returned_items`、`has_more`、coverage `Partial`、execution 不因此变 Partial）。
5. **P1 语义不变**：`Engine::query` 的 `mentions_symbol` 仍按原始符号精确匹配（`Base.foo` 查询不返回 `Sub.foo` 的调用点），这是 A11 需要的对照证据。

### 2.4 的诊断码与边界（实现后固定）

- 诊断码：`resolution_candidate_unresolved`（Warning，provenance = 候选 use-site）；环境与能力类码沿用 1.1–2.3 的既有集合。
- `DeclarationRefQuery.consumers.version` 在 2.4 不校验（`Engine::query` 会拒绝非 1）；统一校验属后续小改动。
- metadata 类成员候选当前永不成为 `items`（见上），因此"不读 Body"的公开证据是**本次查询的 `code_bytes` 等于同 consumers 的同范围 P1 扫描计费**（解析不额外读 Body），不要求某个 fixture 恰好为 0。

### 2.4 实现记录与已知边界

- **closure 自身诊断从 2.5 起有真实生产者**：dispatch 的层级 walk 会自报 `resolution_hierarchy_cycle`（每条环边一条）。因此 2.4 登记的「计费循环当前无生产者」不再成立：该循环必须按上面的计费纪律收费，并有**逐条计费**的用例（同一 fixture 有环 vs 无环，`result_items` 差值等于新增诊断数）。

- **closure 自身诊断的计费是前瞻性条款**：`HeaderClosure::record_diagnostic` 目前只由 `hierarchy_closure` 驱动，而 2.3/2.4 都逐类 `demand`、不启用该 walk，因此解析侧末尾那段 closure 诊断循环**当前无生产者**（代码 fail-safe，2.5 接上后生效）。不得把它当作已测试行为；若复核要求，可在该循环加注释说明。
- 声明查询入口无 hook，**中途取消**不可构造（预取消已覆盖）；`SignaturePolymorphic` 与 `MemberShape` 共享 2.3 的 name-only 近似（有意）。
- 测试写入器在本片修掉一个真实缺陷：接口项原先在常量池落盘之后才 intern，会产生非法索引的 class；修复后接口/`BootstrapMethods`/`InvokeDynamic` 才能被正确写入（幂等性与既有 fixture 字节由复核者独立对照确认）。

### 2.4 的验收

- `Base.foo` 在 `Sub` 调用：`Engine::query(mentions_symbol, Base.foo)` 为空或只含声明侧证据，而声明引用查询返回 `Sub.foo` 的 use-site 且 `resolved` 指向 `Base`；
- 未使用 CP 条目（同名同 descriptor 的 `Methodref` 无指令消费）不产生 `items`；
- 候选 owner `Missing`（平台未提供）→ 计入 `unresolved_candidates`、coverage `Partial`，不出现在 `items`；
- 预算停止（`ResultItems` 或 `ClassHeaders`）→ 保留可靠前缀 + `unresolved_candidates` 或 `has_more`，execution 为对应 `Partial`；
- 跨 loader 同名类：两个 loader 各自命中时，声明引用查询按请求环境解析，不合并结果；
- 每条正例断言 `items` 的 consumer/operation/origin（P1 证据形状）+ `reads` 的 reason 集合 + `usage.method_bodies == 0`。

## 2.5 契约：声明解析与已知范围 dispatch 分离（open-world）

2.5 在 2.3 的声明解析之上加一个**独立的 dispatch 平面**：给定已解析的声明与**显式范围**，返回该范围内的 KnownCandidates 与 open-world 依据；绝不把"已知候选"表述成唯一运行时目标。

### 语义

1. **触发**：`ResolutionRequest.dispatch = Some(DispatchScope { scope, consumers })`。声明解析照常执行（2.1 + 2.3）；dispatch 只在声明 `Resolved` 时计算，否则 `dispatch = None` 并保留 `dispatch_not_implemented` 之外的现有语义（声明未解析时给 `resolution_dispatch_no_declaration` 说明性诊断）。
2. **覆盖平面继承 2.1/2.2 的 skipped 纪律**：dispatch 平面在一次**查找停**处结束时（`ClassHeaders`/`DependencyDepth`/listing 预算、取消或损坏），`runtime_resolution` 必须是 `Partial` **且 `skipped = [examined, positions)`**——闭包已经知道还有多少位置没查，`demand-resolver` 的 MUST（「目录或依赖读取停止后 MUST 保留 skipped/未决范围」）在这里同样适用。**但必须区分查找停与发布停**：「这个停是否结束了一次查找/listing」要显式判定（`DispatchStop::ended_a_search()`：listing 截断与非 `ResultItems` 的拒绝为真，候选发布被拒为假）——发布停让 `execution` 与 `runtime_resolution` 都标 `Partial`（范围确实没有走完），但**不得**伪造 `skipped`，因为请求里更早那些「按规则停在首个命中位置」的查找本来就不需要未决范围。
   **已知粗粒度**（债务，归 5.3）：当一个更晚的查找被拒时，`skipped` 区间会包含此前那些已完成查找的未检查尾部（闭包只暴露整请求的求和）。当前语义是「未检查位置」的**保守上界——高报未决、绝不低报**，因此不会把未决范围说成已决；5.3 的覆盖平面审计要么收紧为逐查找的未决集合，要么在 golden 里固定这个上界语义。
3. **候选发现（CHA-lite，只读 Header）**：枚举 `scope` 覆盖的 Header（`PhysicalScope::SnapshotAll` 或 `ArtifactTree`），对每个类用闭包解析其超类/接口链；若链上包含声明的 owner（**严格在其之上**，因此声明类自身与同名重复定义都不是自己的 override）且该类**自身声明**了同 kind/name/descriptor 的成员，则该成员是一个候选（实现或覆盖）。**候选规则是结构性的，不筛成员标志**：private/static/abstract 的声明与 `<init>` 之类特殊名字都会被发布为候选（P2 不在 dispatch 平面重做 2.3 的规则判定），调用方必须结合 `open_world` 与证据自行判断。**只读 Header，不读任何 Body**（证据是 `usage.code_bytes == 0`，不是尚无计费点的 `method_bodies`），受 `ClassHeaders`/`DependencyDepth`/`ResultItems`/取消约束。
4. **每个候选带 open-world 证据**（`DispatchCandidate { member, evidence }`，证据取自 `OpenWorldEvidence`）：`ExternalSubclass`（范围内存在 `LoadRoot::External` 或未提供内容的 root，可能有未见的子类）、`UnknownLoader`（`Domains` 之外可能有别的 loader 加载同一类）、`RuntimeTransformation`/`ExternalOverride`（`RuntimeUncertainty != None`）、`MissingDependency`（链上有 `Missing`）、`OrderedRoot { index }`（**仅当 `index > 0`**：该候选来自第 index 个 root，之前的 root 本可以定义却被跳过，因此同层可能有别的定义；`index == 0` 之前没有任何位置，**不构成**开放性证据，否则任何查找都会把 `open_world` 顶成 `true`）。
5. **`open_world` 的判定**：只要出现上述任一证据（或范围内存在不可判定位置/预算停止）→ `open_world = true`；**单一候选也照样 `open_world = true`**，报告里没有任何"唯一目标"字段，调用方只能从 `candidates` + `open_world` + 证据自行判断。预算/取消停止时 `execution` 为对应 `Partial`/`Cancelled` 且 `open_world = true`（未知范围本身就是开放世界证据）。
6. **平面组合（停止只落在 execution）**：
   **发布阶段已经停下的请求不再启动 dispatch 平面**（`dispatch = None`，`execution` 说明原因）——请求在那一点已经结束，启动一个明知会立即被拒的平面只会产生重复的停止诊断；预取消请求同理。
   dispatch 平面停止时，`state` 仍是**声明解析**的判定（通常 `Resolved`），停止由 `execution`（`Partial`/`Failed`/`Cancelled`）与 `open_world = true` 表达。不变量 3 里「预算停止 = `Some(BudgetExceeded)`」描述的是**判定平面自身**的停止；声明已判定之后，后续平面在自己的 `execution` 上报告停止，不改写 `state`。
7. **与 2.4 的边界**：声明引用查询回答"哪些 use-site 指向该声明"；dispatch 回答"该声明在该范围内的已知实现/覆盖"。两者不互相替代，也不共享结果缓存（各自独立计费）。

### 2.5 的验收

- **多实现**：接口 + 范围内两个实现 → 两个候选、各自 evidence；当两个实现都来自 `roots` 的第 0 个位置、范围完整且环境无 uncertainty 时 `open_world = false`（这条同时固定上一条 `index == 0` 的语义）；再补一个 `RuntimeTransformation::Possible` 的同一 fixture → `open_world = true` 而候选集合不变（证明 open-world 是独立平面）；
- **外部子类**：分类由 `dispatch` 的纯函数固定（`External`/未提供内容的 root → `ExternalSubclass`，有单元用例），但**公开路径不可达**——只要环境里出现 `External` root，1.1 的环境校验就整次拒绝该请求（`NotPerformed` + `state = None` + `dispatch = None` + 零读取），公开用例钉住的是这条交互；两者合起来才是这条验收的证据，缺一不可；
- **未知 loader/transformer**：`Domains` 未覆盖的 loader 或 `external_override`/`runtime_transformation != None` → 对应证据；
- **单一已知候选不声称唯一**：断言报告结构上不存在"唯一目标"（无该字段）+ `open_world` 语义；
- **只读 Header**：证据是 `usage.code_bytes == 0`（或与同 consumers 的同范围 P1 扫描计费相等），**不是** `method_bodies == 0`——该维度在 3.x 接通前没有计费点，用它当证据是恒真断言（2.2/2.3 复核已两次纠正同一错误）；同时断言 `usage.class_headers > 0` 与 `reads` 的 reason 含 `DispatchScope`；
- **预算/取消**：`ClassHeaders`/`DependencyDepth` 或 `ResultItems` 停止 → 保留已发现候选 + `open_world = true` + `execution` 为对应 `Partial`，且**查找停**的路径上 `runtime_resolution` 为 `Partial` + `skipped = [examined, positions)`（`ResultItems` 是发布阶段的停止，不产生伪造的 skipped）；
- **结构性候选规则**：private/static/abstract 声明与 `<init>` 都会被发布（一条用例固定该边界，避免读者以为 dispatch 已做规则筛选）；
- **回归**：`Engine::query` 的既有行为与证据不变（P1 golden），`resolve_symbol` 的类符号路径不变（2.1 证据），`dispatch = None` 的请求不产生 dispatch 相关诊断。

## 3.1 准入证据与使用约束（petgraph 0.8.3）

准入证据见 `verification.md` 的 3.1 节；结论是**引入**，但带四条硬约束。实验目录与复现命令记在同节。

- **feature 集合固定**：`petgraph = { version = "=0.8.3", default-features = false, features = ["std"] }`。默认 feature 会额外拉入 `stable_graph`/`graphmap`/`matrix_graph` 并把 serde 家族写进 lock；`rayon`/`serde-1`/`all` 不使用（含后台线程池或 proc-macro）。CI 的 feature-tree 步骤加一条 `grep -F 'petgraph feature "std"'` 锁住该集合。
- **SCC 一律用 `kosaraju_scc`，禁止 `tarjan_scc`**：`tarjan_scc` 是**递归**实现，实测在 2 MiB 栈上 20 000 节点的链状图即 stack overflow 并 **abort 进程**（不可捕获）；`kosaraju_scc` 是迭代实现，512 KiB 栈下 100 000 节点正常。
- **顺序自己定**：`kosaraju_scc`/`tarjan_scc`/`toposort` 的输出顺序是插入顺序与 `NodeIndex` 值的函数，`Dominators::immediately_dominated_by` 内部用 hashbrown `HashMap`、**同一二进制不同进程顺序不同**。因此所有对外发布的 `Vec<_>` 必须按 (物理定义, BCI) 显式排序，构建时也按 BCI 顺序加节点/边；不得依赖任何 petgraph 迭代顺序（该纪律由 golden/性质测试固定）。
- **root 归属自校验**：把不属于该图的 `NodeIndex` 传给 `simple_fast` 会在 debug 与 release 都 panic（fixedbitset 的无条件断言）。适配层必须在调用前校验；`Graph::remove_node` 会 swap-remove 并改写末节点索引，构图后不删点。
- **多出口要合成 super-exit**：后支配用 `simple_fast(Reversed(&g), super_exit)`，需先合成一个统一出口节点；入口节点的 `immediate_dominator` 为 `None`、`dominators` 为自身。
- **取消粒度与规模上界**：petgraph 无任何中断/预算钩子，因此**阶段级不可取消**——进入前检查预算与取消、阶段结束后再次检查并据此决定是否继续；单次调用耗时/内存由输入规模上界保证。实测（release、macOS aarch64）：SCC/拓扑 ~13 ns/点·边，`Graph` 本体 52 B/点 + 26 B/边，支配树峰值 ≈ 195 B/block（`predecessor_sets` 的 HashMap/HashSet），65 535 点级图 `simple_fast` ≈ 7.7 ms / 12.6 MB。据此设 `max_blocks` 默认 16 384、硬上限 65 535（= `code_length` 上限），支配阶段按 195 B/block 记账。文档仍声明 `simple_fast` 最坏 O(|V|²)（未复现对抗图），block 上限是唯一保险。
- **引入不等于可用**：依赖进入生产图不改变 A17——`query`/`xref` 仍不得引用 `ir`；首个消费者是 3.3 的 raw CFG，3.1 只做准入与依赖引入（本 change 的 3.1 与 3.2 都不产出图形算法调用）。
- **升级门槛**：0.8.3 之后上游 master 已有破坏性提交，任何升级必须重跑 MSRV、`cargo deny` 两个图、feature-tree 断言与本节的行为证据（尤其是递归/栈与顺序两条）。

## 3.2–3.5 契约：Pass 契约、raw CFG、returnAddress 与有界规范化

### 0.2 共享 reader 补齐（3.4/4.x 前置）

在现有 noak 事件适配上保留 wide 的 effective opcode，并保留 `newarray` atype、`multianewarray` dimensions、`invokeinterface` count；raw opcode、width、BCI 与公共取证事实保持原样。CFG/effects/returnAddress/Frame 使用同一有效操作数事实，不自行重解字节或解析展示文本。宽化 local 的读写、category-2 双槽和 wide ret 的终结行为必须准确。modern 方法只有 wide load/store 时不能因此拒绝 legacy 阶段；51+ wide ret 必须走 dialect 违规分支。

验收含真实字节的普通/宽化 load、store、iinc、ret 对照，数组分配维数/atype、invokeinterface count 的合法与非法样本；重跑 1.2/3.3 oracle、CFG/effects 与 P1 原始 BCI/XRef 回归。D03/D27 是确定的前置缺口，不能留作“若未来需要”。

### 3.2 PassDescriptor 与 invalidation（不引入动态调度）

```rust
pub(crate) enum IrPhase { RawFacts = 1, RawCfg, LegacyNormalization, CanonicalCfg, Frame, Ssa }
pub(crate) enum FactKind { Instructions, ExceptionTable, ThrowSites, RawCfg, CallContexts, CanonicalCfg, Frames, Ssa, Effects }
pub(crate) enum PassBudgetClass { Blocks, Steps, Clones }
// Blocks = IrItems + IrEdges（块与边）；Steps = AnalysisSteps；Clones = NormalizationClones。
// 集合为空 = 该 pass 不计费（例如 raw_facts：解码是 reader 的工作，字节已按 ClassBytes/AttributeBytes/CodeBytes 收过费）。
// **单一 pass 可以计多个维度**：raw_cfg 同时产块与边并迭代工作列表，故其集合是 [Blocks, Steps]——这正是本字段必须是集合而非单一类别的原因。
pub(crate) struct PassDescriptor {
    pub(crate) phase: IrPhase,
    pub(crate) name: &'static str,
    pub(crate) requires: &'static [FactKind],
    pub(crate) produces: &'static [FactKind],
    pub(crate) invalidates: &'static [FactKind],   // 本 pass 改变 CFG/异常边时必须声明
    pub(crate) budget: &'static [PassBudgetClass], // 该 pass 实际计费的维度集合（空 = 不计费）
}
```

- **描述表是静态常量**（`&'static [PassDescriptor]`），按 `IrPhase` 升序声明；引擎只按该表顺序执行，**没有动态注册、插件或运行时图**。
- **阶段词汇一一对应**：公共的 `AnalysisStage`（1.1）与 crate-private 的 `IrPhase` 是 1:1 映射（同序、同名），`IrPhase::RawFacts = 1` 起编；请求里的 `requested_stages` 直接投影成阶段前缀，不引入第二套编号。
- **启动校验**：请求的阶段集合在开跑前解析成"该表的前缀"（与 1.1 的 `scheduled_stages` 同规则），并检查每个被调度 pass 的 `requires` 都能由**更早的已调度 pass** 产出；缺失前置、`IrPhase` 逆序、`requires` 与 `produces` 冲突、表自身成环都在**启动时**返回结构化错误（`ir_pass_prerequisite_missing` / `ir_pass_order_invalid` / `ir_pass_graph_cycle`），不执行任何半初始化 IR。
- **invalidation**：任何 pass 声明了非空 `invalidates` 时，其"之后"的既得事实必须被丢弃（`CanonicalCfg` 改变异常边即让 `Frames`/`Ssa`/`Effects` 失效），后续阶段若要使用必须重算或拒绝使用。运行时用一个"已产出事实集合"检查：使用未被重算的失效事实即 `ir_stale_fact`（结构化错误），不是静默沿用。
- **失败隔离与最后有效阶段**：某个 pass 因输入损坏/预算/取消停止时，`stages` 记录到该阶段为止的 `Completed`/`Partial`/`Failed`，**不发布**半初始化 facts（与 1.1 `StageResult` 语义一致）。
- **计费**：每个 pass 按其 `budget` **声明的维度集合**先计费后分配；pass 边界是 `poll()` 检查点。声明的集合与实现实际计费的维度必须**逐项相等**——多计（未声明的维度被收费）或少计（声明的维度漏收，例如 raw CFG 漏计 `AnalysisSteps`）都是缺陷，由每条 pass 的用例断言该集合。
- **失效后的重算者要指定**：`Frames` 由 `frame` 重算，`Ssa` 与 `Effects` 由 `ssa` 重算——「必须重算」没有指定落点就只是一句空话。
- **invalidate 未产出的事实是 no-op**：状态停在 `NotProduced`，其缺失随后以 `ir_pass_prerequisite_missing` 报出（不是 `ir_stale_fact`）。
- **检查范围分工**：phase 顺序与成环是**整表**性质（不在被调度前缀里的环也报错）；缺前置只判**被调度前缀**（前缀外的悬空 `requires` 不算错）。
- **事实的消费者必须把该事实写进自己的 `requires`**：否则 invalidation 检查对它不生效。当前 3.4 读取 `raw.effects` 却未声明 `Effects`，0.3 必须补齐并以 Effects 为 stale/未产出的反例证明入口拒绝。成功发布阶段时必须保留实际 facts 供下一 pass 使用；不能只在 ledger 标记存在而丢掉 payload。
- **0.3 收紧预算声明**：复用现有 `Blocks`（IR 存储/边）、`Steps`、`Clones` 集合；3.4 至少声明 `[Blocks, Steps]`，3.5 为 `[Blocks, Steps, Clones]`，Frame/SSA 为 `[Blocks, Steps]`。这里声明可能计费的维度类别，不要求无 clone/无边/空方法也产生非零消耗；不能用零工作样本证明漏计正确。计费表须逐结构明确单位，边只在实际创建时计 `IrEdges`。
- **重入语义**：同一执行内重新施加某 pass（3.5 在更大克隆预算下重试、或 5.1 的失败重装配）是合法操作——重入前必须重新满足 `requires`，其 `invalidates` 照常生效；3.5 的默认行为仍是「超限即停 + fallback」，重试由调用方以更大预算重新发起。`last_completed` 的语义是**已完成的最高 phase**（重入不得使其回退）——5.1 装配 `stages` 时以它为准，而不是「最后施加的那个 pass」。
- **phase 命名**：`LegacyNormalization`（3.4）产 `CallContexts`，真正的克隆规范化发生在 `CanonicalCfg`（3.5）；不要把 `legacy_normalization` 读成克隆 pass。
- **运行期复用同一记账**：3.3 起每个 pass 的入口必须走 3.2 的 `FactLedger::apply`（先全量检查 `requires`、再记 `invalidates`、再 `produces`、最后推进 `last_completed`），**禁止**另写一套事实记账——否则启动校验与运行期检查会各自漂移，`ir_stale_fact` 也就失去意义。
- **计费语句在本片只有声明**：3.2 只建立 pass 表与校验，`IrItems`/`IrEdges`/`AnalysisSteps`/`NormalizationClones` 的真实计费点从 3.3/3.4/3.5 起出现；在此之前 `analyze_method` 的 counted usage 全零（1.1 语义不变）。
- **本表的属性 vs 通用规则**：固定表满足"一 phase 一 pass、前缀即 pass 前缀"，由金标单测钉住；校验器本身只拒绝 phase **降序**（同 phase 多 pass 合法），这样 3.3–4.x 若要给一个 phase 拆两个 pass 不必改校验。

### 3.3 raw CFG 与 effect facts

- **形状（crate-private）**：
  ```rust
  pub(crate) struct RawCfg {
      pub(crate) blocks: Vec<RawBlock>,          // 按 class offset/BCI 升序，入口为首块
      pub(crate) edges: Vec<RawEdge>,            // 按 (from, kind, to, throw_ordinal) 排序
      pub(crate) throw_sites: Vec<ThrowSite>,    // 指令级：method_bci → handler(s)
      pub(crate) handlers: Vec<HandlerFact>,     // 保护区间 + handler 序（原始顺序）
      pub(crate) unreachable: Vec<u32>,          // 真值表：入口不可达的块 BCI
  }
  pub(crate) enum EdgeKind { Normal, Exception { handler_ordinal: u32 }, SubroutineReturn { call_site: u32 } }
  pub(crate) struct EffectFacts { /* locals 读/写集合、stack delta、可能的 throw */ }
  ```
- **边的身份**：普通转移按 (块, 不同目标) 一条边（条件分支的目标等于自身 fall-through 时**只有一条**）、异常边按 (块, 异常表记录) 一条边（两个 throw site 落在同一记录上只发一条；两条记录共享同一 handler 入口时**平行边保留**）、subroutine 返回边按 call site 一条。规则写死是为了让「一个转移被记成两条边」可以被判为缺陷而不是读法差异。
- **指令级 throw 语义**：`throw_sites` 必须逐个 throwing instruction 记录（不能只取块尾指令），并按**异常表声明顺序**给出该点可行的 handler 列表；同一 BCI 的多条异常边都要保留（平行边，petgraph `Graph` 多次 `add_edge`）。
- **不可达与自环**：不可达块进 `unreachable` 而不是被丢掉；自环（`goto` 指向自身、保护区间覆盖自身）必须可表达且不破坏 SCC/支配结果。
- **确定性**：块/边/throw site/handler 一律按 (class offset, BCI, kind, ordinal) 显式排序；**不得依赖 petgraph 的迭代顺序或 `immediately_dominated_by` 的顺序**（3.1 的证据）。
- **A17/P1 不变**：raw CFG 只在 `analyze_method` 路径上构建；`Engine::query` 的 BCI、引用数量与证据不变（P1 golden 全绿是证据）；`query`/`xref` 不引用 `ir`。
- **计费**：块/边先计 `IrItems`/`IrEdges` 再构造；工作列表迭代计 `AnalysisSteps`（即该 pass 的 `budget` 集合为 `[Blocks, Steps]`）；超限即停并发布已完成的阶段结果。
- **块上限的落点**：`max_blocks` 默认 16 384、硬上限 65 535 是 `cfg` 的 **crate-private 常量**，不是新增的请求级 limit（1.3 的维度表已定稿，不为它加字段）；请求级控制是 `ir_items`——超限以 `BudgetExceeded{IrItems}` 形式停止，请求级与内部护栏各司其职。
- **不完整方法体**：`RawCfg` 带 `completeness`（`Complete` / `Truncated{stopped_at}`）。截断体上只覆盖**可靠前缀**，且**分两种**：截断且目标无法校验（1.2 的 sound-but-incomplete 视图）⇒ `Partial` + `ir_raw_cfg_incomplete_body`（**不报损坏**）；截断但前缀自洽 ⇒ `Partial` + reader 自己的 stop code（不额外加诊断）。完整体上同样的校验错才是 `Failed{code}`。这正是 1.2 登记的 `stopped_at` 债务的处置点。
- **`jsr` 在原始图里不展开**：`jsr`/`jsr_w` 出 `SubroutineReturn{call_site}` 进子程序，`ret` 无后继并结束其块，call site 记入 `unresolved_returns` 交 3.4；可达性把「可达 `jsr` 的后继块」也算可达，因此 `unresolved_returns` 非空时 `unreachable` 是**欠报**（不是谎称死块）。
- **`jsr` 所在块与续块按「包含该 BCI 的块」查**，不按「块起始 BCI 相等」查：`jsr` 可以出现在块中间（前面是同块的普通指令），此时它的续块仍是**该 jsr 所在块**的后继。BCI→块的查法在 `cfg` 与 `call_context` 之间必须是同一种（`partition_point`：最后一个起始 ≤ bci）。用精确相等（`binary_search_by_key` + 失败即 `continue`）会在 `jsr` 非块首时静默丢掉这条关系，使**活调用点的续块被真值表判为不可达**，进而让 3.4 跳过「可达子程序体无 `ret`」的拒绝、对非法字节码报 `Established`，并把活调用点误列为 `unreachable_call_sites`。3.3 的既有证据（ECJ 语料与 `jsr_returns_stay_unresolved_…`）恰好都把 `jsr` 放在块首，所以该缺陷零覆盖；修正必须配一条**非块首 `jsr`** 的 3.3 回归与一条 3.4 回归（ECJ 45–48 的 `unreachable` 在此修正后由 `[8,11,15]` 变为 `[11,15]`，这才是 JVMS 语义：`ret` 返回到 8）。
- **catch 类型不在本层过滤**：throw site 的 handler 列表 = 保护区间覆盖该 BCI 的记录（声明顺序），**不做 catch 类型匹配**——那需要类型层次（resolver），`cfg` 不得依赖。空 handler 列表也要记录（「此处无人捕获」是事实）。
- **本片新增的报告级诊断码**：`ir_pass_not_implemented`（请求的阶段尚无实现）、`ir_raw_cfg_incomplete_body`（截断体的图不完整）、`ir_method_declared_without_body`（abstract/native 是事实，Info 级）。
- **reader 事实的补充**：`MethodCodeFacts` 增 crate-private 的 `exception_handler_count`（复刻 `BytecodeInspection` 的既有字段），只为 coverage 能命名「未读 handler ordinal 区间」。
- **`wide` 缺口**：当前 3.3 不识别 wide load/store/ret 的有效 opcode；0.2 修正 reader 后同步分块、终结指令与 effects，重新跑对应反例，之后才接受 3.4。
- **载荷的可见性**（不变量 11）：raw CFG 与 effect 载荷建完即在本片内部使用，公共面只暴露状态/覆盖/诊断；消费者是 3.4/3.5/5.1。

### 3.4 raw returnAddress 与调用上下文

**状态**：工作区候选尚未通过本轮 review。R2/R3/R4 是继续到 3.5 的阻塞项；现有 topology walk 可以保留为骨架，不能把其 `Established` 当作已证明的 returnAddress 数据流。

- **值来源**：对每个 `jsr/jsr_w` 创建原始 call-site/return-BCI token，沿 operand stack 与 locals 的存储、覆盖、合流追踪。`ret n` 的后继取自 local n 中已证明的 token；不能仅凭 ret 可达于某个子程序就归属该上下文。未定义、被普通值覆盖、错误槽位、不支持的传递或不可靠合流均不发布 `CallContexts`，保留 raw facts 与明确的 unresolved/fallback。
- **实现边界**：在同一指令事实/effect 适配上做有界的 returnAddress 专用数据流，只区分返回地址来源及必需的栈形状/其他值；不提前交付 4.x 的完整 Frame，也不另写 decoder。无法证明安全的指令形态先明确 fallback。
- **数据流按调用上下文分别进行**（这条决定验收能否通过，务必照此实现）：共享子程序在**每个上下文里各自分析一次**，所以不同调用点的 token 永不互相合流——ECJ 45–48 的 `finallyPath` 正是这个形状（两个 `jsr` 指向同一入口、子程序把返回地址存进同一个槽），它不是合流冲突。合流冲突只可能出现在**同一上下文内**的不同路径（例如 handler 路径改写了返回地址槽后与正常路径在同一 `ret` 汇合）；只有这种同上下文冲突才算「不可靠合流」并停止。若把跨上下文误当合流冲突，共享子程序的验收案例会被整体拒绝。
- **合流判定用 token 身份**：同一槽在两条路径上持有**同一个** call site 的 token 是可接受的合流；持有**不同** call site 的 token、或一条路径是 token 而另一条是普通值，才算冲突。共享及嵌套调用的 token 与调用链分开；嵌套 continuation 必须有实际返回证明，不能无条件排入正常路径。
- **异常路径**：逐 throw-site、handler ordinal 和 active call context 传播。handler 入口清空原 operand stack 后压入异常值，并保留该点 locals；handler 不是普通 fall-through，但也不能整类跳过。若 handler 在当前子程序中继续或回接 ret，其 local 读写与返回地址变更必须参与分析；跨出上下文或无法判定归属时 fallback。嵌套上下文影响传回外层时保持来源，不能把异常路径压成保护区间 ordinal 集合即认为完成。
- **locals**：分别保留分析需要的 accessed/written 信息（category-2 包含两槽）。当前 `affected_locals` 若继续表示写集，名称和消费者需写明；不可把写集当成 ret 状态合流所需的全部访问集。3.5 不得恢复/丢弃实际上已访问或改变的槽。
- **dialect 与可读性**：classfile 51+ 的 jsr/jsr_w/ret（含 wide ret）为违规；保留取证字节和 BCI，verification 仍为 NotPerformed。不可达指令的 dialect 检查与可达上下文证明分开，不能仅因现代方法使用合法 wide load/store 就 unresolved。
- **停止与发布**：有缺口/不支持的值流返回 `ir_call_context_unresolved`，Warning、Partial/Error、Fallback，且不发布 `CallContexts`；预算/取消保持各自终止原因。图与 facts 不一致才用 `ir_call_context_inconsistent`。只有所有可达返回转移与异常状态均有证明时才 Established；原有“五类固定触发”不足，改由上述不变量约束。
- **不可达边界逐触发声明**（不得再用一句“只作用于可达部分”概括，实测该句零覆盖且各触发作用域不一致）：返回点不在已解码前缀 ⇒ 只对**可达**调用点拒绝，死调用点的缺失返回点作为 payload 事实发布（不得让它阻塞活调用点的分析）；子程序体无 `ret`、可达 `ret` 无归属 ⇒ 按可达判定；嵌套成环 ⇒ 只对活在路径上的环拒绝；`wide` 缺口 ⇒ 按 0.2 之后取消整方法粒度的拒绝。每一类各配一条 fixture，并同时断言**反方向**（死代码里的对应形态必须仍 Established），否则删除可达性判定的变异不会被任何测试捕获。
- **Established 的载荷不变量写成等式而非叙述**：`contexts.len()` 等于已解码前缀内 `jsr`/`jsr_w` 的站点数（不按可达过滤）、`returns.len()` 等于前缀内 `ret` 数、每个 `ret` 的 `targets` 是**证明持有其返回地址**的那些调用点；`targets` 为空时 3.5 MUST NOT 据其建边。
- **计费**：0.3 为 plans、(context, block/BCI) 状态槽、visited/worklist、token/return 关系、local 集合成员、handler 关系和输出 origin 制定单位，在增长前计 `IrItems`；真实派生边计 `IrEdges`，传播/合流计 `AnalysisSteps`，建表与最终装配均有 poll。共享/嵌套导致的乘积项按实际数量计费，不能只给外层 vector 或输入 bytes 计一次。无克隆时 `NormalizationClones=0`；crate-private 不豁免预算。
- **验收**：正确 ret 与错槽/覆盖的双侧对照、共享/嵌套和 handler 回接 ret、不同 throw-site locals、wide/版本边界、零/恰好/超限存储与步骤、取消和不可达边界。至少一份真实历史 finally 语料；合成 fixture 断言实际 opcode/operand 及运行状态，不能只断言载荷形状。将本轮反例转成仓内回归，并证明恢复旧实现会失败，独立复核后才能勾选。

### 3.5 有界 jsr/ret 规范化与 CanonicalCFG

进入条件：0.2/0.3 和修订后的 3.4 均验收；依赖真实保留的 CallContexts，不重新推测返回点。规范化同步维护每个 throw-site 的 handler/context/origin。raw 图按 (block, handler ordinal) 聚合的异常边只作结构表示，不能取代值流输入。

- **克隆语义**：一个子程序被 N 个调用上下文共享时，为每个上下文克隆其块集合（`NormalizationClones` 按克隆节点计费）；克隆块保留 **一对多 origin**：`OriginMember::MethodPoint { method, bci }` 指向原始 BCI，且 `OriginSet` 保留全部原始 BCI（不得只留一个）。
- **超级块/边**：`ret` 在规范化后按其上下文确定后继；异常边按原始 handler 序重建；保护区间按上下文映射到克隆后的块范围。
- **界与 fallback**：克隆数超过 `NormalizationClones` 上限、步骤耗尽或上下文无法唯一确定 → **停止**并返回 raw 字节码 + `quality = Fallback` + 原因诊断，**不得**用线性替换伪装语义完整（`ir_legacy_normalization_unbounded`）。
- **PostCondition**：CanonicalCFG 的每个块都可通过 origin 映射回原始 BCI（含克隆块的一对多），且不存在未被任何上下文覆盖的孤立克隆。
- **验收（3.5）**：真实历史 finally（ECJ 45–48 语料，含共享子程序）、嵌套子程序、异常路径上的 `jsr`、恰好/超界克隆、取消与预算停止、以及"规范化不改变 P1 XRef 次数"的对照。

## 4.1–4.3 契约：Frame、未初始化值与 stack/local SSA

### 4.1 描述符驱动的 Frame（缺 debug/StackMap 也能算）

- **输入**：`CanonicalCFG` + 每个块的入口状态（locals 槽类型、栈形状、异常入口的 locals 快照）。指令语义来自 `InstructionOperands`（1.2）与一份按 opcode 的稠密表（push/pop 类别与数量），**不从展示文本恢复语义**。
- **类型格（crate-private）**：值状态需区分不可读取的 `Top`、`UninitializedThis`、带 new-site 的未初始化引用、基本类别、Null 与有 loader 身份的引用；category-2 值占两槽并显式标识第二槽不可独立读。引用信息不足与 `Top` 分开，保守未知引用不能被当成任意基本类型。先在现有私有 Frame 表示中落实这些区别，不要求为每个区别新增公共实体。
- **合流**：operand stack 要求深度/类别兼容，冲突返回 `ir_frame_inconsistent`；locals 允许不兼容或未定义的槽合成不可用 `Top`，之后读取 Top 才失败，不能拒绝只在已死亡 local 上不同的合法路径。category-2 的任一槽被覆盖会使原双槽绑定失效；Null/引用和缺失依赖的合流保持保守，引用身份含 defining loader。
- **不变量（本地）**：每个块的入口状态 = 前驱出口状态的合流；栈深在 JVM 上限内；`dup`/`swap`/`pop` 族按类别配对（category-2 的 `dup2` 语义必须显式覆盖）；`invoke*` 的参数量与返回类型由 descriptor 决定；`<init>` 返回描述符为 void，初始化转换由成功的 `invokespecial <init>` 对 receiver 的作用触发，不来自返回值。
- **`verification` 恒为 `NotPerformed`**：本片只做本地不变量，`semantic_validation` 先 `Unproven`，在 4.3 有不变量证据后可升为 `LocalInvariants`。缺 `StackMapTable`/`LineNumberTable`/`LVT` 不构成失败理由（按契约从 descriptor 与数据流推导）。

### 4.2 未初始化值、handler 入口与引用合流

- **`new`/`<init>` 链**：new-site token 沿 dup/astore 等保留别名。适用的 `invokespecial <init>` 正常完成后，将当前 Frame 的 locals 与 stack 中该 token 的**全部别名**转为已初始化引用；不同 new-site 不合并。异常后继不能复用正常完成后的初始化状态，按该指令的异常规则处理，证明不足则停止而非猜测。
- **`UninitializedThis`**：构造器入口 token；适用的自身/父类构造调用正常完成后，同步所有别名。允许的初始化前访问按具体 opcode 与所属类判断（例如对当前类字段的受限 putfield），不以“所有使用都非法”概括规范。
- **handler 入口**：输入取自每条 throwing instruction 的 locals/effect，栈只有异常引用。raw CFG 可能把同一 block 内多个 throw sites 聚为一条异常边；Frame/SSA 的逻辑输入仍以 `(raw edge, throw-site, normalization context)` 区分。不得因为 petgraph 边数相同就抹平不同 BCI 的状态。handler 声明顺序保留，类型匹配缺信息时保守保留候选。
- **4.1/4.2 必需反例**：分支分别给死亡 local 写 Int/Reference 后合流应可分析；随后读取该槽应拒绝；new/dup/astore/构造调用后的多个别名应一致初始化，构造调用异常后继不得套用正常状态；同 block 两个 throw-sites 写入不同 local 值后进入同一 handler 的输入应区分。
- **保守保留**：缺依赖/未知引用以保守的未知引用状态保留；栈形状或返回地址来源无法证明时明确停止，预算耗尽始终返回相应终止原因，不把预算停止改写为 Unknown 后继续。

### 4.3 stack/local SSA、phi 与 effect 顺序

- **形状**：每个块入口的 Phi 按逻辑值流 predecessor 取值：普通转移按实际 CFG 边，异常转移按 4.2 的 throw-site/context 输入；locals 与 stack 分别建 SSA（stack 在块边界按栈形状对齐）。不可读取的 Top 槽不制造可用值或伪定义。
- **不变量**：每个 value 恰有一个定义（phi 是定义）；每个 use 都能追溯到定义（def-use 双向一致）；phi 输入数 = 该槽实际参与合流的逻辑 predecessor 数（异常输入不能按聚合后的 raw edge 数计算）；phi 的类型 = 输入类型的合流（与 4.1 同一规则）；category-2 值的两槽在 SSA 里作为一个值处理。
- **origin 与 effect 顺序**：每个 SSA 值带 `OriginSet`（`MethodPoint` 指向产生它的指令 BCI）；effect 顺序按**指令级 throw site** 记录（异常边上的 effect 属于该 throw site，不属于块尾）；规范化克隆产生的值保留全部原始 BCI。
- **计费与停止**：`IrItems` 按 frame 槽、SSA 值、phi 输入、origin 成员计；`IrEdges` 按 CFG 边与 def-use 边计；工作列表迭代计 `AnalysisSteps`；停止时保留**最后有效阶段**（`stages` 到该阶段为止），不发布半初始化 facts。
- **4.3 验收**：diamond/loop/不可约控制流/异常合流各一组；高扇出 phi（多 predecessor + 多异常边）与多槽位样本验证 `IrItems`/`IrEdges` 上界；矛盾输入返回 `ir_frame_inconsistent` 或 `ir_ssa_inconsistent` + 最后有效阶段；`verification` 始终保持 `NotPerformed`；5.3 的 fixture/oracle 证据只能说明被测样本，不能把生产报告升为完整 verifier 已执行。
## 5.1–5.4 契约：库/CLI 接通、入口计数、golden/fuzz 与归档

### 5.1 方法分析库与薄 JSON CLI

- **库入口**：`Engine::analyze_method` 从 1.1 的诚实不可用变为真分析，按顺序执行 3.2 的 pass 表（到请求阶段为止）：`RawFacts → RawCfg → (LegacyNormalization → CanonicalCfg) → Frame → Ssa`。
- **五平面各自独立**（不变量 7）：`representation`（`Bytecode`；`Canonical` 只在 3.5 成功时出现）、`quality`（`Conservative`/`Fallback`）、`syntax_status`（`NotJava`：P2 不恢复 Java）、`compile_status`（`NotAttempted`）、`semantic_validation`（无证据时 `Unproven`，有本地不变量证据时 `LocalInvariants`，差分证据归 5.3）、`verification`（**恒 `NotPerformed`**，直到 5.3 的差分证据单独支撑）。
- **body 状态**：`MethodBodyState::{NotInspected, Present, DeclaredWithoutBody { no_body_kind }}`；`abstract`/`native` 方法没有 Body 是**事实**而非失败（`stages` 全 `NotPerformed`、`representation = Bytecode`、`execution = Complete`），诊断说明原因。
- **失败隔离**：同一类里正常方法与失败方法并存时，各方法报告互不影响（一个方法的 `Failed`/`Partial` 不改另一个的 `Complete`）；类级 Header 失败不伪造任何方法结果。
- **CLI**：`jarde-cli` 增 `method` operation（薄转发，JSON 形状与库报告逐字段一致），沿用既有的单请求/单响应与错误码约定；协议错误仍是 transport 级 `error`，报告内的停止仍在成功响应内的 `Partial`/`Cancelled`。
- **验收**：库/CLI 对同一请求的报告**逐字段一致**（可序列化对比，除 `elapsed_millis`）；`Bytecode`/`Conservative`/`Fallback`、`NotJava`、`NotAttempted`、abstract/native、阶段 coverage/execution、成员失败隔离各至少一条端到端用例。

### 5.2 入口计数：只读需要的字节（A14/A16/A17）

- **计数口径**：在**真实入口**（`analyze_method`）上记录每个阶段的读取与构造次数：Header 读取（`ClassHeaders`）、Body 读取（`MethodBodies`，此片起成为真实计费维度）、以及解析/CFG/SSA/Region/AST 构造计数。
- **必须证明为零的项**：单个方法分析不得加载无关 Body（`method_bodies` 只计目标方法）；P1 的 X0/X1 路径不得启动 resolver/CFG/SSA/Region/Java AST（构造计数为零，且 `Engine::query` 的输出与重放名单逐字段不变）。
- **A17 守卫必须覆盖图算法依赖**：3.1 已证实「在受守卫文件里 `use petgraph::…` 并构图」能编译且不被现有 token 表捕获（守卫位于 `tests/p2_contracts.rs` 的 `p2_tokens_in`，其外部 crate 面的清单是 `A17_IMPORT_TOKENS`）。3.3 首个消费者落地时，把 `petgraph::`、`petgraph as`、`extern crate petgraph` 三个 token 加入 `A17_IMPORT_TOKENS`（`petgraph::` 同时覆盖 `use petgraph::algo::…` 与全限定路径 `petgraph::graph::Graph`），并用「注入 `use petgraph::…` → 守卫测试转红」证伪；5.2 的构造计数是这条性质的行为侧证据，两者都要有。
- **确定性**：同一输入重复运行两次，报告的**身份与顺序逐字段一致**（除 `elapsed_millis`）——这同时是 3.1 那条「所有输出按 (物理定义, BCI) 自排序」的可证伪点。
- **验收**：无关 Body 为零、X1 构造计数为零、重复运行一致，各一条可证伪用例（用变异证明断言有牙齿）。

### 5.3 P2 golden、性质与有预算 fuzz

- **固定 replay 名单**（golden）：`tests/fixtures/` 下的 45–52 历史 class（含 ECJ 语料的 `jsr`/`ret` finally）、缺失依赖/debug 的样本、非法版本样本、共享子程序样本、异常重叠样本，以及资源边界样本（超预算、深链、高扇出）。每条记录**执行状态与阶段不变量**，不只记录形状：`stages` 的 `Completed`/`Partial`/`Failed`、`quality`、`representation`、`verification`、`coverage`、以及 origin 映射（克隆块的一对多）都必须可核对。
- **性质测试**：对生成的合法/非法方法体断言阶段不变量——blocks/edges 覆盖可达集、SSA 的 def-use 双向一致、phi 输入数 = predecessor 数、origin 可回溯到原始 BCI，以及停止可解释（每个 `Partial`/`Cancelled`/`Failed` 都有对应诊断与已发布前缀）。
- **有预算 fuzz**：复用维护后的 harness（P1 的 `exercise_query` 模式与 `observe` 钩子），对方法分析入口断言**状态与阶段不变量**，而不只是不 panic；语料包含 3.4/3.5 的 `jsr`/`ret` 与异常重叠形态。
- **验收**：golden 名单可复现、性质测试有变异证伪、fuzz 在固定时长内对不同输入断言不变量且不 panic；新增 harness 与 P1 的既有门禁共存（不替换 P1 的 fuzz target）。

### 5.4 文档、门禁与归档

- 同步五维支持矩阵（resolution 从 Partial 到完成、decompilation-quality 从 NotImplemented 到 P2 的实际能力、output-level 增加方法分析报告）、README、`openspec/acceptance.md` 的 A09/A10/A11/A13/A14/A16/A17 行与 `docs/` 既有文档；**不得宣称 Java 恢复、Region 分析或 verifier 通过**。
- 运行并记录：`cargo fmt --all -- --check`、`cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`、`cargo test --workspace --all-targets --all-features --locked`、oracle（JDK 25，ignored 用例）、MSRV 1.88、两个 workspace 的 supply-chain、**规定时长 fuzz**（既有 smoke 与新增 target 各跑满规定秒数）、`openspec validate --all --strict`、`git diff --exit-code`。
- 归档前置：把本 change 的 spec deltas 同步进 `openspec/specs/`（`analysis-contracts` 的 Purpose 修改、`demand-resolver`/`jvm-ir`/`conservative-output` 的新增能力），记录精确 commit 与 CI run、以及各片的只读复核结论；确认验收映射表里 A09–A11、A13、A14、A16、A17 全部从「部分」变为「通过」且各有指向验证记录的证据链接。

### 5.1–5.4 的风险与失败后果

- **只读需要的字节**是本 change 的边界承诺：若 5.2 的构造计数发现某阶段越界（例如 resolver 启动时枚举全 scope 的 Header），正确处置是**收窄该阶段的触发条件**并把成本计入下一个阶段，而不是放宽计数口径或删除断言。
- **golden 名单如果只记录形状**（例如只断言 `stages` 长度），3.5 的 origin 一对多与 4.3 的 phi 不变量就会失去回归保护——5.3 的验收明确要求状态与不变量，缺一则视为未完成。
- 归档时若 `analysis-contracts` 的 Purpose 仍写 P1 口径，后续读者会把 P2 的行为当成未文档化的偏离；该 Purpose 修改是 5.4 的显式前置。
## Risks / Trade-offs

- [Risk] frame/phi/origin 或 jsr 克隆乘法膨胀 → 分配前计费及高扇出/多槽位用例；P1 输入有界不代替 IR 上界证明。
- [Risk] provider 或 loader 不完整 → 状态、coverage 和 open-world 分开，唯一已知候选不证明唯一运行目标。
- [Risk] 库算法调用期间不能立即取消 → 明确规模上限和可观察取消粒度，不承诺硬 deadline。
- [Risk] 过早恢复源码吞掉缺口 → P2 固定 Bytecode 交付，先验证 IR，不引入 Region 或伪造 Java。

## Migration Plan

本次 review 仅更新规划与证据，保留当前未提交 3.4 代码。下一轮按以下闸口实施；旧任务的勾选和 Approve 记录保留原时点，不作为新反例已经关闭的证据。

1. **0.1 resolver 身份修正**：关闭 R1，重跑 2.1–2.5 的 loader/声明/dispatch 测试；同时固定 driver/caller 的环境绑定，不再让 D10/D15/D25 延后到产品装配。
2. **0.2 reader 与 CFG facts 修正**：关闭 wide/数组维数等确定缺口，给 3.4/4.x 提供唯一操作数事实来源；重新确认 3.3 的分块/effect 与 P1 对照。
3. **0.3 预算与 Pass 依赖修正**：关闭 R4 和 Effects requires 缺口，明确后续阶段的存储/边/步骤/clone 计费，不提前实现 3.5/4.x。
4. **3.4 值流与异常上下文**：关闭 R2/R3，四个本轮探针转为仓内回归；真实语料、失败路径和独立复核通过后才能交接 3.5。
5. **3.5 → 4.1 → 4.2 → 4.3**：先有界规范化，再按修订后的 Top/初始化别名/指令级异常输入契约建立 Frame/SSA；不得以 test oracle 替代 verifier。
6. **5.1–5.4**：库/CLI、真实读取与构造计数、golden/fuzz、文档及全门禁；P3 仍以 P2 整体出口为前置。

每片提供最小反例与旧行为失败证据，记录对应 commit/CI run；本轮工作区的本地门禁不能冒充 HEAD 的 CI 证据。不为“记录文档提交自己的 CI”循环提交。P1 的维护已在独立归档关闭，不重新打开或混入上述修正。

### 保持拆分的既有债务

重复 unit/Code 物化和 Type-only 二次解码继续归 P5；has_more=true/cursor=null、record descriptor 类别、Entry.span、MR/tree aggregate 优先级和冗余 allow 等分别维护，不顺便清理。P2 通过直接 Header、显式停止状态和 class/BCI origin 避开这些依赖；实际受阻再以最小独立 change 修正，不能悄悄改变 query 契约。

`query-api` 主规格写的"`references_definition` 由 P2 的 resolver 处理"在本阶段不兑现：P2 只提供独立的、显式运行环境的解析与声明引用入口，`Engine::query` 的关系语义保持 `UnsupportedAnalysis`（不变量 12）。把 query 接线到 resolver 需要先改该主规格与 `QueryResolution`，属独立变更。5.4 须消除主规格的过时阶段承诺：`query-api` 保持 unsupported 行为但指向独立声明查询入口，`analysis-contracts` 的 Purpose 直接编辑并复核；不得借文档同步接入运行时解析。本 change 归档同步三份新增 capability 和一份 analysis-contracts delta。
