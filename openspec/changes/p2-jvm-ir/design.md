## Context

规划基线为 `8fcdd664b656c4e8cdf383f9c3756a95777ccf7b`，复核日期 2026-09-17。P1 已归档于 `../archive/2026-09-17-p1-query-xref/`，tasks 11/11；本 change 仍未实现。动机和能力边界见 proposal.md。

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

### P2 进入时的接口缺口

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
| `src/resolver.rs`（新） | 解析请求/报告、声明引用查询请求/报告 | 上述 + `budget` + `query::{ConsumerKind, ConsumerSchema, XrefOperation}` 词汇表 |
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
pub struct DispatchCandidate { pub member: ResolvedMemberRef, pub evidence: OpenWorldEvidence }
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
- 1.1 对合法请求的诚实状态：`analysis = NotPerformed`、`state = None`（解析）或 `stages` 全 `NotPerformed`（分析）、`execution = Failed { reason: Unsupported { code: "resolution_not_implemented" / "method_analysis_not_implemented" } }` + 同 code 诊断、三维 `coverage = NotRequested`、所有 counted 维度 usage 为 0（`elapsed_millis` 除外）。code 用能力名，2.x/3.x 落地后消失，相关测试随之退役。
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
- **有意不保留的操作数**：`newarray` 的 atype、`multianewarray` 的 dimensions、`invokeinterface` 的 count（3.x 若需要先改本节）；`immediate` 只表示 CP 条目或指令本身的字面量类型，不承担 ldc 变体与 CP tag 的配对合法性（那属 4.x verifier 领域）。
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

- **起点**：请求目标所在 loader（`CallerContext.loader`）与目标内部名。展开顺序：目标自身 → 其 parent/interface（若目标需要成员解析，2.3 触发）→ 显式范围（仅 2.5 的 dispatch）。
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

- 闭包键 `(loader, internal name)` 的 loader 分量在当前公开路径上**不可证伪**：一次请求只有一个搜索起点（`CallerContext` 不移动起点，1.1 又以 `CallerLoaderMismatch` 拒绝分叉），因此「只按 name 去重」的变异存活；同一防线的另两条（记录用定义所在 loader、记录去重）已被捕获。该分量的可观测条件是「出现第一个以非 `runtime.load_domain.loader` 发起需求的调用方」（可能晚于 2.5），届时应补一条让该分量可观测的用例。
- 层级展开（`ParentChain`/`HierarchyClosure`、`WalkGaps`、环诊断）在本切片无公开入口，语义由 `providers` 的 lib 单测固定，消费者是 2.3/2.5（相关项带 `#[allow(dead_code)]`，移除即产生 9 条 warning）；复核指出的四类未固定语义（`parent_chain` 不跟接口、损坏/读取失败不入记录、Ambiguous 每候选一条、停止的需求不被记忆）已由补测固定。
- 公开层只能构造预取消与预算停止；「两个 demand 之间被取消」由 lib 两层之间的用例证明。

### 2.2 的验收

- **深链**：父类链深度超过 `dependency_depth` → 终止维度为 `DependencyDepth`、保留前缀、`reads` 只含已读深度。
- **高扇出**：一个类实现多个接口且接口再继承 → 每条 Header 只读一次、`reads` 无重复 (definition, loader)。
- **缺失依赖**：闭包中某一层在全部 root 都 `Missing` → 记录该层状态、不伪造定义、`coverage` 标明未完成范围。
- **循环引用**：A→B→A 的继承环（非法 class）→ 不无限扩展（去重键终止）、给出可定位诊断。
- **预算/取消**：预取消与中途取消分别得到 `Cancelled`，`usage` 与 `reads` 一致（`reads.len() <= class_headers`）。
- **无关 Body 读取为零**：闭包请求后断言 `usage.method_bodies == 0`（除非显式请求目标方法 Body），且 `reads` 的 reason 集合不超过本次请求允许的理由。
- **同 bytes 不同 origin/loader 不合并**：同一 class 字节放在两个 loader 的 roots 下时，分别以各自环境请求会得到两次各自的选择与两条记录（单请求只有一个搜索起点，故「一次请求内两条记录」不是本片的形态）。
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
   每步都用 `ReadReason::HierarchyClosure` 读 Header（同 (definition, loader) 去重，2.2 的闭包机器）。
