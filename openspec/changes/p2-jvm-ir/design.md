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
| `src/ir.rs`（新） | 方法分析请求/报告、阶段与产物状态 | `artifact`、`budget`、`environment`、`classfile`（`VerificationStatus`）、`error`、`model`、`view` |
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
    DuplicateLoader, CallerDomainMismatch, MissingParent, ParentCycle,
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
3. 平面分离：`analysis`（能力是否运行）、`state`（语义判定）、`coverage`（范围）、`execution`（终止）、产物状态（representation/quality/syntax_status/compile_status/semantic_validation/verification）互不推断；语义状态集不含 `NotPerformed`，取消由 `execution = Cancelled` + `state = None` 表达，预算停止用 `state = BudgetExceeded` 并同时进 execution。
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

`CountedBudgetDimension` 增 `ClassHeaders`、`MethodBodies`、`IrItems`、`IrEdges`、`AnalysisSteps`、`NormalizationClones`；`BudgetDimension` 另增 `DependencyDepth`（高水位，不累加，不得用 `NestedDepth` 代替）。1.1 先补 `Limits`/`UsageSnapshot` 的构造器或分维访问器，使后续加维是增量；1.3 在同一变更内更新 CLI 请求 schema、goldens 与 fuzz 的 usage 断言，并说明这是 P1 证据的 additive 变化。计数单位：`ClassHeaders`/`MethodBodies` 计"实际读取尝试"，去重后的 (definition, loader) 绑定由结果身份表达。

### 1.1 的验收义务

- 新增可编译的最小 API 与至少一个 `examples/` 入口，构造并打印解析请求与方法分析请求的诚实结果；
- 反例：缺少 parent domain、parent 环、caller domain 不一致、provider root 未归属、内容未提供、未支持 module mode/loader policy → `EnvironmentProblem` + 同 code 诊断、无唯一解析结果、usage 的 counted 维度全零；
- 反例（A17）：`Engine::query` 的 physical X0/X1 不启动 resolver/IR（源码级守卫 + 未接线）；
- 正例：`representation=Bytecode`、`syntax_status=NotJava`、`compile_status=NotAttempted`、`verification=NotPerformed`、`quality`、`coverage`、`execution` 在同一报告里分别取值且互不推断。

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