3. **判定**（与 `specs/demand-resolver` 的 7 状态对齐）：
   - 唯一命中（字段；或方法在 class/超类链上唯一）→ `Resolved`；
   - 方法只有接口候选时，取 maximally-specific 集合：**恰好一个非抽象** → `Resolved`（该 default 方法）；**多个非抽象**（Java 8 default conflict）→ `IncompatibleClassChange` + `resolution_default_conflict` 诊断；**全部抽象** → `Resolved`（那个抽象声明）+ Warning `resolution_method_is_abstract`（JVMS：解析成功，AME 发生在调用时，不由解析阶段伪造）；
   - 什么都没找到 → `Missing`（保留原始 `SymbolRef`/descriptor/origin，不伪造空成员）；
   - 同一步骤内多个无法区分的候选（同一类里同名同描述符重复声明）→ `Ambiguous` + 各自 origin。
4. **调用种类规则**（`ReferenceUse`；违反即 `IncompatibleClassChange` + 具名诊断）：
   - `InvokeStatic` 要求静态方法，`InvokeVirtual`/`InvokeInterface`/`InvokeSpecial` 要求实例方法（`<init>` 仅 `InvokeSpecial`）；
   - `FieldRead`/`FieldWrite` 中 static 指令（`GetStatic`/`PutStatic`）要求静态字段，实例指令要求实例字段——由 2.4/2.5 的 use-site 提供指令级种类时使用；
   - class method resolution 命中接口方法或 interface method resolution 命中类方法 → `IncompatibleClassChange` + `resolution_kind_mismatch`；
   - `InvokeSpecial` 命中抽象方法 → 同上（JVMS 5.4.3.3 对 invokespecial 的额外约束）。
5. **访问规则**（JVMS 5.4.4，与解析分开）：调用方类名由 `CallerContext.enclosing`（其 `owner` 定义读 `this_class`）得到；运行时包 = (loader, 包名)。`public` 通过；`private` 要求同类；`protected` 要求同类/同包/子类；包私有要求同包。**判定不通过 → `Inaccessible` + `resolution_access_denied` 诊断**；调用方类未知（无 `enclosing`）时**不做访问判定**，返回 `Resolved` + Warning `resolution_access_not_checked`（不谎称已检查）。
6. **明确的 unsupported 分支**：
   - **signature-polymorphic**（`java/lang/invoke/MethodHandle` 的 `invoke`/`invokeExact`，JVMS 2.9）：调用点 descriptor 与声明不同，故按 **name 匹配**解析到声明并返回 `Resolved`，同时给 Warning `resolution_signature_polymorphic`（`target` 保留调用点描述符、`resolved.member` 是声明描述符，两者都在报告里可见）；
   - **数组 owner**（`[` 开头，如 `[I.clone()`）：P2 不实现数组类型方法解析 → `UnsupportedPolicy` + `resolution_array_owner` 诊断（不伪造 Object.clone）。

### 2.3 的验收

- 合法/非法对照：静态 vs 实例（各一条 ICCE 反例）、private/protected/包私有/跨包与跨 loader 的访问对照（含"无 `enclosing` 时给 `Resolved` + `resolution_access_not_checked`"）、`<init>` 只经 `InvokeSpecial`、抽象方法解析成功但带 Warning；
- **Java 8 default conflict**：一个接口提供 default、两个接口各提供 default（conflict → `IncompatibleClassChange`）、抽象-only（`Resolved` + Warning）、子接口覆盖父接口 default（唯一非抽象 → `Resolved`）；
- signature-polymorphic 与数组 owner 各一条（含 `target` 与 `resolved.member` 描述符不同的断言）；
- 跨 loader：同一 owner 名在两个 loader 命中不同定义时，解析结果跟随 2.1 的选择（不合并）；
- 每条正例同时断言 items/状态/诊断/`reads` 的 reason 集合（`HierarchyClosure` 出现）、`usage.class_headers` 与"不读无关 Body"（`usage.method_bodies == 0`）。

## 2.4 契约：显式运行环境的声明引用查询（复用结构 consumer）

2.4 交付"找出**解析到该声明**的全部真实 use-site"：不能按 `SymbolRef` 的 owner 精确筛选（`Sub.foo` 的 CP owner 是 `Sub`，却解析到 `Base.foo`），也不能把未使用的 CP 条目当引用。

### 复用方式：给扫描机器加一个 crate-private 候选形态过滤器

- 在 `src/xref/mod.rs` 增 crate-private `CandidateFilter`：`Exact(QueryTarget)`（现有行为，`Engine::query` 用它）与 `MemberShape { name: JvmBytes, descriptor: JvmBytes }`（按原始 name/descriptor 字节匹配 `SymbolRef::Method`/`Field`，**owner 不参与**）。`ScanContext` 持有当前过滤器，`code.rs`/`metadata.rs`/`bootstrap.rs` 的 `use_site_answers`/`asks_symbol`/`target_matches`/`answers` 改为调用同一 `ctx.candidate_matches(..)`（`resource.rs` 只做 literal，不参与）。
- 新增 crate-private 入口 `xref::scan_candidates(snapshot, scope, consumers, filter, budget) -> CandidateScan { items, coverage, execution, diagnostics }`：复用同一 `ProviderScan` 与各 consumer 子扫描、同一计费与 ordering，但**不带分页/游标**（`DeclarationRefQuery` 用 `max_items` + `has_more` 表达截断，规则与 P1 页限一致：coverage `Partial`、execution 仍 `Complete`、不复用 P1 游标）。
- **A17 方向不变**：`CandidateFilter` 与 `scan_candidates` 都不引用 resolver/ir；`src/query.rs` 与 `src/xref/**` 仍不含 P2 记号（守卫测试继续成立）。`Engine::query` 的行为与证据逐字段不变（P1 golden 全绿即证据）。
- 每个候选的 `origin`/`consumer`/`operation`/`evidence` 直接沿用 P1 的 item 证据形状；resolver 侧只做"把候选的 owner 解析成声明并与目标比较"这一步。

### 查询语义

1. 候选来自结构 consumer（未使用的 CP 条目不是引用：`MemberShape` 只匹配**被 consumer 消费**的 use-site，因为过滤器作用在 use-site 判定上，不在池枚举上）。
2. 对每个候选：解析其 owner（2.1 的查找 + 2.3 的成员规则），把解析出的声明与请求的 declaration 比较（loader + 定义身份 + 成员符号）。**只有解析到请求声明的候选进入 `items`**（`resolved` 即该声明、`state = Resolved`）；解析到别的声明的候选计入 `scanned` 但不进 `items`，因此 `items` 的每个条目都满足"该 use-site 确实指向请求声明"这一可复核断言。
3. **未决候选不得当已排除**：候选的 owner 解析为 `Missing`/`BudgetExceeded`/取消/环境问题（或候选自身损坏）时计入 `unresolved_candidates` 并保留 `origin`，`coverage` 标 `Partial`，`has_more` 视截断而定；不得把它们算作"已排除"。
4. `max_items` 截断：按 P1 页限规则（`returned_items`、`has_more`、coverage `Partial`、execution 不因此变 Partial）。
5. **P1 语义不变**：`Engine::query` 的 `mentions_symbol` 仍按原始符号精确匹配（`Base.foo` 查询不返回 `Sub.foo` 的调用点），这是 A11 需要的对照证据。

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
2. **候选发现（CHA-lite，只读 Header）**：枚举 `scope` 覆盖的 Header（`PhysicalScope::SnapshotAll` 或 `ArtifactTree`），对每个类用闭包解析其超类/接口链；若链上包含声明的 owner 且该类**自身声明**了同名同 descriptor 的成员，则该成员是一个候选（实现或覆盖）。接口声明 → 候选是该接口的具体实现方法；类声明 → 候选是覆盖该方法的子类方法。**只读 Header，不读任何 Body**（`usage.method_bodies == 0`），受 `ClassHeaders`/`DependencyDepth`/`ResultItems`/取消约束。
3. **每个候选带 open-world 证据**（`DispatchCandidate { member, evidence }`，证据取自 `OpenWorldEvidence`）：`ExternalSubclass`（范围内存在 `LoadRoot::External` 或未提供内容的 root，可能有未见的子类）、`UnknownLoader`（`Domains` 之外可能有别的 loader 加载同一类）、`RuntimeTransformation`/`ExternalOverride`（`RuntimeUncertainty != None`）、`MissingDependency`（链上有 `Missing`）、`OrderedRoot { index }`（该候选来自有序 root 的第 index 个位置，提示同层可能有别的定义）。
4. **`open_world` 的判定**：只要出现上述任一证据（或范围内存在不可判定位置/预算停止）→ `open_world = true`；**单一候选也照样 `open_world = true`**，报告里没有任何"唯一目标"字段，调用方只能从 `candidates` + `open_world` + 证据自行判断。预算/取消停止时 `execution` 为对应 `Partial`/`Cancelled` 且 `open_world = true`（未知范围本身就是开放世界证据）。
5. **与 2.4 的边界**：声明引用查询回答"哪些 use-site 指向该声明"；dispatch 回答"该声明在该范围内的已知实现/覆盖"。两者不互相替代，也不共享结果缓存（各自独立计费）。

### 2.5 的验收

- **多实现**：接口 + 范围内两个实现 → 两个候选、各自 evidence（若范围完整且无 uncertainty，`open_world = false`）；再补一个 `RuntimeTransformation::Possible` 的同一 fixture → `open_world = true` 而候选集合不变（证明 open-world 是独立平面）；
- **外部子类**：环境含 `External` root → `ExternalSubclass` 证据 + `open_world = true`，即使范围内只有一个候选；
- **未知 loader/transformer**：`Domains` 未覆盖的 loader 或 `external_override`/`runtime_transformation != None` → 对应证据；
- **单一已知候选不声称唯一**：断言报告结构上不存在"唯一目标"（无该字段）+ `open_world` 语义；
- **只读 Header**：`usage.method_bodies == 0`、`usage.class_headers > 0`、`reads` 的 reason 含 `DispatchScope`；
- **预算/取消**：`ClassHeaders` 或 `ResultItems` 停止 → 保留已发现候选 + `open_world = true` + `Partial`；
- **回归**：`Engine::query` 的既有行为与证据不变（P1 golden），`resolve_symbol` 的类符号路径不变（2.1 证据），`dispatch = None` 的请求不产生 dispatch 相关诊断。

## Risks / Trade-offs

- [Risk] frame/phi/origin 或 jsr 克隆乘法膨胀 → 分配前计费及高扇出/多槽位用例；P1 输入有界不代替 IR 上界证明。
- [Risk] provider 或 loader 不完整 → 状态、coverage 和 open-world 分开，唯一已知候选不证明唯一运行目标。
- [Risk] 库算法调用期间不能立即取消 → 明确规模上限和可观察取消粒度，不承诺硬 deadline。
- [Risk] 过早恢复源码吞掉缺口 → P2 固定 Bytecode 交付，先验证 IR，不引入 Region 或伪造 Java。

## Migration Plan

先独立完成 V1/V2 验证维护，再按 tasks 的基础契约、resolver、raw/legacy CFG、Frame/SSA、产品验收逐片执行。第一轮只做 1.1–1.3，退出时证明 reader、预算、结果模型可用。每片记录命令/结果和只读复核结论，阻塞项修复后以定向反例复验，再开始依赖片。交接列明未完成项和允许修改模块，不一次派发整条管线。

P2 本轮只修改规划与过时的 OpenSpec 阶段上下文，不新增 P2 代码、不勾选本 change 的实现任务。P1 的 harness/CI/deny 修复及实际验证归 harden-p1-validation（已归档，见 `../archive/2026-09-17-harden-p1-validation/`）。CI 状态按具体 commit/run 查询，不追加“记录文档提交自己的 CI”的循环提交。

### 保持拆分的既有债务

重复 unit/Code 物化和 Type-only 二次解码继续归 P5；has_more=true/cursor=null、record descriptor 类别、Entry.span、MR/tree aggregate 优先级和冗余 allow 等分别维护，不顺便清理。P2 通过直接 Header、显式停止状态和 class/BCI origin 避开这些依赖；实际受阻再以最小独立 change 修正，不能悄悄改变 query 契约。

`query-api` 主规格写的"`references_definition` 由 P2 的 resolver 处理"在本阶段不兑现：P2 只提供独立的、显式运行环境的解析与声明引用入口，`Engine::query` 的关系语义保持 `UnsupportedAnalysis`（不变量 12）。把 query 接线到 resolver 需要先改该主规格与 `QueryResolution`，属独立变更，不在 P2 的 20 项任务内。P2 归档时只同步本 change 的三份 capability spec，不修改 `query-api` 的既有措辞。
