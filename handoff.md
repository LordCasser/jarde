# 2026-11-10 严格 Clippy 闭环

以实际当前 stable Rust/Clippy1.99.1 运行完整 workspace/all-targets/all-features 和既有 CI 的全部 -A/-D 配置，逐层修复33项原有错误，严格门现通过；仅用 --keep-going 汇总错误，未削弱检查。19项生产写法与14项测试写法保持行为/断言等价，例外表、固定输入、恢复72/schema14/解析6均不变。

同一源码通过受影响的688个原生测试：feature-enabled根层6项、p3_scope12项、shared prepared callee4项、Java库621项、S2b两套件45项。最大target869,353,187字节，默认栈、单任务、源码前后摘要一致，已清理。先前失败的三轮lint原流保留；更早1.88工具链结果不替代本次stable门。下一步按精确目标清单运行默认/all-features各双seed全量回归，重新归类实际失败，再推进剩余顺序/绑定机制。架构33/65及原始S0资源证据缺口不变。

# 2026-11-10 S2b 构造器 witness 与完整生成族

构造器-only Mid 的失败已实际复现：类 Signature 尚待继承证明，但没有成员 Signature，使用扫描未被请求；Required AST 又遗漏已有构造器 witness。现按待证明类 Signature 请求原有完整使用 census，并让已有完整 invocation AST 的消费者保留同次物理构造器 witness。未放宽 direct-parent、依赖缺失或 constructor consumer 证明。首次保存和随后 declared AST 克隆的 witness 字符串均在分配前计 IrItems，原 work/cancellation 检查保留。恢复语义 71→72，schema14/解析6 不变。

新增公共 Engine 控制覆盖 essential/all、实际 invokespecial BCI、缺 Parent/List、相邻 OutputBytes 拒绝/准入切点，并逐次核对 header、constructor text、retained emission 全有或全无。完整族运行门将 Parent、Mid、CustomChild、PlatformChild 四个 Engine 输出原样一起编译，classpath 起初为空，不复制原支持类；Driver 另行只对恢复族编译/执行，与独立原件一致。既有两目标门保持，固定源/类字节不变；本族是开发回归，不是独立 holdout。

实际通过：Java 库621/621、四个 Java S2b 套件50/50、Engine6/6（普通运行2 ignored）、显式完整类 JVM 门2/2、根库299/299、六个邻接泛型套件57/57、rustfmt、OpenSpec1.11.0 strict391/391。新真实构造器控制验证物理/declared AST witness、精确总 IrItems/N−1与预取消。all evidence 在文本提交后停止时，严格保留同一文本并报告 IrItems Partial，同时撤除可复用 AST；essential 的前提交停止无文本。新测试曾错误要求两种停止都无产物，已凭实际诊断和既有 evidence 合同纠正，未改生产停止行为。类似地，essential 不请求公共 SourceMap，但两种模式的 typed emission 均必须保留 BCI；预算相邻点不冒称全局首个切点。

两个最终原生批次使用相同源码摘要、Rust1.99.1/Temurin25.0.4+7、默认栈与单任务，最大 target822,921,875字节，均退出0并清理。严格 Clippy 的17项既有错误尚待独立等价维护；全 workspace两种features/双seed、其他显式JDK门和资源/holdout尚未总验收。架构清单保持33/65，S0原始观测产品/收据缺口不由这些开发控制替代。

# 2026-11-10 S2b 声明投影逐字重放

修复已投影泛型声明的 Signature 标记使后续 declared cohort 误拒绝原物理 body 的问题。Java 层按已有声明投影事实重放完整 member 文本，逐字核对声明、注解、标记、envelope 与 body；不先用忽略标记的旧比较器放行。已有 body 投影仍保持原所有权。根门面只转交事实；重放按既有暂存合同计 AnalysisSteps/OutputBytes，停止先于事务提交，不增影子状态。恢复语义 70→71，schema14/解析6 不变。

实际通过：原 S2b Engine 5/5；显式两目标完整类重编译运行门 1/1（CustomChild/PlatformChild 使用原 Parent/Mid 支持类，不冒称四类全部重新生成）；class_source 56/56；邻接泛型六套件 57/57。随后在官方校验的 Rust/Clippy1.99.1、Temurin25.0.4+7 上通过新增声明重放 2/2、Java 库621/621、四个 Java S2b 套件49/49。新增反例覆盖 body、方法 envelope、漏失/多余/过期标记、投影所有权、注解拼写和位置；精确 OutputBytes/N−1、零预算与取消均保留。只读审查无阻塞发现。

最新原生批次源码摘要前后一致、默认栈、单编译/测试线程，target 峰值812,415,315字节并清理。rustfmt 已通过。严格 CI Clippy 尚红：当前 stable 实测17项既有 lint 错误，均已核对存在于本修改前 HEAD，另行按等价维护处理，不添加 allowlist。早前1.88 Clippy结果不能代替 stable CI。此项不宣告全 workspace/CI 或架构阶段完成，33/65及S0资源缺口不变。下一步独立验证构造器-only Mid 与四类完整生成族，继而继续剩余机制和全量门。

# 2026-11-10 CHECKCAST 双向物理继承证明

CHECKCAST 收集器现在通过原有按身份去重/计费路径同时请求操作数→目标及目标→操作数关系；目标的真实父边可证明缩窄，缺失目标仍保守保留 Object 桥，运行时 cast 不删除。恢复语义 69→70，schema14/解析6 不变。

新增真实 MethodIr 测试和公共 Engine 测试覆盖普通/注解接口改名、宽 aload 导致 BCI1→4、缺失 Header、精确 AnalysisSteps 超限与取消。四组完整未编辑生成类与原始字节码隔离重编译/执行，null、身份和 CCE 均一致。原 nested-cast 两个 spelling pin 经关闭新 reverse 请求的真实反事实运行仍失败；完整类运行门通过后，改为严格钉住缺失 Number/Runnable Header 时的 Object 桥，保留两次 cast、调用次数和所有 BCI 映射，并增加 Runnable-only/Object 反例。没有把真实回归改成通过。

分批实际通过：Java 库 621/621；原 cast 5/5、collection 5/5、snapshot hierarchy 6/6、platform implementer 7/7、generic cast 1/1、array invocation 7/7；新增内部证明与 Engine 各1/1，两个显式 JDK 门各1/1。collection 原件运行首次因主机时区使 Date 字符串长度34而非28失败，现只固定 Java 时区 UTC，原 expected=28 未改。所有批次默认栈、单任务，源码冻结且 target 退出后清理；历史失败收据保留。S2b 声明标记/cohort 问题仍单独修复，架构资源缺口与33/65状态不变。

# 2026-11-10 测试输入恢复：全目标编译恢复，功能与架构验收继续

恢复 821 个实际测试依赖到 `tests/fixtures/recovered-inputs`（1,151,819 字节），逐项与历史 `8ea98e2e11d64f02092ac200d2b596511dcf5e40` 的 Git blob、长度和 SHA-256 比对一致。EX 原始 1,632 字节和原 SHA 不变。Rust 编译期引用、运行期回放输入及两个 EX comparison 默认路径改用永久测试目录；修正 `generic_source_proof` 的模块别名。另应用 rustfmt，不声称格式改动是语义修复。

三个从未跟踪、现不可取得的 S2b dev-v3 JAR 没有冒充原样恢复。新增 `s2b-inherited-generics` 明确固定新的 Corretto 8 源码/编译器/类字节基线，两次独立生成一致，原件 Driver 两次隔离执行通过；原有五测试及断言全部保留。新增显式 JDK 完整类运行门仍待实际执行。

本环境实际通过：默认及 all-features 的 `cargo check --workspace --all-targets`；Reader 184/184、根库 297/297、Java 库 621/621、EX 2/2；N8 的 member_family_identity 20/20、nested_type_source_spelling 6/6；指纹再生 1/1、校验 5/5（1 ignored）。指纹新增 833 项，既有 2,847 项零删除/零改字节。独立原始 Code 计数与 Reader 实测一致：1,898 classes / 8,352 bodies / 1,661 handlers / 6,725 branch targets / 18 jsr。全部使用默认 Rust 栈、单 Cargo job、单测试线程；最终库验证源码前后摘要一致，target 峰值 837,587,101 字节，未触守卫，退出后清理。

**仍红，不宣告全部完成**：S2b 原测试本次为 4 passed / 1 failed；CustomChild 的已投影 `echo` 声明标记使后续类级事务把原 body 误认为被改写，整个继承泛型 cohort 未提交，正按独立机制修复。全 workspace 行为套件、完整重编译门与 CI 尚未重新全部验收。主架构清单仍 33/65，不因恢复编译改勾。S0/S1/S3a 原资源收据与机器/输入固定证据仍缺失；已找到两个真实包的原 SHA，holdout 源文件尚在但不据此复用旧验收。旧本地未发布提交随先前环境丢失，本次以重新验证的新提交为准。

# 2026-11-10 证据清理、历史重写与仓库收敛

本轮按用户要求清理不适合进入 GitHub 的运行证据，并把本地与远端收敛到同一主线。清理完成时的 main 为 `9573be9e`；本提交只追加交接记录。

**证据清理**：删除未跟踪的 `openspec/changes/restructure-demand-driven-recovery/results`（约 23 GB）；从当前树和全部历史移除 `openspec/**/results/**`、`openspec/**/evidence/**` 下的非 Markdown 产物（约 18.6 万个文件）；保留 2,300 个 Markdown 结论摘要。`.gitignore` 继续忽略这些目录的运行产物，但允许 `.md`。`openspec/` 从 3.4 GB 降到 30 MB，`.git` 从 8.2 GB 降到 70 MB。历史已由 `git filter-repo` 重写，旧 SHA 全部失效；远端 `main` 已强制同步。

**分支与引用**：分析并删除 16 个 `codex/*` 分支（15 个已完全并入 main；1 个 `branch-throw-finally-entry` 是 10-17 的过时探针，未进入主线，且依赖已清理的 `EX.class`）。补丁暂存 `/tmp/jarde-codex-branch-throw-finally-entry.patch`；另删除 6 个 `refs/codex/snapshots/*` 与旧 `refs/stash`，补丁分别在 `/tmp/jarde-codex-snapshot-75969137.patch`、`/tmp/jarde-stash-ordinary-try-draft.patch`。当前本地与远端都只有 `main`。

**已知影响（未修复）**：清理后 `cargo check --workspace --all-targets --all-features` 失败（exit 101），多个测试仍用 `include_bytes!`/`include_str!` 引用已删除的 `openspec/evidence/...` 输入；当前主线 `p3_branch_throw_return_finally.rs` 也引用已删除的 `EX.class`。按用户要求未修复，后续需把这些输入迁入 `tests/fixtures` 或恢复对应文件。

**本地临时文件**：`docs/tmp-architecture-adversarial-audit-2026-10-20.md` 与 `docs/tmp-linus-lens-project-review-2026-10-28.md` 保留在本地，已加入 `.git/info/exclude`，不进入仓库。`/tmp` 下保留重写前 bundle 与上述补丁作为回退材料。

# 2026-11-07（续）机制修复链闭环：13 项交付全部 review，机制类红灯清零，剩维护批+两项新显形诊断

本日续前条。自前条更新后完成的派发与 review（全部 resume 原 coder，逐项独立验收）：

**已验收（accepted）**：F1a 初始化器契约 split（lib 601/601 零测试改动转绿；暴露 varargs 暂态）；F2 platform_collection 诊断（推翻 11-05"大表覆盖"前提——真实机制在 build.rs local 规划器 Missing 拒绝声明 + checkcast Object 包装臂）；F1b 渲染链帧瘦身（-62%，86,464B→32,944B，两栈套件转绿）；F1b-2 region.rs Walker 帧瘦身（分发器帧 146KB→22KB，loop45 4/4）；N1 local 规划器 Missing 透明化（platform_collection 5/5 零测试改动；根因：relations 记录调用点 bci 而规划器按 store/load bci 查询，方法级 (source,target) 匹配修复；顺带修 g1_use 双 cast）；N2 cast 楼梯族（platform_implementer 7/7，pin 过时裁决与 post 先例同族；BN 楼梯/顺序漂移被证实不存在）；N3 relations 先行排序（负关系决定性 + release-8 收据门控，+2 单测 bisect 验证）；F3 元素存储点调查（改动集空为裁决性结论：收集已存在且根路径端到端工作，缺口是直接路径无接线）；F4 report.rs 接线（varargs 3/3 零测试改动，最小形态）；N4 facade 诊断（两疑点实证否定，真根因移交）；N5 facade 深诊断（真根因闭环到 jarde-java：新 capture 认领 bridge 调用者 + fold qualify 缺豁免）；N7 WA 渲染对齐（WA 9/9 达成；Diff1 诊断反转——拼写链无罪，缺口在 fold token 扫描改写声明位置 dotted 源名）。

**needs_revision→闭环**：N6 WA 豁免对齐（qualify_member_fold_recovery 对齐 family 检查涉捕获成员豁免，WA 6/9→8/9；单点充分性被诊断推翻，两处渲染加工差异由 N7 闭环）。coder A 原始交付的 needs_revision 残差已全部由上述链条承接。

**当前红灯面（机制类全清，仅剩维护类）**：temporal 2/3、boxed 2/3（拒绝后缀 `(relation: Some(Missing))`，主线裁决保留后缀，golden 维护批重录）；p5_bulk_corpus 3/2、p3_concat_conversion 9/1（billing pin，B 交付归因清单+最新实测由维护批统一更新）；p5_corpus_fingerprint 270 未列夹具、h1_cap/计数 pin（维护批）。N8 在途：member_family_identity 19/1 + nested_type_source_spelling 5/1（N7 抽查发现的新显形预存失败，移除实验已归因非本轮引入，疑似 fold 计划机制变更后首次显形——fold 路径在 HEAD 从未被计划）。

**后续顺序**：N8 → 维护批（golden 重录逐字段 diff 审查、fingerprint 重生成、billing/计数 pin 按最终实测更新、jarde-reader repository 计数 pin 重测）→ 全量重归类 → 最终验证与 handoff 收尾（含 target 磁盘清理）。当前守卫基线：jarde-java --lib 603/603、jarde --lib 297/297、WA 9/9、patterns 92/92、platform_collection 5/5、platform_implementer 7/7、varargs 3/3、reference_slot 8/8、final_static 4/4、loop45 4/4、check 0/0，全部默认栈。

保留红灯边界不变：E7.1 红验收簇、S2b/S3a/S4/S5/S6、8.x、E1–E7、预算跨入口合同、恢复层 fuzz。未提交/推送，工作区脏改动全部保留。

# 2026-11-07 红灯修复执行：项 1/2 交付并 review（A needs_revision / B accepted），F1a+F2 在途，build.rs 残差串行管线排定

本轮续作 11-05 接续顺序，配额已重置。两项修复当日完成派发、交付与 review，工作区脏改动全部保留，未提交/推送。

**项 1（coder A，build.rs 绑定门控+栈）核心达成**：p3_reference_slot_lifetimes 8/8（HeldUse/NonzeroHeldUse 冻结基线逐字节吻合）；`bind_value` 两态提交门控（推送前失败走新增 `abandon_binding` 无痕撤单，不触 refused/deferred，推送后保留半提交保护）；Load 候选消费链存活谓词；copy-input 分支补槽位名检查（WA 访问器恢复基线形状）；deferred_producers 引号吸收规则修复。`no_recovering_method_keeps_a_quote` 3 个套件的栈溢出消除。review 决策 **needs_revision**（残差见下，合流态重验后逐项派发修订）。

**项 2（coder B，source_proof/assignability widening+O(n²)）已验收 accepted**：`reference_intrinsically_assignable` 四张有收据封闭表（调用实参路径与 local 声明规划器共用，matched negative 保持决定性——注意仅对表外对成立，见 F2 观察项）；walk Missing 分支域咨询规则（`platform_allowed && (platform || depth == 0 || loader ≠ runtime.load_domain.loader)`，纯结构事实）；`SsaSiteIndex` BCI 排序向量+二分替换三处逐指令全扫描（按 ceil(log₂(len+1)) 比较数计费，命中后仍按 reads().len() 计费）。array_invocation_widening 7/7、nested_ctor 5/5、varargs 3/3、p3_snapshot_hierarchy_widening 6/6、jarde --lib 297/297、p3_concat_conversion N=1024 在 1M 步内完成。11-05 handoff 的"平台 collection 2 项"实际归属 `tests/p3_platform_collection_widening.rs`（iterable_foreach 基线已绿，属主线派发命名偏差）。

**合流态权威红灯（主线 2026-11-07 亲测）**：jarde-java --lib 600/601（唯一红灯 `initializer_reference_widening_reuses_scalar_and_equal_rank_array_facts`——初始化器 wrapper 继承了 B 的表底线，违反自身契约 pin）；p3_patterns 91/92（`a_shared_deferred_argument_dag_is_quoted_once_and_budgeted` 报告 Mixed→Java 翻转，归因未定谳）；recover_write_accessor_field_types 6/9（WA.main `local1.new S()` 限定 new+空检查惯用法被证据路径拒绝；HEAD 同套件 9/9 绿；A 已证明其绑定候选不再毒化该形状）；p3_final_static、recover_platform_implementer_argument_widening、recover_loop45_original_public 三个栈套件默认栈仍 SIGABRT（A 的帧探针：渲染最大深度两版相同=14，每层帧成本 HEAD 68,224B→当前 86,384B，+26% 全部位于 `render_value_definition` 帧）；p3_platform_collection_widening 3/5（后经 F2 诊断修正：非大表覆盖问题，见下）；p3_concat_conversion 9/1（`a_deep_chain_under_a_short_output_bound_stops_without_an_artifact` 期望步数 <8192 实测 8199，索引化有意变更，维护批更新）；p5_bulk_corpus 3/2（两个 billing pin；B 的 bisect 归因：append_local +1371、prove 主循环 +96、其余 ~4500 在证明路径外——11-05 的 O(n²) 嫌疑对 flat-mixed 不成立，真实收益由 concat 深链证明）；recover_temporal_argument_widening 2/1（golden 缺 `(relation: Some(Missing))` 后缀）、recover_boxed_number_widening 1/2（BNX_MAIN 同后缀 + @bytecode 行顺序漂移同集合不同序 + BN_MAIN pickSeq 行多 `(java.lang.Object)` cast——泛型投影 presented=CharSequence 未投影为 String）。

**主线裁决已定**：(1) 初始化器路径不得继承表底线——split predicate（结构头 identity/Object/array_reference_widens + proved-relations 决定性循环，无四表底线），lib 测试不改动转绿；(2) 拒绝文本删除 `(relation: {relation:?})` Debug 后缀（build.rs ~32842，内部诊断拼写不得进 artifact 文本；trace 设施承载该区分），维护批前一行修复，避免 golden 双录；(3) platform_collection 覆盖按"静态行不得超收据证据，walk 可依实际 header 结构事实超出"裁决。

**F1a 已交付并验收（accepted，review 已记录）**：初始化器契约 split 落地（`initializer_reference_assignable`=结构头+proved-relations 无表底线；`reference_assignable` 行为不变重构），jarde-java --lib 601/601 且 pin 测试零改动转绿，守卫全绿。暴露 varargs 2/3 暂态：初始化器元素存储点无 relation 供给（装箱数 Integer/Long/Double→Number 曾由表底线遮蔽），裁决方向 (a)：向 widening 所有者派 F3 收集器修复（source_proof.rs :7215/:7326/:7523），金标准不动。

**F2 已交付并验收（accepted，review 已记录）——11-05 handoff 的"大表覆盖"前提被证据推翻**：JAVA8_DIRECT_PARENTS 与收据 platform-parents-v1 均不含争议对（无行可收窄、无预期可更新）；两个失败的真实机制全在 build.rs：(i) local 声明规划器在 relation Missing 时拒绝声明 local（域{写类型,用途要求}无最小值），下游实参失类型致拒绝句变形、计数 6→3、main 整体拒绝——精度回归，违背 patrol README"表外拒绝逐字保留"契约；(ii) g1_use 的 `(String)(Object) max(...)` 来自 `reference_check_operand` 对 checkcast 的 Object 包装臂（与 BN pickSeq cast 漂移同一机制，WA.main 证据路径拒绝也可能同源）。观察项确认：`reference_assignable` floor-first 使表内对的 proved negative 永不被咨询，生产可达（platform_allowed=false 环境、快照影子化、release-23 分歧、多域非闭合链、Unknown 五种形态），无测试 pin 覆盖——潜在生产问题，列为 N3。F2 零源码改动、测试 pin 未动。

**修订后管线**。build.rs 单写者串行（resume coder A）：F1b 栈帧瘦身（在途）→ N1 local 声明规划器恢复"声明按写类型成立、用途转换在调用点单独拒绝"（验收 p3_platform_collection_widening 5/5 不改测试，证据链=F2 交付）→ N2 checkcast Object 包装臂（修 g1_use `(String)(Object)`、BN pickSeq cast，重验 WA 证据路径）→ F1c dag 测试诊断+修复（p3_patterns 92/92）→ 拒绝后缀删除一行（build.rs ~32842）。source_proof.rs 串行（resume coder B）：F3 元素存储点 relation 收集（验收 varargs 3/3+lib 601/601+platform_collection 5/5+只依结构事实）→ N3 relations 先行/底线兜底（负关系决定性），一并处理 local 规划器无 release 上下文语义与硬编码 release-8 收据边界。两线写范围互斥时可交替并行。之后维护批（golden 重录逐字段 diff 审查+p5_corpus_fingerprint 270 未列夹具重生成+billing pin 按最终实测与归因更新+jarde-reader repository 计数 pin 重测+h1_cap/p3_scope IrItems 727→728/p3_hoisted_boolean 479→492 等计数 pin）→ 全量重归类 → 最终验证与 handoff 收尾（含 target 39G 磁盘清理）。

保留红灯边界不变：E7.1 红验收簇（owner_census 等 test-support 9 项）、S2b/S3a/S4/S5/S6、8.x、E1–E7、预算跨入口合同、恢复层 fuzz。11-05 与更早各轮记录的这些簇不因本轮红灯修复而宣告验收。

# 2026-11-05 红灯逐簇审查：两机制根因定位，NEG 预期更新完成（含 replay 语义证据），委派被配额中断

本轮按用户"全面审查清理，直到符合架构要求且完成实现"续作。进入时主 change 仍 **33/65**，未勾选任务。上轮（11-04）首次全量量化为 4014/136/80 套件含失败；本轮重跑全量（默认并行）实测 **4029 passed / 121 failed / 72 套件**（差值含时间敏感抖动：p2_contracts 单独跑 30/30 全绿）。全部红灯已逐簇追到机制根因，两项已派发修复，一项已完成。**3 个实现 subagent 因 API 配额耗尽（429，重置 2026-11-06 09:39）中止，无最终报告**；其落地状态逐一核实如下。未提交/推送，进入时脏改动全部保留。

**已完成：NEG 负例族预期更新（原 coder C 范围，由主 agent 收尾）**。`tests/recover_chained_field_assignment.rs`：四个 NEG 负例（mixed/expr/arrayConsumer/callReceiver）从"copy 拒绝契约"更新为 typed-split 保存声明呈现（`NEG_RECOVERED` 表，两编译 leg 输出逐字相同已 CLI 复核）；`the_negatives_keep_their_refusals_on_both_legs` 改为 `the_negatives_recover_with_single_evaluation_saved_declarations_on_both_legs`（每行恰出现一次=单次求值守卫、无 @bytecode/jarde_refused_body=完整恢复、copy 拒绝文本不得回流）；replay 的 NEG"剥离文本必须不编译"特例删除，改走与其他类相同的重编译+执行对照路径。验证：套件 3 passed/1 ignored（`RUST_MIN_STACK=8388608` 下）；**replay `the_recovered_text_compiles_and_runs_identically_on_both_legs` 通过**——NEG 恢复文本两 leg 与夹具 class 行为逐字一致，语义等价证据成立。文件级注释记录有意变更与 reusedSlot 同理由。

**已定位待修（根因+证据齐备，接续者直接按此实施）**：

1. **绑定提交门控回归**（原 coder A 范围，未落地有效修复；build.rs 仅 +6 行残留，无回退风险）。三类同根因形状：(a) p3_reference_slot_lifetimes 3 项——copy_inputs/Load 新候选分支对"消费链已死"形状建立计划→`bind_value` 渲染失败→`mark_binding_refused` 级联替换结构引用（HeldUse/NonzeroHeldUse CLI 实测级联文本）；(b) write_accessor 9 项——从未覆盖槽位也建立 saved 声明（`WA saved0 = arg0;` 取代 `arg0.flag = arg1;`）；(c) 栈溢出——HEAD(30464b083) worktree 探针在 2MiB 栈全绿，当前工作区需 ≤8MiB（4 倍深度增长，debug 帧下 4 个测试二进制中止；release 默认栈通过=有界递归）。修复方向已定：bind_value 推送前失败无痕撤单（推送后路径保留半提交保护）+ Load 候选消费链存活谓词 + 递归深度纪律。验收清单见本轮派发消息存档：p3_reference_slot_lifetimes/write_accessor/4 个栈套件默认栈全绿，p3_local_rewrite 7/7、p3_patterns 92/92、双 lib 不回退。

2. **widening relation 供给链回归 + O(n²) 扫描**（原 coder B 范围，未落地任何源码改动）。两类签名：`relation: None`（prove_reference_relations 对 BCI 无产出：array_invocation_widening 2 项、nested_ctor 2 项、varargs 3 项——含 athrow 形态 Exception→Throwable）与 `relation: Some(Missing)`（walk 返回 Missing：platform_collection 2、snapshot_hierarchy 1、temporal 1、boxed_number 2——presented 为快照类或平台类均应可证）。效率回归同文件：p5_bulk_corpus flat-mixed analysis_steps 1548→7484（4.8 倍，ir_items 仅 +12.6%），p3_concat_conversion N=1024 链恰好耗尽 1M 默认 AnalysisSteps——嫌疑为 prove 循环内逐指令全 SSA 扫描（source_proof.rs ~7210 `ssa.blocks().iter().flat_map` 找 `site.bci()==instruction.bci`）。写范围：source_proof.rs/assignability.rs/facade.rs header 后端；**不得改 build.rs**（与项 1 并行时）。修复后 billing pin 残差列出归因、由主线统一更新。

3. **维护类（合流后统一做）**：p1_xref_golden 3 项+p2_golden 6 项 golden 重录（entry/output_bytes 重分类+environment `domains` schema 演进，逐字段 diff 审查）；p5_corpus_fingerprint 270 个未列夹具重生成（`cargo test --test p5_corpus_fingerprint -- --ignored regenerate_corpus_fingerprint`）；jarde-reader repository 夹具计数 pin 重测（(1382,6048,652,4375,16)）；h1_cap 4 项+p3 各停止类为 S2b 需求驱动成员恢复的顺序变化（方法级可恢复者恢复，类级停止契约保持），预期逐文件更新；p3_scope IrItems 727→728、p3_hoisted_boolean 479→492 等计数 pin 同批。

**保留红灯（非本轮回归）**：E7.1 红验收簇（test-support 9 项：owner_census/capture/enum_switch/assert）对应 `s1-instance-capture-demand-policy-v1/acceptance-r1.json` 的 `owner_census_migrated: false`，change 自身验收未完成；预算跨入口合同、恢复层 fuzz、S2b/S3a/S4/S5/S6/8.x/E1–E7 各簇边界不变。

**验证记录**：`cargo check -p jarde-java -p jarde` 0 error（coder 残留状态可编译）；全量红灯清单与逐簇根因见本轮会话记录，基线日志 `/tmp/jarde-full-test-20261104-r1.log`（临时路径，不保证存在）。CLI 复现命令：`./target/debug/jarde-cli class-source --input <class> --class <名> --policy single-class`。

**接续顺序**：配额重置后先派项 1（绑定门控+栈，build.rs），再并行派项 2（widening+O(n²)）；两项合流后跑全量重归类，再做项 3 维护批与最终验证。stack 溢出在项 1 验收里强制默认栈通过，不得用 RUST_MIN_STACK 掩盖。



# 2026-11-04 全面审查清理：编译/回归/死代码/规格全绿收口，剩余债务如实记录

本轮按用户"全面审查清理，直到符合架构要求且完成实现"执行。主 change `restructure-demand-driven-recovery` 保持 **33/65**，未勾选任何任务、未宣告任何阶段验收；本轮交付的是当前工作区状态本身的收口：编译恢复、提交测试回归修复、死代码清零、OpenSpec strict 校验通过、格式门禁满足。进入时的脏改动全部保留，未提交/推送。

**编译恢复（3 处错误）**。`cargo check --workspace --all-targets --all-features` 此前实际失败（handoff 2026-10-28 的"check 通过"对应其当时状态）：`src/class_source.rs` 测试 `MissingHeaders` 的两处方法签名用裸 `Result`（解析为单参 `jarde_reader::error::Result` 别名）而 trait `RecoveryHeaders` 要求 `std::result::Result<_, Self::Error>`（E0107 ×2）；`crates/jarde-java/tests/p3_patterns.rs` 对 `RecoveryFacts` 调用 `with_declaring_class`（该方法属 `MethodFacts`，E0599），改为按文件既有惯用法在 `MethodFacts` 上链接后构造 `RecoveryFacts`。修复后 workspace 全部 target 0 error。

**提交测试回归修复（5 项，均经 HEAD worktree 探针证实为重构引入：HEAD 上 p3_patterns 92/92、p3_local_rewrite 7/7、p3_reference_slot_lifetimes 8/8）**。

1. `legalize_reference_locals` 接收者域缺失（S2a 新增 pass 的回归）：`local = this` 拷贝的源（slot 0 接收者）无域可播种，arc 一致性清空目标域，`checked()` 的 `Objects.isNull` 参数失去类型事实被拒。修复：pass 内从 `constraints.bounds` 的 slot-0 Entry 播种接收者域（单类型时），队列与验证两处一致使用；分支顺序上接收者先于 `slot < parameters`（实例方法 parameters 含接收者槽但 `parameter_types` 不含 slot 0）。
2. 虚调用接收者检查的门控（S2b 新增检查的过宽）：检查以 `written_type` 为基线，其对 unknown ref 的 `Object` 拼写是自认非证据的（函数自身文档），导致无 flags 时 `arg0.real()` 整体回退 `@bytecode`，违反 `facts.rs` 自己记录的设计（"no flags → writes `arg<slot>`, no claim"）。修复：基线改用 `reference_lifetime_type`（unknown ref → None）并以 `self.has_receiver` 门控——运行未声明接收者时不检查；flags 齐备的接收者类型检查保持原强度。
3. `reusedSlot` 预期更新：重构的 reuse 重写（typed_split 多类别 + exclusive_definition_split）按结构事实拆分 slot 1 的两个生命周期（LVT 两个不相交区间 + 帧类别 + 不相交数据流），恢复出完整 Java（`this.target(); int local1_2 = 1; this.value = local1_2;`）；测试守护的不变量（不发布矛盾的单变量）由拆分机制继续满足，断言更新并在注释中记录该有意变更。
4. `p3_local_rewrite` 两项预期更新：`post(I)I`（`iload_0; iinc 0,1; ireturn`）曾被拒绝引用（Mixed/Fallback，quote BCIs 0+4），现以绑定机制恢复为 Structured/Java（`int saved0 = arg0; arg0 = arg0 + 1; return saved0;`，post(7)=7，return 段携带 BCI 0 的 derived origin）。文件核心不变量（P3-R1：不写 `return arg0;`）与控制组（bump/doubleIt 仍 `return arg0;`）双双保持；`cast` 同样恢复为 `return (java.lang.String) make();`，原"Mixed 映射"测试改钉完整的扫描/来源映射性质（改名 `a_once_mixed_artifact_now_recovers_whole_with_a_complete_scan_and_origin_map`）。
5. 上述修复使 test-support 套件失败数 12→9（anonymous_interface ×2、member_capture ×1 转绿），且禁用本轮修复同样失败的对照证实其余失败与本轮无关。

**既有失败对照（未修复，记录证据与边界）**。根库 test-support 套件 305 passed / 8~9 failed（时间敏感断言在 8/9 间抖动）：owner_census ×2、capture_stop_consumer ×2、enum_switch_group ×1~2、instance_capture ×2、assert_group ×1——对应 change 验收记录中显式未完成的 E7.1 簇（`s1-instance-capture-demand-policy-v1/acceptance-r1.json` 的 `owner_census_migrated: false`），不是本轮回归。`p3_reference_slot_lifetimes` 3 项（HeldUse/NonzeroHeldUse 冻结基线 + unsupported_boundaries）：S3a saved-declaration 绑定机制在 held-reference 负例上先呈现 `saved0` 声明再提交失败（"could not commit" 级联），劣于基线的逐指令干净拒绝；属绑定提交门控的独立闭环，本轮未动（禁用本轮修复同样失败）。

**死代码清零**。workspace `cargo check --all-targets --all-features` 与 test profile 0 warning（此前 22+7 lib 警告与十余个测试目标警告）：删除双 profile 均死的 `known_scalar_initializer_reference_widens`/`reference_array_leaf`/`ACC_NATIVE`/`ACC_ABSTRACT`/`ACC_STRICT`/`source_type`/`class_declaration(_with_types)`/`descriptor_type`/`is_default_member`/`indented_offset`/`lossy_jvm_name`/`bulk.rs strings` 及过期测试 helper；测试专用的 `return_type`/`field_type`/`assert_projection_text` 加 `#[cfg(test)]`；三个有收据的封闭表（Throwable/platform interface/platform array）以 `#[allow(dead_code)]` + ceiling/trigger 注释保留（生产行按 proved relations 走，表由单测钉住，标量初始化元素消费是记录中的开放边界）。`cargo fmt --all` 应用（CI 门禁）。

**OpenSpec**。删除两个杂散脚手架目录（`restructure-demand-driven-generics-v1/` 只有一份与 `results/s2b-declared-generics-v1/dev-v1/sources/custom/Parent.java` 逐字节相同的文件；`s1-generic-physical-names-v1/tools/` 两脚本与 `results/s1-generic-physical-names-v1/tools/` 完全相同），`npx openspec validate --all --strict --no-interactive` 391/391。tasks.md 33/65 计数核对一致。

**验证**：`cargo check --workspace --all-targets --all-features` 0/0；`cargo test -p jarde-java --test p3_patterns` 92/92；`cargo test -p jarde --test p3_local_rewrite` 7/7；jarde-java lib 601/601；jarde lib（无 feature）297/297；s2a/s2b/s1/s3a 聚焦套件全绿（5+2+5+5 / 7+19+25+4+1+5）；局部变量相关套件全绿；HEAD 对照：探针 worktree（30464b083）跑 p3_patterns 92/92、local_rewrite 7/7、reference_slot_lifetimes 8/8，随后清理。

**全量套件首次精确量化（`cargo test --workspace --all-features --no-fail-fast`，默认并行，非 RUST_TEST_THREADS=1 口径）**：431 个 target 中 4014 passed / 136 failed / 80 个套件含失败；4 个用例栈溢出（SIGABRT：`fixture_keeps_the_static_final_and_instance_final_boundaries`、`no_recovering_method_keeps_a_quote`、`original_loop5_…`、`the_enum_positions_are_presented`，stderr 记录，恢复路径递归深度，独立问题）。该红灯面是重构各轮只跑聚焦套件而从未整体量化的提交测试债，本轮按抽样归类为三簇，未在本轮修复：(a) 行为演进预期未更新——最大簇，抽样证实与 `post` 同族（`p3_content` 等期望旧 quote 的用例，新绑定机制给出更完整的 Structured 恢复）；(b) 机制退化——`p3_reference_slot_lifetimes` 的 saved0 半提交级联、`p2_contracts` 2 项等，需逐项追源；(c) E7.1 红验收（owner census/capture 簇，change 记录在案）。RUST_TEST_THREADS=1 的正式口径与 (a)(b) 的逐簇修复属后续独立闭环。

**仍开放（边界不变）**：主 change S2b/S3a/S4/S5/S6、8.x 与 E1–E7 各簇；E7.1 owner census（test-support 8~9 失败为其红验收）；held-reference 绑定提交门控（p3_reference_slot_lifetimes 3 项）；S3a 4.8 资源门 29/30；预算跨入口合同、恢复层 fuzz、D13 语法暂存。本轮不以测试预期更新代替这些机制的验收，新增预算计费（接收者域扫描）在 S0 资源门复核时需一并对照。

# 2026-10-28 架构债务清理：候选收集器与 bridge 投影计划归属

本轮按用户“清理架构债务”推进两个独立闭环，并修复阻塞全目标编译的测试模块归属；主 change `restructure-demand-driven-recovery` 仍 **33/65**，E7 不是该分母里的阶段任务，本次未勾选或宣告 S2b/S3a/S5 验收。

`report.rs` 删除平行的 `RecoveryDemand` 和内核启动时十三个候选输出引用。类级入口由 `ClassSourceCollector` 同时持有 `ClassSourceOptions` 与候选，方法入口传 `None`，不再创建空的类级候选袋；初始化器/枚举构造器的适用性从当前事实推导。三条 class-source 入口统一为 `recover_for_class_source(request, budget, options, gate)`，25 处外部和3处内部调用完成迁移，没有兼容 wrapper。AST 保留的两个重叠 bool 改为 `Required / GenericCalls / All`。独立候选的 Option 保留：请求后可能证明失败，不能将“请求了”误当作“必须 Some”；这条区别已补进 AGENTS.md。失败/停止后的清理和枚举 initializer 特殊保留与原逻辑相同。

bridge 的唯一物理成员匹配、整批暂存和 marker 拼写归 `jarde-java::bridge::stage_class_source_projections`，删除 facade staging 函数和根 ClassSourceMethod 的 marker 拼写方法。root 只供应既有记录的物理事实，保留全部 OutputBytes 计费成功后再一起提交的顺序。原准入政策已在 Java；本次补齐该投影计划的语义所有权，不声称 E7.1 的 owner census/自调用等其余簇已完成。无新依赖，恢复语义67和schema14保持，实际源码指纹随实现变化。

全目标检查发现 `tests/s2b_declared_bounds_r1.rs` 使用 `super::*`，却同时被 Cargo 当独立 target，产生76个编译错误。已归入唯一父测试目录 `tests/s2b_bound_type_identities_r1/declared_bounds.rs`，只更新父路径和三个夹具路径；全部断言保留，父测试19项通过。

验证：`cargo check --workspace --all-targets --all-features` 通过；Java `--lib` 普通/test-support 两种构建各 **601/601**（同组，不重复计独立测试数）；bridge公共入口7、D0/D1/D3与输入来源31、initializer/anonymous15、泛型声明值与bound身份44，共 **97项聚焦集成通过**。输入来源测试的 retained-AST 分支改为实际 All，确认更广收集仍不能认领 read-owner artifact binding。对抗性核对确认所有其它调用参数、gate、适用旗标、清理域、AST消费域、投影文本与计费/提交时机保持；未执行外部目标。

既有失败已做真实前后对照：根库 **295/2**，297项逐项状态相同；p3_patterns **87/5**，92项逐项状态相同。后者5项为 `a_run_that_states_no_access_flags_cannot_decide_a_bridge`、`a_shared_deferred_argument_dag_is_quoted_once_and_budgeted`、`class_source_elides_only_the_closed_this_alias`、`class_source_preserves_aliases_without_closed_receiver_uses`、`straight_finally_reuses_its_guard_verdict_with_a_tight_budget`。首次patterns对照复用了缓存二进制，未计为基线；随后用原始源码重新编译的r2确认上述5项完全一致。没有放宽断言或将它们算通过。原件、最终source manifest及本轮原流保留在 `/private/tmp/jarde-debt-before`、`/private/tmp/jarde-debt-final-source-manifest.json` 和 `/private/tmp/jarde-debt-*.log`。

仍开放：类级源码/映射模型与更大范围门面语义所有权、其它固定 BCI 规则、预算跨入口合同、恢复层 fuzz，以及原 change 的阶段/资源/holdout门。本轮不以源码搬迁代替这些机制验收。专用 `/private/tmp/jarde-debt-20261028-target` 验证后清理；未提交/推送，进入时用户工作区改动保留。

# 2026-10-28 审查改进：枚举静态赋值去除样本准入，整体架构债务仍开放

用户要求理解 `docs/tmp-linus-lens-project-review-2026-10-28.md` 并改进。已新增根 `AGENTS.md` 固化工程原则：结构事实准入、公共入口变形与行为对照、语义单一所有者、真实状态建模、债务分开闭环。审查数字和命名比例只作线索，不当成证明。

本轮闭合 `prove_static_assignment_suffix` 到最终类装配：旧实现只接受 LOW/HIGH、2/5、totalUnits/sumUnits、BCI 34/37/40；先用改名输入复现公开 Engine 失去投影，再从实际 Code 提取调用/字段，并以相邻指令推导偏移。首次修复的端到端测试仍失败，追到 `prepare_enum_constant_source_projection` 中第二处 totalUnits 门控后一并删除。方法/字段物理配对、完整 Code、单次副作用、同次 AST/origin、预算与停止保留。有限域仍是本类静态 `()I` 调用赋给本类包可见 static int 后返回，不宣称任意初始化后缀。恢复语义 66→67，schema 不变。README 澄清清单数不是成功数，支持矩阵记录实际边界。

验证：`cargo test -p jarde --lib -- --test-threads=4` 为 **295 通过 / 2 失败**；其中全部 **66 项枚举测试通过**，含原有额外调用/写入/异常边拒绝、output budget stop、取消及物理 origin 回归。新增四组同源变形（改名、宽整数、三常量、组合）×两种 debug，八份公共 Engine 产物原样完整编译，分别与原源码执行对照，共16次编译运行，值/调用次数/枚举数量一致；这不算独立 holdout 或全项目泛化比例。`cargo test -p jarde-java --lib` **601/601**。仅对受控测试执行 Java，没有运行外部目标。

根库两项失败保留：`class_source::tests::ordinary_parameterized_declaration_requires_array_candidate_and_unbounded_wildcards` 缺 generic_return 数组候选；`facade::snapshot_hierarchy_tests::parameter_ordinals_count_values_not_slots` 的旧预期排除数组，实际包含数组参数。失败位于本次未修改的泛型候选/参数抽取路径；没有本轮改动前的全量运行对照，不把它们声称为已证实的旧失败，也不混入本次修复。未声明全库通过。

剩余债务继续按原 change 和审查拆分：类级 Java 所有权/门面越权与暴露边界（E7）、需求/候选状态耦合、其它固定 BCI/样本准入、类级源码与映射模型、预算跨入口语义及恢复层 fuzz。没有批量删除 prove_/Ok(None)、改报告平面、加预算维度或搬文件，OpenSpec 任务状态未更改。本轮专用 `/private/tmp/jarde-linus-20261028-target` 在验证后清理；进入时未提交改动保留，未提交/推送。

# 当前接续：S2b 类头/字段/方法联合原型已闭合三个公开类，阶段未验收（2026-10-27）

用户“完成所有已记录架构变更”的授权持续，不能停在组件；主change仍 **33/65**。当前事实详见 `openspec/changes/restructure-demand-driven-recovery/results/s2b-declared-generics-v1/implementation-checkpoint.md` 与 `class-field-method-cohort-r1/README.md`。

类级事务复用ClassSourceDeclaration/ClassSourceField/GenericCallProjectionState、原writer/replay/emission，暂存类头、字段和方法并有限裁剪。新继承Header要求完整实例方法及真实零参super()闭合；其它构造/bridge/特殊调用保留。Java保存原descriptor报告和私有typed AST，声明arena不跨方法索引。实际call/field消费证据已接通；owner展示cast丢泛型标记的问题已复现修正，raw local保持。

现有Header map经PreparedMemberOptions在单次class环境共享；缓存命中仍绑定当前方法的实际依赖。新JVM resolve_symbol_from_prepared复用真实PreparedClass Header且不跳过原loader/physical绑定；7项通过。Custom样本Header计数176→51、Body仍7，未充正式资源门。

最新Java650通过；Root11相关目标106/0/1，途中停止16/16，公开Engine4/4。ArrayList<String>/Mid<String>两类完整源码4目标编译/8生成VM通过；共享后同全文，明确复用。OwnedFieldCycle新增2完整目标编译、2原VM/4生成VM，正常/null/heap-pollution原cast异常/写入顺序/泛型反射同原。缺Foo与最后OutputBytes收费失败均无字段/方法半提交；原BCI及新emission范围核对。阶段原流、工具、当前15源pins在结果目录。

下一步：构建/冻结当前CLI与源码，按原dev-v3 8配置/双JDK/反射矩阵核对，再做冻结12真实generic cohort与同上下文基线、资源、规则冻结后的holdout及替代判据清理。s0_map在私有准备矩阵adapter，inventory只读检查真实cohort Header上下文。两者不运行Cargo/JDK/CLI，不读holdout。JDK23 Custom的visibility bridge尚未闭合；自有字段旧拒绝注释在新接受后仍保留，交S5共同候选清理，不做marker字符串过滤。HeaderUnclosed假设实际native已拒绝，不能称确认bug或凭空加guard。

Cargo串行，共用 `/private/tmp/jarde-architecture-20261026-target`；本轮编译均已结束。当前debug CLI仍旧，不能充当前产品；恢复66/schema14/inputs-v11/resolution6，实际fingerprints已随源码更新。原counterfactual和incremental残留待本检查点后精确清理。未提交/推送，进入时脏改动全部保留。

S3a4.8原29/30、sink40时间失败、WAR39无有效基线仍开放。S2b/S3b/S4/S5/S6、8.x及E2/E7全部目标继续；D13具体语法仍最后。不能用本组件绿灯或旧producer快照替代阶段验收。

## 前一接续（保留历史）

# 当前接续：声明调用适配；E2 故障子集已重放（2026-10-27）

用户“完成所有已记录架构变更”的授权持续，主 change 仍 **33/65**，不能停在局部组件。S2b 3.4/3.5/3.8尚未验收；S3a4.8原时间门29/30失败保留，S3b等待。S4/S5/S6、8.x、E2/E7与D13排期继续。

S2b 当前新增事实与消费：raw实例成员按实际descriptor擦除，static成员保留自己的Signature；bound成员/SSA固定点会随上游泛型事实重新求解，避免缓存第一次raw结果。`unique_calls`证明完整selected hierarchy中的源码调用绑定；泛型替换后同形但原descriptor不同的重载不合并。Builder消费最终局部/参数类型，复用原Cast节点、原c0与原SSA复制/数组关系；新增source cast须同时有声明赋值与原runtime erasure上转证明。调用的source结果与原runtime结果分开，get→add不能凭Signature新增Foo检查。Engine仍未供应新arena，整类联合准入/候选回退尚未接通，无新CLI或阶段收益。

[final-declaration-calls-r1](openspec/changes/restructure-demand-driven-recovery/results/s2b-declared-generics-v1/final-declaration-calls-r1/) native-r4三目标 **33/33**、root-check-r1成功；这两项在随后返回位置扩展之前运行，当前正在补return controls，不能称最终freeze。native-r1/r2/r3失败保留：先修raw早成功结果冻结，再分别核正raw get应有Object事实；缺Collection/Iterable的原输入保留不授权/typed拒绝对照，同一target字节使用完整13 Header才接受泛型调用。旧None-domain descriptor基线仍可取。fromGetter来源核对已显式请求SourceMap，避免空map比较。新文件与两新Builder方法仅做edition2024局部格式化。

[call-adaptation-r1](openspec/changes/restructure-demand-driven-recovery/results/s2b-declared-generics-v1/call-adaptation-r1/)三份原样public Recovery body的Java8组件wrapper编译/javap通过：typedAdd保留原Foo c0，genericForward没有新增Foo c0，rawWrite保留raw/Object；不是Engine整类验收，未执行目标。static-generic-members-r1与call-binding-r1各focused1/1含真实class/javap控制；source spelling/runtime erasure旧收据保持。return-consumption-r1输入已冻结，返回消费代码新接入，待focused测试。S2b独立holdout仍未读取。

E2已修Prepared/Refused/首因/交付前缀及准备后panic方法进度。lifecycle5/5、cancel8/8、serial7/7、retention2/2、delivery3/3通过。S0原40–46七行现已用独立冻结源码快照完成70次重放：无guard、协议70/70；按原上界当前时间6/7、RSS7/7、token延迟通过，row40 sink耗时3060µs仍超过原上界1898µs。预算、token与sink都是有报告和worker join的停止前缀，不要求停止后再发ClassEnd，也不算正常类终态。快照与producer绑定并不含之后的S2b、bridge及reader/query源码编辑。原计划中的WAR row39仍是0/10有效完成、10/10 RSS guard，无当前完整请求阈值；其余正常bulk行24–31、35未重放。详见[阶段复核](openspec/changes/restructure-demand-driven-recovery/results/architecture-completion-v1/s0-e2-resource-stage-review.md)、[row40审查](openspec/changes/restructure-demand-driven-recovery/results/architecture-completion-v1/s0-bulk-fault-current-r1/sink-row-forensics-r1.md)及[原协议清点](openspec/changes/restructure-demand-driven-recovery/results/architecture-completion-v1/e2-postprepared-followup-r1/s0-replay-readiness.md)。S0/E2/S6仍未验收，任务未勾选。

query reader共享事实修复及r4-v2资源批次已有收据：30配置完整，原时间门29/30、RSS30/30；`18-fixture-query-cold-essential-normal` 为1379µs，超过冻结上界1277µs（oldmax301µs），仍是独立开放红灯，未归因OS调度。重复digest处理和测量协议见[s3a资源复核](openspec/changes/restructure-demand-driven-recovery/results/s3a-ordered-baseline-v1/resources-r4-v2/semantic-resource-review-v2.json)。不放宽门、不用其他成功行覆盖，也不与E2 row40混为一项。

E7 bridge只读审查在e7-bridge-migration-r1.md：当前纯形状/target/contract已在Java，剩余耦合Root assembly/I/O/提交；未做半簇迁移，不能据此宣告E7完成。以下为历史检查点，保留当时状态。

# 当前执行：S2b bound类型与成员代换接入原型（2026-10-27）

主线仍 **33/65**。[S2b检查点](openspec/changes/restructure-demand-driven-recovery/results/s2b-declared-generics-v1/implementation-checkpoint.md)更新了当前状态：`generic_types.rs` 已建立带物理定义/loader身份的bound arena，class/field/method Signature 与同次Reader erasure proof配对；尚无Source consumer/Engine接入或阶段收益验收。`RecoveryRequest`已有可选 `declared_types` 输入，digest为v11、artifact schema14、recovery semantics66；`Declarations`局部规划有有限generic-local路径，当前仅有cargo check r1通过，root仍在编辑 `generic_types.rs`/`build.rs`，不视为final freeze。

现有controls累计11+7+4项：前两组已有11/11、7/7收据；SSA局部4项的历史4/4是另一版 `resolve_member` 接口下运行，当前已回到 `resolve_member(symbol, use_kind, budget)`，最终接口重跑待补。物理绑定小测 `member-defining-loader-r1/native-r5.log` 1/1：child loader claim parent-defined IR仍被原合同拒绝；合法parent IR解析到parent Foo，独立child查找解析到child Foo，两路径method-body均为0。

开发输入与基线已准备：[dev-v3](openspec/changes/restructure-demand-driven-recovery/results/s2b-declared-generics-v1/dev-v3-inputs/README.md)含13类真实Corretto8 Header/Signature闭包、R4 8/8；[real-generic-freeze-v1](openspec/changes/restructure-demand-driven-recovery/results/s2b-declared-generics-v1/real-generic-freeze-v1/README.md)是单独冻结的真实12泛型类cohort。均非S2b收益验收。独立S2b holdout尚未读取。

下一步按当前resolver接口完成SSA局部公开入口测试，再接Engine同次class facts供给和类级候选共同准入/全部使用回退；之后对冻结12类做同配置净收益比较并通过适用资源门。3.4/3.5/3.8未勾选；不要把Header可见、helper green或旧接口4/4当阶段完成。S3a 4.8原资源门时间29/30红灯保留，S3b仍等待；D13排期、E2/E7及S4/S5/S6和8.x剩余目标继续保留。

# 当前执行：S3a组件完成、资源阶段门未过；推进独立S2b（2026-10-27）

用户“完成所有已记录架构变更”的授权持续。主change当前33/65；S3a4.1/4.2/4.5/4.6完成，4.8仍未验收。R4恢复65/解析6/schema13/inputs10，[检查点](openspec/changes/restructure-demand-driven-recovery/results/s3a-ordered-baseline-v1/README.md)与[544文件归档](openspec/changes/restructure-demand-driven-recovery/results/s3a-ordered-baseline-v1/source-r4-manifest.json)可接续；prod194来源冻结，当前没有后续产品语义改动。

核心机制/行为证据：当前12开发报告严格复用24完整类编译48隔离运行；S0 E1独立于本轮S3调规则，S2a8编译失败→当前8编译16行为成功。新独立集一组完整保持、一组原短路shared-value/catch scope拒绝仍阻断整类，不虚报改善；具体失败是后续区域/变量边界输入。固定包12类363方法174→194 Structured，20增零回退，未执行真实目标。2469报告全退出/限额/类状态等S2a，7040方法状态保持，6368绑定与28002 map区间验证；33方法只有quote/rejection注释变动，22方法map来源增补无旧origin丢失。当前native Java600/JVM251/copy6/atomic-origin5及13公开目标56/0/10通过，workspace all-target/all-feature check过。

资源必须保持红灯：r4原30/30通过批次协议复制残留S2当前metadata，保留为非最终；准确v2独立60+600批次29/30时间、30/30RSS，纯query cold essential单样本1379µs>原1277µs（oldmax301µs），未证是OS调度。不得用第一次成功覆盖、放宽阈值或无实质理由反复重跑。原始两批/metadata erratum完整保留，17E2/S6仍不计。S3a4.8和依赖它的S3b不开；按真实任务依赖，S2b3.4/3.5只依赖3.9等既已完成S2a，可以继续推进，后续实质产物再共同验收原资源门。

S2b只读入口图在results/s2b-declared-generics-v1/entry-map.md；当前缺已声明类泛型实参沿继承路径组合到成员/变量/使用的统一连接，不能只加漂亮class header或类名白名单。Dev双族由inventory冻结；独立两族holdout由s0_map原始8编译16运行已冻结，root不读其源/driver，规则冻结前不评估。后续保持S2b→S3b→S4/S5/S6、8.x及E2/E7全目标，D13具体语法暂存不混入。

资源release target303287780B已移除，immutable CLI/harness保留；architecture debug target仍约16GB供后继复用，阶段/最终收尾清理。未提交/推送，所有进入时脏改动保留。

# 当前执行：S3a有序基线原型r3验证中（2026-10-27）

用户“完成所有已记录架构变更”的授权持续。主change仍29/65；S2a已有限域验收，S3a的4.1/4.2/4.5/4.6/4.8未勾选。当前[检查点](openspec/changes/restructure-demand-driven-recovery/results/s3a-ordered-baseline-v1/README.md)记录JVM实际Frame/SSA复制来源、producer位置typed temp、r1算术扩展退化/r2类型来源编译失败及r3最小修复，原流全部保留。原型恢复65/解析6/schema13/inputs10，r3 CLI与producer pin在结果目录，最新产品仅随后整理了source_proof注释，不能把r3二进制说成当前源码逐字构建。

当前开发完整类16编译/32隔离运行全过，含LocalUpdate严格复用8编译16运行和Lvalue新增8编译16运行。Java600、JVM251、stack-copy6及来源正负2通过。固定真实12类363方法174→194 Structured，20新增零成功回退，LongArray原Partial保持；未执行真实包，不充完整类/行为证明。宽数值/浮点、原子拒绝/停止、有界Java fuzz继续；独立holdout仅冻结原oracle，尚未读取/评估/调参。后续按D12继续S3a→S2b/S3b→S4/S5/S6、8.x和独立E2/E7，D13具体语法问题继续暂存。

# 当前接续：S2a已验收，继续S3a顺序基线（2026-10-26）

用户“完成所有已记录架构变更”的授权持续。主change `restructure-demand-driven-recovery` 29/65，S1/S2a已按有限域实际验收，下一步4.1/4.2/4.5/4.6/4.8；S2b/S3b、S4/S5/S6、8.x及独立E2/E7仍继续，D13具体语法暂存不提前混入。不能停在局部阶段或宣称全部完成。

S2a当前producer r65，恢复64/schema13/解析5/inputs10，CLI SHA2414d624e74a881f9240acb9d00f113b436c1713f26069547e203106a07c93eb。[阶段收据](openspec/changes/restructure-demand-driven-recovery/results/s2a-variable-legalization-v1/acceptance.json)、[源码542文件归档](openspec/changes/restructure-demand-driven-recovery/results/s2a-variable-legalization-v1/source-r65-manifest.json)、[检查点](openspec/changes/restructure-demand-driven-recovery/results/s2a-variable-legalization-v1/reference-constraints-checkpoint.md)和[s3a入口图](openspec/changes/restructure-demand-driven-recovery/results/s2a-variable-legalization-v1/s3a-entry-map-r65.md)可接续。

3.6生产路径重复RHS语法白名单已移除：声明证明SSA写入/到达性，实际Builder拒绝负责整方法回滚与原诊断/origins保留；Store独占消费的引用条件式及lambda捕获消费最终声明。原物理边Store、循环交换临时值与handler throw前局部保持。r64复用IR持有的物理Header交给既有有序解析，6条原class_headers=2退化同预算修复；物理facts共享与loader绑定仍分开。

当前600Java、13demand、102公开、38lambda、251JVM、4Header复用测试通过；既有10ignored保持。136×3核心、6控制编译/12隔离运行和2真实有效类4编译通过。固定包12类363方法173→174净+1、无成功回退；LongArray原IR partial及PKIX整类编译失败保留，未执行真实包。

独立ReferenceCases首次盲验的完整类0/8→8/8类型合法化收益当前text/输入/driver/依赖pins完全相同；r44少量driver曝光记录保留且未用于调规则。新r65独立双族32完整编译/64隔离运行均过，但S1已全Structured，只计保持/作用域精度。已用于诊断的IndependentCases及r48两lineage不再冒充独立集，当前完整编译/运行通过，原失败原流保留。

2469固定原预算报告无新增exit0→4，48条4→0；14更早AnalysisSteps停止在既有充分预算兄弟中具有与S1相同方法文本/来源。正式600进程的30配置time/RSS均通过原S0阈值，60准备进程另列；全部配置观测live/peak最大值与最终S1一致，纯结构查询零Body/IR/AST。原17E2/S6项未被本批接受，历史失败不改写。

# 当前执行：S2a 全写入/使用类型队列已接入，验收继续（2026-10-26）

用户“完成所有已记录架构变更”的授权持续有效，不能停在S2a局部。主线24/65；3.1完成，3.2/3.3实现中，S2a未验收。[当前检查点](openspec/changes/restructure-demand-driven-recovery/results/s2a-variable-legalization-v1/reference-constraints-checkpoint.md)记录恢复53/schema13/inputs10的新机制、冻结dev完整类8编译16行为证据、后继变化、已保留失败和剩余门。holdout仍未读/用于调参。后续继续S2a→S3a→S2b/S3b→S4/S5/S6及记录的独立架构债务，具体语法缺口遵守D13。

# 当前执行：S2a变量分组3.1完成，类型约束与环境绑定继续（2026-10-26）

用户授权完成所有已记录架构变更。主线当前24/65；[3.1收据](openspec/changes/restructure-demand-driven-recovery/results/s2a-variable-legalization-v1/task-3.1-receipt.json)已交付，S2a尚未验收，下一步3.2/3.3。复用Plan/LocalVariable，SSA证明多类别槽生命周期和无debug互斥分支的独立变量；live aliases/未证明phi入边/异常边继续拒绝。原物理store、loop临时值和handler prethrow事实保留。

最新R6 CLI `/private/tmp/jarde-s2a-r6-20261026-bin/jarde-cli` SHA256 `ab79fa84408450aae1ad431e2819f5b08309024fe592d4e5aa9e56f4779e8b86`，恢复51、schema12/解析5/inputs9。四dev原样源码重编及JDK8/23行为均通过；R6 debug按text SHA复用R3、no-debug新增4编译8运行。旧category2Adjacent和nestedHandler负例升级有原class/javap与完整派生控制证据，原golden/整类Partial保持。测试曾把catch结构origin误当handler primary、return关键字误当load origin，失败流保留，分别校验derived handler和return21/operand20后通过；没有为测试改来源产品。

固定bcprov12类同预算报告已配对：11 complete、LongArray仍partial；363方法中PKIXNameConstraintValidator数组重载由mixed/fallback变java/structured，尚需整类编译与来源审查，不算真实行为证明。holdout已独立冻结且原oracle验证，但未用于恢复/调参；S2a资源尚未跑。

3.3已确认反例：平台Header override r2中，选中无List接口的物理ArrayList后，参数仍被Builder表放行。下一步统一环境绑定assignability，明确Header/平台事实来源和Missing/Ambiguous/Unknown/不兼容，不重做E7迁移。7纯关系helper已机械提取assignability.rs（Java598通过）；还未改变关系消费语义。专用debug target当前约11GB供后续复用，阶段结束须清理。S1最终source归档和500MB性能cache清理已记录；所有既有脏改动保留，无提交/推送。

# 当前执行：S1有限退出已通过，转S2a（2026-10-26）

用户已授权完成所有已记录架构变更。当前主线`restructure-demand-driven-recovery`为23/65，2.9–2.14完成；下一项3.1，先实现S2a变量分组/全写入使用类型约束/声明与phi、handler可见值，再按D12做S3a。纯归属整理E7继续独立；具体语法问题按D13等架构完成后复核，当前不先补旧writer。

[S1验收](openspec/changes/restructure-demand-driven-recovery/results/architecture-completion-v1/acceptance.json) · [逐项审查](openspec/changes/restructure-demand-driven-recovery/results/architecture-completion-v1/finite-exit-review.md) · [S2a输入准备](openspec/changes/restructure-demand-driven-recovery/results/architecture-completion-v1/s2a-input-readiness.json)。固定R01–R16与R06/R07后继闭合，typed stop保留原配额/elapsed首因与原Report；Code停止截断后续callee，owner census原诊断透传，预取消保留未搜索范围。恢复50、解析5、binding12、inputs9各自独立。

最终2469完整报告及408核心完整报告在纯Clock优化后保持；136×3、6完整编译/12受控JVM、真实2类4重编通过，真实包未执行。正式600进程、30配置各10轮，原时间/RSS门30/30通过；300对实际工作/计费/留存不变，100窗600次纯查询无Body/IR/AST。三行性能修复只删除成功计费/深度更新后未消费的elapsed缓存写入；公开usage现算，计费和deadline/cancel检查不变。独立Reader183及ledger/公开停止控制通过。

资源初探和两批时间失败保留，最终批次前共同60进程准备独立留档，正式fresh process/FactsCache与原配置/阈值不变；沿用S0未清OS缓存条件，不保证冷机或首次执行时延。原47行中的17个bulk行、10个WAR RSS失败和bulk故障门仍归E2/S6。八项历史失败中五项合同修正有独立证据；WA九accessor逐项已过，main/整类三项仍失败，不改为绿；row171独立probe额外9字节未归因，未声称历史隐藏文档全等。

当前可用S1基线CLI为`/private/tmp/jarde-s1-clock-cost-20261026-bin/cli-jarde-cli`，源码/producer与输入证据均在结果目录。临时测量/profile测试源已移除并保存副本；`/private/tmp/jarde-architecture-20261026-target`仍供后续Rust实现复用，阶段结束/最终交付须清理自建缓存。用户预先存在的工作区改动全部保留，未提交或推送。

# 前序执行约束：语法问题先登记，架构完成后再修（2026-10-26）

用户要求避免“先修语法、再随架构迁移代码”的重复工作。具体语法恢复问题统一登记在[OpenSpec design D13](openspec/changes/restructure-demand-driven-recovery/design.md#d13-语法恢复问题先记录架构变更后复核)，本轮仅更新规划与交接，未改代码、测试或历史证据。

当前登记：SYN01为已观察的字段声明`this$bad_name`与方法体`this.this$bad-name`不一致，物理子类未独立编译验证；SYN02为SSA变量作用域/声明/phi合法化风险，SYN03为求值顺序/次数/窄化/异常优先级风险，后两项沿用现有设计，未新增已确认反例。等本次架构变更完成后先用最终接口复现，自然消除的补证关闭，仍存在的另行修复；不先改旧发射路径，不将它们新增为架构前置。S2a/S3a共享机制按原任务继续，既有验收和失败状态保留。

R07已确认的必要origin修复维持；损坏MUTF的Partial/Complete传播仍归2.10，不混进语法暂存。2.9仍未勾选、进度17/65；下一步仍是下方冻结清单的证据汇总，无需为语法待办扩大当前范围。

# 当前接续：2.9 R07必要origin缺口已修；有限闭包待汇总（2026-10-26）

[R07结果与边界](openspec/changes/restructure-demand-driven-recovery/results/s1-synthetic-field-origin-v1/README.md) · [52对公开入口](openspec/changes/restructure-demand-driven-recovery/results/s1-synthetic-field-origin-v1/public-comparison.json) · [检查点](openspec/changes/restructure-demand-driven-recovery/results/s1-synthetic-field-origin-v1/checkpoint.json)

只推进已登记的R07，没有职责迁移。DBS匿名双括号两来源控制的supplementary字段原本不会阻断投影；N1混合成员族仅改一个CP字段名this$0→this$𐐀后，公开入口确认丢失捕获写入必要origin，实例Inner不再折入。projection_needles改用现有严格MUTF解码，发射/物理证明保持；恢复48→49，schema12/解析4/inputs9保持。新增回归从0/1失败到通过，并核对真实child owner、构造器描述符、BCI2与物理raw字段。

当前源码基线与修后各52调用：supp N1根类在source7/8×三evidence中0/6→6/6折叠，当前正文与ASCII相同；其余46份正文、52对物理field/method item保持。7原生目标39项通过；6完整类空classpath编译、12生成JVM与4原夹具对照通过。不可表达组的物理子类旧拼写、损坏组N1根类仅静态投影的Complete/子类Partial边界保留，未顺手修名称政策或2.10。详情与原流在结果目录，不把这些组算作新增整类成功。

R07该来源缺口已闭合；旧R01–R16清单、旧源码/收据不改写。下一步按冻结清单汇总其余行的当前/历史证据与2.9退出条件，不能直接宣布2.9完成。进度仍17/65，S1未验收，S2/S3未进入；typed-stop归2.10、资源归2.12、E7独立。136×3、2469和S0门未重跑，不沿用为本轮通过。本轮独占cargo target已按检查点记录清理；预先存在的工作区改动保留。

# 前序：2.9 有限清单已固定；raw字段owner阻塞已修（2026-10-26）

[结果与验证](openspec/changes/restructure-demand-driven-recovery/results/s1-raw-consumer-closure-v1/README.md) · [16组消费者清单](openspec/changes/restructure-demand-driven-recovery/results/s1-raw-consumer-closure-v1/inventory.json) · [逐行审查](openspec/changes/restructure-demand-driven-recovery/results/s1-raw-consumer-closure-v1/consumer-review.md)

本轮只完成用户指定的有限清单/已有证据核对及一个确认的raw修复，不延续CG旧迁移顺序。冻结R01–R16、118生产源；CG19项证据bytes/SHA-256全匹配，40个目录引用存在。原metadata Option风险已有后继闭合，ASCII access$数字桥名解析不是有效MUTF反例；未重开它们。

确认same_class_field_receiver_sites把owner文本转标准UTF8再比较MUTF字节，合法supplementary类因此漏掉字段receiver证明，T字段误保留Object。仅report.rs三行分开raw身份/源码拼写，artifact恢复语义47→48；schema12/解析4/inputs9保持，无职责迁移或新抽象。新公开回归在修改前当前源码上失败，最终接收者组7/7及14相邻目标64/64通过（去重71项），新生成类空目标classpath原样重编通过；24 CLI调用/12新旧对核对ASCII全文6保持、supp投影0/6→6/6、物理成员和三evidence正文保持。不充新增行为/holdout或整包恢复收益。

2.9仍未勾选，进度17/65，S1未整体验收，S2/S3未进入。下一步仅核R07已登记的synthetic supplementary字段是否因projection_needles的UTF8过滤丢掉必要origin：先公开入口复现，有确认影响再最小修复；不得借此迁owner census/call-site/专用家族。其余行保留已有证据等级，完整当前闭包尚未宣告通过。typed-stop归2.10，资源归2.12，E7整理继续独立。

CG源码归档与本轮开始时facade存在尚未封存的cfg(test) seam差异；保留用户工作区。CG CLI仅作历史对照，当前报告绑定本轮实际构建。2469/136×3历史全门和S0资源未在本轮重跑，不沿用为新通过。

# 规划修订记录：S1 有限退出 → S2a 变量合法化 → S3a 顺序基线（2026-10-26）

本次仅更新 OpenSpec 规划，未修改产品代码、测试或历史证据，也未新增实现/验收通过项。当前产品检查点仍为下方 CG；S1 未验收，S2/S3 未启动。任务从历史 17/59 调整为 **17/65**，仅增加六个未完成的分解/里程碑任务，原 17 个完成项不变。

当前执行顺序以 [proposal](openspec/changes/restructure-demand-driven-recovery/proposal.md)、[design D12](openspec/changes/restructure-demand-driven-recovery/design.md#d12-s1-有限退出与恢复主线优先级)、[tasks](openspec/changes/restructure-demand-driven-recovery/tasks.md) 为准；后附收据保留当时结果，其“下一组继续迁移”不再决定主线优先级。

- S1 只按 2.9–2.14 退出：固定 raw/来源消费者闭包、typed stop、有效成功/旧失败核对、适用资源、当前 API 与实际双语义路径、最终 A1.7 汇总。身份、首因、停止后零新 Body/IR、无半提交及适用旧时间/RSS 门不豁免；未触达 bulk 行在原矩阵保留失败并归 E2/S6。
- 剩余 owner census/call-site/bridge、nested/super/double-brace 的纯职责整理及薄适配收口归独立 E7 工作组，不作为 S2a/S3a 前置。若实际破坏上述不变量，只提升对应最小修复为阻塞，不能要求整簇搬完。
- S1 实际退出后先做 **S2a：3.1/3.2/3.3/3.6/3.7/3.9**，交付源码变量、声明和 phi/handler 值合法化；继而做 **S3a：4.1/4.2/4.5/4.6/4.8**，交付保持求值顺序的有类型语句/临时值基线。
- S2b 泛型扩展与 S3b 糖化各自保留验收，不阻塞已证明有限域的 S3a；S4 仍等待完整 S2/S3，S5/S6 依赖保持。两项早期里程碑都须有完整类原样重编、隔离行为、独立保留集、固定包净收益及适用资源证据，不能用迁移数量或中间计划通过代替。

下一步实施任务为 **2.9 的有限清单与已有证据核对**，随后只修其确认的 raw/来源阻塞并进入 2.10；本轮不执行。

# 历史实现检查点与收据

# 当前接续：普通/instance capture需求已归Java；owner census与专用策略待续（2026-10-26，CG）

### 2026-10-26 / CG：instance需求与IR持有期已验收

Java member_capture::prove_instance_capture接管instance物理methods的原顺序需求、每项先poll再筛Code、PhysicalMethodId、IR身份/Complete门与同一prove_family_instance_capture调用；Root保留物理重读、实际IR callback/typed执行诊断、fold target/bridge/census与提交。原Vec::new、拒绝文案及instance capture refused前缀、允许普通构造体/实际capture write BCI的扩展域保持，不误用普通member六指令证明。所有旧Java证明/ordinary/anonymous政策及Root其他流程逐字不变；普通proof的Root导入限定cfg(test)，instance proof不再Root重导出，CF新增unused import已清。恢复47/schema12/解析4/inputs9保持，无新产品DTO/Runtime/cache/read。

先增加仅test-support的真实MethodIr构造/drop计数，正常构建无该字段/状态；已有事件Counts和产品报告不扩展。基线同两新测试1pass/1fail：完整N1捕获后的2个IR及2个ClassFacts在proof、bridge、census到done仍存活，returned才归0；五种实际IR/后置proof停止已释放且停止后实际reader calls/bytes、Body、六IR全0。迁移后Java只返回自有MemberCaptureProof，分析/借用缓冲在证明返回前释放；同一五阶段IR和ClassFacts均2→0，五stop保持、所有六控制通过。ClassFacts为值载荷数而非物理类数/Arc引用数；只证明该受控路径的对象持有期，不当heap字节、RSS/时间或完整S0资源改善。

当前17目标489/0/0：Root308、原14 integration174、Java assert2、追加既有inner_class_static_mixed_folding5；其中只有两项是本轮新测试，五项为已有整族源码/重编/受控行为/停止回归。原83完整控制记录严格保持，CG6的请求、身份、执行、diagnostics、usage及fold全文保持；仅normal五阶段的两个存活计数变化逐点保存，原预算/取消与已发布prefix保留。复用旧N1三class及原fixture，不计新holdout或恢复增量。

核验实际engine、registry.jvm来源指纹并仅规范直属usage.elapsed_millis后，2469继承完整JSON/退出码与408核心完整JSON保持一致，零语义/计费例外；原r1规范下377份相同、2092份共6538处仅JVM来源身份变化，均与父/当前实际Cargo绑定；r2复用原2469次CLI输出，不增加CLI。168枚举/20匿名profile已含，无额外矩阵CLI；168源码全文/78声明、136×3、核心6编译12受控VM和真实两类4重编保持，真实目标未执行。2745源/资产、归档、18当前产物与三正常构建绑定，target清理。r1迁移静态断言误拒report.execution读取，source scope门在Cargo前拦截未完成转移，原脚本/错误和空目录保留；r2修正断言后构建，无证明或测试断言调整。观测基线与迁移后用同一测试源码，21项纯文本迁移核对及正常CLI无test-support features验证通过。两次审查脚本误判及原false输出、纠正收据分列保留，不改生产/测试断言；构建磁盘不充运行资源。

下一组owner census/自调用准入与call-site/bridge资格，然后nested/super/double-brace完整资格及暂存事务；shared emitter Interrupted、其余read/bridge/sink停止、资源/API、raw消费者闭包和历史native失败仍开放。第三轮六项原JDK8/历史限制、S0原记录保持；2.9/2.10/S1未关闭，OpenSpec17/59，S2未进入。

最新证据：`openspec/changes/restructure-demand-driven-recovery/results/s1-instance-capture-demand-policy-v1`，canonical current-build-r2/actual-producer-r1。


### 2026-10-26 / CF：普通 member capture 需求政策已迁

承接CE已封存的capture原始Report消费者闭包，继续迁移普通member capture的方法需求政策。Java member_capture::prove_member_capture接管原Vec::new、物理methods原顺序、每项先poll再筛Code、PhysicalMethodId构造、IR身份/Complete校验、IR借用及同一prove_family_capture调用。Root仅保留同一实际analyze_capture_dependency callback、typed停止/执行diag合并与ClassSourceMemberCapture枚举拼装。无新DTO/trait/Runtime/cache/read，原拒绝文案、严格六指令Outer证明域、计费/poll/clone顺序保持；所有旧Java证明与anonymous pipeline、Root其他函数/测试逐字未动。恢复47/schema12/解析4/inputs9保持，instance的普通构造体扩展域未混入。普通证明的Root重导出现仅测试使用，新增unused import警告已逐项对照记账，随member_inner剩余调用者/重导出收口清理，不声称零警告。

当前16目标482/0/0，原25政策、CD10 typed与CE48公开控制共83份完整记录严格保持；48包括8正例、32真实IR Report及8 proof Error，原报告/诊断一次消费、prefix与实际停止工作截断保持。CE原代码2/1和24公开Report传递失败、修后全通过的原证据不改，不重复算新恢复/holdout。本轮无新测试/目标输入。

2469继承全JSON/退出码、408核心全JSON严格相同，零语义/计费例外，含168枚举/20匿名profile子集不另跑矩阵；168完整源码/78声明、136×3、核心6编译12受控VM及真实两类4重编保持，真实目标未执行。2745源/资产、归档与17当前产物及三正常构建绑定，target清理。r1脚本parent路径预检错误发生在Cargo/native启动前，原流保留；r2纠正路径后正常构建，不改生产代码。构建磁盘不充S0运行资源。

下一组依次处理instance需求编排及IR持有期、owner census/call-site/bridge资格与nested/super/double-brace专用事务。共享Interrupted、全部read/bridge/sink停止/资源/API和历史native失败仍归2.9/2.10，不能用482/0覆盖。第三轮六项原JDK8/历史限制、S0记录保持；2.9/2.10/S1开放，OpenSpec17/59，S2未进入。

最新证据：`openspec/changes/restructure-demand-driven-recovery/results/s1-member-capture-demand-policy-v1`，canonical current-build-r2/actual-producer-r1，后继instance持有期与owner census见next-boundaries。


### 2026-10-26 / CD：完整共享 capture 证明簇归 Java

[结果](openspec/changes/restructure-demand-driven-recovery/results/s1-member-capture-source-policy-v1/README.md)、[验收](openspec/changes/restructure-demand-driven-recovery/results/s1-member-capture-source-policy-v1/acceptance-r1.json)、[六项](openspec/changes/restructure-demand-driven-recovery/results/s1-member-capture-source-policy-v1/six-gates.md)、[后继任务/依赖/验收](openspec/changes/restructure-demand-driven-recovery/results/s1-member-capture-source-policy-v1/next-boundaries.md)。member_capture模块接管完整Outer/instance/double/val捕获、constructor参数分区、local单写、SSA使用/唯一写/MethodHandle/exception及两receiver域，两个原类型同迁，root仅重导出同一实现；anonymous闭表/Code预检、method调度与三分流归Java，root callback留实际analyze/执行合并/可选原Report relay。无镜像DTO/trait/runtime/cache/read，恢复46/schema12/解析4/v9保持；一般family/instance IR编排、call-site/bridge/owner census与专用匿名族仍待迁，不称整族完成。

原代码同三测3/0，当前16目标479/0/0；新14控制（2成功/4早拒/8真实IR stop）与原25控制逐项保持。四早拒wrapped-reader/Body/六IR为0；八stop均1请求、停止后三类计数全0。legacy四次Ok(None)无原Report/diag sink已复现，opt-in四次原execution/diagnostics保留；两种局部execution均已合并，尚未证明公开假Complete或修typed消费者，2.10继续开放。2469继承全JSON/退出码及408核心严格保持，零语义/计费例外，含168枚举/20匿名子集不另跑矩阵；168全文/78声明/136×3及核心6编译12受控VM、真实两类4重编保持，真实目标未执行，不计新holdout。2745源/资产（仅新增机制模块）、归档/17当前产物与构建绑定，target清理。r1 lib递归格式化三无关文件被scope门拦在Cargo前并按封存字节恢复；r2仅旧单测缺两helper导入而编译失败，r3补同实现别名/隐藏pub，不改证明或断言，原流保留。S0原门/历史失败不豁免，2.9/2.10/S1开放，17/59，S2未进入。


### 2026-10-26 / CC：匿名接口跨 IO 阶段资格与 emission 归 Java

[结果](openspec/changes/restructure-demand-driven-recovery/results/s1-anonymous-interface-source-policy-v1/README.md)、[验收](openspec/changes/restructure-demand-driven-recovery/results/s1-anonymous-interface-source-policy-v1/acceptance-r1.json)、[六项](openspec/changes/restructure-demand-driven-recovery/results/s1-anonymous-interface-source-policy-v1/six-gates.md)、[后继职责/依赖/验收](openspec/changes/restructure-demand-driven-recovery/results/s1-anonymous-interface-source-policy-v1/next-boundaries.md)。site/root-child typed nesting、capture参数与返回类型、平台/实际接口契约、构造器/完整方法/scan闭集和捕获AST重写—child文本—root匿名返回emission归Java；共享唯一allocation/nested-site/complete-method政策及三个既有模型同迁。root仍负责实际resolve/read/IR准备、执行合并、锚点/物理装配和发布；flags/facts/bytes getter只借用原数据，placement调用CB已迁政策，无新DTO/trait/read/cache，恢复46/schema12/解析4/v9保持。共享capture/owner census仍含政策、nested/super/double-brace专用决策仍待迁，不称纯IO或整族完成。

旧政策同三新测试3/0，当前16目标476/0/0；4正例/7早晚拒绝/3停止共14控制逐项保持，原11控制保持，三停止后实际Body/六IR全0且无半提交；elapsed仍Unsupported(jre_budget_interrupted)，单列2.10，不冒称首因正确或全部read点通过。2449继承+20匿名profile（5既有JAR×source7/8×essential/all，40CLI）=2469完整报告/退出码严格相等，408核心相同，零语义/计费例外；168全文/78声明/136×3及核心6编译12受控VM、真实两类4重编保持，真实目标未执行。新控制只用既有冻结class/JAR和内存facts变体，不计恢复增量/holdout。r1选错capture夹具2pass/1fail原流保留，r2修正输入不改断言；2744源/资产/归档、17当前产物及正常构建绑定，target清理，磁盘不充运行资源。临时row/definition所有权范围提前仅静态观察，S0原门/历史失败不豁免。2.9/2.10/S1开放，17/59，S2未进入。


### 2026-10-26 / CB：共享成员放置政策归 Java

[结果](openspec/changes/restructure-demand-driven-recovery/results/s1-anonymous-member-placement-v1/README.md)、[验收](openspec/changes/restructure-demand-driven-recovery/results/s1-anonymous-member-placement-v1/acceptance-r1.json)、[六项](openspec/changes/restructure-demand-driven-recovery/results/s1-anonymous-member-placement-v1/six-gates.md)、[后继职责/依赖/验收](openspec/changes/restructure-demand-driven-recovery/results/s1-anonymous-member-placement-v1/next-boundaries.md)。八个关联body/span/匿名成员方法的声明/marker/注解准入、child flags、默认缩进、root-return/current-prefix放置与UTF-8范围核验归Java；root仅供原字段/Recovered变体事实并构造原index/kind/anchors，私有parts移除，无新模型/trait/read/cache。各接受域差异、clone/文本顺序和预算/发布保持，恢复46/schema12/解析4/v9不变。高层匿名interface/nested/super/double-brace候选、Header/owner/capture资格与专用暂存/冲突决策仍在root逐成员编排中，不能称整族已迁完；物理提交继续留root。

原政策同三新边界测试3/0；当前16目标473/0/0，原11政策/停止控制完整保持。2449继承（已含168枚举）及408核心全JSON/退出码严格相等，零语义/计费例外；CA42份metadata修正沿接受基线核对，不改其原false流。168全文/78声明/136×3及核心6编译12受控VM、真实两类4重编通过，真实目标未执行；新测试只复用旧JAR和布局事实变体，不计新恢复或holdout。2744源/资产、归档、17产物与正常构建绑定，target清理，磁盘仅构建护栏。第三轮历史六项与原JDK8/旧native失败限制保持。共享Interrupted、其他停止/资源/API、raw闭包及高层匿名族仍按2.9/2.10承接，17/59，S1未整体验收，S0原记录保持，S2未进入。


### 2026-10-26 / CA：enum helper typed-stop 与后继需求截断

[结果](openspec/changes/restructure-demand-driven-recovery/results/s1-enum-helper-stop-v1/README.md)、[收据](openspec/changes/restructure-demand-driven-recovery/results/s1-enum-helper-stop-v1/acceptance-r1.json)、[六项](openspec/changes/restructure-demand-driven-recovery/results/s1-enum-helper-stop-v1/six-gates.md)、[后继任务/依赖/验收](openspec/changes/restructure-demand-driven-recovery/results/s1-enum-helper-stop-v1/next-boundaries.md)。复用原DependencyProofFailure Error/Report，Java三政策入口透传泛型失败，root opt-in read与IR回调在全局停止时传原execution/diagnostics，外层消费一次；形状拒绝仍inner String、局部接受域保持，无新模型/trait/read/cache。恢复45→46/schema12/解析4/v9保持，不捏造报告缺失的Error quota。

baseline r2同7测试6pass/1fail，完整复现四Report路径2helper请求及停止后raw IR+1、三个预算首因被Cancelled覆盖、原报告/诊断丢失；两Error路径原正确。当前16目标470/0/0，六控制均1helper请求、停止后实际Body/六IR全0，无method/marker/proof半提交，四原Report/诊断保留，两原Error及BY三marker/BZ两helper保持。r1 proof控制误用AnalysisSteps未触发，第五断言失败/第六未跑原流保留；r2只改IrItems控制，不改预期。2449继承（含168枚举）中2407全JSON相同/42份Header停止metadata修正，408核心相同，退出码全部保持；42份仅追加原预算诊断与明确停止拒绝，正文/身份/execution/usage不变、零未审查差异，168全文/78声明/136×3核心及核心6编译12VM、真实两类4重编保持，真实目标未执行。2744源/资产/归档与17产物绑定，target清理。共享Interrupted、其他后继/read切点/资源/API、raw/匿名族及旧失败仍开放，17/59，S0原记录保持，S2未进入。


### 2026-10-26 / BZ：enum helper准备与shared-helper整组职责归Java

[结果](openspec/changes/restructure-demand-driven-recovery/results/s1-enum-helper-source-policy-v1/README.md)、[收据](openspec/changes/restructure-demand-driven-recovery/results/s1-enum-helper-source-policy-v1/acceptance-r1.json)、[六项](openspec/changes/restructure-demand-driven-recovery/results/s1-enum-helper-source-policy-v1/six-gates.md)、[后继任务/依赖/验收](openspec/changes/restructure-demand-driven-recovery/results/s1-enum-helper-source-policy-v1/next-boundaries.md)。Java持有完整selector/ordinal/helper/enum/values与常量身份准备，及helper table-set、同table enum一致性/key合并、单map/group证明选择和结果展开；原Prepared记录同迁，root只供原scope/target、ConfirmedRead借用facts与实际resolve/read/IR/执行合并和按index收集，无新模型/trait/read/cache。两pub政策入口不提供Engine artifact绑定权，恢复45/schema12/解析4/v9不变。

原政策同6测试6/0；当前16目标469/0/0，新两控制实际closed-set在IR前Body/六IR全0，定义冲突在一次共享IR后拒绝，Body1/六IR各1，与正例相同；无方法/marker半提交，三marker停止记录保持。2449继承（含BY168枚举）全JSON/退出码相等、零语义/计费例外，不重复跑矩阵；168全文/78声明/136×3核心与408全JSON、核心6编译12受控VM和真实两类4重编保持，真实目标未执行。BY raw summary false/48份修正按封存acceptance核对，不改历史。2744源/资产/归档、17产物与正常构建绑定，target清理。helper IR报告型停止消费/共享Interrupted及其余全停止/资源/API、raw与匿名专用族/旧失败仍归2.9/2.10，17/59，S0原记录保持，S2未进入。


### 2026-10-26 / BY：枚举同方法整组政策与相邻停止守卫

[结果](openspec/changes/restructure-demand-driven-recovery/results/s1-enum-group-source-policy-v1/README.md)、[验收](openspec/changes/restructure-demand-driven-recovery/results/s1-enum-group-source-policy-v1/acceptance-r1.json)、[六项](openspec/changes/restructure-demand-driven-recovery/results/s1-enum-group-source-policy-v1/six-gates.md)、[后继任务/依赖/验收](openspec/changes/restructure-demand-driven-recovery/results/s1-enum-group-source-policy-v1/next-boundaries.md)。Java接管完整方法分组/资格/extra-use/同方法拒绝/marker/AST emission/placement/整类克隆暂存和纯map label/coverage，三个既有模型同迁；root供事实/跨类读取IR、执行诊断及最终发布，无新生产模型/read/cache。helper准备/source资格和shared-helper table-set仍待审，不计已迁。

先复现预算被后续cancelled覆盖（baseline3/1、第一层守卫fixed-r3仍3/1），补enum后的ended、array与initializer入口守卫后fixed-r4=4/0，再迁移。恢复44→45；当前16目标467/0/0，固定内联/迁移后三marker记录相同，真实两map之后停止保留原方法/markers、无proof投影，实际Body decoder及六IR starts零增量；output/cancel付1257、elapsed0。2281继承中2275全JSON相同/6份停止修正、168枚举中126相同/42份header2停止修正、408核心相同，退出码全部保持；48份差异逐项审查，包括明确Refused、停止后IrItems减少及3个compact交付requested+68，未屏蔽费用字段；168全文/78声明/136×3核心、核心6编译12受控VM及真实两类4重编保持，真实目标未执行。2744源/资产/归档与17产物绑定，r7三构建target清理，r5/r6主动终止及失败原流保留。第三轮历史六项保持原限制；共享Interrupted、bridge等全部停止、raw闭包、实际资源/API与旧失败仍开放，2.9/2.10不勾选，17/59，S0原记录保持，S2未进入。


### 2026-10-26 / BX：整数常量名称资格、placement 与整组暂存归 Java

[结果](openspec/changes/restructure-demand-driven-recovery/results/s1-integer-name-source-policy-v1/README.md)、[收据](openspec/changes/restructure-demand-driven-recovery/results/s1-integer-name-source-policy-v1/acceptance-r1.json)、[六门](openspec/changes/restructure-demand-driven-recovery/results/s1-integer-name-source-policy-v1/six-gates.md)、[后续任务/依赖/验收](openspec/changes/restructure-demand-driven-recovery/results/s1-integer-name-source-policy-v1/next-boundaries.md)。Java持有整组/方法资格、原descriptor/AST匹配、prior staged冲突、专用text/span、唯一源范围与字段匹配、普通整类重放证书和发布前计费暂存；root只适配同次事实/锚点、原物理组装器及最终无失败发布，外层family编排保持。5函数政策、3生产文件，无新模型/read/cache；恢复44/schema12/解析4/v9不变。基线同4新测试4/0，当前16目标463/0/0，4公开控制前缀与实际reader Body decoder及六阶段IR starts零增量；late conflict/output/cancel无半提交，重复needle/重复调用保持。

2185直属报告（含BW96 assert）+96整数专项+408核心全JSON/退出码保持，零语义/计费例外；168全文/78声明/136×3核心及核心6编译12受控VM、真实两类4重编保持。额外匿名/double-brace相邻4目标18项覆盖共享证书，不混计JDK分母。elapsed仍Unsupported(jre_budget_interrupted)已实际复现，单列2.10共享错误消费闭包，不将4/0视为停止类型正确；原历史失败不豁免。名称Vec在stage返回时提前drop、重放String在投影组装前drop仅静态范围，不充资源门。r2主动终止/r3编译命名空间失败原流保留，r4三构建通过，2744源/资产及最终17二进制绑定，target清理。下一组enum-switch同方法整组事务，共享Interrupted原因独立修复；2.9/2.10未勾选，17/59，S0原记录保持，S2未进入。


### 2026-10-26 / BW：assert 类型化停止与 enclosing 链有界化

[结果](openspec/changes/restructure-demand-driven-recovery/results/s1-assert-stop-boundary-v1/README.md)、[收据](openspec/changes/restructure-demand-driven-recovery/results/s1-assert-stop-boundary-v1/acceptance-r1.json)、[六门](openspec/changes/restructure-demand-driven-recovery/results/s1-assert-stop-boundary-v1/six-gates.md)、[后继职责/依赖/验收](openspec/changes/restructure-demand-driven-recovery/results/s1-assert-stop-boundary-v1/next-boundaries.md)。复用既有失败通道，scanner 不再吞 budget/cancel，emitter 保留 typed stop，root 合并状态；outermost 每行计费并有限拒绝环/重复出边，成功计划前 poll。无新生产模型/read/cache，恢复43→44。441/0/0，四个实际公开边界控制保持五个已完成成员、逻辑Body数与六阶段实际IR starts；ir_items的13点暂存增量不伪装IR启动。2089+408完整报告相等；assert96中78相等、18仅明确的analysis_steps增加；168全文/78声明/136×3核心及核心6编译12受控VM、真实两类4重编保持。

r2新测试把逻辑ir_items当实际IR启动而失败，修正仅在cfg观测，r3生产代码与r2逐字相同、三构建通过；失败原流及68次CLI探索均保留，CLI探索没有复现旧公共假Complete。第三轮历史六门保持，不豁免旧native失败，不作全S1停止/资源通过。下一组迁移整数常量名称投影的完整资格/span/暂存事务，其后enum-switch同方法整组及匿名族专用事务；依赖同次facts、原子性及效果/完整输出验收，不按单样例拆补丁。2.9/2.10未勾选，17/59，S0原记录保持，S2未进入。


### 2026-10-26 / BV：assert整组政策与nested分类职责迁移

[结果](openspec/changes/restructure-demand-driven-recovery/results/s1-assert-group-source-policy-v1/README.md)、[验收](openspec/changes/restructure-demand-driven-recovery/results/s1-assert-group-source-policy-v1/acceptance-r1.json)、[后续机制/依赖/验收](openspec/changes/restructure-demand-driven-recovery/results/s1-assert-group-source-policy-v1/next-boundaries.md)。完整assert准入、synthetic field/clinit/outermost、Fieldref census、全fold/current-text、clinit stripping及暂存归Java，两个nested classifier/两旧候选类型同迁；root留3借用getter与同slice position发布，去掉逐项find后可半提交的分支，真实physical index保持。4函数/三生产文件，无新模型/read/cache，原接受域/拒绝/clone及0显式poll/2charge顺序保持，恢复43/schema12/解析4/v9不变，Rust-public面扩大不授予Engine绑定权。

旧生产+同一3新测：2原子性desired测试及1含2stop情形的audit；当前11目标438/0/0，assert7原样家族重编/default禁用与-ea启用及guard/detail效果顺序通过。late conflict691及wide1101/stopped1099/cap1100均无半提交。2089继承+96专项完整JSON/退出码严格保持，零语义例外；12wide投影/12不投影、72有限预算不冒充全部到达阶段或holdout。168全文/78声明、136×3及408完整JSON、核心6编译12受控VM、真实两类4重编保持，真实目标不执行。已有A1资产首次进入include清单，baseline/current2744（BU2743），无新fixture文件；源/归档/12产物和两当前成功构建绑定，target清理，616275429bytes仅构建磁盘。

**已实测但未修的停止缺口**：budget/cancel scanner分别Err，private assert边界均refused；emitter output stop仍为Unsupported assert_projection_stopped。S1_stop_gate=false，不能用438/0覆盖。outermost无预算/环检测仅静态风险，未执行环/变异目标，未外推Engine假Complete。下一组先闭合typed stop/outer链有界化，再清其余projector/raw消费者与2.10原资源/全部故障/API门。BQ EM11/BP两旧native失败未重跑/豁免；17/59，S1未整体验收，S0原记录保持，S2未进入。

# 当前接续：共享成员 writer 归 Java（2026-10-26，BU）

### 2026-10-26 / BU：共享成员 artifact/block writer 职责闭合

[结果](openspec/changes/restructure-demand-driven-recovery/results/s1-member-writer-source-policy-v1/README.md)、[验收](openspec/changes/restructure-demand-driven-recovery/results/s1-member-writer-source-policy-v1/acceptance-r1.json)、[后续职责/依赖/验收](openspec/changes/restructure-demand-driven-recovery/results/s1-member-writer-source-policy-v1/next-boundaries.md)。共享strict artifact parser、quote/block writer、精确UTF-8长度、raw marker label、current body/explanation及span mapper统一归Java，Artifact/Placed两原类型同迁；root只适配这些数据及提交。BT构造族去掉2个反向writer callback，改为第四个借用metadata getter，Java直接判断。13函数/三生产路径，无新模型/fixture/read/cache；class_source生产-243行、facade+8/Java+321行；20poll/26charge/0clone和最终poll保持。Rust-public面扩大但不授予Engine绑定/认证/提交权，恢复43/schema12/解析4/v9保持。

加同一单测的baseline root279/0，当前10目标428/0/0。72字符串组合实际UTF-8长度/annotation前缀与预算预估一致；原artifact正负/三placement/真实explanation当前正文冲突及span、BT两late failure继续通过，853/1583/1217/1582原流数值相同。2089继承完整JSON/退出码严格保持，零语义例外，仅规范核验后的actual engine及直属usage.elapsed_millis；168全文/78声明、136×3核心及408完整JSON、核心6编译12受控VM、真实两类4重编保持，真实目标未执行。72不是独立程序/报告/holdout，未新增恢复接受域。

2743源/归档与11产物、baseline1/current2正常构建绑定并清理target；601170834bytes仅构建磁盘。共同writer不重列待迁；剩余projector资格/marker事务和assert整组政策需真实消费者清点，2.9及2.10原资源/全部停止/历史失败/API双轨门保持。BQ EM11/BP两旧native失败未重跑/豁免；17/59，S1未整体验收，S0记录保持，S2未进入。

# 当前接续：共同 instance-array 构造族政策归 Java（2026-10-26，BT）

### 2026-10-26 / BT：共同 instance-array 构造族政策与暂存职责迁移

[结果](openspec/changes/restructure-demand-driven-recovery/results/s1-instance-array-source-policy-v1/README.md)、[验收](openspec/changes/restructure-demand-driven-recovery/results/s1-instance-array-source-policy-v1/acceptance-r1.json)、[后续职责/依赖/验收](openspec/changes/restructure-demand-driven-recovery/results/s1-instance-array-source-policy-v1/next-boundaries.md)。整个共同primitive-array constructor group的census、AST/member配对、所有构造器RHS/写入一致性、raw字段/调用descriptor资格、字段/成员冲突及全部暂存归Java；root供借用事实、原共享writer适配，并保留最终整组提交。4函数/两生产文件，root生产-419行/Java+517行，无新模型/fixture/read/cache；20poll/26charge/0clone与最终poll保持。共享ClassSourceMethod current-body/markers/annotations writer仍在root，下一组按完整writer迁移，不冒称所有callback是纯事实。恢复43/schema12/解析4/inputs v9保持，两个doc-hidden pub入口不授予Engine artifact绑定权。

baseline新增2事务测试后9/0；当前10目标427/0/0，late conflict付费853、末尾output cap wide1583/stopped1217/cap1582均零提交，迁移前后实测相同。1897继承+192专项完整JSON/退出码保持、零语义例外，仅规范核验后的actual engine及直属usage.elapsed_millis。专项8既有byte-array输入×目标7/8×三evidence×四预算：30wide投影/18不提升、144有限预算，不称全部到达group或所有primitive/独立holdout。168全文/78声明、136×3核心及408完整JSON保持，核心6编译12受控VM、真实两类4重编通过，真实目标未执行。2743源/归档/11产物、两成功构建绑定并清理target；601003696bytes仅构建磁盘。

baseline r1测试Debug误用、current r2错误Quality路径两次编译失败原流保留；分别定点修正后baseline r2/current r3成功，生产判据不变。native旧后处理多行匹配失败只重读原9/0流，无测试重放。BQ EM11及BP两旧native失败未重跑/豁免。下一组共享member writer，其余assert政策与2.9/2.10完整门仍开放；17/59，S1未整体验收，S0原记录保持，S2未进入。

# 当前接续：enum 外部静态 int 字段资格归 Java（2026-10-26，BS）

### 2026-10-26 / BS：enum 外部静态 int 字段资格完整职责迁移

[结果](openspec/changes/restructure-demand-driven-recovery/results/s1-enum-static-field-source-policy-v1/README.md)、[收据](openspec/changes/restructure-demand-driven-recovery/results/s1-enum-static-field-source-policy-v1/acceptance-r1.json)、[后续职责/依赖/验收](openspec/changes/restructure-demand-driven-recovery/results/s1-enum-static-field-source-policy-v1/next-boundaries.md)。enum 有限 int 构造表达式的完整外部字段资格、owner/shadow 政策归 Java；root 保留同次 enum 属性、selected read、owner pool/属性供给与执行停止编排。3 函数整体迁移、两个生产文件，root -183 行/Java +221 行，无新模型/测试/fixture/路径或读取/cache。原 raw 三元组去重/首次 source_owner、守卫/拒绝次序、3 poll/3 charge/5 clone 保持；owner mask 不含 ACC_ENUM，未扩大或收缩接受域。3 doc-hidden pub 扩大 Rust 可见面但不授予 Engine artifact 绑定权；恢复43/schema12/解析4/inputs v9 不变，实际实现摘要更新。

直属 BR 的 10420 封存载荷/2743 源及旧 315/0 原流重核后复用，不重复旧构建。当前 9 目标315/0/0，8 项字段机制测试通过；1825 继承与72字段专项完整 JSON/退出码保持、零语义例外，仅规范核验后的实际 engine 和直属 usage.elapsed_millis。72 为3输入×目标7/8×三evidence×四预算模式，18充分预算投影/54有限预算；不声称全部到达资格函数或独立 holdout/完整编译。168全文/78声明、136×3核心/408全JSON保持，核心6编译12受控VM及真实两类4重编通过，真实目标未执行。2743源/归档/10产物与两正常构建绑定，target清理，584082894bytes仅构建磁盘。

精确字段循环取消、缺失字段/owner及所有版本/flags未逐branch动态穷尽，原状态分流保持。BQ EM11 相邻断言与 BP 两旧 native 失败未重跑/豁免。BR静态组与BS字段组不重复列为待迁；下一完整组为 instance-array constructor 政策，要求同次 AST/读事实、原子性、效果/放置和停止验收。2.9消费者闭包与2.10实际资源/全部停止/历史失败/API双轨仍开放，17/59，S0原记录保持，S2未进入。

# 当前接续：普通静态初始化政策归 Java（2026-10-26，BR）

### 2026-10-26 / BR：普通静态初始化完整政策职责迁移

[结果](openspec/changes/restructure-demand-driven-recovery/results/s1-static-initializer-source-policy-v1/README.md)、[收据](openspec/changes/restructure-demand-driven-recovery/results/s1-static-initializer-source-policy-v1/acceptance-r1.json)、[后续职责/依赖/验收](openspec/changes/restructure-demand-driven-recovery/results/s1-static-initializer-source-policy-v1/next-boundaries.md)。普通class/interface初始化的完整proof、RHS field@1/constant-expression phase/效果与顺序政策、全部发射staging归Java；root只供同次借用事实，保留读取/IR、执行诊断与完整成功后的原子提交。7函数/5原有类型归Java，无新模型/Session/cache/read；物理字段顺序、class/interface不同接受域、原拒绝优先级和3charge/0显式poll/8clone调用保持，删除旧无用私有class_internal参数。报告两类型re-export；doc-hidden pub错误/helper扩大Rust可见面，但不提供新artifact绑定权或Engine注入入口。恢复43/schema12/解析4/inputs v9保持，实际Java/root摘要更新。

旧生产加两新对抗测试先实跑5目标24/0；当前9选定目标315/0/0，新增8类field@1错配拒绝理由逐字保持，late mismatch在首RHS已发射计费后仍零提交。1765继承+60专项完整JSON严格相同、零语义例外；专项10class×两个SourceTarget×三evidence为24 Proved/30 Refused/6 NotApplicable，不充60完整编译或holdout。168全文/78声明、136×3核心/408全JSON、核心6编译12受控VM及真实两类4重编保持，真实目标未执行。2743源/归档/10产物与两正常构建绑定，target清理，583731025bytes仅构建磁盘；首次baseline测试Option访问编译错误保留，修正后不改变生产判据。

BQ相邻EM11断言与BP两旧native失败继续原记录，不因本轮315/0改变其状态。静态组已完整迁移，不重复计为待迁；下一完整组为enum外部static int字段资格，然后核instance-array constructor族政策。2.9raw消费者闭包与2.10实际资源/全部停止/API双轨仍开放，17/59，S0原记录保持，S2未进入。

# 历史接续：enum 构造链类型化停止（2026-10-26，BQ）

### 2026-10-26 / BQ：enum 构造链类型化停止与第三轮六门复核

[结果](openspec/changes/restructure-demand-driven-recovery/results/s1-enum-constructor-stop-v1/README.md)、[收据](openspec/changes/restructure-demand-driven-recovery/results/s1-enum-constructor-stop-v1/acceptance-r1.json)、[第三轮六门](openspec/changes/restructure-demand-driven-recovery/results/s1-enum-constructor-stop-v1/six-gates.md)、[剩余职责及验收](openspec/changes/restructure-demand-driven-recovery/results/s1-enum-constructor-stop-v1/next-boundaries.md)。Java shape proof 保留外层 Budget/Cancelled，源码拒绝留内层；无新模型/read/cache，guard/拒绝文案/charge/poll/clone顺序保持，恢复42→43/schema12/解析4/inputs v9。实际helper反例由Ok(Some字符串拒绝)转为typed BudgetExceeded；公开local96点保留父方法文字/身份，10点避免后继Body解码（8/10→7），IR启动均7，不冒充S0资源或任意全局零读门。

当前6 native目标296通过/1 EM11 cap-two相邻断言失败，原r1退出101保留，r2只续未执行目标；旧/当前CLI同EM11请求均Complete且全JSON相等，旧原生精确测试未重跑，归2.10核对，不改预期或宣称全绿。1765完整报告严格保持、零语义例外；168全文/78声明、136×3核心及408全JSON、核心6编译12受控VM、真实两类4重编通过，真实目标未执行。新夹具只编译，未执行或充holdout。2743源路径/归档/7产物与三成功构建绑定，target清理；峰值538557236bytes仅构建磁盘。

第三轮六门历史通过及6218载荷重核已同步。strict验证器两次错误原流保留：138进程后的路径字符串误比、763进程处沿用输出路径误写BP一份交付；原文件已按封存hash精确恢复，10872个BP载荷与原索引重核。r3校核输入与输出重定向、复用763进程，只补1002，不重跑成功168/78/核心门。剩余普通静态初始化整组及enum外部字段源码资格仍按既有S1职责迁移，root保留物理事实/读取/停止/事务；2.9闭包与2.10资源/全部停止/历史失败/API双轨未完成，17/59，S0记录保持，S2未进入。

# 历史接续：enum constant-body raw owner机制（2026-10-26，BP）

[结果](openspec/changes/restructure-demand-driven-recovery/results/s1-raw-enum-owner-boundary-v1/README.md)。

BP关闭一个已公开复现的共享机制缺口：constant-body enum candidate把MUTF raw owner当标准UTF8，四处descriptor连接与物理原字节不在同一编码域。现在统一复用已有object_descriptor/array_descriptor/values_descriptor/value_of_descriptor；source拼写仍由严格MUTF names处理，reader/物理选择、同次Code/scan、flags/属性/构造链、interface→shape→child读取、预算与事务守卫保持。无新命名模型、DTO/Session/reader/resolver/cache或读取编排，不按样例名放行；facade和enum_constants源码逐字节不变。恢复语义41→42，schema12/解析4/inputs v9保持，实际Java实现更新；root实现指纹不变。

精确BO生产实现加同一永久测试/fixture的baseline为13通过/1失败，唯一新增失败是合法supplementary owner完整投影；当前raw目标8/0、lookup边界4/0、原enum-string2/0。公开三组/ASCII控制×SourceTarget7/8×essential/source_map/all：合法owner投影0/6→6/6；ASCII6、不可拼写6拒绝、坏MUTF6 Failed保持，其中18完整JSON对严格一致；六新恢复报告的root物理身份、声明、字段与方法报告保持。原样完整生成类编译0/3→3/3（JDK8 source7/8、JDK23 release8），共9次javac；ASCII当前全文等于旧已编译源，三次编译按pins复用。真实LabeledValue源码作依赖、无stub/原目标classpath；新变体未执行，不充行为oracle或holdout。

继承1780完整JSON对（1740CLI/24库/16目标入口）仅规范核验actual engine及直属usage.elapsed_millis，零语义例外；原生39目标1247通过/2同BO既有失败/4既有忽略，两旧红不改测试或预期。168全文/78声明/136×3核心、408 JSON/outline与AA12/130/3保持；核心实际6编译/12受控VM、真实两类4重编通过，真实包未执行。AS54＋Dual12共66源码/32历史行为对按23/21跳全文/pins复用，该复用新增JDK/产品0。旧五个raw测试和check正文逐字节保持，仅抽取result适配以断言拒绝/Failed并追加三测试。

2735封存BO来源+4显式新增fixture路径形成2739测量baseline；产品与其同路径、测试/fixture相同，仅两生产源码不同。五成功构建/40产物绑定actual producer，baseline/产品target均清理；21工具current构建/产品前核pin。主审另复算报告/原流/进程/input、39native原流、三组矩阵和编译、21脚本与BO10608payload；独立审查仅静态。初期摘要器读未序列化字段和空projection_inputs省略的验证器错误原流保留，修正读取后复用已有进程，不重跑成功前缀。

合法owner新成功沿既有有理由依赖链增加Header/Body/IR推进，逻辑usage另列，不能声称零读取增量或通过S0实际资源门。当前态校核已排除旧draft三项过时缺口（enum child方法名、integer raw去重、array声明匹配）。2.9仍待A1.6全消费者证据闭包审计，不凭本例勾选；arbitrary constructor Error→String属A1.3/2.10，须实际复现/不可达证明。2.10实际资源/全部停止/八旧失败/API双轨仍开放；17/59，S1未整体验收，S0原记录保持，S2未进入。

# 历史接续：整类源码目标拒绝政策归 Java（2026-10-25，BO）

[结果](openspec/changes/restructure-demand-driven-recovery/results/s1-source-unit-target-policy-v1/README.md)。

Java 7 整类源码拒绝政策已迁至现有 Java declaration::source_unit_target_refusal；root 的 source_text_with_member 只借用现有字段/方法事实。保持 release<8 短路、interface default/static+Code 判断、type_uses 原计算位置、interface 理由优先，以及 escaped→comment_text 的原文与换行。root 保留物理报告、anchors、预算/停止、可靠前缀和外层事务，不机械迁移报告装配。无新 DTO/Session/cache/read/测试/fixture；class_source -10 行，declaration +37 行；facade、source_proof、测试和其他来源逐字节保持，clone/poll/charge 调用点不增。恢复41/schema12/解析4/inputs v9保持，实际实现指纹更新。

保留核验完成：36原生目标1233通过/2同名既有失败/4既有忽略，与直属BN的逐目标计数/退出码/原流结果相同；两失败不改为通过，仍归2.10的测试形状/过宽断言及同夹具fold效果覆盖。1740CLI+24库入口完整JSON严格保持；额外四真实class×Java7/8×essential/all共16对完整CLI报告/32次调用，拒绝理由与正文、物理身份/方法报告保持。只规范已核验actual engine及直属usage.elapsed_millis，零语义例外。168完整源码/78声明/136×3核心与408 JSON/outline、AA12/130/3保持。核心实际6编译/12受控VM及真实两类4重编通过；真实目标未执行。AS54＋Dual12共66源码连接/32历史行为对按22/20跳复用，该复用新增JDK/产品0，不冒充新执行或holdout。

2735来源同路径/归档与五构建37产物同次绑定，target清理；原19脚本及补充1脚本在各自产品执行前核pin（报告工具schema在只读预检中修正，初始脚本/pins/错误记录保留；目标CLI对照r1误配PlainJar给standalone被BN拒绝，r2按既有测试改用SingleClass，错误调用另列，不称所有脚本均在构建前最终冻结）。主审另从原流复算1780完整JSON对、36原生结果/同名旧失败、20脚本及直属BN10544封存payload；独立复核仅静态。当前功能测试未单独分离static-only和两原因同类的输入，逐式源码等价支撑该边界，不冒充穷尽分支或实际资源收益。

本轮只收口这一处剩余语言政策；其余报告混排/物理anchors/提交留root。后续优先2.9 raw enum owner/descriptor三组的完整公开入口复现，再处理arbitrary constructor Stop与2.10实际读取/IR/live/RSS/时间/取消/全部停止门、八旧失败/API双轨。2.9/2.10/S1未整体验收，OpenSpec17/59，S0原记录保持，S2未进入。

# 历史接续：narrow member writer与方法放置完整政策（2026-10-25，BN）

[结果](openspec/changes/restructure-demand-driven-recovery/results/s1-narrow-member-writer-policy-v1/README.md)、[验收](openspec/changes/restructure-demand-driven-recovery/results/s1-narrow-member-writer-policy-v1/acceptance-r1.json)、[后续](openspec/changes/restructure-demand-driven-recovery/results/s1-narrow-member-writer-policy-v1/next-boundaries.md)。

完整render_member_class和append_family_method源码与范围政策归Java。Java决定narrow/generic/capture的row flags、annotations/header、static插入、capture/generic/hidden来源位置、字段/bridge/omitted与projected方法选择、无capture时constructor改名，以及nested/flat方法放置。root保留原validated child declaration expect、ClassDeclarationFacts clone、原MemberFamily/GenericStatic/capture对象与全部anchors factories；capture field查找/原expect通过lazy factory仍在header成功后，真实读取/恢复/停止/外层事务不下移。复用已有Java声明/indent/成员/range helper，无新DTO/Session/cache/read/测试/fixture。

class_source减少80行、Java增加217行。facade文件、Java旧前缀和root除完整renderer/append外所有代码及测试逐字节相同。原13个.clone、generic.header与staged整对象clone和D clone、原零poll/charge保持；普通class_declaration语义未换成更严格nested header。generic字段仅在原Some declaration分支替换并保留空markers，其anchor仍独立guard；bridge用原identity，omitted仅当hidden_fields非空；nested核缩进source_span原文相等，flat只原加偏移，拒绝/原expect时点/clone顺序不变，不放宽S5合同。没有实测资源收益声明。恢复41/schema12/解析4/inputs v9保持，实际实现指纹更新。

六项局部核验中五项通过；原生门36目标1233通过/2既有失败/4既有ignored，新增纳入member_family_identity的20项为18通过/2失败。r1流水线因此中断，原流保留；从直属BM封存2735源码单独重建同一target，在相同JDK/default stack实测同名18/2，未改产品/测试/预期。只补运行未执行的最后5项，35个已运行目标不重跑；两失败仍归2.10，不将六门称为全绿；1740CLI+24库入口完整JSON严格一致/零语义例外，仅规范已核验actual engine及直属usage.elapsed_millis；168全文/78声明/136×3核心、408 JSON/outline与AA12/130/3原字段保持。既有51 enum/nested/static-fold、11 generic/family相邻、5 mixed、7 annotation与20 family identity共94条本次原流（92成功、2同前继失败），机制提取不重复执行当前native（另一次前继对照分列）、不称全部guard/任意假helper、新恢复收益或holdout。核心实际6编译/12受控VM、真实两类4重编；native原受控编译/行为也通过，无逐JDK调用总数收据，真实目标未执行。AS54源报告/28历史对二十一跳、Dual12报告/4对十九跳全文/pins复用，该复用新增JDK/产品0。

当前r1五构建/37产物与2735同路径来源/归档绑定，另有BM源码对照的一次受控构建及一个产物，两个target清理，914279854bytes只作构建磁盘峰值。原17脚本构建/产品前冻结，后补4脚本另行冻结；r2在前继对照完成后误断剩余target名称，错误记录保留，r3按构建receipt修正编排且不重跑前继；r3原产品runner亦因native全绿前置未满足而停止，r4只独立运行尚未启动的八个保留检查，不称原六门流水线通过；成功门未重跑；独立source仅静态，主审另复算1740报告/交付流/进程/input/producer、24原report行、36stdout计数、94测试行、17脚本和直接BM父10518payload。下一步按2.9对raw enum owner/descriptor及arbitrary stop做完整公开入口复现，剩余职责审查确认Java7整类拒绝门是下一最小迁移候选；物理报告装配/anchors/事务留root，不机械搬整个writer。2.10实际read/BodyIR/live/RSS/时间/取消/停止/八旧失败/API双轨仍待S0原门；本次新纳入的两项旧形状/过宽断言及同夹具fold效果覆盖缺口单列known-native-failures-r1.md，不混改。2.9/2.10/S1开放，17/59，S0原记录保持，S2未进入。

# 历史接续：嵌套enum writer与annotation源码阶段（2026-10-25，BM）

[结果](openspec/changes/restructure-demand-driven-recovery/results/s1-nested-enum-annotation-writer-policy-v1/README.md)、[验收](openspec/changes/restructure-demand-driven-recovery/results/s1-nested-enum-annotation-writer-policy-v1/acceptance-r1.json)、[后续](openspec/changes/restructure-demand-driven-recovery/results/s1-nested-enum-annotation-writer-policy-v1/next-boundaries.md)。

完整nested enum writer及annotation纯源码阶段归Java。迁移前源码没有nested_annotation_source_text，annotation在facade::project_class_source_nested_annotation内：本轮迁移原单元素资格（raw Item name/descriptor、value()F、default、NoBody、单一no-body marker）、owner源码引用排除和annotation块拼写；root保留family/complete/声明、child/root原source_text重放、原facts clone、poll/anchors/OutputBytes/事务。enum constants/字段/initializer/constructor/implicit/clinit/method/descendant次序及header完整纯政策归Java，root保留原ClassDeclarationFacts clone/模型/anchors factory/NestedClassSourceText封装；仅失用的root header适配删除，复用既有Java声明/indent/成员拼写/indented_offset。无新DTO/Session/cache/read/测试/fixture。

class_source减少87行，facade减少20行，Java增加233行。旧Java前缀、两个root文件除完整enum writer/annotation projector和唯一失用wrapper外所有代码/测试逐字节相同。原6个.clone、annotation的1poll/1OutputBytes及两个ClassDeclarationFacts clone保留；child/root replay及poll→anchors→commit代码块逐字节相同。enum method span保留原文相等，descendant只沿原非空与UTF-8边界合同，不强合并；annotation逐行补四空格包括空行，不改用跳空行indent。annotation binary_source_name/header/header_text阶段返回时更早drop，不充RSS/时间收益。恢复41/schema12/解析4/inputs v9保持，实际实现指纹更新。

六局部门通过：35native1215/0/4既有ignored，既有annotation七项与mixed预先纳入主计划；1740CLI+24库入口完整JSON严格一致/零语义例外，仅规范已核验actual engine元组和直属usage.elapsed_millis；168全文/78声明/136×3核心、408JSON/outline与AA12/130/3原字段保持。51enum/nested/static-fold机制、11generic/header/family相邻、5mixed及7annotation为74条本次原生成功原流，不增加执行次数，不称全部guard、任意伪helper、独立holdout或新恢复收益。核心实际6编译/12受控VM、真实两类4重编；原native夹具受控编译/行为亦通过，无逐JDK调用总数收据，真实目标未执行。AS54源报告/28历史对二十跳、Dual12报告/4对十八跳全文/pins复用，该复用新增JDK/产品0。

r1五构建/36产物与2735同路径来源/归档绑定，target清理，913762419bytes只是构建磁盘峰值。17脚本产品前冻结，成功门不重跑。独立source只静态，主审复算1740报告/交付原流/进程/input/producer、24原report行、35stdout计数、74测试行、17脚本与父BL10520payload。下一完整职责组是render_member_class与append_family_method的narrow family源码与范围政策，root保留物理/capture/generic事实和anchors/事务，不照抄static-fold接受域；raw owner/arbitrary stop、2.10实际read/BodyIR/live/RSS/时间/取消/停止/八历史失败/API双轨仍待原门。2.9/2.10/S1开放，17/59，S0原记录保持，S2未进入。

# 历史接续：嵌套成员writer完整源码政策归属（2026-10-25，BL）

[结果](openspec/changes/restructure-demand-driven-recovery/results/s1-nested-member-writer-policy-v1/README.md)、[验收](openspec/changes/restructure-demand-driven-recovery/results/s1-nested-member-writer-policy-v1/acceptance-r1.json)、[后续](openspec/changes/restructure-demand-driven-recovery/results/s1-nested-member-writer-policy-v1/next-boundaries.md)。

嵌套static/instance member writer完整源码政策归Java：row flags/物理raw parents与重写父类型/header、annotation/marker、字段和方法的staged→retained→physical来源优先级、constructor改名和capture首参数消除、hidden-write clamp、缩进前后UTF-8范围与原source span逐字节核对。两个专用参数扫描/范围位移helper及三个共享declaration_member/comment_member/indented_offset单一实现归Java；根层其余调用保留薄包装。root保留原ClassDeclarationFactsclone、同次ClassSource/MemberFamily对象和全部物理anchors工厂、原NestedClassSourceText封装、真实恢复/停止与事务。泛型accessors只借用现有对象，factories创建原对象，无镜像DTO/Session/cache/read/测试/fixture。

class_source减少225行，Java增加395行；facade文件和Java旧前缀逐字节保持，root除完整writer、两专用helper和三个共享薄包装外其余代码及测试逐字节保持。原15个.clone和1个staged整对象.cloned调用点、原零poll/charge保持；原ClassDeclarationFactsclone留root。header已用尽的父类型String/接口Vec在header返回时早于原writer结束释放，不宣称实际RSS/时间收益。原首参数扫描的泛型/注解/字面值/嵌套括号、位移次序、hidden-write重夹与span不一致拒绝均保持，不修或放宽S5域。恢复41/schema12/解析4/inputs v9保持，实现指纹更新。

六局部门通过：34native1208/0/4既有ignored，mixed目标预先纳入主计划；1740CLI+24库入口完整JSON严格一致/零语义例外，仅规范核验后的actual engine元组与直属usage.elapsed_millis；168全文/78声明/136×3核心、408JSON/outline及AA ordinary12/budget130/names3原字段保持。51既有enum/nested/static-fold机制、11generic/header/family相邻回归及5mixed/instance集成取自同次成功原流，不额外执行，不称新恢复收益、全部私有guard覆盖或独立holdout。核心实际6编译/12受控VM、真实两类4重编；native夹具既有受控编译/行为也通过，无逐JDK调用总数收据，真实目标未执行。AS54源报告/28历史行为对十九跳、Dual12源报告/4对十七跳按全文及pins复用，该复用新增JDK/产品0。

r1五构建/35产物与2735原路径来源/归档绑定，target清理，913728667bytes仅作构建磁盘峰值。17脚本构建/产品前冻结，成功门未重跑；独立source只静态，主审另核1740原报告/交付流/进程/input/producer、24原report行、34stdout计数、67既有测试行、17脚本及直接BK父10516payload。下一完整职责组审nested enum/annotation两个writer，沿不同接受域整体迁移并复用已有纯helper，不为表面相似统一资格；raw owner/arbitrary停止、2.10实际read/BodyIR/live/RSS/时间/取消/停止/八历史失败/API双轨仍待原门。2.9/2.10/S1开放，17/59，S0原记录保持，S2未进入。

# 历史接续：member-fold整体组合资格与目标准备（2026-10-25，BK）

[结果](openspec/changes/restructure-demand-driven-recovery/results/s1-member-fold-composition-policy-v1/README.md)、[验收](openspec/changes/restructure-demand-driven-recovery/results/s1-member-fold-composition-policy-v1/acceptance-r1.json)、[后续](openspec/changes/restructure-demand-driven-recovery/results/s1-member-fold-composition-policy-v1/next-boundaries.md)。

member-fold的完整组合资格与目标准备归Java：先三轮核对物理complete/声明、Signature承载、enum/initializer承载，再按成员物理顺序准备精确root/child源码名、child重放后row/binary一致性与原targets；re-recovery完整Java质量门和bridge owner MUTF拼写一并归属。root保留真实IR/recovery、execution/identity、原evidence/instance/capture feeds、retained上下文构建及source_text重放供给、物理模型/anchors/writer和最终事务。四纯函数使用借用accessor与原target factory，无新DTO/Session/cache/read/fixture/测试函数。

facade减少64行、Java增加137行；原2处poll/1处OutputBytes/25个clone调用点保持。三轮拒绝顺序、enum短路后才算initializer order、有order才查clinit、child源码名成功后才重放、重放后才核row/binary与完整targets之后才re-recovery保持。实际IR/recovery调用、execution/identity与原selection代码块逐字节相同，其余root/全部测试和Java旧区域逐字节保持。child重放临时对象较原迭代结束提前drop，不充RSS/时间收益；重复expect仍位于原complete不变量之后。当前accessors均纯借用，Fn本身不保证纯度，静态审查记录了重复调用/元组求值边界，不将底层helper当可信artifact构造器。恢复41/schema12/解析4/inputs v9保持，实际实现指纹更新。

六局部门通过：主33native1203/0/4既有ignored；覆盖复核后补充既有mixed/instance目标5/0/0，合计34目标1208/0/4，1740CLI+24库入口完整JSON严格一致/零语义例外，仅规范核验后的实际engine元组与直属usage.elapsed_millis；168全文/78声明/136×3核心、408JSON/outline及AA ordinary12/budget130/names3原字段保持。51既有enum/nested/static-fold机制与11项既有generic/header/family相邻回归取自本次成功原流，不增加执行次数，不声称每条穿过全部内部gate或独立holdout。核心实际6编译/12受控VM、真实两类4重编，真实目标未执行；native原fixture受控编译/行为亦通过但无逐JDK调用总数收据。AS54源报告/28历史行为对按十八跳、Dual12报告/4对按十六跳全文/pins复用，该复用新增JDK/产品0。原r1五构建/34产物及独立补充mixed一次构建/1产物均与同一2735来源/归档绑定，两个target清理，932443512bytes仅作构建磁盘峰值；17脚本产品前冻结，成功产品门未重跑。

独立source审查只静态；主审复算1740报告/原流/进程/input/producer、24原report行、主33stdout计数、17原脚本、独立补充脚本/来源/5项mixed原流、51机制和父10500payload。下一完整职责组是nested_static_member_source_text的header/字段方法来源优先级/constructor捕获参数消除/缩进范围政策，root保留原模型/anchors及提交，不按旗标零散打补丁。raw enum_owner、arbitrary停止、2.10实际read/BodyIR/live/RSS/时间/取消/停止/八历史失败/API双轨仍待原门。2.9/2.10/S1开放，17/59，S0原记录保持，S2未进入。

# 历史接续：static-fold owner源码改写完整政策归属（2026-10-25，BJ）

[结果](openspec/changes/restructure-demand-driven-recovery/results/s1-static-fold-source-policy-v1/README.md)、[验收](openspec/changes/restructure-demand-driven-recovery/results/s1-static-fold-source-policy-v1/acceptance-r1.json)、[后续](openspec/changes/restructure-demand-driven-recovery/results/s1-static-fold-source-policy-v1/next-boundaries.md)。

完整static-fold owner texts纯政策归Java：字段声明、override/retained projected来源组合、所有覆盖origins、声明资格、Code/handler/SSA来源、bridge/capture/PendingEdit和完整替换范围。root保留原override/retained物理事实、原条件惰性owner/Exceptions与IR读取及stop/execution/identity，raw throws身份成员关系供给、原ClassSource放置、anchors/OwnerTexts和事务。原局部PendingEdit单一实体整体移Java，bridge/capture沿既有Java单一helper，无新DTO/Session/cache/读取/报告镜像。

facade减少507行、Java增加750行。原5处poll/5处AnalysisSteps费用、28处clone、全部字面值保持；IR/stop/identity与owner read/headers/pool/attributes代码块、其余根层/测试和Java原区域逐字节保持。static-fold所有覆盖段并集、CP首匹配→handler→SSA不能改用BI最短origin/唯一enum字段规则；projected不支持组合拒绝、text-only投影不回退physical、声明only零IR、descriptor不命中才读Exceptions、原physical-only scratch/text与relation clone保持。descriptor_entry/declaration_edits等耗尽对象阶段返回时较早drop，不充RSS/时间收益。恢复41/schema12/解析4/inputs v9不变，实现指纹更新。

六局部门通过：33native1203/0/4既有ignored，1740CLI+24库入口完整JSON严格一致/零语义例外，仅规范实际engine元组和直属usage.elapsed_millis；168全文/78声明/136×3核心、408JSON/outline与AA ordinary12/budget130/names3原字段保持。51既有机制为43enum/nested回归+8static-fold集成，取自本次两个目标原流，不计新测试/恢复收益或穷尽全部guard。24非enum库入口只是相邻回归，不充编译oracle/独立holdout。核心实际6编译/12受控VM、真实两类4重编，真实目标未执行；AS54源报告/28历史行为对按十七跳、Dual12报告/4对按十五跳全文/pins复用，复用新增JDK/产品0。r1五构建/34产物与2735来源/归档同次绑定，target清理，913441918bytes只作构建磁盘峰值；17脚本产品前冻结，成功产品门不重跑。

独立source审查只静态；主审另复算1740报告/原流/进程/input/producer、24原report行、33stdout计数、17脚本、51机制和父10505payload。原native resume日志未pin风险未混修，本次不恢复旧prefix。下一完整组审member-fold整体组合资格与目标准备，root真实恢复/一致性重放/放置/提交不下移；raw enum_owner、arbitrary停止及2.10实际read/BodyIR/live/RSS/时间/取消/停止/八历史失败/API双轨仍待原门。2.9/2.10/S1开放，17/59，S0原记录保持，S2未进入。

# 历史接续：nested enum方法引用改写完整政策归属（2026-10-25，BI）

[结果](openspec/changes/restructure-demand-driven-recovery/results/s1-nested-enum-reference-policy-v1/README.md)、[验收](openspec/changes/restructure-demand-driven-recovery/results/s1-nested-enum-reference-policy-v1/acceptance-r1.json)、[后续](openspec/changes/restructure-demand-driven-recovery/results/s1-nested-enum-reference-policy-v1/next-boundaries.md)。

nested enum方法引用改写的完整纯政策按原读取前后分三阶段归Java：源码token/必要origins/声明候选，已读取IR的getstatic/CP/物理字段证明，完整替换与字节范围调整。root保留原owner词法prefix与Recovered事实选择、原位置body edits非空才analyze_method_ir、error/stop与execution/identity合并，原ClassSource span/member放置、四类anchors/MethodText和事务。声明only零IR；回调只供借用fact或装配原输出，不隐藏reader，不新建DTO/Session/cache/报告镜像。

facade减少187行、Java增加331行，三处poll与三处AnalysisSteps费用、15处原clone、原字面值保持；原IR/stop/identity代码块及函数外根层/测试、Java原有区域逐字节保持。MethodItem raw descriptor与field.identity.member按原来源，必要projection_origins不替换为可选source_map；最小覆盖origin并列合并、唯一FieldRef常量匹配、overlap/offset/placement拒绝顺序不变。Code/field纯阶段仅String Refused；prepare预算Stop与render offset硬Error保留。原单声明/RecoveryReport scratch clone不增不减，耗尽的借用索引工作Vec在阶段返回时较旧root迭代末尾早drop，不宣称RSS/时间收益。恢复41/schema12/解析4/inputs v9保持，实现指纹更新。

六局部门通过：33native1203/0/4既有ignored；1740CLI+24库入口完整JSON严格一致、零语义例外，仅规范已核验engine元组与直属usage.elapsed_millis；168全文/78声明/136×3核心、408JSON/outline及AA ordinary12/budget130/names3原字段保持。43既有enum/nested机制取自本次原流（40继承+3当前重点），不计新测试/恢复收益，不穷尽私有guard/任意伪helper；24非enum入口不充编译oracle/独立holdout。核心实际6编译/12受控VM、真实两类4重编通过，真实目标未执行；AS54源报告/28历史行为对按十六跳，Dual12报告/4对按十四跳全文/pins复用，复用新增JDK/产品0。r1五构建/34产物与2735来源/归档绑定，target清理，912416690bytes仅构建磁盘峰值；17脚本构建/产品前冻结，无成功产品门重跑。

首份静态复核因任务相对路径歧义误写repo/results并用HEAD，原件与位置/基线说明保留，不作为前继证明。新复核使用正确BI/pre-edit绝对路径，源摘要与封存BH manifest匹配；其结论仍只静态，主审另外复算1740报告/原流/进程/input/producer、24原report行、33stdout计数、17脚本和父10500payload。原native resume日志未pin内容的风险未混修，本次不恢复旧prefix。下一完整共享机制为static-fold字段/方法引用改写；raw enum_owner、arbitrary停止及2.10实际资源/停止/八历史失败/API双轨仍待原门，不被逻辑usage/构建磁盘或局部门替代。2.9/2.10/S1开放，17/59，S0原记录保持，S2未进入。

# 历史接续：ordinary enum源码投影完整政策归属（2026-10-25，BH）

[结果](openspec/changes/restructure-demand-driven-recovery/results/s1-enum-ordinary-source-policy-v1/README.md)、[验收](openspec/changes/restructure-demand-driven-recovery/results/s1-enum-ordinary-source-policy-v1/acceptance-r1.json)、[后续](openspec/changes/restructure-demand-driven-recovery/results/s1-enum-ordinary-source-policy-v1/next-boundaries.md)。

完整ordinary enum源码投影政策归Java：不同Signature marker、隐藏参数/源码参数注解、delegating/terminal构造器、varargs/user-tail、initializer目标与空enum分号统一在原函数完整阶段内检查和拼写。原prefix_method_annotations整体迁为Java单一实现供全部原消费者共用。root三accessor只借用原FieldItem/MethodItem的独立raw字段、owner、声明/markers及既有Java注解实体，原group借用，terminal/initializer文本move；Java返回六个原局部值，root装配唯一原Projection，继续持有ClassSource、读取/发射、retained与事务。

class_source减少372行、Java增加501行，原字符串顺序、1处poll、1处IrItems与2处OutputBytes收费及先分配/后收费/拒绝顺序保持。普通recognized marker仍只核refused+len1、arbitrary仍核len<=1/substring，未混同或放宽；initializer原totalUnits/Map固定条件保持。root其余字节与测试、Java原有区域保持；无新增DTO/Session/cache/read/clone/test/fixture/源路径，恢复41/schema12/解析4/inputs v9不变，实现指纹更新。

六局部门通过：33native1203/0/4既有ignored，1740CLI+24独立库入口完整JSON严格一致、零语义例外，仅规范实际engine元组与直属usage.elapsed_millis；168全文/78声明/136×3核心、408JSON/outline与AA ordinary12/budget130/names3原字段保持。40项当前既有机制来自本次原流（16普通投影重点+24body相邻），不是新测试或全部私有guard覆盖；24非enum库入口不计编译oracle/独立holdout。核心实际6编译/12受控VM、真实两类4重编通过，真实目标未执行。AS54源报告/28历史行为对按十五跳，Dual12报告/4对按十三跳全文/pins复用，复用新增JDK/产品0。r1五构建/34产物与2735来源/归档同次匹配，target清理，917616228bytes只作构建磁盘峰值；17脚本构建/产品前冻结，无成功产品门重跑。

完整原流审计复算1740报告/进程/input/producer、24入口原行、33stdout计数、17脚本与父10533payload。独立source复核仅静态，不冒充动态验收；本轮不从旧native prefix恢复，原resume日志未pin内容风险仍单列。下一完整组为nested enum引用改写的词法/origins/资格与替换政策，root保留条件IR读取、execution及物理anchors/提交；其recovery.clone成本仅为静态待测事项。raw enum_owner与arbitrary停止传播继续完整公开入口复现，2.10实际read/BodyIR/live/RSS/时间/取消/停止/八历史失败/API双轨不被局部门替代。2.9/2.10/S1仍开放，17/59，S0原记录保持，S2未进入。

# 历史接续：body源码投影完整资格与拼写归属（2026-10-25，BG）

[结果](openspec/changes/restructure-demand-driven-recovery/results/s1-enum-body-source-policy-v1/README.md)、[验收](openspec/changes/restructure-demand-driven-recovery/results/s1-enum-body-source-policy-v1/acceptance-r1.json)、[后续](openspec/changes/restructure-demand-driven-recovery/results/s1-enum-body-source-policy-v1/next-boundaries.md)。

完整body-source projection资格与拼写归Java，root只用四个纯借用accessor供给原field/method/source/constant facts，并消费五个原局部值组装唯一EnumConstantSourceProjection。ClassSource、child方法Arc、retained/emission安装与事务继续root持有；Java不查询、不读Body、不复制报告。原共享indent与user-tail构造器拼写整体迁Java，全部原消费者共用单一实现，不留重复helper，不新增DTO/候选实体/Session/cache/读取路径。

class_source减少163行、Java增加262行。常量/field/BCI/owner/name、child配对与非空methods、declaration/markers/Recovered/text闭合、backing与implicit原物理字段、SingleString/arbitrary构造器资格与文本保持。MethodItem raw name/descriptor与identity分别借用，不相互替代；原字面值、两处poll、四处OutputBytes charge、先收费再拒绝和单次constructor String clone保留，方法Arc只借用。root其余代码与测试、Java原有区域逐字节保持，0新测试/fixture/源路径。恢复41/schema12/解析4/inputs v9不变，实际实现指纹更新。

六局部门通过：33native共1203 passed / 0 failed / 4既有ignored；1740 CLI+24独立库入口完整JSON严格一致，零语义例外，仅规范经核验engine元组与直属usage.elapsed_millis。168全文/78声明/136×3核心、408完整JSON/outline以及AA ordinary12/budget130/names3原字段子集保持。当前原流确认24项既有body候选/关系/投影/原子性/停止机制；24非enum库入口只是相邻回归，不充编译oracle/独立holdout，也不穷尽所有私有guard或任意伪helper。

核心实际6编译/12受控VM，真实两类4重编通过，真实目标未执行。AS54源报告/28历史行为对按十四跳、Dual12源报告/4对按十二跳，以完整源码和历史pins复用，复用新增JDK/产品调用0。r2五构建/34产物与2735来源/归档同次绑定，target清理；910916341bytes仅是构建磁盘峰值。17脚本构建/产品前冻结，成功产品门不重跑。

r1被主审主动SIGINT中断：适配误用identity中同名字段，原规则应核MethodItem独立raw字段；exit130、0完整命令/二进制/native产品JDK，finally清理target，receipt.failure=null不代表成功。原源码/归档/plan/预检/静态审查/原流保留，r2仅修借用字段、四只读预检重跑并用全新target/bin；当前product-build-r1为固定输出名，实际build_attempt2。原静态source复核未识别差异，主审r2单列补核。工具native resume仅查日志存在的静态风险不声称已修，本轮不恢复既有native前缀，主审终验复算全部33原流计数、1740报告与进程/input/producer、24原report行、17脚本和封存父10532payload。

下一完整组为ordinary enum source projection的构造器/注解/initializer资格与文本，不按单flag堆helper。旧raw enum_owner UTF8门槛与arbitrary Error转String仍需完整公开入口分组复现；2.10实际read/BodyIR/live/RSS/时间/取消/停止/慢sink/panic、八历史失败和API临时双轨继续原门。逻辑usage与构建磁盘不替代实际资源门；2.9/2.10/S1未整体验收，OpenSpec17/59，S0原记录保持，S2未进入。

# 历史接续：child读取前候选与shape完整政策归属（2026-10-25，BF）

[结果](openspec/changes/restructure-demand-driven-recovery/results/s1-enum-body-candidate-policy-v1/README.md)、[验收](openspec/changes/restructure-demand-driven-recovery/results/s1-enum-body-candidate-policy-v1/acceptance-r1.json)、[后续](openspec/changes/restructure-demand-driven-recovery/results/s1-enum-body-candidate-policy-v1/next-boundaries.md)。

child读取前的完整候选/shape政策已分为Java三个纯阶段：候选预检、已读接口资格、最终Code/factory/prefix/constructor/direct-BCI shape。root在原位置显式读取必要接口，shape完成后才执行原child loop；Java没有content/environment/loader/execution对象，也不通过回调隐藏resolver/reader。原constructor/marker辅助和物理member构造闭包归Java，同一私有构建函数保留原字段clone；root测试只调整导入，测试体不改。

阶段tuple转交原identity、Code、constructions、raw成员/常量、source_ctor/shared参数和可选接口owner，不新增DTO/候选实体/Session/cache、报告深拷贝或Body。仅耗尽的借用索引工作Vec在纯阶段返回时较早释放，拥有的facts/AST/scan payload和后续必需值不复制，也不据此宣称RSS收益。facade减少564行、Java增加706行，七处预child费用与四处poll、候选/常量/API/bridge/abstract/interface原guard及字面值保留。root原child读取/恢复、group/census、preview/emission/诊断/事务及后续测试、Java既有区域逐字节保持。恢复41/schema12/解析4/inputs v9不变，实现指纹更新，公开raw/helper不授予可信Engine binding。

六项局部门通过：33native共1203 passed / 0 failed / 4既有ignored；1740 CLI与独立24入口完整JSON严格一致，零语义例外，只规范已核验的实际engine元组和直属usage.elapsed_millis。168全文/78声明/136×3核心、408 JSON/outline及AA body12/budget130/names3原字段子集保持。当前r2原流确认24项既有candidate/relation/nested/child/group/prefix/行为/停止机制，不穷尽全部私有guard；24非enum入口只作相邻回归，不是编译oracle或独立holdout。

核心实际6编译/12受控VM、真实两类4重编保持，真实目标未执行。AS54源报告/28历史行为对按十三跳，Dual12源报告/4对按十一跳，以完整源码和历史pins复用，复用新增JDK/产品调用为0。当前r2五构建全部通过，2735来源/归档与34产物同次绑定，target清理；910876185bytes仅是构建磁盘峰值。11tools/2adapters/4辅助脚本产品前冻结，成功产品门未重跑。

首次check因descriptor解析None分支仍返回旧空Vec而E0308退出101，0二进制/产品；失败源码、manifest/archive/freeze、plan/预检/原流保留。只将该分支包装改为Ok(None)，root仍立即返回原空关系；四只读预检重跑后用全新r2 target/bin构建，17脚本未改。当前product-build-r1是固定检查点名，build_attempt实际为2，不能写成首次全绿；源审查r1仅静态，r2单独证明一处分支包装修复。

剩余完整source projection资格/拼写按下一职责组推进。std::str::from_utf8(enum_owner)旧门槛及arbitrary constructor Error转String路径仍保留；前者需以完整公开入口覆盖合法supplementary/不可拼写/坏MUTF后修正，后者需区分本地quota先于ledger计费和operation stop闩锁，再用实际read/BodyIR复现，当前不宣称缺陷或修复。2.10继续S0原资源/各类停止/慢sink/panic/八历史失败/API临时双轨门，逻辑usage与构建磁盘不能代替。2.9/2.10/S1未整体完成，17/59，S0原记录保持，S2未进入。

# 历史接续：body-family group 完整政策归属（2026-10-25，BE）

[结果](openspec/changes/restructure-demand-driven-recovery/results/s1-enum-body-group-policy-v1/README.md)、[验收](openspec/changes/restructure-demand-driven-recovery/results/s1-enum-body-group-policy-v1/acceptance-r1.json)、[后续](openspec/changes/restructure-demand-driven-recovery/results/s1-enum-body-group-policy-v1/next-boundaries.md)。

Java 已拥有 prove_enum_constant_body_group 的完整 shape/API/sidecar/成员与子类资格政策，以及原 GroupShape 单一实体。root wrapper只借用原方法/字段/关系事实，仍区分 raw name/descriptor 与物理 identity；ClassSourceMethod 及其 Arc、两个现有 Proved 输出继续归 root，由原字段装配回调保持源参数、subclass身份与同一child methods handle。Java 不查询、不读Body、不复制报告；没有镜像DTO、新实体、Session/cache或读取路径。

facade 减少441行，Java增加645行；67个拒绝/格式字符串顺序保持，一次 source_methods.len()+relations.len() 计费、两个主循环poll及内层Refused/外层Stop保持。固定String/zero/arbitrary域、same-shape、隐式API、terminal clinit/structured sidecar/write、abstract override、用户隐式成员使用、常量顺序、direct/anonymous关系和child唯一性、assigned-field/构造器展示均按原guard与原顺序检查。String仍要求signature-refused=true，markers为空或len==1，原本不查marker文本；arbitrary保留独立的len<=1/substring门。原entry/物理读/child恢复/census/preview/emission/事务与测试体、Java既有区域逐字节保持，不新增测试/fixture/固定样例分支。恢复41/schema12/解析4/inputs v9保持，实现指纹更新，公开raw/helper不授予可信Engine binding。

六项局部门通过：33 native共1203 passed / 0 failed / 4既有ignored；1740 CLI与独立24入口完整JSON严格一致，零语义例外，只归一化经核验的实际engine元组和直属usage.elapsed_millis。168全文、78声明、136×3核心及408 JSON/outline保持；AA body12/budget130/names3仍仅验原字段子集。当前原流确认20项既有group/child/field/prefix/原子性/行为/停止机制测试，不穷尽全部私有guard。24入口是非enum相邻回归，不计作group编译oracle或独立holdout。

核心实际6编译/12受控VM、真实两类4重编通过，真实目标未执行。AS54源报告/28历史行为对按十二跳，Dual12源报告/4对按十跳，以完整源码和历史pins复用，复用新增JDK/产品调用为0。当前r1五构建全部通过，2735来源/归档和34产物同次绑定，target清理；909621335bytes仅为构建磁盘峰值。11 tools、2 adapters、4辅助脚本在构建/产品前冻结，成功产品门未重跑。源码审查原件误写当前BE/子目录且为英文，原件保留，根层中文整理/JSON复制与精确marker说明另有摘要承接；没有因此修改源码、工具或封存父证据。

下一组先按完整阶段处理 resolve_enum_constant_body_relations 的main-class候选/shape准备政策，保留实际dependency读取、loader/definition、收费与停止位置；随后处理完整 source projection 资格/拼写，root继续持有ClassSource、emission安装与事务。raw enum_owner/MUTF、method/BCI/span和A1.6证据闭包继续2.9。2.10实际read、Body/IR、live/peak、RSS/耗时/取消、各类停止/慢sink/panic、八历史失败与API/临时双轨仍按S0原门；逻辑usage与构建磁盘不能替代。2.9/2.10/S1未整体完成，17/59，S0原记录保持，S2未进入。

# 历史接续：body-family 构造器与 initializer-prefix Code 政策归属（2026-10-25，BD）

[结果](openspec/changes/restructure-demand-driven-recovery/results/s1-enum-body-code-policy-v1/README.md)、[验收](openspec/changes/restructure-demand-driven-recovery/results/s1-enum-body-code-policy-v1/acceptance-r1.json)、[后续](openspec/changes/restructure-demand-driven-recovery/results/s1-enum-body-code-policy-v1/next-boundaries.md)。

body-family 的五个完整 Code 政策（constructor instructions/candidate、String constructor candidate、values factory、initializer prefix）及五个原事实记录已归 Java。root 从原调用点供给选中的 Code/member/fields，保留物理读取、MethodBodies/AnalysisSteps/IrItems 费用与 poll、arbitrary 分支、direct constants、跨定义解析、use census、ClassSource body 和提交。GroupShape 仍由 root 组装，待下一完整 shape/展示资格组处理；不复制 DTO，不新增 Session、cache 或读取。

facade 减少 490 行，Java 增加 496 行；五个函数体、五类记录字段只有路径/可见性/导入调整。物理读取函数、resolve relations 起的全部后续代码及测试体、Java 原有区域逐字节保持。构造参数逐槽转发、String putfield 物理索引、prefix 名称/ordinal/顺序、factory 与 suffix 的原拒绝域均保留；不新增固定样例或加固旧规则。恢复合同 41、schema 12、解析合同 4、inputs v9 保持，实现指纹更新。公开 raw 记录构造不能授予可信 Engine binding。

六项局部门通过：33 native targets 共 1203 passed / 0 failed / 4 既有 ignored；1740 CLI 与 24 独立入口完整 JSON 严格一致，零语义例外，只归一化经核验的实际 engine 元组和 usage.elapsed_millis。168 全文、78 声明、136×3 核心和 408 JSON/outline 保持，AA body12/budget130/names3 继续仅比较原字段。当前 native 原流确认 16 项既有 body-family 机制测试，覆盖参数/字段身份/prefix 正负、原子性和停止；不穷尽所有私有 guard。24 入口是非 enum 相邻回归，不是本组编译 oracle 或独立 holdout。

核心实际 6 编译/12 受控 VM、真实两类 4 重编通过，真实目标未执行。AS 54 源报告/28 历史行为对按十一跳，Dual 12 源报告/4 对按九跳，以完整源码和历史 pins 复用；复用新增 JDK/产品调用为 0。当前 r1 五构建全部通过，2735 来源/归档和 34 产物同次绑定，target 已清理；909047401 bytes 仅是构建磁盘峰值。11 tools、2 adapters、4 辅助脚本在构建/产品前冻结，成功产品命令未重跑；父 BC 的首次失败及 r2 修复仍是父历史，不归入本轮。

下一组整体迁移 prove_enum_constant_body_group 的 shape、sidecar、成员展示与已选子类结论消费，root 继续持有读取/查询与提交。raw enum_owner/MUTF、其余 method/BCI/span 与 A1.6 证据闭包继续 2.9；2.10 仍需实际 read、Body/IR、live/peak、RSS/耗时/取消延迟、各类停止/慢 sink/panic、八项旧失败和 API/临时双轨原门。逻辑 usage 与构建磁盘不替代这些门。2.9/2.10/S1 未整体完成，17/59；S0 原记录保持，S2 未进入。

# 历史接续：initializer-return 与 suffix 完整政策归属（2026-10-25，BC）

[结果](openspec/changes/restructure-demand-driven-recovery/results/s1-enum-suffix-policy-v1/README.md)、[验收](openspec/changes/restructure-demand-driven-recovery/results/s1-enum-suffix-policy-v1/acceptance-r1.json)、[边界](openspec/changes/restructure-demand-driven-recovery/results/s1-enum-suffix-policy-v1/DESIGN-BOUNDARY.md)、[后续](openspec/changes/restructure-demand-driven-recovery/results/s1-enum-suffix-policy-v1/next-boundaries.md)。

has_only_terminal_initializer_return、静态赋值和map后缀的完整Code/AST/元数据源码政策及两个既有结果类型归Java。root EnumStaticAssignmentInput不下移，仅以原位method/field事实借用适配；facade继续terminal-return优先、再static、只有static None才map，emitter/停止/安装/整组提交保持。static强字段匹配、map原弱字段/marker文本门槛分别保留，不合并守卫或新增recovered要求。

两生产文件 enum -772/Java +931行，原测试体及Java既有政策区域逐字节相同，0新测试/资产/来源路径/DTO/Session/cache/read。初始guard、static/map不同表费用、逐项poll、Code/sidecar原身份、BCI/primary与derived origins、clone/receiver清空/元素type重建原样保留。恢复41/schema12/解析4/inputs v9保持，实际指纹更新。旧固定名称/BCI/局部/步数与IndexLoop带否定any仅记录S5/待入口复现，不混修语义，不称泛化收益。

六项局部门通过：33native1203/0/4，1740CLI与独立24入口完整JSON全strict，0语义例外；168全文/78声明/136×3核心、408 JSON/outline、AA12/130/3原字段子集保持。14项既有suffix/行为/evidence/停止机制测试从当前r2原生输出确认，不主张全部私有guard、公开rawhelper任意假输入或资源总门；24非enum入口只作相邻回归。

核心实际6编译/12VM、真实两类4重编保持，真实目标未执行。AS14变体/54源报告/28行为对按十跳、Dual12源报告/4对按八跳全文/历史pins复用，该复用项新增JDK0。修正后五串行构建/34产物与2735源/归档绑定，r2 target清理；909989581bytes只作构建磁盘采样峰值。13工具/adapter与4辅助脚本产品前冻结，三执行组和八子门通过，无成功产品/验收重跑。

首次check因不存在的facts::ACC_NATIVE导入退出101，0二进制/产品；失败源码/归档/plan/预检/原流/receipt保留。只将原root私有0x0100位值随政策迁Java，移除错误导入；四只读预检重跑后以全新r2 target/bin成功构建。当前product-build-r1是固定检查点目录而非尝试次数，不把首次失败追认为通过。源审查r1对应原迁移、r2定点复核修正，旧绝对路径仅描述失败时点。

下一组核body-family构造器/initializer-prefix纯Code闭包及既有raw metadata记录；root保留query使用闭包、跨定义读取、pending与ClassSource/事务。2.9其余raw/方法/BCI/span与2.10实际资源/各类停止/八旧失败/API双轨仍按原门，2.9/2.10/S1未整体验收，17/59，S2未进入。


# 前继：ordinary-enum group 与成员使用 census 政策归属（2026-10-25，BB）

[结果](openspec/changes/restructure-demand-driven-recovery/results/s1-enum-group-policy-v1/README.md)、[验收](openspec/changes/restructure-demand-driven-recovery/results/s1-enum-group-policy-v1/acceptance-r1.json)、[边界](openspec/changes/restructure-demand-driven-recovery/results/s1-enum-group-policy-v1/DESIGN-BOUNDARY.md)、[后续](openspec/changes/restructure-demand-driven-recovery/results/s1-enum-group-policy-v1/next-boundaries.md)。

ordinary-enum group 的完整源码资格、常量/backing/构造器/API、全方法 Code 完整性及 direct/bootstrap 使用 census、prefix/clinit 使用/sidecar 准入和结果组装归 Java source_proof。ProvedEnumConstant/ProvedOrdinaryEnumConstantGroup 两个既有输出类型移动为唯一实现，root 重导出，未复制镜像 DTO。root 只保留 ClassSource/Body-family、原位物理表 owner/counts/completeness/初始收费/raw 配对，以及 Code、声明、Signature、PhysicalMethodId、构造链事实回调和原 proofenum/事务包装。

标准 Result<Result<Option<既有OrdinaryGroup>,String>> 保持外层 Stop、内层 Refused 与 None NotApplicable 的原含义；非枚举/不适用资格仍先于物理回调，首次表收费仍在完整性检查之后、逐项配对之前。五个预选 Code、随后 census、edge→terminal、String/任意参数字面量费用、clinit sidecar 与原结果字段顺序均保持；body-family fallback、int-field certification 和 nested 消费者继续连通。两生产文件 enum -707/Java +745 行，原测试体逐字节相同，0新测试/资产/来源路径/DTO/Session/cache/read。recovery41/schema12/resolution4/inputs v9 保持，实际实现指纹更新。

六项局部门直接通过：33native1203/0/4、1740CLI与独立24入口完整JSON全strict，0语义例外；168全文/78声明/136×3核心、408 JSON/outline及AA12/130/3原字段子集保持。20项既有group/完整性/census/预算取消机制测试从当前原生输出确认；不主张全部私有guard、公开raw helper任意假输入或资源总门，24非enum入口只作相邻回归。

核心实际6编译/12VM、真实两类4重编保持，真实目标未执行。AS14变体/54源报告/28行为对按九跳、Dual12源报告/4对按七跳全文及历史pins复用，复用项新增JDK0。五串行构建/34产物与2735源/归档绑定，target清理；908555195bytes仅为构建磁盘采样峰值。11工具/2adapter与4辅助脚本产品前冻结，三个执行组和八子门通过，无成功产品/验收重跑。

下一完整机制是 initializer-return/static-assignment/map suffix，之后再核 body-family 中仍混源码政策的阶段，root 事实/读/停止/提交边界不变。既有 method_code 在 census 中重复扫描候选仅为静态成本观察，留2.10按原测量协议验证，不在本组改费用或引入索引缓存。raw enum_owner 仍先完整入口复现再单列语义修复。2.9其余raw/方法/BCI/span及2.10实际资源/各类停止/八旧失败/API双轨未关门，2.9/2.10/S1未整体验收，OpenSpec17/59，S2未进入。


# 前继：enum 构造链与 initializer-prefix 政策归属（2026-10-25，BA）

[结果](openspec/changes/restructure-demand-driven-recovery/results/s1-enum-constructor-chain-policy-v1/README.md)、[验收](openspec/changes/restructure-demand-driven-recovery/results/s1-enum-constructor-chain-policy-v1/acceptance-r1.json)、[边界](openspec/changes/restructure-demand-driven-recovery/results/s1-enum-constructor-chain-policy-v1/DESIGN-BOUNDARY.md)、[后续](openspec/changes/restructure-demand-driven-recovery/results/s1-enum-constructor-chain-policy-v1/next-boundaries.md)。

source-tail 的既有枚举与完整 Signature 分类、implicit constructor、delegation edge、terminal 的源码资格和完整 Code/AST、initializer-prefix/factory 及共享 raw helper 归 Java source_proof；七个既有类型/输入/输出移动为唯一实现。root 保留两既有 input structs、初始 terminal 表计费/poll、source/header/index/raw 配对、Code.member 绑定与 method_code 原位借用回调。字段 callback 在唯一 Fieldref 匹配后才供物理配对/声明/markers/annotation refusals 事实，源码组合判据在 Java；原读取、停止与整组提交不穿层。

三生产文件净变化 enum -1047、class-source -53、Java +1213 行。两个 root 测试模块逐字节相同，Java 既有区域仅增 ACC_PRIVATE 导入；0 新测试函数/资产/来源路径/DTO/Session/cache/read。raw headers 的唯一性与三个 method_code 回调仍按原阶段和次序；terminal 的 Code.member 失配仍先于连续指令 poll；原 Signature、quality/content/Complete、flags/handler/效果与 BCI 拒绝保持。initializer-prefix 原本无 Budget，仍依赖原上层费用；各 charge/poll/错误映射不改。recovery41/schema12/resolution4/inputs v9 保持，实际实现指纹更新。

六项局部门直接通过：33 native1203/0/4、1740 CLI 与独立24入口完整 JSON 全 strict，0语义例外；168全文/78声明/136×3核心、408 JSON/outline与AA12/130/3原字段子集保持。18项既有构造链/prefix/Signature/停止机制测试从当前原生输出确认，不主张全部私有guard、逐pair取消注入或公开raw helper任意伪输入覆盖；24非enum入口只作相邻回归。

核心实际6编译/12VM、真实两类4重编保持，真实目标未执行。AS14变体/54源报告/28行为对按八跳、Dual12源报告/4对按六跳全文与历史pins复用，复用项新增JDK0。五串行构建/34产物与2735源及归档绑定，target清理；908568694bytes仅为构建磁盘采样峰值。11工具/2adapter和4辅助脚本产品前冻结，三个执行组与八子门通过，无成功命令重跑。

本组开始时发现后续审阅任务误写AZ两份已封存审稿；已保存改写稿，从首次写入工具payload只提取AST literal（未执行日志命令），按原SHA精确恢复两文件。AZ10497载荷/目录清单、原索引和验收复核通过；产品源码/测试/原始结果未改，未重跑旧产品。另一次迁移脚本函数定位错误发生在写源前，修正后实施；不混入构建/产品成功记录。审阅只允许当前检查点新路径，封存后禁止覆盖。

后续按完整ordinary group/census及suffix共享机制迁移剩余源码政策，保留物理/请求/停止/事务；raw enum_owner仍先完整入口复现再单列语义修复。2.9其余raw/方法/BCI/span与2.10实际资源/各类停止/八旧失败/API双轨未关门，2.9/2.10/S1不整体勾完，OpenSpec17/59，S2未进入。


# 前继：enum 构造器 Code/field 政策归属（2026-10-25，AZ）

[结果](openspec/changes/restructure-demand-driven-recovery/results/s1-enum-constructor-policy-v1/README.md)、[验收](openspec/changes/restructure-demand-driven-recovery/results/s1-enum-constructor-policy-v1/acceptance-r1.json)、[边界](openspec/changes/restructure-demand-driven-recovery/results/s1-enum-constructor-policy-v1/DESIGN-BOUNDARY.md)、[后续](openspec/changes/restructure-demand-driven-recovery/results/s1-enum-constructor-policy-v1/next-boundaries.md)。

四种既有固定 int、scalar String、String-varargs 与任意用户参数尾部的构造器 Code/field 证明、两个尾部类型及 Enum super descriptor 与带预算的连续 BCI helper 归 Java source_proof。root 重导出唯一实现；两个实际供应者在原唯一字段/重复字段判断之后，以借用闭包提供同次字段声明存在性，不预扫描、不增加读取/DTO，缺项仍拒绝。root 保留 ClassSource 记录、物理配对、请求/停止、constructor chain 与整组提交。

三生产文件净变化 root enum -403、facade +4、Java +411 行，0 新测试函数/资产/来源路径；原 root 测试体及 Java 既有政策区域逐字节相同。三个固定证明与预算连续 helper 仅改可见性，任意参数证明只替换事实供给；固定 int 既有弱检查没有顺手加固。三个带 Budget 的证明保留逐相邻指令 poll，原 IrItems/参数 poll、短路和错误优先级不变。recovery41/schema12/resolution4/inputs v9 保持，实际 Java/facade 指纹更新。

六项局部门直接通过：33 native 1203/0/4、1740 CLI 与独立 24 入口完整 JSON 全 strict，0 语义例外；168 全文/78 声明/136×3 核心、408 完整 JSON/outline 与 AA12 body/130 budget/3 name 原字段子集保持。12 项既有构造器/整组机制测试从当前 native 原流确认；missing declaration 与每个相邻 pair 的取消点未逐 guard 独立动态覆盖。24 非 enum Scope 入口只作相邻回归。

核心实际 6 编译/12 VM、真实两类 4 重编保持，真实目标未执行。AS14 源变体/54 报告/28 对按七跳、Dual12 报告/4 对按五跳全文/pins 复用，复用项新增 JDK0。五串行构建/34 产物与 2735 源/归档绑定，target 已清理；907228173 bytes 仅是构建磁盘采样峰值。11 工具/2 adapter 与 4 辅助脚本产品前冻结，三个执行组及八子门无成功命令重跑。

下一项先核 implicit constructor、delegation edge 与 terminal body 的完整事实/源码资格边界，沿现有 reports/candidates/reader facts 拆除 ClassSource 耦合，不搬 root 请求/物理事务，不新造 DTO。raw enum_owner UTF8 仍需完整入口复现再单列语义修复。2.9 其余 raw/方法/BCI/span、2.10 实际资源/各类停止/八旧失败/API双轨未关门；2.9/2.10/S1 不勾完成，OpenSpec17/59，S2 未进入。


# 前继：enum 参数与实参纯政策归属（2026-10-25，AY）

[结果](openspec/changes/restructure-demand-driven-recovery/results/s1-enum-argument-policy-v1/README.md)、[验收](openspec/changes/restructure-demand-driven-recovery/results/s1-enum-argument-policy-v1/acceptance-r1.json)、[边界](openspec/changes/restructure-demand-driven-recovery/results/s1-enum-argument-policy-v1/DESIGN-BOUNDARY.md)、[后续](openspec/changes/restructure-demand-driven-recovery/results/s1-enum-argument-policy-v1/next-boundaries.md)。

九个既有参数/实参/字面量/条件类型及impl、constructor descriptor分类、用户与固定int/String Code实参证明及源码拼写闭包归Java source_proof；共享raw方法唯一匹配、ASCII String解码、constant/连续BCI helper一并迁移。root消费唯一实现，保留initializer prefix、ClassSource字段声明/构造器耦合、物理配对、Pending/census、读取/停止与组commit。plain-int与user-int不同原判据、max3参数、String ASCII限制、字段符号和单次条件求值不改；不跨类求值、不新增DTO/Session/cache/read。

两生产文件净减808/增820行，0测试函数/资产/来源路径新增，2735同路径。recovery41/schema12/resolution4/inputs v9保持，实际Java/facade指纹更新。33native1203/0/4、1740CLI与独立24完整类JSON全strict，0语义例外；12既有参数/窄化/表达式/拒绝/预算机制测试实际通过。168全文/78声明/136×3与408完整JSON/outline、AA12/130/3原字段子集保持。24非enum入口是相邻回归，不充实参helper覆盖。

核心实际6编译12VM、真实成功两类4重编保持；AS14源变体/54报告/28对按六跳、Dual12报告/4对按四跳全文/pins复用，该复用项新增JDK0，真实目标未执行。修正后五成功构建/34二进制与源归档绑定，target清理，907404314bytes只作构建磁盘护栏。

构建前收敛未用重导出/多余public可见性；首check因漏ACC_STATIC导入退出101、0产物/产品，旧收据和对应源/归档保留。修正复用Java现有0x0008常量，重新冻结并用r2 target/bin完成构建；没有成功产品/验收命令重跑，不称首次全绿。13工具和4辅助脚本在产品前分别pin，24审阅重算完整producer并绑定native实际binary/receipt。

下一项按共享constructor Code/field机制迁固定与任意用户尾部的纯证明及已有输出类型，通过既有facts/窄事实供给拆除ClassSource耦合；root不永久持有纯源码政策，也不把request/identity/stop/commit搬入Java。raw enum_owner仍单列完整入口复现。2.9其余raw/方法/BCI/span、2.10实际资源/停止/八旧失败/API双轨未关门，2.9/2.10/S1不勾完成，17/59，S2未进入。


# 前继：enum Code candidate 与纯 API 证明的所有权（2026-10-25，AX）

[结果](openspec/changes/restructure-demand-driven-recovery/results/s1-enum-code-candidate-policy-v1/README.md)、[验收](openspec/changes/restructure-demand-driven-recovery/results/s1-enum-code-candidate-policy-v1/acceptance-r1.json)、[边界](openspec/changes/restructure-demand-driven-recovery/results/s1-enum-code-candidate-policy-v1/DESIGN-BOUNDARY.md)、[后续](openspec/changes/restructure-demand-driven-recovery/results/s1-enum-code-candidate-policy-v1/next-boundaries.md)。

四个既有 EnumCodeReference/Instruction/Use/MethodCodeCandidate 定义、cheap eligibility、同次 MethodIr 候选捕获及完整 CP/bootstrap/operand 闭包归入 Java source_proof；values/valueOf 的纯 Code 证明及七个共享 descriptor/operand helper 一并迁移。root 通过重导出消费唯一实现，保留 ClassSource 物理索引/member/complete/handler 守卫、Pending、census、读/停止/递归请求和最终commit。公开 metadata/形状 bool 不授予可信 artifact。candidate名字、克隆内容、direct/bootstrap uses顺序、单层handle处理及原收费/poll逐项保持，不改恢复接受域。

两生产文件净减436/增445行，0测试函数/资产/来源路径新增，无新DTO/Session/cache/read；recovery41/schema12/resolution4/inputs v9保持，实际Java/facade指纹更新。33native1203/0/4、1740CLI完整JSON及独立24多domain完整入口JSON全strict，0语义例外；168全文/78声明/136×3与408完整JSON/outline、AA12/130/3原字段子集通过。五项既有捕获/预算取消/changed-values机制测试实际通过，不充所有私有guard覆盖。

核心实际6编译12VM、真实两类4重编保持，真实目标未执行。AS14源变体/54报告/28对按五跳、Dual12报告/4对按三跳全文/pins复用，该复用项新增JDK0。2735源/归档/五成功构建/34产物绑定，target已清理，908520366bytes仅为构建磁盘峰值。11工具/2adapter在产品前冻结，runner从plan取33目标，三执行组和八产品子门均直接通过；不重复成功命令。

下一项沿已有用户参数/实参类型、descriptor分类与纯Code证明的依赖闭包继续所有权迁移，不复制第二套模型；依赖ClassSourceField.declaration的构造器尾部和root-private Pending编排先保留。raw enum_owner仍先完整入口复现再修语义。2.9其余raw/方法/BCI/span、2.10实际资源/停止/八旧失败/API双轨未整体验收；2.9/2.10/S1不勾完成，17/59，S2未进入。


# 前继：完整类入口的 loader 图与 widening scope（2026-10-25，AW）

[结果](openspec/changes/restructure-demand-driven-recovery/results/s1-widening-entry-scope-v1/README.md)、[验收](openspec/changes/restructure-demand-driven-recovery/results/s1-widening-entry-scope-v1/acceptance-r1.json)、[完整入口](openspec/changes/restructure-demand-driven-recovery/results/s1-widening-entry-scope-v1/entry-report-review-r1/summary.json)、[后续](openspec/changes/restructure-demand-driven-recovery/results/s1-widening-entry-scope-v1/next-boundaries.md)。

完整 class-source 请求原先只能构造单个无 parent 的 loader，不能表达 AV 计划的跨 loader 验收。现有 EnvironmentPolicy 增加 ExplicitDomains，借用现有 LoadDomain/HeaderProvider 声明完整图；EnvironmentRequest 的 loader 精确选择唯一 caller，缺失或重复直接 invalid input，不合成默认 domain。scope/profile/有序 domains/providers 原样进入 ResolutionEnvironment，后续继续原 validator、环境身份、defining loader 后继、缓存、读取、预算和停止；不新增 DTO/Session/cache/解析入口，也不改 Java widening 接受规则。该能力通过公开库入口验证，CLI 保留现有 flat 政策配置，不声称新图配置已由 CLI 交付。

同一 Scope 的 alpha(S) 与 beta(U) 经 reader 解码均只有 BCI1 的 invokestatic，分别需要 T/V。四组完整入口覆盖 child 链成功、child 链断裂、parent-defined S 不受 child Mid 遮蔽、parent T 与 child 同字节 T 不能合并；先核实际 loader/snapshot 选中身份，再核最终 cast/拒绝、独立 sibling、physical method 和 evidence 正文。RuntimeProfile11、SourceTarget7/8×三evidence，共24报告：alpha正12/拒12，beta24正。独立审查168个method-origin引用未串用。输入由冻结 Rust 测试内存构造，不新增binary fixture，未编译/执行本组输入；24份不计独立holdout或编译/行为oracle。

六项局部门通过：33 native1203/0/4（原30目标加2新测试及3既有widening目标；4忽略均为既有测试），1740完整JSON全strict、0语义例外；168全文/78声明/136×3核心、408完整JSON/outline和AA12/130/3原字段子集保持。完整报告仅规范已核验实际engine及usage直属elapsed_millis。核心实际6编译12VM、真实成功两类4重编保持；扩大native还有受控fixture的JDK调用，不由native汇总推算总调用数。AS14源变体/54报告/28对按四跳、Dual12报告/4对按两跳完整text/pins复用，该复用项新增JDK0；真实目标未执行。

一生产路径facade净增34行，一现有测试路径新增2测试/320行；2735来源同路径，recovery41/schema12/resolution4/inputs v9保持，Java指纹保持、facade指纹更新。五构建/34二进制绑定，target清理，909280925bytes只是构建磁盘峰值。工具审查在产品前补R2清单并让AA直达入口验证完整producer；之后runner残留30-target预检在零子命令时退出。原11tools及失败原流保持，外置continuation只将计数/报错绑定冻结plan33，其余检查与八子命令原样；成功native/1740及产品命令未重跑。原remaining-checks保留失败，权威产品结果为product-gates及continuation，不能改写为首次全绿。

本组闭合了既定widening组合的公开入口盲区，没有把薄wrapper再迁一层。下一项继续设计 enum group 现有 raw Code candidate/源码 policy 的所有权边界，root 保留 pending/身份/读/停止/commit；raw enum owner 仍先完整入口复现。2.9其余raw/方法/BCI/span及2.10实际read/Body/IR/live/peak/RSS、各类停止/背压/panic、八旧失败当前复核和API/双轨收尾未整体验收。2.9/2.10/S1不勾完成，OpenSpec17/59，S2未进入。


# 前继：S1 bridge 当前类源码准入职责迁移（2026-10-25，AT）

[结果](openspec/changes/restructure-demand-driven-recovery/results/s1-bridge-source-policy-v1/README.md) · [收据](openspec/changes/restructure-demand-driven-recovery/results/s1-bridge-source-policy-v1/acceptance-r1.json) · [范围](openspec/changes/restructure-demand-driven-recovery/results/s1-bridge-source-policy-v1/coverage-scope.md) · [后续](openspec/changes/restructure-demand-driven-recovery/results/s1-bridge-source-policy-v1/next-boundaries.md)

Java source_proof接管bridge当前类27个准入守卫、同次Report/analysis完整性、raw name/descriptor唯一配对、flags/MethodParameters、参数cast form及covariant return判据；两个既有Signature是否参数化函数一并迁入。门面供给借用事实iterator和selected-header callback，保留resolver/平台absence/访问检查、reloaded-current、Stop清空bridge proofs、原成员和staging/commit。当前类证明是完整阶段迁移；最终继承header再生gate仍由门面编排，后续单列收口。

六项局部门通过：30 native1183/0/2；AS权威1452加新bridge84，共1536完整JSON严格保持，0语义例外，仅规范校验后的actual engine及usage直属elapsed_millis。168全文、78声明、136×3核心、408同evidence完整JSON/跨evidence outline及AA12/130/3原字段子集保持。核心实际6受控编译/12VM、真实两类4重编；既有bridge完整源码/erased dispatch/跨层级协变与停止测试本轮实际通过。AS14完整程序/54源报告/28行为对经一跳text、输入和pins核验复用，该项新增JDK0，不把新84份计入旧Oracle。

新84来自14既有context×SourceTarget7/8×三evidence，共7完整JAR，0新fixture编译/目标执行/变异；54报告有投影、30拒绝。跨配置102条proof记录为72 admitted/projected、30 refused，不能当独立holdout或27内部守卫探针。两个生产文件、facade减少359行/Java增加390行，新增一个阶段证明入口、两既有函数迁移，无新DTO/Session/cache/read/Body/测试函数；恢复40/schema12/解析4/inputs v9保持，实际Java/facade指纹更新。五构建/31产物/2735来源/11原工具绑定，target清理，908139246bytes仅为构建磁盘峰值。

下一项先收口继承header再生gate，并复现平台Comparable契约未明确匹配raw方法名的静态盲区；只有完整入口反例支持后才修接受域和版本。随后核raw enum owner/descriptor、widening同次method/BCI/loader。2.10真实资源/各类停止/背压/panic、八历史失败当前复核、API/双轨收尾仍未验收。2.9/2.10/S1未整体通过，OpenSpec17/59，S2未进入。

## 前继：S1 enum 方法忠实名称恢复闭包（2026-10-25，AS）

[结果](openspec/changes/restructure-demand-driven-recovery/results/s1-enum-method-name-v1/README.md) · [收据](openspec/changes/restructure-demand-driven-recovery/results/s1-enum-method-name-v1/acceptance-r1.json) · [范围](openspec/changes/restructure-demand-driven-recovery/results/s1-enum-method-name-v1/coverage-scope.md) · [后续](openspec/changes/restructure-demand-driven-recovery/results/s1-enum-method-name-v1/next-boundaries.md)

Java child 方法资格与既有方法声明共同消费 names::source_method_name：严格 MUTF-8，保留 Java 7/8 单独 _。raw name/descriptor、base 唯一配对、flags/visibility、Code/core 完整性、预算/停止及组提交保持。仅 source_proof.rs 与 artifact.rs 改变，facade 与 AR 逐字节一致；0 新 helper/测试函数/DTO/Session/cache/read/永久 source 路径。接受域改变使用恢复39→40，schema12/解析4/inputs v9保持。

六项局部门通过：30 native1183/0/2；1452报告中1440完整JSON保持，12份 supplementary/单独_由无enum_projection变为闭合投影。12份完整文本及完整projection_inputs由已有ASCII有效机制与独立child物理事实参数化核对，其余physical/method/BCI/artifact/status保持。唯一逻辑费用变化是既有constants_text计费：supplementary6份3190→4907、_6份3174→4859，只出现在usage及execution.usage的output_bytes；两者为累计快照，不求和充实际I/O或RSS。168全文、78声明、136×3核心、408同evidence完整JSON/跨evidence outline与AA原字段子集12/130/3通过。

七个有效现代contexts×SourceTarget7/8生成14完整源码程序（字符串族包含当前Jarde IOps），三evidence先核全文一致，空classpath原样编译；Driver另编译1次。复用冻结原class，在JVM8/23执行原件14次/生成28次，28对方法名/参数返回类型、enum name/ordinal、常量体调用、tag/getOp输出一致，校验root/children/IOps的Loader与CodeSource。核心另实际6编译/12VM、真实两类4重编；真实目标和变异不执行。新child108/名称变异24/接口12报告复用完整AR JAR，原fixture成功编译不重跑；4名称变异18exit0/6exit4，前端失败不充Java helper覆盖。同源变体不充独立holdout，语言目标7不充JVM7。

普通widening主体已在Java；后续先核raw/String handoff、bridge共同源码证明与同次方法/BCI/span，不重复迁移薄适配包装。Corretto8缺$values的旧lowering单列S5，整类map/重定位同属S5。2.10实际read/Body/IR/live/peak/RSS及取消/预算/elapsed/sink/backpressure/panic、八旧失败和API/双轨收尾仍待原门验收。2.9/2.10/S1未整体通过，OpenSpec17/59，S2未进入。

## 前继：S1 enum child 源码资格职责迁移（2026-10-25，AR）

[结果](openspec/changes/restructure-demand-driven-recovery/results/s1-enum-child-source-policy-v1/README.md)、[收据](openspec/changes/restructure-demand-driven-recovery/results/s1-enum-child-source-policy-v1/acceptance-r1.json)、[范围](openspec/changes/restructure-demand-driven-recovery/results/s1-enum-child-source-policy-v1/coverage-scope.md)、[后续](openspec/changes/restructure-demand-driven-recovery/results/s1-enum-child-source-policy-v1/next-boundaries.md)。

Java source_proof接管四组分阶段判据：child声明/无字段；旧UTF8方法名、identifier与flags；raw唯一base配对后的override可见性；同次RecoveryReport/analysis execution/声明与marker事实的可展示资格。facade保留实际class_source_with_evidence、物理identity、完整Code coverage、constructor/clinit、raw name/descriptor唯一配对、method poll、停止/原错误与group提交。root wrapper只解包Outcome并传已有类型/两bool，无ClassSourceMethod穿层或新DTO。两生产路径，Java既有测试和root cfg(test) helper assert_positive扩展，0新测试函数；4新纯helper，无Session/cache/read/永久source资产。恢复39/schema12/解析4/inputs v9保持，实际实现指纹更新。

六局部门通过：30 native1183/0/2；AQ最终semantic-r2的1170 current报告加AR138，共1308完整JSON保持，仅规范校验后的engine与usage直属elapsed_millis，raw/费用/partial/BCI/来源和错误优先级不变。168全文/78声明、136×3及408同evidence完整JSON/跨evidence outline保持；核心实际6编译12VM、真实两类4重编通过，真实目标未执行。AA body12/budget130/name3为原字段子集；AQ14完整源码编译变体/42源报告/28运行对经AQ→AR一跳text与pins核验复用，不计本轮新增JDK或新138份的行为oracle。

9同源contexts在Corretto8 source8/target8与JDK23 --release8各编译一次，共18成功编译、108报告（runtime8、SourceTarget7/8×三evidence）。现代54中ASCII两族/debug与BMP共30发布enum_projection，supplementary/_/fields/extra-method24无投影；旧compiler54缺$values形状，不能当源码helper覆盖。5 query-only变异30中18exit0、12exit4（instruction decode/method ambiguous），不编译/执行、不由终态断言内部拒绝切点。同源变体不充独立holdout，语言目标7不充JVM7覆盖。mutator初版parser失败、后续修正private/真duplicate及AA漏适配脚本的零产品调用失败均保留，未修改原11tools或重跑成功产品门。

五串行构建/31产物/2735来源/11原tools绑定，target清理，908144335bytes仅是磁盘护栏；AQ8887封存payload保持。下一组以严格MUTF与source_method_name统一child方法资格和既有忠实声明（不是保守source_identifier），保留raw配对/flags/Code/core证明与事务；接受域变更单列39之后版本及完整类/隔离行为/元数据/来源验收。现代supp/_12无投影仅为入口事实，不推断唯一根因；旧$values lowering差异独立归S5类级机制，不能混入名称修复。随后widening及raw/方法/BCI/span按共享机制推进，整类map/重定位属S5。2.10实际read/Body/IR/live/peak/RSS、各类停止/背压/panic与八旧失败仍未验收；2.9/2.10/S1未完成，17/59，S2关闭。

## 前继：S1 整型常量 raw 名称恢复闭包（2026-10-25，AQ）

[结果](openspec/changes/restructure-demand-driven-recovery/results/s1-integer-source-name-v1/README.md)、[收据](openspec/changes/restructure-demand-driven-recovery/results/s1-integer-source-name-v1/acceptance-r1.json)、[范围](openspec/changes/restructure-demand-driven-recovery/results/s1-integer-source-name-v1/coverage-scope.md)、[后续](openspec/changes/restructure-demand-driven-recovery/results/s1-integer-source-name-v1/next-boundaries.md)。

Java准入复用既有严格MUTF-8 source_identifier，mask从既有FieldItem传raw key，全部物理字段先去重名、再计候选重复值；源码String不再回编码充物理身份。root保留Integer/声明/static final/I/单ConstantValue、完整field表、读取/预算/poll、停止清空与提交。三个生产文件含恢复版本，扩展一个既有Java单测，无新增helper/DTO/Session/cache/read或永久source路径。恢复38→39，schema12/解析4/inputs v9保持，实际实现指纹更新。

六项局部门通过：30 native1183/0/2；AP1080加新合法42/变异48共1170，1152完整JSON严格保持，18份精确名称/UTF-8 byte spans及原Field/Method/BCI增量通过（AP6由0→2，新12由0→4）。仅规范已验证engine与usage对象直属elapsed_millis；新增AST保留及后续累计预算快照的费用变化逐位置保存，方法级只允许steps/ir_items非负，不能对累计快照求和充实际资源。原比较18失败、R1路径标签TypeError和R2修正都保留，11原tools未改、CLI未重跑。6个审查负例均拒绝。

168全文/78声明、136×3及408同evidence完整JSON/跨evidence outline保持；核心实际6编译12VM、真实两类4重编通过，真实目标未执行。AA body12/budget130/name3继续为原字段子集。新7合法contexts（名称×debug和同值控制）×SourceTarget7/8生成14完整源码编译变体，三evidence先核全文相同；JDK8/23下14原件VM/28生成VM形成28对反射/行为一致，额外Driver编译1次，隔离加载并校验实际CodeSource。新7原fixture各成功编译一次，变异只查询；12份编码前端拒绝不计Java helper覆盖，raw重名/同值先后次序及既有g()独立return边界保持。同源变体不充独立holdout，语言目标7不充JVM7覆盖，也不将名称可读性改善计作新增可编译类。

五串行构建/31产物/2735来源/11原tools绑定，target清理，907839575bytes仅为构建磁盘护栏；AP8468封存payload保持。下一组先冻结enum constant anonymous-body完整JAR与有效/拒绝/停止域，再迁源码/flags/可展示资格，root保留物理配对/coverage/read/recover/费用/提交，避免crate反向依赖和新DTO。该处旧UTF8仅为静态风险，未冒称已复现；若改接受域另声明相对39的恢复版本。其后widening和raw/方法/BCI/span按共享机制推进，整类map/重定位属S5。2.10实际read/Body/IR/live/peak/RSS、各类停止/背压/panic与八旧失败仍未验收；2.9/2.10/S1未完成，17/59，S2关闭。

## 前继：S1 共享元数据源码准入职责迁移（2026-10-25，AP）

[结果](openspec/changes/restructure-demand-driven-recovery/results/s1-metadata-source-policy-v1/README.md)、[收据](openspec/changes/restructure-demand-driven-recovery/results/s1-metadata-source-policy-v1/acceptance-r1.json)、[范围](openspec/changes/restructure-demand-driven-recovery/results/s1-metadata-source-policy-v1/coverage-scope.md)、[下一组验收](openspec/changes/restructure-demand-driven-recovery/results/s1-metadata-source-policy-v1/next-boundaries.md)。

static target的读前static/root Signature、读后child字段/Signature资格和完整表唯一raw构造器selector，以及integer constant的准入和名称先于值去重，成组迁入Java source_proof。facade保留物理读取/关系/identity、字段/方法完整性、CP/IR/recover、逐项计费/poll、停止清空及提交；selector返回原Header引用，mask保留原FieldItem和顺序，不以源码拼写授予物理身份。

相对AO仅两生产文件，Java文件同时新增一个cfg(test)测试函数；新增5纯helper，无DTO/Session/cache/read或永久source inventory资产。2735来源路径、恢复38/schema12/解析4/inputs v9保持，实际Java/facade指纹更新。局部分配与生命周期可变，不从纯迁移推断时间/RSS不变。

六项局部门通过：30 native1183/0/2；继承AO960加新合法90/变异30，共1080份完整JSON保持，仅规范actual engine与usage.elapsed_millis。AA body12/budget130/name3为字段子集，65compact与32个constructor+两个generic body均complete的partial点保持，header0:31/2:1；145份中170条engine identity全匹配。168全文/78声明、136×3及408同evidence完整JSON/跨evidence outline保持；实际6受控编译12VM、真实两类4重编通过，真实目标未执行。AM34完整源码编译变体/68运行对经AM→AN→AO→AP三跳text与pins核验后复用，不计为本轮新增JDK或新120份的行为oracle。

新15合法JAR覆盖static六变体与integer九变体，固定Corretto8、release8、SourceTarget7/8×三evidence。合法90份均exit0；五query-only变异30份中duplicate ctor/truncated table合计12份exit4，是classfile前置拒绝，不充内部source helper定点负例；变异不编译或执行。KEY𐐀有完整字段声明和ConstantValue但0常量名投影，已从静态风险升级为完整入口复现的可读性缺口，不等于整类失败。本轮保持旧std UTF-8准入与display bytes比较域；后续必须同时改严格MUTF名称出口及candidate.field.name.raw()去重key，不能只替换decoder。

R1 ZIP排序误判和首次变异父verifier失败原流保留；R2复用首输入成功编译pins，另14输入各首次编译，15次成功fixture编译均只执行一次。父adapter错误lineage/current checkout比较缺失在freeze前修复；初版与diff保留，11原tools固定为native前AST版本，协议统一独立承接。五构建命令/31产物绑定，target清理，907843864bytes仅为构建磁盘护栏；AO7485封存payload与精确清单保持。

下一组按整型常量raw identity→忠实源码名称→switch/return/int[]消费者闭环推进：先保存旧完整入口，再同时修准入/raw key；保留声明/flags/I/ConstantValue/完整表/停止清空与anchors，接受域变更恢复38→39，建立新producer绑定，并以完整生成类的隔离编译、反射字段名/flags/value及行为对照验收。malformed/源码不可写/重名/同值/遮蔽/停止分组，变异仍只查询。随后按原阶段继续enum anonymous-body、widening及raw/方法/BCI/span；整类map/重定位属S5。2.10实际read/Body/IR/live/peak/RSS、各类取消/停止/背压/panic与八旧失败仍按原退出门。2.9/2.10/S1未完成，17/59，S2未进入。

## 前继：S1 static target 三段源码判据迁移（2026-10-25，AO）

[结果](openspec/changes/restructure-demand-driven-recovery/results/s1-static-target-policy-v1/README.md)、[收据](openspec/changes/restructure-demand-driven-recovery/results/s1-static-target-policy-v1/acceptance-r1.json)、[范围](openspec/changes/restructure-demand-driven-recovery/results/s1-static-target-policy-v1/coverage-scope.md)、[下一组](openspec/changes/restructure-demand-driven-recovery/results/s1-static-target-policy-v1/next-boundaries.md)。

static target的空Object构造器三指令/CP形状、原ClassSourceRecovery资格、root/child名称解码与root路径三个源码判据迁入Java source_proof。root保留物理选择/完整性、实际Header/IR/recover、依赖登记/计费/stop、target装配；各判据仍在原阶段，名称未提前到读取前，static child仍仅需严格解码。早期static/root-child Signature/fields与constructor selector的元数据资格仍未迁完，单列下一组，不宣称整个static函数策略完成。

相对AN，两生产文件及一处既有names cfg(test)断言扩展，新增3纯helper，无DTO/Session/cache/read、新测试函数或永久source资产。2735来源路径不变，恢复38/schema12/解析4/inputs v9保持；实际Java/facade指纹随构建变化。分配生命周期可变，不以纯迁移零文本差异推断性能不变。

六项局部门通过：30 native1182/0/2；前继936加新static24共960完整JSON保持，仅规范actual engine与usage.elapsed。AA body12/budget130/name3子集及65compact/32双generic body原子partial保持；168全文/78声明、136×3与408同evidence完整JSON/跨evidence outline保持；核心实际6受控编译12VM、真实两类4重编通过，真实目标不执行。AM的34源码编译变体/68运行对经AM→AN→AO两跳text/pins核验后复用，未当作本轮新增JDK或新24份的行为oracle。

新24为固定Corretto8的四完整root+Box JAR、release8与SourceTarget7/8×三evidence：12份明确static_proved；_和effectful的12份仅prepared_static、未发capture/calls，不充内部helper切点负例。五成功构建/31产物绑定，target清理；907410424bytes仅为磁盘采样护栏。AN7413封存payload与精确清单不变；继承工具标签依actual schema/source/build核对，不把组名当producer。

下一组按共享元数据源码准入机制批量迁static header资格与integer constant候选，保留读前/读后时序、reader完整性/物理绑定及预算提交；integer supplementary限制仍只是静态风险，先独立复现再修语义。随后继续widening、enum anonymous-body及raw/方法/BCI/span；整类map/重定位属S5。2.10实际read/Body/IR/live/peak/RSS、各类停止/背压/panic与八旧失败仍按原退出门，2.9/2.10/S1未验收，17/59，S2未进入。

当前CLI `/private/tmp/jarde-s1-static-target-policy-r1-bin-20261024/cli-jarde-cli`，SHA `c59e9b1ec74d18af9a8c75f71473f9db1a1bb1324502214d54358edf1cb301cb`。持续目标active，无提交/推送/归档，保持S1。

# 前继接续：S1 类族调用名称职责迁移（2026-10-24，AN）

[结果](openspec/changes/restructure-demand-driven-recovery/results/s1-member-call-name-policy-v1/README.md)、[收据](openspec/changes/restructure-demand-driven-recovery/results/s1-member-call-name-policy-v1/acceptance-r1.json)、[覆盖范围](openspec/changes/restructure-demand-driven-recovery/results/s1-member-call-name-policy-v1/coverage-scope.md)、[后续职责](openspec/changes/restructure-demand-driven-recovery/results/s1-member-call-name-policy-v1/next-boundaries.md)。

Java source_proof接管类族调用证明的root严格MUTF解码、child源码资格、root既有保守名称资格及dotted拼写；facade保留物理关系/capture/header事实、完整方法/IR、raw new/constructor/BCI、计费/停止和提交。三条拒绝文本与语义优先级不变，没有借纯迁移放宽`_`等名称。两个生产文件、一处既有cfg(test)断言扩展，新增一个纯helper，无新DTO/Session/cache/read/测试函数或永久资产；2735来源不变，恢复38/schema12/解析4/inputs v9保持，Java与facade实际实现指纹更新。dotted String和参数clone分配时机改变，不宣称内存/时间不变。

六项局部门通过：30 native1182/0/2；AM既有918份加新18份完整JAR入口，共936份完整JSON保持（仅规范actual engine与usage.elapsed，raw/正文/费用/coverage/partial/BCI/来源保留）。AA body12/budget130/name3子集门及65compact/32双body原子partial保持；168全文/78声明、136×3及408同evidence完整JSON/跨evidence outline保持。核心实际6受控编译12VM、真实两类4重编通过，真实目标不执行。AM的34个完整源码编译变体/68对运行输出在text相同和pins有效后复用，本轮不重跑这组JDK，也不将新18份计为行为oracle。

有效R3为Corretto8编译的root+Inner完整JAR，ASCII/supplementary各6份calls proved；单独`_`6份最终prepared_fold而无calls字段，不充narrow调用证明成功或外部拒绝。R1模板guard误判、R2单CLASS缺成员/Outer_错误名称及javac23、工具parent路径修正和初次构建漏参均分别保留，不作为恢复收益。五构建命令/31产物同次绑定，target清理；907388435bytes仅为构建磁盘采样护栏，AM8545冻结payload与精确清单未变。

下一组先冻结static family target末尾名称资格的完整入口及停止前缀，再迁Java；该组child仅要求可严格解码，不能整套复用更强的member-call helper。调用仍位于物理/descriptor/IR/Java recovery证明之后。随后继续widening、enum anonymous-body、integer constant及同次raw/方法/BCI/span；整类map/重定位仍属S5。2.10实际read/Body/IR/live/peak/RSS、各类取消/停止/背压/panic与八旧失败保持原退出门。2.9/2.10/S1未验收，17/59，S2未进入。

当前CLI `/private/tmp/jarde-s1-member-call-name-policy-r1-bin-20261024/cli-jarde-cli`，SHA `2c996e6bb09f6e6ad39375f65a6cf7056b83e4401bc8b804d094e8581a6c3257`。持续目标active，无提交/推送/归档，保持S1。

# 前继接续：S1 当前声明与类型变量名称闭包（2026-10-24，AM）

[结果](openspec/changes/restructure-demand-driven-recovery/results/s1-own-generic-name-boundary-v1/README.md)、[收据](openspec/changes/restructure-demand-driven-recovery/results/s1-own-generic-name-boundary-v1/acceptance-r1.json)、[覆盖边界](openspec/changes/restructure-demand-driven-recovery/results/s1-own-generic-name-boundary-v1/coverage-scope.md)、[后续职责](openspec/changes/restructure-demand-driven-recovery/results/s1-own-generic-name-boundary-v1/next-boundaries.md)。

Java层收拢当前声明与类型变量严格名称出口：类头、字段、方法、构造器、throws/数组及class/method同名遮蔽共同支持Java7/8合法`_`；raw scope、erasure、完整使用、预算/停止和提交守卫保持。新增两个names helper，三生产文件/一测试文件变化，未新增DTO/Session/cache/read/永久资产或测试函数；2735来源不变，恢复37→38，schema12/解析4/inputs v9保持。该名称语义修复与前继纯职责迁移分别验收。

六项局部门通过：30 native1182 passed/0 failed/2既有ignored；当前typed126中60预期修复/66完整JSON保持，owner30中12修复/18保持，继承762完整JSON保持。72是报告数，包含目标/evidence同源变体，不作为独立问题族分母。AA body12/budget130/name3子集合同保持，另核65compact/32双body原子partial（header0:31、2:1）；168全文/78声明、136×3及408同evidence完整JSON/跨evidence outline保持。核心6编译12受控VM/真实两类4重编通过，真实目标和字节变异未执行。

新增18个合法受控程序、34次完整生成源码编译；生成全部输入目标类、空编译classpath、运行仅生成classes与Driver，无原目标或原依赖后备。Java8/23共104条VM命令、68对输出一致，观察GenericDeclaration归属、bounds/数组/throws及字段/构造/普通调用。仅为定向小型程序及两组完整Parent+Child，不外推任意JAR/WAR、任意行为或JVM7执行。family同字节后复用旧oracle，无重复家族JDK。

owner旧采集51条含18条重复argv，初版三份ASCII控制非泛型且原脚本未另存，旧工具无timeout；原流和偏差如实保留。新增typed工具先固定再执行并设置90s超时。offline preflight的evidence字典/选择字符串误判在产品执行前修正，13原工具在native前固定。native summary.scope误留AL标签，实际schema/来源/30二进制为AM，不以标签冒称额外覆盖。

五构建命令与31产物同次绑定，target清理；907408680bytes仅为磁盘采样护栏。AL7380冻结payload与目录清单保持。下一组仍按2.9审查实际源码策略/事实编排边界，widening、enum anonymous-body、integer constant与raw/方法/BCI/span分组推进；整类map/重定位仍属S5。2.10实际read/Body/IR/live/peak/RSS、取消/停止/背压与八旧失败继续按原协议承接。2.9/2.10/S1未验收，OpenSpec17/59，S2未进入。

当前CLI `/private/tmp/jarde-s1-own-generic-name-boundary-r1-bin-20261024/cli-jarde-cli`，SHA `e45673118da18a156bab5837a19e3cc74f708a240da167e0a025d8d92f5d4b4c`。持续目标active，未提交/推送/归档，下一步保持S1。

# 前继接续：S1 父定义与 Override 严格名称边界（2026-10-24，AL）

[结果](openspec/changes/restructure-demand-driven-recovery/results/s1-parent-override-name-boundary-v1/README.md)、[验收收据](openspec/changes/restructure-demand-driven-recovery/results/s1-parent-override-name-boundary-v1/acceptance-r1.json)、[独立名称审查](openspec/changes/restructure-demand-driven-recovery/results/s1-parent-override-name-boundary-v1/name-results-independent-review.md)、[覆盖范围](openspec/changes/restructure-demand-driven-recovery/results/s1-parent-override-name-boundary-v1/coverage-scope.md)、[后续边界](openspec/changes/restructure-demand-driven-recovery/results/s1-parent-override-name-boundary-v1/next-boundaries.md)。

Java层既有方法/类名严格出口接入直接Override与已选父类/接口资格；父定义的单一Object-bound类型参数严格解码，并允许Java7/8的`_`，该变量不在Child发射。全局source_identifier/局部别名策略未改。root环境/loader、实际读取、raw唯一配对、物理来源、flags/arity/erasure、收费/停止和原子提交保持。三生产文件、两测试文件，无新产品helper/DTO/读取/缓存/资产，2735来源路径不变；恢复36→37，schema12/解析4/inputs v9不变。这是名称语义修复，与前继纯职责迁移分别验收。

六项局部验收通过：30 native1182 passed/0 failed/2既有ignored；21上下文三evidence共63份，12个合法名称上下文的36份修复、27份完整JSON保持。三组方法名新增9份精确raw Override proof，九组父类/接口解除header拒绝。新增12次完整生成Child重编、24次driver编译、Java8/23共48条VM命令及24对输出对照通过；生成classpath无原Child，父/接口为声明的原始依赖，观察限于方法反射调用和泛型继承元数据，不声称整包、父定义自身源码或JVM7覆盖。真实目标和字节变异未执行。

654份继承机制及45份旧Override完整JSON保持；body12/budget130/name3专项保持，另核65compact和32个0/2-header原子partial点，不冒称这些专项均为全JSON。168全文/78声明、136×3与408同evidence完整JSON/跨evidence outline保持；核心6编译12受控VM及真实两类4重编通过。family全文不变后复用8组生成隔离oracle，无新增家族JDK。全JSON保持门仅规范actual engine与usage.elapsed，名称修复的36份另核完整差异及raw/方法正文，不靠删除字段放宽。

旧基线15份方法名负例因generic基底提前拒绝，原流保留为setup，普通无Signature父子类另采；有效63份含9复用和54新查，历史新旧CLI调用69次。名称工具R1误把成功Signature说明注释当拒绝，已结束的31次查询全部复用，R2只新增余下32次；原20工具与失败流保留，不称首次全绿。非法父类名exit2是原始父类声明拼写失败，不是CLI选择参数错误或selected-parent helper覆盖；四字节名称负例是stdout failed ClassSourceReport。

五成功构建、2735来源/归档/构建前/当前与31产物同次绑定，精确target清理；907386668bytes仅为构建磁盘采样护栏，不代表实际read/IR/RSS收益。新名称进入更深的原证明路径，费用变化单列；内部定点取消未穷尽。当前类自身`_`泛型header/变量等静态剩余消费者先做完整入口复现，再闭合名称与相关使用；widening、enum anonymous-body、integer constant及同次raw/方法/BCI/span继续按2.9推进，整类map/重定位仍属S5。2.10实际read/Body/IR/live/peak/RSS/取消/停止/背压与八旧失败按原协议承接。2.9/2.10/S1未验收，OpenSpec17/59，S2未进入。

# 前继接续：S1 直接 Override 源码资格检查点（2026-10-24，AK）

[结果](openspec/changes/restructure-demand-driven-recovery/results/s1-direct-override-policy-v1/README.md)、[有效收据](openspec/changes/restructure-demand-driven-recovery/results/s1-direct-override-policy-v1/acceptance-r1.json)、[覆盖边界](openspec/changes/restructure-demand-driven-recovery/results/s1-direct-override-policy-v1/coverage-scope.md)、[后续职责](openspec/changes/restructure-demand-driven-recovery/results/s1-direct-override-policy-v1/next-boundaries.md)。

Java declaration接管直接Override的child class flags/scope、parent class flags/scope、child method普通flags/name/Signature/已有声明与注解/正文/descriptor以及parent method资格/visibility。root保留环境/loader、读取/完整性、raw index/name/descriptor/flags绑定、同origin/同包、唯一配对、原计费/停止、唯一offset与output charge后的原子提交。child资格在父表扫描费前，parent资格在唯一匹配及计费后；descriptor是已证明与报告相等的同一raw member。两生产文件纯迁移，无新DTO/Session/read/cache/永久测试函数/产品资产；2735来源，恢复36/schema12/解析4/inputs v9保持。

六项有效局部门通过：30 native1181 passed/0 failed/2既有ignored；16上下文三evidence48份完整JSON保持（42exit0、3compact exit2、3partial exit4），三成功上下文的9份Override proof保持，11无提示上下文保持。继承六cohort501/annotation39/package36/family36/selected-parent42共654完整JSON、body12/budget130/name3、65compact/32原子partial保持；168全文/78声明、136×3与408同evidence完整JSON/跨evidence outline保持，核心6编译12受控VM/真实两类4重编通过。family同字节后复用8生成隔离oracle，未重复家族JDK；真实目标与变异未执行。仅规范已核验actual engine与usage.elapsed，raw/physical/正文/marker/limits/费用/coverage/来源/partial范围保持。

R1 AA wrapper旧变量名在产品调用前失败；R2复用六组501报告并通过三个AA门，随后首annotation查询后因收据pin类型错误停止。原19工具与两轮失败原流保留，R3复用九成功门和首annotation，只跑剩余五门，14项最终有效通过；门后201份父/Override报告逐条核对engine/输入/实际命令/原流/收据及完整JSON。前三份visibility setup误打public class的真实报告另存excluded，修正protected输入另存；不称首次全绿、参数错误或恢复收益。重复工具协议债务[单独记录](openspec/changes/restructure-demand-driven-recovery/results/s1-direct-override-policy-v1/separate-harness-debt.md)，不混入产品迁移。

五成功构建、2735来源/归档/构建前/当前和31产物绑定，精确target清理；907830747bytes只作磁盘采样护栏。AJ6838冻结payload与精确清单未改。预算compact三行无artifact engine，analysis三行在方法恢复期停止，不冒称helper内部覆盖；native精确output charge原子性保留，内部定点取消仍不宣称穷尽。

下一组在实际入口统一父定义与Override的严格名称出口：U+10400方法名的旧无提示已复现，本组不修接受域；合法supplementary/Java7-8 `_`、源码不可表达、损坏MUTF/孤立surrogate分组，保留raw匹配/环境/loader/erasure/Signature/visibility及原预算事务。widening、enum anonymous-body、integer constant、raw/BCI/span可信闭包和2.10实际read/Body/IR/live/peak/RSS/取消/停止/背压、八旧失败继续承接；整类map/重定位归S5。2.9/2.10/S1未验收，OpenSpec17/59，S2未进入。

当前CLI `/private/tmp/jarde-s1-direct-override-policy-r1-bin-20261024/cli-jarde-cli`，SHA `fc36d691ed7721e64c9ebb6862a2dcc0bc6d40a3b7fb19665da8b0da73c13c76`。持续目标active，无提交/推送/归档，下一步保持S1。

# 当前接续：S1 已选泛型父定义源码资格检查点（2026-10-24，AJ）

[结果](openspec/changes/restructure-demand-driven-recovery/results/s1-selected-parent-policy-v1/README.md)、[有效收据](openspec/changes/restructure-demand-driven-recovery/results/s1-selected-parent-policy-v1/acceptance-r1.json)、[覆盖边界](openspec/changes/restructure-demand-driven-recovery/results/s1-selected-parent-policy-v1/coverage-scope.md)、[后续职责](openspec/changes/restructure-demand-driven-recovery/results/s1-selected-parent-policy-v1/next-boundaries.md)。

Java接管已选直接泛型父类和接口读取后的源码资格、Signature形状/arity及原预算化解析；门面保留环境/loader选择、实际读取、SingleClass读前短路、缺定义平台fallback、execution/停止合并与提交。父类bool和接口三态不合并，接口仍只证明现有kind/arity等窄条件，不声称bounds/实参与bound或interface erasure证明。两生产文件纯迁移，无新DTO/Session/read/cache/永久测试函数/产品资产；2735来源，恢复36/schema12/解析4/inputs v9保持。

六项有效局部门通过：30 native1181 passed/0 failed/2既有ignored；14上下文三evidence42份完整JSON保持（39exit0、3预算停止exit4）；继承六cohort501/annotation39/package36/AI family36共612份完整JSON，body12/budget130/name3、65compact/32原子partial保持；168全文/78声明，136×3与408同evidence完整JSON和跨evidence outline保持，核心6编译12受控VM/真实两类4重编通过。AI八份生成源码相同后复用隔离oracle，未重复家族JDK，真实目标及变异未执行。仅规范已绑定实际构建的engine tuple和usage.elapsed，raw/physical/正文/marker/limits/费用/coverage/来源/partial范围保持。

R1执行分支误用contract35 resolver，在第一份实际CLI后拒绝真实AI36；原16工具/失败流不改，R2离线复核并复用000后续验，无成功Cargo/native/该次CLI重跑。门后独立核验42selected与36family报告的非空engine、原流/进程/输入和完整JSON，闭合selected原工具只查tuple互相一致的缺口。基线三份最初SingleClass参数错误原流因路径覆盖丢失，excluded为原argv实际补采，明确不入42分母。

五构建命令成功，2735来源/归档/构建前/当前与31产物绑定，精确target清理；907426558bytes只作构建磁盘采样守卫。AI6976冻结payload与精确目录清单未改。父类变体在erasure前shape拒绝、接口结构变体在更早原始检查拒绝，不充后续helper覆盖；CLI无token取消入口，native只证明既有回调取消/投影原子性。

下一组按原候选阶段迁直接Override源码资格，root保留same-package前筛选、raw唯一配对、物理来源、读取/费用/停止及偏移事务。旧父定义UTF8 gate修复另经完整入口复现；widening、enum anonymous-body、integer constant、raw/BCI/span可信闭包及2.10实际read/Body/IR/live/peak/RSS/取消/停止/背压与八旧失败继续承接。整类map/重定位属S5。2.9/2.10/S1未验收，OpenSpec17/59，S2未进入。

当前CLI `/private/tmp/jarde-s1-selected-parent-policy-r1-bin-20261024/cli-jarde-cli`，SHA `159528a2801b01c03256d3307761cb493efc4bc3713b1608ca0d743e1f691e63`。持续目标active，无提交/推送/归档；下一步保持S1。

# 当前接续：S1 类族 raw→source 名称边界检查点（2026-10-24，AI）

[结果](openspec/changes/restructure-demand-driven-recovery/results/s1-family-source-name-boundary-v1/README.md)、[有效收据](openspec/changes/restructure-demand-driven-recovery/results/s1-family-source-name-boundary-v1/acceptance-r2.json)、[覆盖边界](openspec/changes/restructure-demand-driven-recovery/results/s1-family-source-name-boundary-v1/coverage-scope.md)、[后续职责](openspec/changes/restructure-demand-driven-recovery/results/s1-family-source-name-boundary-v1/next-boundaries.md)。

generic static、nested enum、nested annotation 和 common fold 的实际名称消费者收拢至既有 Java 严格命名出口；descriptor/CP/Code 身份保持 raw，嵌套路径仅转换已证明的 InnerClasses 边，保留标识符内字面$。两个生产文件、两个已有测试增强，无新DTO/read/cache/test函数/产品资产；2735来源保持，恢复35→36，schema12/解析4/inputs v9不变。既有A<T>/compareTo切片、bridge/闭包、完整执行、BCI/span、预算/停止与原子提交不放宽。

六项有效门通过：30 native1181 passed/0 failed/2既有ignored；十二开发上下文三evidence共36当前报告，四个名称拒绝上下文恢复、四旧有效控制保持、四JDK8形状拒绝保持。当前24 projected、6 generic JDK8 prepared/refused、6 enum JDK8 family refused，均exit0；不是退出码通过率或真实包整体恢复率。八组完整生成源码原样编译/隔离执行成功，原始六组Driver复用、两组fold原始Driver补验，生成classpath无原目标后备。继承六cohort501完整JSON、annotation39/package36、body12/budget130/name3、65compact/32双body原子partial；168全文/78声明、136×3与408同evidence完整JSON保持，核心6编译12受控VM/真实两类4重编通过，真实目标和字节变异不执行。

R1脚本误要求essential公开SourceMap，且两fold原始Driver因文件名编译失败；原流/原21工具与弱版续验保留。最终离线复核使用同context已保存source_map peer，MethodPoint仅来自physical recovery source-map段，物理ID取冻结旧事实；旧JSON缺少的两个Minor ID另用2次旧AH CLI只读查询补齐（contract35，单列不并36恢复分母）。8生成oracle和36当前恢复全部复用，只补失败原始Driver，没有重跑成功native/CLI/JDK。新增验证器对完整报告的伪造digest/BCI/span拒绝，不允许derived自证；不得称首次全绿。

五构建命令同次成功，2735来源/归档/构建前与31产物绑定，精确target清理；907183333bytes仅是构建磁盘采样峰值。AH6575冻结payload及目录清单未变。下一组迁已选泛型父定义读取后的源码资格至既有Java module，再按原阶段迁直接Override方法资格，root保持解析/读取/费用/停止/提交。旧UTF8 gate的语义修复独立复现；enum anonymous-body、不同raw widening、integer constant候选和2.10实际read/IR/live/peak/RSS/取消/停止/背压及八旧失败继续承接。整类map/重定位归S5。2.9/2.10/S1未验收，OpenSpec17/59，S2未进入。

当前CLI `/private/tmp/jarde-s1-family-source-name-boundary-r1-bin-20261024/cli-jarde-cli` SHA `98309a4b4e87a1b187eb22a82662ec6eddf627c4b75eb78caad54e7e6629d363`。持续目标active，无提交/推送/归档，下一步保持S1。

# 当前接续：S1 package-info源码证书检查点（2026-10-24，AH）

[结果](openspec/changes/restructure-demand-driven-recovery/results/s1-package-info-policy-v1/README.md)、[收据](openspec/changes/restructure-demand-driven-recovery/results/s1-package-info-policy-v1/acceptance-r1.json)、[覆盖分布](openspec/changes/restructure-demand-driven-recovery/results/s1-package-info-policy-v1/coverage-scope.md)、[后续边界](openspec/changes/restructure-demand-driven-recovery/results/s1-package-info-policy-v1/next-boundaries.md)。

Java接管package-info窄源码证书与raw JVM包名合法性，按原顺序检查版本/flags/header/空成员、完整执行、属性/注解和单空Deprecated，再拼写package文本。root仅供给既有事实/切片，保持触发、Reader、预算/停止/diagnostics、物理fallback和完整提交。三生产文件纯迁移，无新DTO/read/cache/永久测试函数/产品资产；2735来源，恢复35/schema12/解析4/inputs v9保持。

六项有效门通过：29 native1173 passed/0 failed/2既有ignored；package-info36条同模式完整报告保持（31exit0、4exit4、1exit2）；继承annotation39、classSignature52/classField56/ordinary56/family65/constructor248/NoBody24、body12/130cutoff/name3及65compact/32双body原子partial保持；168全文/78声明、136×3与408完整JSON保持，核心6编译12受控JVM/真实两类4重编通过，真实目标和字节变异未执行。只规范核验过的actual engine tuple与usage.elapsed，physical/raw/marker/limits/费用/coverage/来源保持。

36行是11上下文×三evidence加3预算观察：4行成功投影、27行证书拒绝、4行前置partial和1行compact输出错误；class_headers=1是成功控制，不称停止。截断annotation只证明Reader/完整执行门。另3条误改魔数的早期探针及原流独立保留，不计源码证书分母；不冒充合法包名反例。 合法fixture原始/生成源在JDK8和23下各编译并运行，共4编译4JVM，输出均true；没有compiler/runtime笛卡尔积。当前三个正常evidence及成功预算控制全文相同后复用该oracle，未重复JDK；生成侧只有2编译2JVM，其余2编译2JVM是原始侧。

2735源码/构建前/归档/当前与30产物、五成功命令同次绑定，精确target已清理；907193463bytes仅为构建磁盘采样峰值。AG6332冻结payload和精确目录清单核验未改。12个工具AST与离线preflight绑定；跨evidence核心/正文检查与同evidence408严格完整报告检查分开，两个保存流复核均不再调用CLI。

证书仍只接受Java8固定shape和单空Deprecated；FieldEnum整体编译、数组return维度注解、Holder匿名嵌套partial与八项旧失败继续保留。本轮不声称恢复域扩大、完整来源map或实际资源性能收益。

下一组按[raw消费者清点](openspec/changes/restructure-demand-driven-recovery/results/s1-package-info-policy-v1/remaining-raw-consumers-audit.md)先复现完整入口，再统一实际raw到source消费；generic static/nested family、父定义/override和widening的静态可疑点不冒充已复现收益。span同run/BCI/唯一性属S1边界，整类map与编辑后重定位另归S5。继续2.10实际read/Body/IR/live/peak/RSS/取消/停止/背压和八旧失败的原合同。2.9/2.10/S1未验收，OpenSpec17/59，S2未进入。

当前CLI `/private/tmp/jarde-s1-package-info-policy-r1-bin-20261024/cli-jarde-cli` SHA `228428cd92e5dc161cc9d2a7a2923153fa3b375d9a8848e50fd37610b4e6ada1`。所有本轮会话终态见session-status-r1.json；无提交/推送/归档，目标保持active。

# 当前接续：S1 annotation位置与重复策略检查点（2026-10-24，AG）

[结果](openspec/changes/restructure-demand-driven-recovery/results/s1-annotation-position-policy-v1/README.md)、[收据](openspec/changes/restructure-demand-driven-recovery/results/s1-annotation-position-policy-v1/acceptance-r1.json)、[覆盖范围](openspec/changes/restructure-demand-driven-recovery/results/s1-annotation-position-policy-v1/coverage-scope.md)、[后续边界](openspec/changes/restructure-demand-driven-recovery/results/s1-annotation-position-policy-v1/next-boundaries.md)。

Java接管八个既有annotation报告类型、raw重复域、descriptor参数对齐、owner/target/info/path/冲突与拼写、field实际类型放置和method的clinit/无声明拒绝；root保留Reader shells/facts、计费、错误/停止、报告装配和发布。四生产文件纯迁移，无新DTO/Session/cache/read/永久测试函数/产品资产，2735来源，恢复35/schema12/解析4/inputs v9保持。

六项有效门通过：29 native1173 passed/0 failed/2既有ignored；13注解上下文×3evidence的39份完整报告保持（36complete/exit0、Holder3份旧partial/exit4）；继承classSignature52/classField56/ordinary56/family65/constructor248/NoBody24及body12/130cutoff/name3、65compact error/32双body原子partial点保持；168全文/78声明、当前136×3及408完整JSON保持，核心6编译12受控JVM/真实两类4重编通过，真实目标及字节变异未执行。仅规范核验过的actual engine tuple与usage.elapsed，正文/marker/raw与physical snapshot/limits/费用/coverage/来源保持。

39份CLI矩阵不是完整畸形输入分母：类级重复与非法element name等由既有native补充，任意畸形info未穷尽。Holder记录static声明与旧anonymous nesting partial，空注解列表不能冒充拒绝收益。AE FieldEnum四源整体编译失败、AD数组return维度注解旧拒绝，以及八项既有失败继续保留；本轮无恢复域或实际资源性能收益声明。

2735来源/构建前/归档/当前与30产物、五成功命令同次绑定，target已清理；907163740bytes为构建磁盘采样峰值，仅用于构建守卫。AF6146冻结payload及精确目录清单核验未改。工具在native前补强真实input/process绑定并改为只读复用408保存报告；native后、产品调用前又发现跨evidence误用全JSON比较，以resume-r2复用AF原outline检查核心，并继续由独立408同evidence完整JSON门严格验收。原10工具保持native AST pin，修订前preflight保留，没有因修正重复CLI或冒充产品收益。

下一组独立迁package-info窄源码证书，保持Java8/flags/空members/单空Deprecated及原拒绝次序；随后继续2.9 raw/bridge/span真实来源闭包和2.10 actual read/Body/IR/live/peak/RSS/取消/停止/背压及八旧失败的原合同。2.9/2.10/S1未验收，OpenSpec17/59，S2未进入。

当前CLI `/private/tmp/jarde-s1-annotation-position-policy-r1-bin-20261024/cli-jarde-cli` SHA `5699910ed174d5903013490e17fdad7f86cfbb3934e10e2bf09cb44542e1ef2d`。本轮无提交/推送/归档，持续目标active；当前会话终态以session-status-r1.json为准。

# 当前接续：S1 完整 class Signature 源码策略检查点（2026-10-24，AF）

[结果](openspec/changes/restructure-demand-driven-recovery/results/s1-class-signature-policy-v1/README.md)、[收据](openspec/changes/restructure-demand-driven-recovery/results/s1-class-signature-policy-v1/acceptance-r1.json)、[剩余职责审计](openspec/changes/restructure-demand-driven-recovery/results/s1-class-signature-policy-v1/remaining-source-responsibilities.md)、[后续边界](openspec/changes/restructure-demand-driven-recovery/results/s1-class-signature-policy-v1/next-boundaries.md)。

完整class Signature的shape/source准入、interface-only fallback、direct-parent/interface决定、bounds/Object省略与header组装已归Java。root仍管parse/raw发布、nested/header/erasure物理证明、所选环境父定义budgeted callback、最后输出计费与原子提交。两生产文件纯迁移，无新DTO/Session/cache/read/永久测试/产品资产，2735来源，恢复35/schema12/解析4/v9保持。

六项有效门通过：29 native1173/0/2；新14contexts42complete+10budget完整报告，5投影/5拒绝/4无投影；继承class-field56/ordinary56/family65/constructor248/NoBody24/body12/130cutoff/name3及65compact/32原子partial保持；168全文/78声明、136×3及408完整JSON保持，当前核心6编译12受控JVM/真实两类4重编通过，真实目标未执行。direct-parent proved/missing补8报告时未重跑初始44或重建JAR。16类/字段完整源码相同后复用AE非enum oracle；FieldEnum/数组return维度注解旧缺口及首组exit重建层级保留。

当前CLI `/private/tmp/jarde-s1-class-signature-policy-r1-bin-20261024/cli-jarde-cli` SHA `4606123560e52a2d840f9450135b4bd2c1786362d4148e49af65b224c1398cf0`。build43379/native48051/product92887均终态exit0；R1机制一次同步exit1在任何当前CLI前，R2机制98745 exit0；核心只读R1一次同步exit1，R2 87069 exit0复用408保存报告。self-test早期导出错误/原shell总体0与独立成功receipt分别记录。原9工具native AST冻结后未改，修正仅在resume-r2；无成功native/CLI/JDK重跑，不称首次全绿。专用target缺席，30产物绑定，906653750bytes仅为构建磁盘护栏，AE5980payload未改。

下一组完整迁annotation现有事实类型所有权、重复/参数对齐/target/path/source position，并收拢field实际类型位置；复用已有metadata_literal/declaration，不复制formatter，不让Java依赖root。package-info窄证书随后独立迁移，Java应接管包名合法性。然后完成2.9 raw/bridge/span真实来源闭包和2.10实际read/Body/IR/live/peak/RSS/取消/停止/背压及八旧失败；不得拿局部六门或构建峰值关闭S1。17/59，2.9/2.10/S1未完成，S2不进入；持续目标active，未提交/推送/归档。

# 当前接续：S1 类与字段源码策略检查点（2026-10-24，AE）

[结果](openspec/changes/restructure-demand-driven-recovery/results/s1-class-field-policy-v1/README.md)、[收据](openspec/changes/restructure-demand-driven-recovery/results/s1-class-field-policy-v1/acceptance-r1.json)、[下一组职责](openspec/changes/restructure-demand-driven-recovery/results/s1-class-field-policy-v1/next-boundaries.md)。

类/字段普通声明与共享names/flags/type/ConstantValue/marker、field Signature源码准入已归Java；root保留读取、物理/SSA/erasure/CP证明、预算/停止、deferred和完整提交。四生产文件纯迁移，2735来源，合同35/schema12/解析4/v9保持。六项局部门禁通过：29 native1173/0/2；新类/字段48normal+8budget、继承ordinary56/family65/constructor248/NoBody24/body12/130cutoff/name3及65compact/32原子partial保持；168全文/78声明、136×3与408完整JSON保持，当前6编译12受控JVM/真实两类4重编通过，真实目标未执行。

非enum四组的ClassField/Iface/Anno原样生成源隔离编译、反射/ConstantValue通过，当前16份完整源码逐字节相同后复用。FieldEnum所选域四源整体编译失败与AD数组返回维度注解旧拒绝保留。首partial只补采一条，未重建JAR；非enum首组汇总解析失败，已成功JDK原流离线复用，后三组续采，首exit控制流重建与派生元数据纠错明确；不称首次全绿。30产物/五命令/源码绑定，target已清理，AD5759冻结payload未改。

当前CLI `/private/tmp/jarde-s1-class-field-policy-r1-bin-20261024/cli-jarde-cli` SHA `2cb3886155ac2b004bc45b4908745f1663d6bac236ec0b176fa23ee505da02ad`。build47806、native84237、mechanism93821、product61714、core只读1873均终态exit0。本任务CLI/JDK均终态，不重启这些句柄或重复成功前缀；不宣称其他项目全局无进程。构建峰值906665720bytes仅为磁盘护栏，非产品资源结论。

下一组先完整迁class Signature的shape/source决定，保留原raw Signature发布时机、erasure/nested位置/父定义budgeted callback和interface-only回退；之后迁annotation位置/重复、package-info，再做2.9 raw/span/bridge闭包与2.10 actual read/Body/IR/live/peak/RSS/取消/停止/背压及八旧失败。不得只迁formatter就声称class Signature完成。17/59，2.9/2.10/S1未完成，S2不进入；持续目标active，未提交/推送/归档。

# 当前接续：S1 普通方法声明与类族header策略检查点（2026-10-24，AD）

[结果](openspec/changes/restructure-demand-driven-recovery/results/s1-ordinary-method-policy-v1/README.md)、[收据](openspec/changes/restructure-demand-driven-recovery/results/s1-ordinary-method-policy-v1/acceptance-r1.json)、[后续机制](openspec/changes/restructure-demand-driven-recovery/results/s1-ordinary-method-policy-v1/next-boundaries.md)。

Java承接完整普通method的descriptor源码准入、flags/default方法、ctor/clinit、debug与同次恢复参数名、wide slots/MethodParameters final、varargs/throws/default/type annotation装饰及marker，既有参数事实类型与共享命名/可见性/类型装饰helper迁移后内部重导出。has_supported_generic_member_family_header的源码子域也归Java；root保留物理读取、身份、属性事实/报告、scope可用性、原阶段/预算/停止及完整提交。五生产文件纯迁移，无新DTO/Session/read/cache/产品test函数/资产，恢复35/schema12/解析4/inputs v9保持。

六项有效门通过：26 native 1158 passed/0 failed/2既有ignored（新增纳入class_source110与annotation_default14旧测试）；16上下文48complete与8非零预算边界共56份普通声明完整报告保持；家族65/构造器240+8/NoBody24及body12/130cutoff/name3、65compact error/32原子partial点保持；168全文/78声明及当前136×3和408完整JSON保持，6编译12受控JVM/真实两类4重编通过，真实目标未执行。16份当前生成全文相同后复用4组旧生成编译/4定向隔离Driver；数组return维度注解旧拒绝造成四组reflection差异仍明确保留，不称完整反射等价。完整JSON仅排除actual engine与usage.elapsed，physical snapshot/raw/正文/marker/limits/费用/来源/coverage不变。

2735来源/归档/构建前/当前、27产物与五成功命令同次绑定，target清理，769844016bytes仅作构建磁盘护栏；AC5527payload未改。原机制driver在constructor调用前误把历史label当pin，R2又在调用前误比较相对/绝对索引路径表示；原脚本和失败流保留，R3核验80个冻结JAR的实际path/bytes/SHA后只续验后缀，复用decl56/family65，无重复成功native/CLI/JDK，不称首次全绿。

前轮清单误称generic_enclosing_scope_is_eligible，实际源码一直是has_supported_generic_member_family_header，当前规划已更正，AC封存不改。重新审计明确class/field普通声明、class/field Signature准入、annotation位置/重复与package-info仍有源码策略在root，按共享机制继续迁移，不把所有String assembly一概当债务。随后继续2.9 raw/span闭包与2.10 actual read/IR/live/peak/RSS/停止/背压及八旧失败；不据局部门禁关闭S1，进度17/59，S2未进入。

当前CLI `/private/tmp/jarde-s1-ordinary-method-policy-r1-bin-20261024/cli-jarde-cli` SHA `b08a0611961947ac8d899d337503b85b479ddb7bb3d58cf5a3a7a9a9be95b31f`。build65537/native68071/product87042/core离线27408均终态exit0；机制R1 63153终态exit1，R2一次同步preflight命令exit1且无constructor CLI；R3 31120终态exit0。成功ordinary56/family65被复用，constructor248仅R3执行；nobody/body/budget/name后缀均通过。构建与本任务产品调用无重叠；本任务进程均终态、专用target缺席，不宣称其他项目全局无进程。持续目标active，未提交/推送/归档。

下一步从类/字段共用声明flags/type/name及field Signature源码准入继续，随后class Signature、annotation位置和package-info。真实函数与局部caller/测试盲区见remaining-source-responsibilities。当前已迁普通method与family header源码子域不重复算残留；2.9/2.10/S1仍未勾选，不进入S2。下方AC及更早记录为历史范围，冻结结果不修改。

# 当前接续：S1 泛型类族方法策略检查点（2026-10-24，AC）

[结果](openspec/changes/restructure-demand-driven-recovery/results/s1-member-family-policy-v1/README.md)、[收据](openspec/changes/restructure-demand-driven-recovery/results/s1-member-family-policy-v1/acceptance-r1.json)、[后续边界](openspec/changes/restructure-demand-driven-recovery/results/s1-member-family-policy-v1/next-boundaries.md)。

Java 接管已选泛型类族 method 的三阶段源码策略：属性读取前的准入、parse 后 erasure 前的作用域拒绝、erasure 后的完整 shape/声明与 proof 分类；root 保留同次读取、raw 身份、erasure、物理关系/候选/闭包、停止和原子提交。消除 child 为清 marker 复制整份 record 的临时 clone。两生产文件纯迁移，无新 DTO/read/cache/test 函数/产品资产，恢复35/schema12/解析4/inputs v9保持。

六项局部门禁通过：24 native 1034 passed/0 failed/2既有ignored；19类族输入×三evidence及8非零预算边界共65份报告保持；继承构造器240+8、NoBody24/body12/130cutoff/name3及65 compact error/32原子partial点；168全文/78声明保持；当前136×3与408完整JSON保持，6编译12受控JVM/真实两类4重编通过，真实目标未执行。四组原件/旧生成8 oracle通过，当前4份生成全文相同后复用，无重复JDK。完整JSON仅排除已核验actual engine tuple与usage.elapsed，物理snapshot、正文/marker、limits/费用、来源/coverage及partial范围不变。

类族normal57为45complete/12partial；24份prepared_fold保留原普通回退，effectful_id/method_local_root12份保持partial，extra_child6份家族refused但执行complete。2735来源/归档/构建前与25产物、五成功命令同次绑定，target清理；769748398bytes只作构建磁盘护栏。AB5934payload未改。collector遗漏javac版本stdout的原空字段保留，以采集后元数据补记说明；不补造原流。

剩余has_supported_generic_member_family_header的源码前置选择、member descriptor/普通声明、bridge/raw来源闭包与2.10实际资源/停止/八旧失败继续按清单推进。本局部不证明恢复域收益或实际CPU/RSS改善。2.9/2.10/S1仍未验收，OpenSpec17/59，S2未进入。

当前CLI `/private/tmp/jarde-s1-member-family-policy-r1-bin-20261024/cli-jarde-cli` SHA `08cda835541a32f2a098270fc16ec074ee3a4e7f63e8eb0a373db2cf27cf3b43`。build36505/native4021/mechanism93844/product97616/core离线69425均终态exit0。工具wrapper导出及非零exit假设在native前修正；首次无参数build guard只打印usage并退出，未运行Cargo或写构建结果，随后带计划参数五命令均成功。所有成功native/CLI/JDK不重复。无活动Cargo/产品CLI/JDK，专用target缺席；持续目标active，未提交/推送/归档。

下一步先分离Facade的has_supported_generic_member_family_header中的源码选择，再按member descriptor/普通声明与2.9/2.10门推进；本轮已选family method策略不再算root残留。下方AB与更早历史属于各自冻结范围。

# 当前接续：AB 泛型构造器源码策略已局部验收（2026-10-24）

[结果](openspec/changes/restructure-demand-driven-recovery/results/s1-generic-constructor-policy-v1/README.md)、[收据](openspec/changes/restructure-demand-driven-recovery/results/s1-generic-constructor-policy-v1/acceptance-r1.json)：Java接管class-scope构造器选择、完整准入、candidate/body-proof回退、InitRecord/参数槽/作用域/字段写入证明、raw字段唯一匹配及声明/proof分类，root保留同次facts、subject身份、erasure/CP/同类调用与委托物理门、停止和完整提交。两生产文件纯迁移，无新DTO/read/cache/test函数/产品资产，恢复35/schema12/解析4/inputs v9保持。有效六门通过：24选定native1034 passed/0 failed/2既有ignored；80上下文240报告与8非零预算边界全文、marker、声明、物理身份、费用/limits、来源/coverage保持；52上下文有成功构造器投影，28没有，RawNewHold旧Partial保留。NoBody24/body12/130cutoff/name3、65compact error与32原子partial点、168全文/78声明保持；当前136×3全文及408完整JSON除已验证engine与usage耗时外匹配，核心6编译12受控JVM/真实两类4重编通过，真实目标未执行。2735来源/归档/构建前、25产物及五成功命令同次匹配，target清理。六份旧GenericConstructor缓存报告与重新生成的JAR身份错配已保留原流，以现存冻结JAR补采AA旧6份，当前235前缀复用+5续跑，不改snapshot、不重建或重跑成功native/JDK，不称首次全绿。subject纯比较求值提前的实际CPU成本归2.10；剩余family/descriptor/普通声明、bridge/raw与资源门及八旧失败继续承接，2.9/2.10/S1未验收，OpenSpec17/59，S2未进入。

当前CLI `/private/tmp/jarde-s1-generic-constructor-policy-r1-bin-20261024/cli-jarde-cli` SHA `dfc3748fbfeb188674343f2754ac1c8433d002941bb14e6c41f237c17eaefa88`。build45830、native33822、product90564、core离线66368、R2续验74243均正常终态exit0；首mechanism81769终态exit1，第235当前结果对错误旧基线失败，已复用到R2验证，不重复成功native/CLI/JDK。无活动Cargo/CLI/JDK，target缺席。持续目标active，未提交/推送/归档。

下一步按[后续边界](openspec/changes/restructure-demand-driven-recovery/results/s1-generic-constructor-policy-v1/next-boundaries.md)处理`project_member_family_generic_signature`完整shape/拼写策略与原始物理family关系分离；随后member descriptor/普通声明和2.9/2.10总门。2.9/2.10/S1不勾选，S2不进入。下方AA及更早记录均为各自已冻结范围。

# 当前接续：AA NoBody 泛型策略已局部验收（2026-10-24）

[结果](openspec/changes/restructure-demand-driven-recovery/results/s1-no-body-generic-policy-v1/README.md)、[收据](openspec/changes/restructure-demand-driven-recovery/results/s1-no-body-generic-policy-v1/acceptance-r1.json)：Java 接管 NoBody 的层级源码准入、method-local 泛型完整声明和 slot/annotation/varargs/header 拼写，root 保留原 erasure 后预检位置、同次物理事实、CP 费用、调用绑定、停止和提交；没有新 DTO/read/cache。三生产文件与两现有测试增强，恢复 34→35，schema12/解析4/inputs v9 保持。纠正 ordinary NoBody 误称 AST/SSA 的说明，新旧均39字节，原 method-local 说明保持。有效六门：22 native 1023 passed/0 failed/2既有ignored；7上下文21报告、2实际预算stop与1零值输入拒绝保持；继承body12/130cutoff/name3，65 compact error完整JSON和32原子partial点通过；168全文/78声明保持。当前136×3对Z完整JSON除物理NoBody说明、实际engine tuple和usage.elapsed外相等；20上下文72次物理marker，其中16上下文48份顶层文本改变，声明/raw/费用/limits/来源/coverage保持。当前核心6编译12受控JVM、真实两类4重编通过，真实目标与变异未执行。2735来源/构建前/归档、23产物及五成功命令同次绑定，target清理；770145514bytes仅为构建磁盘护栏。首core driver全文断言exit1与合同preflight字段错误均保留，R2复用已成功原流离线验收，没有重复CLI/JDK或重跑native；不称首次全绿。后续构造器、family source policy、member descriptor与普通声明拼写及bridge/raw/资源退出门见后续清单；八项旧失败不在本次分母，2.9/2.10/S1仍未验收，OpenSpec17/59，S2未进入。

当前 CLI `/private/tmp/jarde-s1-no-body-generic-policy-r1-bin-20261024/cli-jarde-cli` SHA `82ac69184a852ff3872861342420493f10ed43a5bbc62685c8be3073533fb1c8`。构建63127、native27222、current定向命令及contract R2 26539正常终态exit0；首次product39740终态exit1，core保存结果R2离线7429终态exit0。无活动Cargo/CLI/JDK，target缺席。所有成功native/CLI/JDK原流已保留，不重启这些终态句柄或重复成功oracle。未提交、推送、归档；持续目标active。

下一步先按[后续边界](openspec/changes/restructure-demand-driven-recovery/results/s1-no-body-generic-policy-v1/next-boundaries.md)梳理 generic constructor 的完整准入/字段匹配与物理绑定消费点，再迁整体源码策略，保留原候选顺序、费用、停止和原子提交；并补其既有测试族对照。family/descriptor/普通声明与2.9/2.10资源仍是后继，不进入S2。

# 当前接续：Z ordinary parameterized策略已验收（2026-10-24）

[普通参数化声明策略迁移](openspec/changes/restructure-demand-driven-recovery/results/s1-ordinary-parameterized-policy-v1/README.md)、[收据](openspec/changes/restructure-demand-driven-recovery/results/s1-ordinary-parameterized-policy-v1/acceptance-r1.json)（2026-10-24）：Java接管完整ordinary准入/返回来源/throws/声明与proof分类及exact-null选择，root保留同次事实、物理身份、有序路由、停止和提交；Signature/descriptor/varargs现有实体迁入Java并统一generic解析，无新DTO/read/cache。三生产文件纯迁移，合同34/schema12/解析4/v9保持。六门有效验收：22native1023/0/2，9开发context的27evidence+2预算逐mode源码/方法/execution/coverage/logical usage保持；实际19方法成功/4旧拒绝，Generic族单列。继承body12/130cutoff/name3保持，65compact error完整JSON核对，32个双body完整partial点无半项泛型头；168源码/78声明及当前136×3、核心6编译12受控JVM/真实两类4重编保持，真实目标与变异未执行。2735来源/归档/构建前、23产物与五成功命令同次匹配，target清理；峰值769858021bytes仅是构建护栏。旧CLI误选和name验证器KeyError原流保留；修正后复用成功门与all前缀，不称首次全绿。NoBody误称AST/SSA的旧文案、未覆盖的List参数primitive结果、临时return拼写成本和其余职责/资源门按[next-boundaries](openspec/changes/restructure-demand-driven-recovery/results/s1-ordinary-parameterized-policy-v1/next-boundaries.md)承接；八个旧失败未重测或修复，2.9/2.10/S1不勾选，OpenSpec17/59，S2未进入。

当前CLI `/private/tmp/jarde-s1-ordinary-parameterized-policy-r1-bin-20261024/cli-jarde-cli` SHA `2039174ddb8bfd60469b04626cb1339af6a26952a7452379d0d886c1f150380f`。build12316正常终态exit0；driver7050在name验证器KeyError后终态exit1，native22/ordinary29/body12/budget130均成功不重跑；nameR2手动命令exit0复用all且只新运行两evidence；剩余三门session46185终态exit0。无活动Cargo/CLI/JDK；target缺席。后续按Z/next-boundaries推进NoBody/method-local/class-scope/constructor职责与2.9/2.10退出门，不能只迁formatter；目标active，不进入S2，未提交/推送/归档。

# 当前接续：Y 同类generic body声明已验收（2026-10-24）

[同类generic body声明迁移](openspec/changes/restructure-demand-driven-recovery/results/s1-generic-body-declaration-v1/README.md)、[收据](openspec/changes/restructure-demand-driven-recovery/results/s1-generic-body-declaration-v1/acceptance-r1.json)（2026-10-24）：完整bounds/formals/return sources/throws准入与header组装归Java，root只核对certificate↔physical member、供事实/原顺序/停止/提交；复用descriptor/严格命名/共享header，无新DTO/Session/cache/read。两生产文件纯迁移，合同34/schema12/解析4/v9保持。现有预算sweep加入method-local T/U的MethodShadow，不新增test function；扩入既有same_class_generic_binding16项，17选定native977/0/1。四开发context×三evidence的12报告真实进入本route，当前源码/方法/execution/完整coverage/logical usage与旧X一致（仅测量elapsed排除）；旧完整生成8编译16VM及原oracle16VM已绑定，当前同字节复用，非holdout/恢复域收益。旧/当前同130cutoff全匹配，32个双body完整停止点均0或2泛型头。普通UTF8非MUTF name只改CP #8，旧/当前各3报告均Reader Failed，methods[1]拒绝且保留init前缀/完整coverage，未进genericroute，变异未编译执行；任意raw helper域差异不能冒充产品可达域变化。168源码/78声明、当前136×3/6编译12受控JVM/真实两类4重编保持，真实目标未执行。2735来源/归档/构建前与18产物、四正常命令匹配，target清理，峰值770427606bytes仅是构建守卫。target对象审查、elapsed比较、复制stem与tuple序列化四类工具失败原稿/原流保留；续验复用成功native、3个CLI前缀和130预算原流，不称首次七门全绿。剩余ordinary完整路由/no-body/constructor/class-scope、bridge/needle与实际read/IR/资源/停止门见[next-boundaries](openspec/changes/restructure-demand-driven-recovery/results/s1-generic-body-declaration-v1/next-boundaries.md)；八项既有失败不在本轮分母，2.9/2.10/S1不勾选，OpenSpec17/59，S2未进入。

当前CLI `/private/tmp/jarde-s1-generic-body-declaration-r1-bin-20261024/cli-jarde-cli` SHA `6bd369e5befaebb80f949b583b32d07e2d1071a2bb13544a85c2cfb7beeb9d9c`。build72695、gate23993(R1 body比较器失败)、37335(R3预算汇总序列化失败)、89746(R4四门exit0)均终态；native与130预算CLI已成功执行，不重启/重复成功native/JDK/oracle。完整有效七门选择见Y/effective-gates.json，源码归档与当前2735资产匹配，target缺席。工具修正未改产品源码。

下一步按Y/next-boundaries.md推进ordinary parameterized完整route+strict policy，再处理no-body/class-scope/constructor；不能只搬formatter。2.10实际read/IR/live/RSS/取消/背压及旧失败承接待完成。持续目标active，未提交/推送/归档；S2不进入。

# 当前接续：X 泛型专门声明策略已验收（2026-10-24）

[泛型专门策略迁移](openspec/changes/restructure-demand-driven-recovery/results/s1-generic-declaration-policies-v1/README.md)、[收据](openspec/changes/restructure-demand-driven-recovery/results/s1-generic-declaration-policies-v1/acceptance-r1.json)（2026-10-24）：Java接管method-local empty-void throws、instance null-return、static local throws三条策略的Option<Result>路由/严格准入/完整声明，root保留事实、原顺序、停止与提交；复用共享header，无新DTO/Session/cache/read。实际以W33 CLI复现只改Signature CP #11的interface-only bound导致exit101/panic，Code不变；Java显式要求class-bound且无附加interface-bound后局部拒绝，保留物理方法/正文/Exceptions，合法控制保留泛型。恢复33→34/schema12/解析4/v9。16选定native961/0/1，新增永久反例1/0；静态throws既有3测试扩入cohort，不称新增恢复收益。当前原/变异6报告、W正6报告/负18报告通过；对W33的18负例源码/正文/完整coverage/前缀严格保持，对W32另18比较仅3个bridge旧拒绝文案不同并显式记录，不称完整JSON一致。168完整源码/78声明及当前136×3/6编译12受控JVM/真实成功两类4重编保持，真实目标及变异未执行。2735来源/构建前/归档、17产物与四成功命令同次匹配，target清理，构建峰值755250698bytes不冒充S1资源验收。W6218冻结payload全部核验保持；剩余generic body/ordinary/no-body/constructor/class-scope策略及bridge/needle/2.10资源停止门按[next-boundaries](openspec/changes/restructure-demand-driven-recovery/results/s1-generic-declaration-policies-v1/next-boundaries.md)承接，八项既有失败未修复或重测，本次非全仓门；2.9/2.10/S1不勾选，OpenSpec17/59，S2未进入。

当前CLI `/private/tmp/jarde-s1-generic-declaration-policies-r1-bin-20261024/cli-jarde-cli` SHA `ef9b3f72a938fba0f23c174956246422c8b015e5ffbda40e1e1c14fe816ff9e2`。build session76622与gate session72956均正常终态exit0，target缺席；不要重启/轮询成功句柄或重复成功native/JDK/oracle。X/source-binding-review-r1及本目录归档与当前源码匹配。前继W冻结目录不改写。

下一步按X/next-boundaries.md继续S1剩余职责，先依据实际调用划清generic_method_body、ordinary/no-body、constructor与class-scope throws源码政策；不能只搬formatter，保留同次事实/身份/停止/完整提交。2.10实际read/IR/live/RSS/取消/背压与旧失败承接尚未完成。持续目标保持active，未提交/推送/归档，S2不进入。

# 当前检查点：S1 generic声明边界第三轮完成（2026-10-24，W）

[结果](openspec/changes/restructure-demand-driven-recovery/results/s1-generic-declaration-boundary-v1/README.md)、[收据](openspec/changes/restructure-demand-driven-recovery/results/s1-generic-declaration-boundary-v1/acceptance-r3.json)：Java接管完整generic-void Option<Result> route/strict生成及共享generic method header，root只供事实/顺序/停止/提交；raw sibling严格MUTF出口保留身份守卫。恢复33/schema12/解析4/v9；三生产文件+1永久test/2jar，3新Javaunit+1engine。

R3四命令正常成功，2732当前source/source-before/archive与15binary匹配；target清理，峰值725760701bytes。六gate session10976终态exit0，14native957/0/1，6正/36负，168源码/78声明保持；当前136×3/6编译12受控JVM/真实两类4重编通过，真实目标/变异未执行。selectedleaf泛型反射2/4→4/4，JDK23编译2/2保持；JDK8 leaf旧exit4崩溃保留，不称完整family/holdout。CLI `/private/tmp/jarde-s1-generic-declaration-r3-bin-20261024/cli-jarde-cli` SHA `ff7e7c80fa38574863ae2f7aed32cde4c5679d9ababdf6df52898febfd4371ee`。

R1编译E0609原流保留，R2六门通过后发现route仍在root，R3已补全；独立source review初误选R2 archive，pre-path-correction保留，实际R3全符。W最终静态/独立复核通过并封存，见evidence-seal-r3.json；R3 Cargo60778和gate10976均终态，不poll/重启/重复成功oracle。

用户已授权接着推进S1剩余职责：优先同一generic声明族的method-local empty-void throws、null-return、static local throws，须把各自route+strict policy一并归Java，保留原顺序/None与拒绝区分/费用点/事务，复用当前commonheader，不增DTO。不进入S2。OpenSpec17/59，2.9/2.10/S1仍false；goal active，未提交/推送/归档。

# 本次指定范围：S1 / 2.6 当前构建复核完成（2026-10-24，v25）

用户本轮仅要求2.6。[结果](openspec/changes/restructure-demand-driven-recovery/results/s1-environment-recheck-v25/README.md)：九条native命令终态正常达到预期，旧loader2/身份7失败实际重现；当前288passed/0failed/1既有ignored，其中含A1.4全11。2729当前来源/构建前清单/归档、七项同次V R1二进制匹配，5895冻结payload核验。恢复保留引用同源码/CLI的V/core-r1原流，本轮未运行CLI/JDK/Cargo，未改生产源码或测试体，target缺席。

2.6保持通过，OpenSpec17/59，2.9/2.10/S1仍未完成，S2未进入。旧V目录不修改；只有新相关源码或失败证据才重开2.6验证。此前V的未完成架构边界仍按下方记录承接，本轮没有推进它们或授予额外验收。持续目标保持active，未提交/推送/归档。

# 当前检查点：S1 / 2.9 显式bound cast已局部验收（2026-10-24，V）

持续目标接续S1，S2未进入。[结果](openspec/changes/restructure-demand-driven-recovery/results/s1-generic-cast-source-v1/README.md)、[收据](openspec/changes/restructure-demand-driven-recovery/results/s1-generic-cast-source-v1/acceptance-r1.json)：overload cast的AST Type复用names::source_class_name，SignatureType::Class.binary_name保持raw MUTF；不放宽候选/explicit bound/most-specific/flags/census。两个生产文件，恢复32/schema12/解析4/inputs v9；Driver的relay T反射4/8→8/8（E只做声明+编译验证），生成编译4/4保持。supp新2编译4JVM，ASCII同字节引用旧2编译4JVM；六报告/新永久test1/0，七metadata变体42报告保持，截断Failed可靠前缀1/4、skip[1,4)，变异未编译执行。

九命令同次正常成功，2729当前来源/归档/source-before、75binary/74cleanup匹配，target清理，target采样峰值962873835bytes。74native1959passed/8旧failed/11ignored，没有新增失败。14实际门全部满足；U正负39/T-memberpath15/数组12/旧能力保持；A1.4全11/相邻288保持；当前32CLI实际重跑136×3/6编译12JVM/真实两类4重编通过，真实目标未执行。U全部4952payload/133748779bytes核验，U/v24/T保持冻结。

CLI `/private/tmp/jarde-s1-generic-cast-product-r1-bin-20261024/cli-jarde-cli` SHA `5f79b250132ed87b3b5ac878b954839fa8f357fc746a01adcc0059285a661ebd`。build60379、gate19762、finalize均终态正常；不得继续poll或重复成功native/JDK/oracle。工具静态假设修正（coverage helper归属/input支持class项/路径）原稿保留，未为比较器改产品门。前继U原native失败仍保留，不称历史首次全绿。

下一步按V/next-boundaries.md：先实际复现root selected_nested_sibling_bound_spelling的ASCII/supplementary sibling路径（现只有静态3处from_utf8风险，尚未称产品失败），保持raw同族/唯一row/physical spelling门；再统一到共享严格命名。V/responsibility-audit-input.md已修订职责边界：整个generic void的binder/void/throws/bound/erasure源码适用与声明拼装归Java，root仅供事实/subject/停止/提交。只搬一个sibling spelling helper不能关闭A1.1；过窄原审查存pre-layer-review。不要求兼容旧内部路径，但保持现有有效恢复。bridge span/projection needle及2.10全部readIR/停止/资源门仍未闭合，八项旧失败按原归因承接；禁止先进入S2。

OpenSpec17/59，2.9/2.10/S1仍未勾选；持续goal active，未提交/推送/归档。下方U/v24为各自已冻结历史范围，不重复循环已完成2.6或U/V检查点。

# 前继检查点：S1 / 2.9 泛型消费者局部验收完成（2026-10-24，U）

持续目标接续OpenSpec，本轮在既有S1范围完成U验收，没有重复2.6循环，未进入S2。[结果](openspec/changes/restructure-demand-driven-recovery/results/s1-generic-source-consumers-v1/README.md)、[收据](openspec/changes/restructure-demand-driven-recovery/results/s1-generic-source-consumers-v1/acceptance-r2.json)：四生产文件接通共享严格类名和同次AST真实成员路径，raw descriptor/Signature不变，恢复31/schema12/解析4/inputs v9。正式六输入泛型反射8/24→24/24，完整生成编译12/12保持；本轮新8编译16JVM，旧同字节4编译8JVM核验复用。两字面$类仍是旧Partial/anonymous_typed_nesting_unproved。新永久测试3/0，旧List目标19/0/1；73native1958passed/8knownfailed/11ignored，七变体42拒绝报告、15局部保留和旧能力门满足。A1.4全11/相邻288保持；v24同CLI的136×3/6编译12JVM/真实两类4重编核验1797payload后引用，未重复oracle，真实目标未执行。

R1九命令正常成功、R2单命令只重建修正测试；当前2726来源/资产、归档和74binary匹配，选择R1排除oldbadtest+R2testfix、73test，不称完整R2同次构建。target均缺席，max948672446bytes。CLI `/private/tmp/jarde-s1-generic-source-product-r1-bin-20261024/cli-jarde-cli` SHA `43c86e05f54b2bdfc372f050d336d2984cd8f80c97399ca930843f5ee6480734`。session40700十三门、finalize均正常终态；R1原新test2/1错误控制、原流和R2三test-only差异保留，不能重跑或改写成首次全绿。

下一安全动作已有实际证据：`U/overload-probe-r2`（session5197正常exit0）原2compile/4JVM正常，旧生成4compile正常，ASCII泛型4/4而supplementary0/4；`U/overload-current-r1`两报告原文与旧完全相同，证明当前仍有缺口。class-scope T/E、overload第二参数String/Object使cast必要性可达，普通效果均bound，丢的是T/E泛型。修复只将 `generic_source_proof.rs::prove_same_class_overload_argument_upcasts` 的AST Type接共享严格命名，semantic ClassTypeSegment.binary_name须直接保留raw target_name，禁止将source UTF-8重编码成raw身份；不放宽候选/explicit bound/most-specific/flags/census。下一项先复用现有2原控制/Driver/oracle并添加永久测试，再冻结来源构建。详见U/next-boundaries.md；bridge span/projection needle、嵌套bound与2.10职责/readIR/停止/资源仍未闭合。OpenSpec17/59，2.9/2.10/S1不勾选；持续goal active，未提交/推送/归档。

## 前继指定范围：2.6 v24（该轮范围不外推）

### S1 / 2.6 已完成复核（2026-10-24，v24）

[本轮结果](openspec/changes/restructure-demand-driven-recovery/results/s1-environment-recheck-v24/README.md)：实际重现旧 2 个 loader / 7 个环境身份失败，当前 A1.4 全 11 项及相邻 **288 passed / 0 failed / 1 既有 ignored**。当前 2726 来源/归档匹配，七个二进制明确选自 generic-source R1，生产源码与当前 R2 相同；R2 仅泛型外置测试/两 fixture 共三项变化。三个环境实现/测试完整文件与 v23 相同；facade 差异仅数组 helper 职责提取，不谎称整个 facade 不变。136×3 正文/核心零差异、6 编译/12 受控 JVM、真实两类4重编本轮实际通过，真实包目标未执行。

本轮未改产品源码/测试体，未启动新 Cargo；沿原 session94431 确认进入本轮时在途的 test-only R2 正常 exit0/target 清理，native23811 和恢复87374 均终态成功。不要重启/轮询这些句柄或重复成功 oracle。CLI `/private/tmp/jarde-s1-generic-source-product-r1-bin-20261024/cli-jarde-cli`，SHA `43c86e05f54b2bdfc372f050d336d2984cd8f80c97399ca930843f5ee6480734`。只验收2.6，OpenSpec17/59；2.9/2.10/S1总门未完成，S2未进入。持续目标保持active，未提交/推送/归档。

## 先前 U 泛型消费者检查点仍待验收；不得由历史下一步扩大本轮范围

`results/s1-generic-source-consumers-v1` 的 R1 九命令构建成功、74二进制，首次 native74241 终态exit1，新 target 2pass/1fail，原因是 top-level `$` class<T> 控制先被既有 class header 门拒绝。已保存错误控制和失败原流，改用 nongeneric owner/method-local<T> 的 R4 正控制；旧两 dollar 方法丢泛型，当前局部探针恢复方法泛型，但全类仍保留既有 Partial/anonymous_typed_nesting_unproved，不得叫 Complete。

R2只重建修正的新泛型测试，1命令正常成功、source-before/归档2726项、target缺席；delta恰为测试源和两个jar，生产源码没有后续变化。新的R2测试尚未执行，R1旧badtest不能计入通过。List两输入、普通rawreceiver两输入、修正dollar两输入为六正式控制；真实member-path保留另计。现有工具静态修正和未跑专项保留，后续仅在获准继续2.9时执行，不把本次2.6/核心保留门当其验收。

旧U交接头原文保存于v24/handoff-prior-head.md，供追溯；其中“74241 live”等旧状态已由这里的终态覆盖。下方T及更早检查点均为其各自冻结范围。

## 当前检查点：S1 / 2.9 write-accessor raw descriptor 已局部验收（2026-10-24）

[结果](openspec/changes/restructure-demand-driven-recovery/results/s1-write-accessor-raw-v1/README.md)、[收据](openspec/changes/restructure-demand-driven-recovery/results/s1-write-accessor-raw-v1/acceptance-r1.json)：两个生产文件用reader组件/现有raw owner与JvmBytes贯穿字段/header/frame/双语句发射，原结构/SSA/flags/census门保持，无新增实体/读取路径。四合法开发控制helper Structured1/4→4/4，生成编译2/8→8/8；新6编译12受控JVM，ASCII同字节旧2/4引用，原4编译8JVM核验。12控制与六变体36选定负例通过，变异未编译/执行；损坏字段Partial可靠前缀/skipped fields0..1，不伪造方法枚举。

九命令同次正常成功，2718来源/归档、72二进制/71cleanup副本匹配，target已清理，峰值937919200bytes；恢复30/schema12/解析4/inputs v9。71目标1936passed/8既有failed/10ignored；A1.4全11与相邻288/0/1、旧能力及当前136×3/6编译12JVM/真实两类4重编保持，真实目标未执行。旧2+7环境失败引用已核验v23，不重跑。CLI `/private/tmp/jarde-s1-write-accessor-product-r1-bin-20261024/cli-jarde-cli`，SHA `536915b02256473f527a5e89162dc17ee7ec0b6783e25c5fbce482a13098c670`。

构建84067、首门68395、保留39485/66072、finalize30440均终态。68395在控制成功后因比较器误要求Failed退出；39485在enum嵌套binding29遗漏退出，修正后续验。原工具/原流保留。产品选定首个resume，followup冗余负例重跑保留不计数；不重跑成功JDK/native。详细选定关系见accepted-gates-r1及product-selected-r1。

下一步按[后续边界](openspec/changes/restructure-demand-driven-recovery/results/s1-write-accessor-raw-v1/next-boundaries.md)先复现其余六行/五机制raw消费者，随后2.10职责/停止/实际readIR/资源总门；不把静态清点当失败，不外推本项修复ASCII WA三项旧失败。OpenSpec17/59，2.9/2.10/S1未完成，S2未进入；持续目标active，未提交/推送/归档。不重新循环验收已完成2.6。

## 前继：S1 / 2.9 数组 helper raw 交接已局部验收（2026-10-24）

[结果](openspec/changes/restructure-demand-driven-recovery/results/s1-array-helper-raw-v1/README.md)、[收据](openspec/changes/restructure-demand-driven-recovery/results/s1-array-helper-raw-v1/acceptance-r2.json)：删除内部helper字符串重复，保留同次IR BCI/CP与raw目标；发射共用严格命名，整项声明/返回类型匹配归Java，facade只供物理选中事实/停止/事务。四合法开发输入数组投影1/4→4/4；旧8生成编译/16JVM本就成功，不算编译收益。R1九命令成功但新native0/1，factory supplementary漏门在R2补齐；原流保留。

R2九命令同次正常成功，2713源码/资产和归档、71二进制/70cleanup副本匹配，target清理；恢复29/schema12/解析4/inputs v9。70目标1935 passed/8既有failed/10ignored，新永久测试1/0、旧数组16/0；12控制报告/6新生成编译12JVM、六变体36恢复通过，变异未编译执行；截断编码Failed及可靠三成员前缀/skipped3..4保持。A1.4全11/相邻288/0/1、旧能力及当前136×3/6编译12受控JVM/真实两类4重编保持，真实目标未执行。

构建55237、native2245、保留门3530、负例71183均已终态；2245在比较器失败后退出，negative续跑复核引用已成功controls，不重复JDK或native。essential空map、合法别名和损坏prefix三个工具误假设及失败原流均保存，不称首次全绿。CLI `/private/tmp/jarde-s1-array-helper-product-r2-bin-20261023/cli-jarde-cli`，SHA `9d62c4fd038058ad07f01c9b7ef54953d61ee36bef4602be9b49f1b9bb05fd0d`。

持续目标active，下一步按[后续边界](openspec/changes/restructure-demand-driven-recovery/results/s1-array-helper-raw-v1/next-boundaries.md)先复现剩余raw消费者并补2.10总门。OpenSpec17/59，2.9/2.10/S1未完成，S2未进入；不重跑已完成2.6或旧成功oracle。未提交、推送或归档。下方R3与v23均为前继范围。

## 前继：S1 / 2.9 lambda 最终消费检查点已局部验收（2026-10-23，R3）

[结果](openspec/changes/restructure-demand-driven-recovery/results/s1-lambda-final-consumption-v1/README.md)、[收据](openspec/changes/restructure-demand-driven-recovery/results/s1-lambda-final-consumption-v1/acceptance-r3.json)：五生产文件保留raw Member/同次Code BCI envelope，公共名称终结与私有provisional类候选闭合；root复用精确ExplanationOnly marker及原文/span门。未闭合引用、独立方法和碰撞残留明确quote，整方法丢弃的合法lambda/accessor同步撤销presented。正式旧十项7/3，R3原十项+合法sibling共11/0、root marker/span单元通过；R1投影退化、R2合法sibling误报及两个工具假设失败原流保留，补跑后验收，不称首次全绿。

R3九命令同次正常成功，2708来源/归档与69二进制匹配、target清理；恢复合同28/schema12/解析4/inputs v9。68目标1918 passed/8既有failed/10ignored；27新拒绝、三组合9控制/18变异、sibling三evidence、16公共预算stop/8零预算拒绝通过；122lambda/captured-this（9旧非法源码改quote）、84invocation、132enum、adapters、168旧源码/78声明和当前136×3/6编译12受控JVM/真实两类4重编保持，真实目标未执行。组合3原编译6JVM/6生成编译12JVM在R2已成功，R3逐项核验后引用，不重复运行；变异只恢复。固定captured-this实际Body4、IR六阶段各4、read2/833bytes保持，只是局部观测，不替代完整S1资源门。

构建65543和主验收83304已正常exit0，不轮询/重启；R2旧验收16451因比较器退出，50518补跑正常；组合12114因region-fallback误假设退出，R3复用控制门已正常。CLI `/private/tmp/jarde-s1-lambda-final-product-r3-bin-20261023/cli-jarde-cli`，来源见product-build-r3/receipt。下一步按[next-boundaries](openspec/changes/restructure-demand-driven-recovery/results/s1-lambda-final-consumption-v1/next-boundaries.md)盘点其余raw精确消费者及2.10最终呈现/停止/实际readIR/资源，不按单样例加允许分支。OpenSpec17/59，2.9/2.10/S1未完成，S2未进入；持续目标active，未提交、推送或归档。下方v23止于2.6属于此前单轮范围。

## 前继指定范围：S1 / 2.6 当前源码复核完成（2026-10-23，v23）

[本轮结果](openspec/changes/restructure-demand-driven-recovery/results/s1-environment-recheck-v23/README.md)：实际重现旧 2 个 loader / 7 个身份失败；当前 A1.4 全 11 项通过，连同相邻回归 288 passed / 0 failed / 1 既有 ignored。当前 2707 项源码/资产及归档匹配，显式选择 R1 JVM 与 R2 根层/CLI 共 7 项二进制；四个 A1.4 完整源码/测试文件与 Q 一致，无新增产品或测试体修改。本轮实际重跑 136×3 正文/核心、6 编译/12 受控 JVM、真实两类 4 重编通过，真实目标未执行。

进入本轮时在途的 final-consumption R2（session 22635）已正常 exit0、七命令成功且 target 清理；当前 native 24445、恢复保留 41372 均 exit0，不轮询或重启。R2 CLI `/private/tmp/jarde-s1-lambda-final-product-r2-bin-20261023/cli-jarde-cli`，来源见本轮 binary-selection。本次止于 2.6，不继续 2.9；R1 投影回归失败原流及 R2 未验收专项保持，不能因本轮通过宣称其已修复。OpenSpec17/59，2.9/2.10/S1 未验收，S2 未进入；未提交、推送或归档。

## 前继指定范围：S1 / 2.6 复核完成（2026-10-23，v22）

[本轮结果](openspec/changes/restructure-demand-driven-recovery/results/s1-environment-recheck-v22/README.md)：实际重跑固定旧构建 2 个 loader / 7 个身份失败；当前 A1.4 全 11 项通过，连同相邻回归共 288 passed / 0 failed / 1 既有 ignored。2701 项当前源码/资产、归档、69 二进制及 5434 冻结载荷匹配 Q 构建，未发现新的 2.6 产品缺口，无产品或测试体修改。恢复保留引用同源码/二进制 Q 的既有 136×3、6 编译/12 受控 JVM、真实两类 4 重编；本轮没有 Cargo/CLI/JDK 运行，工具 preflight/parser 失败原流单列。

本轮范围止于 2.6。下方 2.9 接续与新建 final-consumption 复现保持待办，不因历史下一步自动继续实施；2.9/2.10/S1 未验收，OpenSpec17/59，S2 未进入。未提交、推送或归档。

## 当前检查点：S1 / 2.9 captured-this共享需求门（2026-10-23）

[结果](openspec/changes/restructure-demand-driven-recovery/results/s1-helper-demand-v1/README.md)与[收据](openspec/changes/restructure-demand-driven-recovery/results/s1-helper-demand-v1/acceptance-r1.json)：private/synthetic/lambda$+Code提出同类census需求，不再只认static，后续raw/kind/flags/完整使用/单site/同次AST及提交守卫保持。两生产文件，恢复26→27，schema12/解析4/inputs v9不变；没有新实体或读取路径。前继captured-this kind7开放门在ASCII/supplementary×inline/rename四开发输入转绿：生成编译0/8→8/8、16JVM一致，七永久测试旧6/1→当前7/0。固定输入essential/all实际Body4、IR各阶段4、entry读取2次833字节与红基线相同，不是零scan/AST/逻辑成本。68目标1913 passed / 8既有failed / 10ignored，A1.4全11/相邻288保持；五变异守卫15报告、明确IR预算停止无提交、旧能力回放及136×3、6编译12受控JVM和真实两类4重编通过，真实目标未执行。

中断11366的5步仅有Cargo成功事件、没有最终receipt，进程退出码/旧峰值/独立rustc记录保持unknown。39前缀产物逐项来源核验；24270在同target接续4步实际正常exit0产生30产物，target清理；选择69产物，不称九条正常退出。门6330十一项及finalize均已正常exit0，不轮询/重启旧句柄或oracle。2701来源/归档/二进制pin匹配。CLI `/private/tmp/jarde-s1-helper-demand-product-continuation-r1-bin-20261023/cli-jarde-cli`，SHA `3feee740c08598211e43e2e0214913c46be2fbf7dc836ec70d7f7e458b73e1b5`。

下一步先复现未闭合/shared/external/非private或nonsynthetic的非法lambda$引用、独立方法入口及碰撞后caller残留，复用现有Plan/Program/candidate与raw目标/BCI/CP闭合同次fallback来源，保留dash helper内联及本轮captured-this成功；见[next-boundaries](openspec/changes/restructure-demand-driven-recovery/results/s1-helper-demand-v1/next-boundaries.md)。其余raw消费者与2.10职责/停止/实际read/IR/资源/E1/E3总门继续承接。OpenSpec17/59，2.9/2.10/S1未完成，S2未进入；持续目标active，未提交、推送或归档。

## 前继检查点：S1 / 2.9 MethodHandle/helper（当时instance门开放，现由上方需求门检查点闭合）

持续目标接续S1，S2未进入。[结果](openspec/changes/restructure-demand-driven-recovery/results/s1-lambda-handle-names-v1/README.md)和[收据](openspec/changes/restructure-demand-driven-recovery/results/s1-lambda-handle-names-v1/acceptance-r4.json)：共享方法命名、严格MUTF helper连接与声明/暂存别名冲突守卫已实施；六永久测试旧2/4→当前6/0，16合法矩阵32/32生成编译、64JVM一致。68目标1912 passed / 8既有failed / 10ignored，A1.4全11、相邻288/0/1；既有有效完整源码与136×3、6编译12受控JVM及真实两类4重编保持，真实目标未执行。

**实际开放门**：captured-this kind7/ASCII helper原编译1/原JVM2成功，旧生成2编译失败，当前all完整源码逐字相同，未投影且诊断为空。13个独立门通过不能抹掉instance失败；没有补写其它3输入或其余evidence通过。下一步先定位同类census/候选/提交的真实阻塞，与未闭合/prefix-only helper及碰撞caller的fallback共同闭合，见[next-boundaries](openspec/changes/restructure-demand-driven-recovery/results/s1-lambda-handle-names-v1/next-boundaries.md)。不可只按新样例放开守卫。

当前产品2700来源/归档匹配，69二进制是12个R1下层+57个R4根层/CLI显式选择，不是同次构建；各自source-before与来源清单绑定、二进制SHA核验、target均清理。CLI `/private/tmp/jarde-s1-lambda-names-product-r4-bin-20261023/cli-jarde-cli`，SHA `825cc83d04f2e2c16f959d09d326736a3c723804a4503ee239c5f656c95a56b9`。构建21403正常exit0；首门66323在javap文本断言失败，补门10017在真实instance缺口失败，21300因collision工具目录重名失败，最后82312九独立门正常exit0，后处理正常exit0。所有句柄已终态，不重启旧oracle。

恢复26/schema12/解析4/inputs v9；OpenSpec17/59，2.6保持完成，2.9/2.10/S1未完成。持续总目标active，未提交、推送或归档。

## 前继用户范围：S1 / 2.6 复核完成（2026-10-23）

本次用户指定环境身份、defining loader 与 cache key，范围止于2.6。已有实现满足该项验收，本轮没有修改产品或测试体，没有新增Cargo/native/CLI/JDK运行。只读逐项核验[当前构建收据](openspec/changes/restructure-demand-driven-recovery/results/s1-invocation-name-boundary-v1/acceptance-r1.json)：2,669项当前源码/资产与归档完全一致，68个二进制及6,541个冻结载荷（111,851,156 bytes）摘要匹配，专属target不存在。该固定构建的A1.4全11项、相邻288 passed / 0 failed / 1既有ignored继续有效；不是本轮重新运行的计数。旧失败原流与机制说明见[2.6验收](openspec/changes/restructure-demand-driven-recovery/results/s1-environment-v1/README.md)及[v20复现](openspec/changes/restructure-demand-driven-recovery/results/s1-environment-recheck-v20/README.md)。OpenSpec strict及git diff --check通过。

完整domains/providers与snapshots身份、defining loader后继、loader/raw name缓存键、caller快捷路径和热命中取消仍保留。恢复保留引用上述同源码构建的136×3正文/核心、6编译/12受控JVM与真实两类4重编；本轮未重复运行。OpenSpec17/59，2.6保持完成，2.9/2.10及S1总门仍未完成。不得由下方历史下一步自动扩展本次范围，不进入S2；未提交、推送或归档。

## 前继检查点：S1 / 2.9 调用名称共享出口（局部验收）

[结果](openspec/changes/restructure-demand-driven-recovery/results/s1-invocation-name-boundary-v1/README.md)与[收据](openspec/changes/restructure-demand-driven-recovery/results/s1-invocation-name-boundary-v1/acceptance-r1.json)：五生产文件使普通声明、普通调用/特殊调用拼写共用raw方法名出口，Java7/8 `_`忠实保留，非法引用raw/BCI fallback；恢复24→25，schema12/解析4/v9保持。四永久旧1/3→4/0；16控制32生成编译30→32、JVM60→64，target7 `_`另4编译8JVM。67目标1906 passed / 8既有failed / 10ignored；A1.4全11/相邻288保持。12负例24错误Structured→Mixed/Fallback，12损坏报告Failed保持；前继R132完整报告及10 `_`源码/15负例、168旧完整源码+78声明、136×3/6编译12JVM和真实两类4重编保持，真实目标未执行。

构建79010正常exit0，2669来源/归档和68二进制匹配、target清理；CLI `/private/tmp/jarde-s1-invocation-product-r1-bin-20261023/cli-jarde-cli`，SHA `1fc672849ec29743a283e5b3979e66d0a5553e0d30639b224989ffa5a37204e5`。首pipeline99261在构造器比较器错选委托ctor后exit1；v2按物理descriptor修正，补验96228 exit0。原空源码harness及基线排序假失败/后处理名字解析错误均保存并如实排除，原始参考没有重复运行。所有上述句柄及最终后处理76561已终态，不轮询/重启。

持续目标active，下一步先实际复现lambda/method-reference的MethodHandle名称旁路，优先复用现有方法名出口，再按既定raw消费者清单闭合2.9/2.10；不要只据grep扩展实体或放宽原grammar。generic/enum独立名称门、复杂参数的拒绝来源闭包尚待核对；OpenSpec17/59，2.9/2.10/S1未完成，S2不进入。当前未提交、推送或归档。

## 当前检查点：S1 / 2.6 保持通过；在途 enum R3 已局部验收

[当前结果](openspec/changes/restructure-demand-driven-recovery/results/s1-enum-raw-names-v1/README.md)及[收据](openspec/changes/restructure-demand-driven-recovery/results/s1-enum-raw-names-v1/acceptance-r3.json)：A1.4全11项、相邻288 passed / 0 failed / 1既有ignored；环境/loader/cache代码片段保持，未增加2.6补丁。旧2+7失败引用v21已核验实际原流，本次不重复运行。

进入本轮时在途的enum R3构建76335已正常exit0，九步成功、2643来源及归档/67二进制匹配、target清理。验收23005在可选fields比较器误判后exit1；补验19653五门exit0，原流保留。66目标1902 passed / 8既有failed / 10ignored；132矩阵源码/报告限定差异保持，十个`_`控制20生成编译/40JVM，14×额外evidence、15变异负例、168旧完整源码+78声明、136×3/6编译12JVM与真实两类4重编均通过，真实目标未执行。CLI `/private/tmp/jarde-s1-enum-raw-product-r3-bin-20261023/cli-jarde-cli`，SHA `56efdf240957ad3f3d8d19e45c5a2cd1952788a8df731704e72e35920f252e76`。已终态句柄不轮询或重启。

负例另确认terminal helper dash/surrogate名称原样文本调用缺raw/alias，属于2.9待补的实际缺口，不能以enum投影拒绝视为全类正确。

本次指定2.6保持完成，不进入S2，不扩展下一raw候选。OpenSpec17/59；2.9/2.10/S1未完成，静态next-boundaries不算已复现。持续总目标保持active；工作区未提交、推送或归档。

## 前继完成：S1 / 2.6 当前构建验收（2026-10-23，v21）

[验收结果](openspec/changes/restructure-demand-driven-recovery/results/s1-environment-recheck-v21/README.md)：旧2个loader/7个身份失败引用已实际运行且本次核验的v20原流，不重复计数；当前A1.4全11项及相邻回归288 passed / 0 failed / 1既有ignored，另6项既有enum回归通过。完整环境声明、defining loader后继、三处memo/快捷路径及热命中取消保持，五个对应源码/测试文件与v20完全一致，本次没有新2.6补丁。136×3正文/核心、6原样编译/12受控JVM、真实两类4重编实际重跑保持，真实目标未执行。

借用进入本轮时在途的enum R1构建：9命令成功，2,640来源/资产及归档、67二进制匹配，target清理；不是67个目标已运行。CLI `/private/tmp/jarde-s1-enum-raw-product-r1-bin-20261023/cli-jarde-cli`，SHA `6a6f2bbdf143ff64704a155edcdfe03604aef5e6048e136b201ffde039623c73`。构建86287、验收53086、finalize64133均已正常exit0，不再轮询或重启。本次指定2.6已完成，OpenSpec17/59，2.9/2.10/S1未完成；已有enum草稿仍未取得产品验收，不继续扩展，S2未进入。工作区未提交、推送或归档。

## 历史在途：S1 / 2.9 enum raw 名称（产品仍未验收）

[嵌套enum上下文复核](openspec/changes/restructure-demand-driven-recovery/results/s1-nested-enum-context-v1/README.md)关闭先前未闭合的可到达假设：terminal body Some在当前Engine唯一生产点必带delegation与constructor field index，nested render在二次emit前拒绝；正式8输入的实际拒绝一致，两个javac23物理家族重编和4次JVM保持。248个pins匹配，历史harness/空stdout核验错误已保留并分列，只有closure-v3作正式结论。不新增行表或放宽委托域；死分支清理归2.10。沿链发现活跃enum名称连接的11类非BMP失败，进入[enum raw名称实施](openspec/changes/restructure-demand-driven-recovery/results/s1-enum-raw-names-v1/README.md)，当前旧源码四测试1通过/3失败，产品未验收；2.9/2.10/S1不勾选，S2未进入。

修正红基线49346、旧矩阵v2 99931、enum产品构建86287均已正常终态，不再轮询或重启；v2旧CLI为132格/40投影/80重编成功/160生成JVM一致。产品R1的9条构建完成、target清理，仅借给本次2.6验收。2.9产品矩阵/负例/evidence及`_` Class边界仍未验收，不自动继续。S2未进入。

## 当前接续：S1 / 2.9 Body 类型上下文（2026-10-23，局部验收）

[Body 类型上下文检查点](openspec/changes/restructure-demand-driven-recovery/results/s1-body-type-names-v1/README.md)将完整 raw InnerClasses rows 沿 MethodDeclaration/Java facts/Declaration/retained AST 共享，commit/replay/fragment 复用同一 SourceClassNames，删除成员名列表双轨；具名边核验后投影，local/anonymous/无行保留物理拼写。两项永久反例0/2→2/2，60开发输入重编48/120→120/120、240隔离JVM精确匹配；65目标1897 passed / 8既有failed / 10ignored，R3两项匿名回归已在R4恢复且原流保留。300绑定/60raw与map/120额外evidence通过，既有泛型、Signature、metadata与136×3/真实两类4重编保持，真实包未执行。2,624源码/资产和66二进制匹配，target清理；schema12/恢复23/解析4/inputs v9。v1原流遗失与v2历史generator pin失配已单列，不冒充全部输入历史可核验。嵌套enum child-context待实际复现，逐方法解析/资源总门归2.10；2.9/2.10/S1不勾选，S2未进入。

本轮持续目标接续既定S1重构。R4构建3611已正常exit0；首验收64119在检查器处exit1，首次离线补验亦exit1，最终补跑32691 exit0。原流均保留，不再轮询或重启旧句柄。CLI `/private/tmp/jarde-s1-body-names-product-r4-bin-20261023/cli-jarde-cli`，SHA `f3cd70a4fd830f076654d43c6970b827e6281ce8fcb500de0dbd35b2713e1394`。下一步先复现next-boundaries的嵌套enum外部helper类型上下文，再闭合其余2.9/2.10；不进入S2。OpenSpec17/59，工作区未提交。

## 历史单轮范围：S1 / 2.6 当前源码验收（2026-10-23，v20）

[v20 验收](openspec/changes/restructure-demand-driven-recovery/results/s1-environment-recheck-v20/README.md)实际重现固定旧构建 2 个 loader / 7 个身份失败；当前最终构建 A1.4 全部 11 项及相邻回归 **288 passed / 0 failed / 1 既有 ignored**。环境/loader/cache 机制未发现新增可复现缺口；已有 2.9 Body 命名迁移造成两个 E0061，首次构建失败原流保留，只适配两个旧 emitter 调用，另 6 项既有 enum 回归通过，未扩大恢复规则或改测试体。R2 三构建成功，678 源码/资产与 7 二进制匹配，target 清理；117 个 include 输入补充归档如实标记编译后/测试前采集，关键 A1.4 输入完整字节与实际测试二进制匹配。

本轮实际重跑 136×3 正文/核心、6 完整类编译/12 受控 JVM、真实成功两类 4 重编全部保持，未执行真实目标。构建 70819、验收 84560 都已正常 exit0，不再轮询/重启；初次构建 10719 exit1 亦保留。CLI `/private/tmp/jarde-s1-environment-v20-bin-20261023/cli-jarde-cli`。当前用户范围止于 **2.6**，已有 Body 命名迁移保持未验收，不自动继续 2.9/2.10，S1 总门未完成，S2 未进入。OpenSpec 17/59；工作区未提交。

## 历史接续：S1 / 2.9 泛型物理名称（2026-10-23，局部验收）

后继 [泛型物理名称检查点](openspec/changes/restructure-demand-driven-recovery/results/s1-generic-physical-names-v1/README.md)让同类调用key/库存保留raw三元组，field/method词法及六处AST连接共享严格MUTF标识符出口，constructor规范owner连接保留同次SSA/完整使用/停止守卫。五生产文件，无新增读取/resolver/cache/Session/crate或序列化字段，恢复合同22，schema11/解析3/inputs v8。五项同源码永久反例0/5→5/5；60输入泛型反射40/60→60/60，120重编/240正常运行均保持，收益为元数据恢复。61目标1869 passed / 8既有failed / 10ignored，其中补入三个既有generic目标49项全通过；45evidence、60raw/body/map、108套绑定预期更新及完整报告差异门通过。六负例保留raw/显式别名/泛型拒绝或准确Partial/Failed前缀，未执行变异输入。Signature96编译192JVM、metadata40/80及112/224、78源码/24状态/四预算、136×3与真实两类4重编保持，真实目标未执行；field-write/setter两组R2已绿控制48编译96JVM保持。674来源/资产、62二进制和八成功构建匹配，target清理，10122前继载荷保持；准确同字节CLI复用已实际运行的60组矩阵，无重复计数。R1 check/R2 native及R3检查器误判Partial的失败原流保留，补跑12门后覆盖18门，不称首次全绿。后续Body类型上下文与2.10总门承接，2.9/2.10/S1仍不勾选，S2未进入。

持续目标已接续既定S1重构；下方v19止于2.6是此前单轮范围。R3构建68534八步exit0，初验39847在负向检查器处exit1；修正准确Partial/Failed边界后补跑30821十二门exit0，均不再轮询或重启。CLI `/private/tmp/jarde-s1-generic-physical-product-r3-bin-20261023/cli-jarde-cli`。下一步先按next-boundaries复现Body命名上下文，再闭合剩余raw消费者与2.10；当前OpenSpec17/59，不进入S2。工作区未提交，整体目标继续。

## 历史单轮范围：S1 / 2.6 当前源码验收（2026-10-21，v19）

本次指定 2.6 的 [v19 当前源码验收](openspec/changes/restructure-demand-driven-recovery/results/s1-environment-recheck-v19/README.md)先实际重现固定旧构建 2 个 loader / 7 个身份失败；当前源码新构建 A1.4 全 11 项及相邻回归 **288 passed / 0 failed / 1 既有 ignored**。四个环境/loader/facade/身份测试文件与 v18 归档逐字节一致，未发现新增可复现 2.6 缺口，本轮未改生产代码或测试体。624源码/资产、归档及7二进制匹配三成功构建，target已清理；136×3正文/核心、6编译/12受控JVM、真实成功两类4重编实际重跑保持，未执行真实目标。前继Signature 10122冻结载荷核验；既有2.9 generic physical overlay的R2新测试4/5及后续未验收改动保留，不授予2.9通过。本轮止于2.6，2.9–2.10/S1未完成，S2未进入。

当时该轮仅复核2.6；随后持续目标已按上方R3接续2.9。本段记录当时终态。定向构建51477和验收62746均已正常终态exit0，无需重启。当前CLI `/private/tmp/jarde-s1-environment-v19-bin-20261021/cli-jarde-cli`。2.9 R2构建90310及验收26106亦已终态；R3未启动，后续六处连接改动仍未授予恢复验收。OpenSpec17/59，工作区未提交。

## 历史未验收改动：S1 / 2.9 泛型物理名称与 owner（2026-10-21，已由上方R3承接）

持续目标已进入 `openspec/changes/restructure-demand-driven-recovery/results/s1-generic-physical-names-v1`。60 输入原编译/120 原 JVM 已固定；旧 CLI 120/120 重编、240/240 运行成功，但完整泛型反射仅 40/60。五项永久测试在旧源码新构建为 0 passed / 5 failed，624 源码/资产已固定。当前五生产文件把 field/method 词法门、五处 InitRecord owner join、prologue this/super 和同类调用库存迁到现有严格命名/raw CallTarget 决定，恢复合同22；**当前产品未验收**。

R1 check 因删除旧 this_class 临时变量后遗漏一个 field-write 引用而 exit101，target 已清理、没有产品二进制；修为直接 raw owner 比较。R2 构建 session **90310** 已正常exit0，七构建与target清理完成。验收26106在native首门因新测试4/5而exit1，原流保留，不能宣称通过；后续六处名称连接已修正但R3未启动。最新用户范围为2.6，当前暂停2.9实施及矩阵；不再轮询或重启这些旧句柄。基线构建80060 exit0已清理，baseline-tests exit101为预期0/5红例。

初次编辑脚本 Unicode 字面量编码异常曾截断 init.rs，已在任何产品构建前从 baseline 精确恢复并重应用；implementation-r2.patch只对应已构建的R2阶段；后续六处连接的当前源码由v19 current-sources归档固定，尚未取得2.9恢复验收。原流与edit-attempt.json保留。独立静态审查确认完整性/SSA/停止守卫保持；该结论不是运行验收。Body `$` 命名上下文另有只读定位，未实施。2.9–2.10/S1未完成，OpenSpec17/59，S2未进入。下方已验收 Signature 检查点是前继，不代表当前改动通过。

## 当前接续：S1 / 2.9 Signature raw 名称（2026-10-21，局部验收）

后继 [Signature raw 名称检查点](openspec/changes/restructure-demand-driven-recovery/results/s1-raw-signature-names-v1/README.md)将递归 Signature 拼写及 body TypeVariable 出口迁入现有 Java 模块，全部类型变量声明/引用共享严格 MUTF-8 标识符决定；root 保留 raw scope/erasure、成员事实、停止与类事务。四生产文件，无新读取/resolver/cache/crate；恢复合同21，schema11/解析3/inputs v8。四项同源码永久反例0/4→4/4；48同源javac输入泛型32/48→48/48、反射行为128/192→192/192，96/96编译保持，不称编译净收益或holdout。57目标1813 passed / 8既有failed / 10ignored；36 evidence、48 raw/body/map、完整报告预期差异及逻辑Header/Body零增量通过，12负例拒绝泛型/损坏Failed且未执行。旧metadata112编译224JVM、20类型路径40批次80JVM、78完整源码、24状态/四预算、136×3及真实两类4重编保持，真实包未执行。621来源/资产、58二进制与七成功构建匹配，baseline/product target清理，8405前继载荷核验。首负向门因验证器误要求损坏输入exit0失败，原流保留并独立核对exit4后补跑八门，不称首次全绿。后续物理field/method名、constructor/private owner、Body类型上下文及2.10门沿[next-boundaries](openspec/changes/restructure-demand-driven-recovery/results/s1-raw-signature-names-v1/next-boundaries.md)承接；2.9–2.10/S1不勾选，S2未进入。

本轮按持续目标恢复既定 S1 架构重构；下方 2.6 “本轮止于”是此前单轮范围。Signature R1 构建73847已正常终态exit0；本轮复用该确切构建，不重启。native77333 exit0（保留八已知失败），初流水线89750在负向检查器处exit1，补跑63883八门exit0，独立pin56166 exit0；旧句柄不再轮询。当前CLI `/private/tmp/jarde-s1-signature-names-product-r1-bin-20261021/cli-jarde-cli`、SHA `171c6d1198a2ab243e32b51942712809ee1add004d8ad363113965764f1e0454`。下一步先复现泛型投影的物理field/method名、constructor/private owner raw消费，再迁移共享决定；Body命名上下文与2.10总门仍待处理，不进入S2。工作区未提交，整体目标继续。

## 本次指定范围：S1 / 2.6 当前源码验收（2026-10-21，v18）

本次指定 2.6 的 [v18 当前源码验收](openspec/changes/restructure-demand-driven-recovery/results/s1-environment-recheck-v18/README.md)实际重现旧构建 2 个 loader / 7 个身份失败；当前 A1.4 全 11 项及相邻回归 **288 passed / 0 failed / 1 既有 ignored**。三处 memo、defining loader 后继、caller 快捷路径及取消热命中保持，本轮无生产代码或测试体修改。进入本轮时在途的 Signature R1 正常完成，621 源码/资产、58 二进制与七成功构建匹配，target 清理；仅借该构建验收 2.6，未执行 Signature 产品矩阵或追认 2.9。136×3 正文/核心、6 编译/12 受控 JVM、真实两类 4 重编实际重跑保持，8405 前继载荷核验，真实目标未执行。本轮止于 2.6；2.9–2.10/S1 未验收，S2 未进入。

本次明确范围覆盖下方历史接续建议，不继续 2.9 实现。进入本轮时已启动的 Signature R1（session 73847）七构建完成 exit0、target 缺席；本轮只运行 A1.4 与保留门，native 76212、recovery/pins 82908 均 exit0，不再轮询或重启。CLI `/private/tmp/jarde-s1-signature-names-product-r1-bin-20261021/cli-jarde-cli`。Signature 的四个旧反例及48输入 baseline 已保存，但当前产品新增测试/矩阵/evidence/negative 未执行，仍属未验收 overlay；对应目录 `results/s1-raw-signature-names-v1`。本轮没有修改生产源码或测试体，没有授予该改动或 S1 整体验收。

## 当前接续：S1 / 2.9 metadata 类型路径（2026-10-21，局部验收）

后继 [metadata 类型路径检查点](openspec/changes/restructure-demand-driven-recovery/results/s1-metadata-type-paths-v1/README.md)将已存在 ResolvedInnerClass 迁入 Java naming，借用同次 raw rows 为普通 descriptor 声明、注解使用及递归默认值提供统一路径；root 只读取 raw default、供给上下文并保留停止/隔离/类族事务。五生产文件，无新增resolver/read/cache/Session/crate；恢复合同20、schema11/解析3/inputs v8。20同源输入的40完整编译批次0/40→40/40（80 source units）、80生成JVM一致；保留真实 `$`、深层及supplementary边界，40 raw报告/30 evidence对照保持，60处type annotation spelling逐项验证。两root旧反例转绿，另两Java unit；新重复/矛盾row拒绝与BodyProbe逻辑Body4→3通过，不声称actual IR。R5七构建成功，56目标1807 passed / 8既有failed / 10ignored；78完整类文本、24旧状态/四预算、旧metadata112编译/224JVM及42 evidence、136×3与真实两类4重编保持，真实包未执行。618源码/资产、57二进制匹配，六target缺席，5079前继载荷核验。R1/R2错误、R3新增失败、R4中断及验证器错误原流保留；R5首流水线因raw/spelling DTO比较退出，修正后补跑七门并独立验收十门，不称首次全绿。Signature/generic、Body类型上下文、constructor/private owner及2.10总门继续按[next-boundaries](openspec/changes/restructure-demand-driven-recovery/results/s1-metadata-type-paths-v1/next-boundaries.md)承接，2.9–2.10/S1不勾选，S2未进入。

本轮按持续目标恢复既定 S1 架构重构；下方 2.6 “本次止于”是此前单轮范围。当前 R5 构建 session 56844 exit0；初次验收9284因比较器exit1，补跑42491七门exit0，独立pin58823 exit0。旧session不再轮询或重启；当前CLI `/private/tmp/jarde-s1-metadata-types-product-r5-bin-20261021/cli-jarde-cli`，工作区未提交。

下一步先复现 Signature 类型变量/字段/方法的 supplementary raw 消费问题，再迁移共同源码决定；完整职责、actual读IR/停止/资源门仍由2.10承接。不得跳至S2。

## 本次指定范围：S1 / 2.6 当前源码验收（2026-10-21，v17）

本次指定 2.6 的 [v17 当前源码验收](openspec/changes/restructure-demand-driven-recovery/results/s1-environment-recheck-v17/README.md)实际重现旧构建 2 个 loader / 7 个身份失败；当前新构建 A1.4 全 11 项及相邻回归 **288 passed / 0 failed / 1 既有 ignored**。当前 618 项源码/资产与归档、三成功构建命令和 7 二进制匹配，target 清理；136×3 正文/核心零差异、6 完整类编译/12 受控 JVM、真实成功两类 4 重编全部实际重跑通过，未执行真实目标。5079 前继冻结载荷校验通过。既有 2.9 overlay 的 R4 构建中断、无 receipt，未重启或追认通过，仅清理废弃 target；本轮未改生产源码或测试体。2.6 保持通过；本轮止于 2.6，2.9–2.10/S1 未验收，S2 未进入。

本次明确范围覆盖下方历史接续建议；不继续 2.9 实现。新构建 session 8058 三步 exit 0；验收 session 74718 三门 exit 0；两个 target 均缺席。CLI `/private/tmp/jarde-s1-environment-v17-bin-20261021/cli-jarde-cli`。2.9 的 metadata type paths R4 是中断检查点，不能沿用 R3 结果宣称当前 2.9 通过。

## 当前接续：S1 / 2.9 metadata 缺省与拒绝状态（2026-10-21，局部验收）

后继 [metadata 拒绝状态检查点](openspec/changes/restructure-demand-driven-recovery/results/s1-metadata-refusal-v1/README.md)让 Java 的已存在值 resolver 返回 Result<MemberDefault>，root None 仅表示属性缺失；方法复用 Refused，字段复用标记/诊断并隔离局部错误，全局预算/取消仍截断。四生产文件，无新 DTO/Session/cache/crate；恢复合同 19，schema11/解析3/inputs v8。相同测试源新两项旧0/2→当前2/2，原10项保持；24开发输入的16个错误Complete改为Partial，8损坏仍Failed且保留邻接成员。56目标1803 passed / 8既有failed / 10ignored，A1.4全11、既有metadata112编译批次/224JVM、42evidence与112raw/body-map、136×3、6编译12受控JVM及真实两类4重编保持，未执行真实目标。616源码/资产、57二进制与七成功构建匹配，三个target清理；四紧预算探针通过，只声称逻辑Body。首次测试API错误、状态验证器摘要算法假设及预算成本假设原流保留，纠正后六门独立验收，不称首次流水线全绿。2.9–2.10/S1仍未验收，S2未进入。

- 构建终态：初次 baseline61599 因新增测试API错误exit1；修正后baseline94260及其10/2红测试结束；product98659七步exit0。初次验收44187在状态验证器假设处exit1；补跑88648四门exit0，四有效预算探针与独立finalize通过。旧session不再轮询/重启。
- 下一步按后继边界复现真实nested source path及generic/constructor/private owner raw消费，再推进2.10职责/actual读IR/停止/资源门。下方metadata Option盲区属于旧检查点，本轮已闭合；不进入S2。
- CLI `/private/tmp/jarde-s1-metadata-refusal-product-r1-bin-20261021/cli-jarde-cli`；工作区保留未提交改动，整体目标仍继续。

## 本轮指定范围：S1 / 2.6 当前构建复核（2026-10-21，v15）

本次指定 2.6 的 [v15 当前构建复核](openspec/changes/restructure-demand-driven-recovery/results/s1-environment-recheck-v15/README.md)实际重现旧构建 2 个 loader / 7 个环境身份失败；当前 A1.4 全 11 项和相邻回归 **288 passed / 0 failed / 1 既有 ignored**。完整环境声明、defining loader 后继、三处缓存/caller 快捷路径及热命中取消保持，无新增 2.6 产品修改。613 项源码/资产及 57 二进制匹配 metadata R2，核验 8,452 个冻结 payload；target 缺席，本轮无新增 Cargo。实际重跑 136×3 正文/核心、6 编译/12 受控 JVM、真实成功两类 4 重编均保持，未执行真实目标。首次旧脚本数量断言失败与修正后完整复跑分别保留。2.6 保持通过，2.9–2.10/S1 未验收，本轮止于 2.6，S2 未进入。

下方 2.9 是已有工作区检查点，保留其未完成状态；本次用户指定范围为 2.6，不继续其实现。

## 当前接续：S1 / 2.9 metadata字面量职责与raw边界（2026-10-21，局部验收）

- [结果](openspec/changes/restructure-demand-driven-recovery/results/s1-raw-metadata-literals-v1/README.md)：原MemberDefault/ConstantValue/default及注解元素语义移Java，root保留同一属性读取、预算/停止、成员隔离与类组装；复用JvmString/现有转义表保留完整UTF-16，名称另走严格identifier门；损坏编码Err明确传播。六生产文件，schema11/恢复合同18/解析合同3/inputs v8。
- 同源码/资产新永久测试0/2→2/2，旧8项保持。56输入×Marker/Subject双目标，编译96/112批次→112/112（224完整source units），行为32/56→56/56、224生成JVM一致；反射字段避免常量内联。损坏MUTF的NoBody Marker由错误Complete/替换字符改为明确Failed；损坏目标未执行，合法不可表达enum名raw属性保留且不输出非法token。
- 56目标 **1801 passed / 8既有failed / 10默认ignored**；42evidence、112份raw属性/物理来源与112普通body-map保持，136×3、6编译12受控JVM和真实两类4重编保持，未执行真实包。Marker零Body与Header/Body计费保持是逻辑usage，不冒充actual IR观测；其他旧raw专项未全量重跑。
- baseline构建86717/定向90810均结束；R1构建97542因缺char_literal导入exit1且target清理，R2修复导入及generic reexport后51246七步exit0。613源码/资产、57二进制匹配，三个target清理，峰值974082806 bytes。验收25666六门exit0，finalize75372已exit0；均不再轮询/重启。CLI `/private/tmp/jarde-s1-raw-metadata-product-r2-bin-20261021/cli-jarde-cli`。
- **下一步优先闭合确认的metadata拒绝状态**：外层Option仍混淆无AnnotationDefault与有属性但源码不可表达，helper拒绝不代表完整类语义正确。按[后续边界](openspec/changes/restructure-demand-driven-recovery/results/s1-raw-metadata-literals-v1/next-boundaries.md)保留raw/成员来源、明确拒绝及邻接隔离，再处理Signature/generic、constructor/nesting与private owner；不跳去S2。
- OpenSpec **17/59**，2.9–2.10/S1未验收；八失败、取消skipped、资源/E1/E3门保留。未提交、推送或归档。

## 当前接续：S1 / 2.9 interface-super raw 身份（2026-10-21，局部验收）

- [结果](openspec/changes/restructure-demand-driven-recovery/results/s1-raw-interface-super-v1/README.md)：删除String候选，raw owner/name/descriptor交接至Java精确匹配；共用canonical类名/严格MUTF-8/完整descriptor门，不保留双轨，原default/层级/loader/entry-this/停止守卫保持。六生产文件；schema11/恢复合同17/解析合同3/inputs v8。
- 同源码/资产永久反例0/2→2/2；56同源开发输入qualified-super32→56、完整类编译64/112→112/112、224生成JVM一致；CP raw目标/descriptor/UTF-8来源全部核对。bad-name从错误Structured改为raw/alias/BCI保留的拒绝；surrogate和损坏MUTF拒绝保持，损坏输入未执行，不称holdout。
- 55目标 **1790 passed / 8既有failed / 10默认ignored**，仅special-dispatch新增2通过；A1.4全11、24新evidence、80旧Outer编译/160JVM、136×3、6编译/12受控JVM及真实成功两类4重编保持，未执行真实包目标。其余旧raw专项本轮未全部重跑，不冒充新增运行。
- baseline构建session36374和product89911均exit0，606源码/资产及56二进制匹配，七步构建成功、两个target清理、峰值958433241 bytes；CLI `/private/tmp/jarde-s1-raw-interface-super-product-r1-bin-20261021/cli-jarde-cli`。主验收80777在前四门后因比较器混入可选init/declaration/rules详情exit1，原流/原脚本保留；修正可选发布边界后补跑83002已exit0，九覆盖门独立核验。最终收据见acceptance，不再轮询/重启旧session。
- 下一步按[剩余边界](openspec/changes/restructure-demand-driven-recovery/results/s1-raw-interface-super-v1/next-boundaries.md)先复现annotation/default，再核对Signature/generic、constructor/nesting与private owner；facade interface源码策略和实际Body/IR复用仍归2.10总审查。
- OpenSpec **17/59**；2.9–2.10/S1未验收，八既有失败/取消skipped/资源/E1/E3门保留，S2未进入。下方v14止于2.6是此前单轮范围，本轮持续目标已接续既定架构重构。未提交、推送或归档。

## 本次指定范围：S1 / 2.6 当前构建验收（2026-10-21，v14）

- [结果](openspec/changes/restructure-demand-driven-recovery/results/s1-environment-recheck-v14/README.md)：本轮实际重现旧构建2个loader / 7个身份失败；当前A1.4全11项及相邻回归 **286 passed / 0 failed / 1既有ignored**。完整环境身份、defining loader后继、三处缓存及caller快捷路径/热命中取消均保持，无新增已确认2.6缺口，未改产品代码或测试体。
- 本轮实际重跑136×3 evidence正文/核心、6编译/12受控JVM、真实成功两类4重编，全部保持；未执行真实包目标。复用源码职责迁移R1：601源码/资产、源码包、56二进制匹配，10529冻结payload核验通过；target不存在，无新增Cargo。定向session12164和恢复session41031均exit0，不再轮询或重启。
- 本次止于 **2.6**。下方2.9接续仅描述此前范围，不覆盖本次指令；OpenSpec **17/59**，2.9–2.10/S1未验收，八项既有失败及停止/资源/格式门仍由2.10承接，S2未进入。未提交、推送或归档。

## 当前接续：S1 / A1.1 Outer.super 源码职责迁移（2026-10-21，局部验收）

- [当前结果](openspec/changes/restructure-demand-driven-recovery/results/s1-outer-super-source-policy-v1/README.md)：泛型/Exceptions、重载、发布参数/SSA direct-load与严格子类判据从facade移入Java；root保留物理一致性、选定Header/分析供给、原execution合并及原census/类级事务。仅两生产源码变化，无测试体修改或接受域扩大；schema10/恢复合同16/解析合同3/inputs v7保持，源码实现身份实际更新。
- 七步构建成功，55目标 **1788 passed / 8既有failed / 10默认ignored**。40完整输入80编译/160生成JVM保持；40报告及新增36开发输入的42调用（含6入口紧预算）在校验两个确切旧→新实现摘要后，其余字段、逻辑usage、origins/raw spans和退出码保持。部分变体PreparedFold保留低级桥，不能统称直接binding拒绝；旧overload测试提前失败未跑到的变体由独立矩阵补齐。
- 7×3 evidence、A1.4全11、136×3、6编译12受控JVM、真实成功两类4重编及全部旧raw保留矩阵保持，未执行真实目标。601源码/资产、56二进制与R1相符，target清理，峰值958488223 bytes；此前raw Outer.super的12121冻结payload保持。
- 构建session31907已exit0；验证session50511在前16门通过后因比较器遗漏实现摘要迁移而exit1，原流保留，不能报原流水线全绿。补充policy session11793与finalize session12004均已exit0；40完整报告亦实际核验。均不再轮询/重启。CLI `/private/tmp/jarde-s1-outer-super-source-policy-r1-bin-20261021/cli-jarde-cli`，摘要见receipt。
- 下一步按[后续边界](openspec/changes/restructure-demand-driven-recovery/results/s1-outer-super-source-policy-v1/next-boundaries.md)先复现interface-super的raw→普通UTF-8往返，再闭合annotation/default、generic/constructor/nesting；2.10继续盘点所有职责及同次live facts/SSA复用，用实际read/IR核查，本轮不声称资源改善。
- OpenSpec **17/59**；2.9–2.10/S1未验收，八失败与停止/资源/E3门保留，S2未进入。下方v13止于2.6是此前单轮范围，本轮收到持续目标后已接续既定架构迁移。未提交、推送或归档。

## 本次指定范围：S1 / 2.6 当前构建复核（2026-10-21，v13）

- [结果](openspec/changes/restructure-demand-driven-recovery/results/s1-environment-recheck-v13/README.md)：旧构建实际重现2个loader / 7个身份失败；当前A1.4全11项及相邻回归 **286 passed / 0 failed / 1既有ignored**。三处cache、caller IR/prepared快捷路径与热命中取消保持；未发现新增2.6缺口，未改生产代码/测试体。
- 当前601项源码/资产、源码包与56二进制逐项匹配Outer.super R3，12121个冻结payload均通过摘要核验，build target当前不存在；本次复用现有固定构建，无新增Cargo。恢复保留引用同源码/二进制既有oracle，本轮未重跑矩阵/JDK或执行真实目标。
- 本次止于 **2.6**，下方2.9后继方向仅是历史检查点，不覆盖当前范围。OpenSpec **17/59**，2.9–2.10/S1未验收，S2未进入。未提交、推送或归档。

## 当前接续：S1 / 2.9 Outer.super raw 交接（2026-10-21，局部检查点已验证，整项未完成）

- [结果](openspec/changes/restructure-demand-driven-recovery/results/s1-raw-outer-super-v1/README.md)：OuterSuperCallInput目标三元组改JvmBytes；facade直接raw交接/最终needle严格MUTF解码，Java共用完整descriptor/规范类名门并直接规划raw参数/返回。六源码文件，原physical/SSA/BCI/闭合/overload/异常/原子提交守卫保持；schema10/恢复合同16/解析合同3/inputs v7。
- 两项永久同测试源码旧0/2→当前2/2；effects28完整类编译24/56→56/56、112受控生成JVM一致；合法`_`父类4组8编译/16JVM含其中，无专用分支。reference12补齐分析Header后8/24→24/24、48JVM一致，12raw descriptor/物理方法/最终跨度对照通过。均为开发矩阵，不称holdout。
- reference两次harness盲区原流保留：先误查Member，再发现分析JAR缺Parent/Token，后者只是缺Header的probe，不是完整依赖能力基线。最终complete-inputs使用原class字节与已有24条真实原JVM记录，原件字节/pins核对，新生成运行无原Outer/Member后备。
- R3统一55目标 **1788 passed / 8既有failed / 10默认ignored**，A1.4全11保持；7×3 evidence正文/raw projection/span一致，136×3正文/核心、6编译12受控JVM、真实既有成功两类4重编及enum/private/匿名/super/cast/初始化/成员/varargs/数组/名称矩阵均保持，真实目标未执行。
- 601源码/资产、56二进制及7成功构建匹配，四个phase target已清理，R3峰值957765685 bytes。R1 Option类型错误原流、R2七构建/55目标及`_`反例保留，最终完全用R3重建/重跑。构建session36419、验收session23220均已终态，不再轮询或重启。CLI `/private/tmp/jarde-s1-raw-outer-super-product-r3-bin-20261021/cli-jarde-cli`，摘要见receipt。
- 最终文档、OpenSpec严格校验/diff检查与来源核验通过，17步原流已收齐；**12121个payload已冻结并逐项校验**，前序enum的9537个payload也保持。932是独立来源pin核验数，不是freeze payload数。冻结后不修改本phase；如需后继实现须新建下一检查点。
- 下一步优先核对并迁移facade残留的Outer.super纯源码重载/异常判据（已列S1 2.10的A1.1职责承接），供给/停止/事务留root；再按[剩余raw消费者](openspec/changes/restructure-demand-driven-recovery/results/s1-raw-outer-super-v1/remaining-raw-consumers.md)闭合interface-super/annotation/default/generic/constructor/nesting的实际反例。先复现ASCII控制，不进入S2，不按样例名放行。
- OpenSpec **17/59**；2.9–2.10/S1整体仍未验收，八个原失败/资源停止/格式门未关闭。未提交/推送/归档。下方v12“止于2.6”只描述此前单轮范围。

## 此前指定范围：S1 / 2.6 当前源码验收复核（2026-10-21，v12）

- [本轮结果](openspec/changes/restructure-demand-driven-recovery/results/s1-environment-recheck-v12/README.md)：旧构建实际重现 2 个 loader / 7 个身份失败；当前 A1.4 全 11 项与相邻回归 **286 passed / 0 failed / 1 既有 ignored**。三处 cache、caller facts 快捷路径及热命中取消保持，无新增已确认 2.6 缺口，未改生产代码或测试体。
- 601 项源码/资产重新归档构建，六个二进制/两条成功命令与来源匹配，target 已清理，峰值 406305363 bytes。构建 session76816、定向验收 session29747 均已结束，不再轮询/重启。
- 生产源码与 enum-switch R4 相同，仅此前待验收的 Outer.super 两项测试及 12 资产不同；核验 R4 的 9537 个冻结 payload，保留既有恢复 oracle。本轮未重跑矩阵/JDK，未执行真实目标。
- 此前已启动的 Outer.super baseline class_source 构建 session76031 已结束（153.86 秒、exit0、target 清理），只有编译收据，本轮未执行两项新测试或继续其实现。该 baseline 当前包含601源码/资产，后续如获准继续2.9应先验证来源及实际运行两项测试。已有 effects24组和 reference-root12组原流保留；不得把 reference 首次错误查询 Member 的失败计为根类产品失败。
- 本次止于 **2.6**；OpenSpec **17/59**，2.9–2.10/S1 未验收，S2 未进入。下方接续记录只描述此前范围，不覆盖本次指令。未提交、推送或归档。

## 此前接续：S1 / 2.9 enum-switch raw 身份（2026-10-21，检查点已验收，整项未完成）

- [当前结果](openspec/changes/restructure-demand-driven-recovery/results/s1-raw-enum-switch-v1/README.md)：CallTarget/IndexCall→enum owner/descriptor→values field/factory→map proof保留raw，最后严格MUTF拼写label；同次IR的anewarray CP Class核对物理元素，原shape/ordinal/BCI/闭合helper/原子提交守卫保持，恢复合同15。
- 28同源输入投影8→14、原样完整类编译16/56→28/56、56生成JVM一致。12个raw拒绝中6个取得整类恢复、6个仅显露canonical后继阻塞；14组javac8原形状拒绝由S5/6.9承接。两组private values字段/factory metadata由0/2投影、0/4编译→2/2、4/4、8JVM；先有4次原JVM校验，无原目标后备，不称holdout。
- R3测试在45目标处失败原流不动；R4保留来源注释，helper禁用检查限定可执行行并核对raw proof，常量注释复用已验证label。修订后的同一测试源码在旧实现重新构建仍0/2、当前2/2。不能把R3原45目标或失败断言写成全绿。
- R4全部55目标 **1785 passed / 8既有failed / 10默认ignored**；A1.4 11项、enum 4×3 evidence、136×3、匿名/cast/初始化/成员/varargs/数组/名称保留矩阵与真实成功两类4重编保持，真实包未执行。
- 当前589源码/资产、56二进制、7条成功构建匹配；本phase六个target清理，R1/R2中断完整峰值未知。构建session98804、主验证session66697、补充session77805均已结束，不再轮询/重启。CLI为 /private/tmp/jarde-s1-raw-enum-switch-product-r4-bin-20261021/cli-jarde-cli，摘要以receipt为准；schema9/恢复合同15/解析合同3/inputs v6。
- 下一步仍为2.9：[剩余raw边界](openspec/changes/restructure-demand-driven-recovery/results/s1-raw-enum-switch-v1/next-raw-boundaries.md)的Outer.super及annotation/default、generic/constructor等需先实际复现，再按共享身份/拼写边界迁移。八个既有失败和2.10停止/资源/格式门仍未关闭。OpenSpec **17/59**，2.9–2.10/S1未验收，S2未进入。
- v11“止于2.6”是此前单轮范围；收到持续OpenSpec目标后本轮已继续2.9，原冻结收据不改。未提交、推送或归档。

## 本次指定范围：S1 / 2.6 当前来源复核（2026-10-21，v11）

- [结果](openspec/changes/restructure-demand-driven-recovery/results/s1-environment-recheck-v11/README.md)：旧构建重新复现 2 个 loader / 7 个身份失败；当前 A1.4 全 11 项及相邻回归 **286 passed / 0 failed / 1 默认 ignored**。三处 cache 消费、caller 快捷路径与热命中取消均保持，无新增已确认 2.6 缺口，本轮未改生产代码或测试体。
- 当前 589 源码/资产与此前已启动的 enum-switch R3 构建及 56 二进制匹配；七构建命令成功，target 已清理。本轮实际重跑 **136×3** 正文/核心、**6 编译 / 12 受控 JVM**、真实成功两类 **4 重编**，全部保持，真实包未执行；没有再次运行 Cargo。
- 收取此前回归 session59054，45 目标执行后在两项新增 enum-switch 测试失败处终止（1742 passed / 10 failed / 8 ignored，含原八失败），没有运行后续步骤，不能报55目标全通过。其 `fields-class_source` 106旧项通过；新两项拒绝 helper 名/替换字符出现在文本的断言仍失败，不能改预期隐去。此2.9 overlay保留而未验收，本轮不继续修复或metadata补充探针。
- 定向验收 session19742、恢复保留 session76427 均已完成，不再轮询；全部 target 缺席。CLI 为 /private/tmp/jarde-s1-raw-enum-switch-product-r3-bin-20261021/cli-jarde-cli，摘要以receipt为准。schema9/恢复合同15/解析合同3/inputs v6 是当前overlay版本，不计本轮新增。
- 本次止于 2.6；OpenSpec **17/59**，2.9–2.10 与 S1 整体未验收，S2 未进入。下方2.9接续仅为历史记录，不覆盖本次范围。未提交、推送或归档。

## 当前接续：S1 / 2.9 匿名类 raw 身份（2026-10-21，检查点验收，整项未完成）

- [当前结果](openspec/changes/restructure-demand-driven-recovery/results/s1-raw-anonymous-v1/README.md)：已选candidate.class保留到interface/outer/nested/super的typed row、解析、capture/xref与scan；源码拼写复用MUTF边界。selector完整census/唯一verified守卫不变，恢复合同14。
- 三项永久反例0/3→3/3。48同源输入20→48投影、40/96→96/96编译、192JVM一致；super16组2→8内联、4/32→16/32编译、32JVM一致，旧2次CLI panic消失。8组javac8原shape拒绝保留，已交S5/6.9矩阵，不称holdout。
- R1全部55目标 **1783 passed / 8既有failed / 10默认ignored**；新增3测试，另10通过来自扩入两个既有super目标。A1.4 11项、136×3、cast/初始化/成员/varargs/数组/名称与真实成功两类4重编保持，真实包未执行。
- 577源码/资产、56二进制与7成功构建匹配，baseline/product两个target已清理。构建session45982、验收session33506均已结束，不再轮询/重启；最终CLI见 /private/tmp/jarde-s1-raw-anonymous-product-r1-bin-20261021/cli-jarde-cli，摘要以receipt为准。
- 下一步仍为2.9：[剩余raw消费者](openspec/changes/restructure-demand-driven-recovery/results/s1-raw-anonymous-v1/remaining-raw-consumers.md)中的enum-switch owner/constant与Outer.super源码门先实际复现。OpenSpec17/59；2.9–2.10/S1未验收，S2未进入。未提交/推送/归档。
- 下方v10“止于2.6”是此前单轮范围，本轮收到持续OpenSpec目标后已接续2.9；v10原冻结收据与历史结论保持。

## 本次指定范围：S1 / 2.6 当前源码复核（2026-10-21，v10）

- [结果](openspec/changes/restructure-demand-driven-recovery/results/s1-environment-recheck-v10/README.md)：旧构建实际重现 2 个 loader / 7 个身份失败；当前 A1.4 全部 11 项及相邻回归 **286 passed / 0 failed / 1 默认 ignored**。
- 577 项源码/资产重新构建六个目标，来源与归档/二进制匹配，两条命令成功，target 已清理。本轮未改生产代码、测试体或恢复合同，现有完整环境与 loader/cache 修复保持有效。
- 生产源码与 raw-projections R2 逐文件摘要相同，差异仅三个匿名类测试/22 项资产；核验其 6786 个冻结 payload，恢复保留引用已有 oracle。本轮未重跑矩阵/JDK或执行真实目标。
- 本次止于 2.6，下方 2.9 记录仅为此前接续。匿名 baseline 的48组已运行（20投影、40/96编译、80生成JVM一致），三个新永久测试已编译但尚未执行；本轮未继续匿名迁移。baseline构建session84694已结束，target清理收据在 s1-raw-anonymous-v1/baseline-build-r1，不再轮询/重启。
- OpenSpec **17/59**，2.9–2.10/S1 整体未验收，S2 未进入。未提交、推送或归档。

## 当前接续：S1 / 2.9 raw 初始化与字段投影（2026-10-21，检查点已验证，整项未验收）

- [当前结果](openspec/changes/restructure-demand-driven-recovery/results/s1-raw-projections-v1/README.md)：调用target精确定位改用raw，access bridge源码字段名复用字段声明拼写；物理FieldRef/表索引与source_field_name职责分开，原gate/预算/原子提交保持，恢复合同13。
- 两项永久旧失败转绿；28同源输入预期投影12→28、原样完整类编译48/56→56/56、112受控JVM一致。数组只计投影误拒绝修复；8次编译收益来自supplementary字段组，不称holdout。
- R2 **53目标1770 passed / 8既有failed / 10默认ignored**，失败名单不变；新增2测试，另4通过来自扩入既有N1。A1.4的11项、cast/成员/varargs/数组/名称、136×3正文核心和真实两类重编保持，真实包未执行。
- baseline/R1/R2三个target已清理；当前555源码/资产、归档与54个统一R2二进制匹配，七构建命令成功。R1 census失败已用旧reader复现，R2只更新新增两class/八Code的cfg(test)人口并完整重建。构建session2430、验收session52794均已结束，不再轮询或重启。
- 下一步按[剩余raw消费者](openspec/changes/restructure-demand-driven-recovery/results/s1-raw-projections-v1/remaining-raw-consumers.md)优先复现匿名捕获外类名、enum switch owner/constant、Outer.super桥源码绑定；这些仍是静态风险。OpenSpec17/59，2.9–2.10/S1未验收，S2未进入，目标保持active；未提交/推送/归档。

## 本次指定范围：S1 / 2.6 复核与测试补齐（2026-10-21，v9）

- [验收结果](openspec/changes/restructure-demand-driven-recovery/results/s1-environment-recheck-v9/README.md)：旧构建实际复现 2 个 loader / 7 个身份失败；当前 A1.4 11 项及相邻回归 **286 passed / 0 failed / 1 默认 ignored**。
- 发现 bridge admission 缓存此前只有静态核验，本轮在既有 cfg(test) 测试中增加直接调用，覆盖跨 loader 同物理定义、准备 facts 快捷路径、零 Header 热命中与取消热命中。未修改生产代码或恢复规则。
- 555 项源码/资产与本轮最终构建、六个二进制匹配；两条串行构建成功，中断重建有独立收据，两个 target 已清理。恢复保留引用生产代码相同的 raw-casts R3 oracle并核验 5737 个冻结 payload；本轮未重跑恢复矩阵/JDK，未执行真实包。
- 本次止于 2.6。进入本轮前的两个 2.9 测试/六项资产保留但未继续推进；未运行全工作区测试。OpenSpec 17/59，2.9–2.10 与 S1 整体未验收，S2 未进入。下方 2.9 下一步仅为此前接续记录。

## 当前接续：S1 / 2.9 cast/type-test 与 bridge raw 身份（2026-10-21，整项未验收）

- [当前检查点](openspec/changes/restructure-demand-driven-recovery/results/s1-raw-casts-v1/README.md)贯通 CP 类型操作、BridgeParameterCast、canonical/recheck 与 bridge 目标声明定位的 raw 字节；源码拼写在边界严格进行，保留整体描述符门与对象类型 bridge 域，无新事实层或类型推断。
- 永久整类反例实际由 0/1 转为通过；24 同源输入完整类编译 40/48→48/48、96 JVM 和 24 raw descriptor 对照通过。包名旧可编译但未证明的泛型 Header/bridge projection 也修正，不重复计编译收益。
- **52 目标 1764 passed / 8 基线 failed / 10 默认 ignored**，全部用本 R3 二进制重跑；新增 3 Java + 1 整类测试，另 5 passed/1 ignored 是本次纳入的既有引用 cast 目标。A1.4 11 项、136×3、原成员/数组/varargs/名称及真实两类重编保持，未执行真实包。
- 549 项源码/资产与 R3 归档/构建及 53 二进制匹配，7 构建命令成功，四个阶段 target 均清理。R1/R2 主动中断与责任边界修正保留；schema 9/恢复合同 12/解析合同 3/输入域 v6，迁移的类型不序列化。
- 下一步仍是 **2.9 raw 方法跨度与静态初始化等剩余精确交接**。2.10 原失败/停止/资源/E3 门未完成，OpenSpec **17/59**，S1 未验收，S2 未进入，目标保持 active；未提交、推送或归档。

## 此前指定范围：S1 / 2.6 当前工作区复核（2026-10-21，v8）

- [本次结果](openspec/changes/restructure-demand-driven-recovery/results/s1-environment-recheck-v8/README.md)：旧构建实际复现 2 个 loader / 7 个身份失败，当前 A1.4 11 项与相邻回归合计 286 passed / 0 failed / 1 默认 ignored。只读对抗复核未见新增 2.6 产品缺口，无产品修改。
- 当前 545 项源码/测试资产匹配 raw-member R4 归档/构建；根层 R4 与同源码下层 JVM R1 分开绑定，target 仍不存在。核验原检查点 8,382 个证据 payload；恢复保留引用该同构建既有 oracle，本轮未重跑 Cargo、恢复矩阵或 JDK。
- 本次止于用户指定的 2.6。OpenSpec 17/59，2.9–2.10 和 S1 整体未验收，S2 未进入。下方 2.9 下一步仅为此前接续记录，本轮不推进。

## 此前接续：S1 / 2.9 allocation 与成员路径（2026-10-21，整项未验收）

- [当前检查点](openspec/changes/restructure-demand-driven-recovery/results/s1-raw-member-paths-v1/README.md)将 CP→Allocate/Site/NewRecord/census→成员目标/source path→声明投影的物理字段保留 raw；源码拼写复用严格 MUTF-8 名称门，无新事实层、Session、类型推断或 BCI/cast 放行。
- 两项旧产品实际失败的永久测试现已通过；32 组完整类编译 16/64→R1 56/64→R4 64/64，生成类 128 JVM、32 raw class 对照通过。R1 supplementary package 声明拼写漏点及修正原流保留；同源变体不是 holdout，无原 Use.class 后备。
- **51 目标 1755 passed / 8 基线 failed / 9 默认 ignored**：R4 42 根层重跑，R1 9 下层的源码/依赖输入完全相同而复用，不重复累计。A1.4 全 11 项、136×3、旧数组 24 编译/48 JVM、21 名称与真实两类 4 重编保持，未执行真实包。
- 当前 **545 项**源码/测试资产绑定 R4；43 个 R4 二进制和 9 个 R1 下层二进制分别有来源/执行收据。R1 七条、R2 五条与 R4 五条完整构建命令成功，11 个本阶段 target 均清理。schema 9/恢复合同 11/解析合同 3/输入域 v6；NewRecord.class JSON 改为 byte array，独立报告格式合同由 E3/2.10 承接。
- 下一步仍为 **2.9 Bridge cast 及其余精确 raw 消费者**；2.10 原失败/停止/资源门未完成。OpenSpec **17/59**，S1 未验收，S2 未进入，持续目标 active；未提交、推送或归档。下方为各历史检查点。

## 此前接续：S1 / 2.9 raw 数组链路（2026-10-21，整项未验收）

- [本检查点](openspec/changes/restructure-demand-driven-recovery/results/s1-raw-arrays-v1/README.md)将 class literal、anewarray/multianewarray 和 Named array 两处识别改为 raw descriptor/object leaf 消费；保留 nested literal、`_` 类型名、opcode/rank 和 malformed 拒绝规则，无新事实层或类型推断。
- 先实际复现 Java 572 passed / 3 新增 failed，当前同套测试 575 全通过。12 组同源输入原样完整类编译由 8/24 到 24/24，生成类 48 次 JVM 对照通过；不是独立 holdout，没有原目标 fallback。首次 JDK8 Unicode classpath 目录问题与后继 ASCII 目录标签的工具修正分开保留。
- 当前 **41 目标 1709 passed / 8 基线 failed / 8 默认 ignored**；A1.4 环境/loader/cache 11 项全部保持。136×3 正文/核心、6 编译/12 受控 JVM、21 名称 13 编译/26 JVM、真实两类 4 重编通过，未执行真实包。原八失败、四合法根类失败及四损坏拒绝不掩盖。
- **534 Rust/manifest + 3 embedded test 资产共 537 项**与 R1 源码包、构建及全部 42 二进制匹配；6 命令退出 0，target 峰值 942704507 bytes，两个本阶段 target 均清理。schema 8/恢复合同 10/解析合同 3/输入域 v5 保持，Java 实现摘要随代码变化。
- 下一步仍为 **2.9 member-inner enclosing 身份、bridge cast 和其余精确 raw 交接**；2.10 旧失败/停止/资源门未完成。OpenSpec **17/59**，2.9–2.10/S1 整体未验收，S2 未进入；持续目标 active，未提交、推送或归档。下方仅描述各自历史检查点。

## 此前接续：S1 / 2.9 共享源码名称边界（2026-10-21，整项未验收）

- [本检查点](openspec/changes/restructure-demand-driven-recovery/results/s1-source-name-boundary-v1/README.md)将类/包/descriptor/owner 拼写统一到严格 MUTF-8 与冻结 Java 8 Character 门；修正 Frame/catch 的 lossy 类型消费。producer census 使用 raw descriptor，保留数组、Lfoo、字面 `$` 与 7/8 的 `_` 类型名，无新产品依赖或 JVM 运行依赖。
- 17 空类由旧产品 7 组非法/身份不保真输出、2 组合法包名拒绝，变为 8 组双编译器通过、9 组明确拒绝。supplementary catch 由 0/2 到 2/2 编译、4 JVM 对照；不可表达字段 owner 不再发布 Structured 非法表达式，而保留 raw 字段及 Mixed/Fallback/ExplanationOnly。原失败、测试迁移和 runner 取证位置错误均保存。
- 当前 R2 **41 测试目标 1706 passed / 8 基线 failed / 8 默认 ignored**；A1.4 全 11 项、136×3 正文/核心、6 编译/12 受控 JVM、真实两类 4 重编保持。21 名称仍 13 编译/26 JVM、字段四组 4 编译/8 JVM、参数六组 18 编译/36 JVM、alias 3×3 保持。未执行真实包，不宣称完整公开报告迁移、反射或 ABI 等价。
- 当前 **534 项 Rust/manifest** 与 product-sources-r2、构建及全部 42 二进制匹配；target 已删除。schema 8、恢复合同 10、解析合同 3、输入域 v5；下方版本/源码数量只描述各自冻结检查点。
- 下一步仍按 **2.9 的共享 raw 链路**推进：先 class literal/array creation 及 Named array 识别，随后 member-inner/bridge 精确交接，禁止把 display 重新编码成身份。2.10 旧失败/停止/资源门保持未完成，OpenSpec **17/59**，S2 未进入；持续目标 active，未提交、推送或归档。

## 此前指定范围：S1 / 2.6 当前工作区复核（2026-10-21，v7）

- [本轮结果](openspec/changes/restructure-demand-driven-recovery/results/s1-environment-recheck-v7/README.md)：修复前实际复现 2 个 loader / 7 个环境身份失败，当前 A1.4 的 11 项及相邻回归共 286 passed / 0 failed / 1 默认 ignored。完整环境、defining loader 后继、两个 initiating-loader/raw-name memo 与 defining-loader/physical 节点保持；未发现新增 2.6 修复项，本轮没有新增产品修改。
- 接续完成已启动构建；当前 **534 项** Rust/manifest 与 `s1-source-name-boundary-v1/product-sources-r1`、最终构建及全部 42 个二进制匹配。6 条构建命令退出 0，target 已删除。本轮实际重跑 **136×3** 正文/核心决定、**6 编译/12 受控 JVM**、真实两类 **4 重编**，全部保持；未执行真实包，不声称完整公开报告迁移或全工作区测试全绿。
- 进入本轮时已有的名称边界 overlay 保留，但本轮未继续其消费者修复或专项验收；`s1-source-name-boundary-v1` 的构建成功不等于 2.9 已完成。当前 schema 8、恢复合同 10、解析合同 3、输入域 v5，仅描述本构建；下方 532 项/恢复合同 9 是此前 raw-alias 检查点。
- 本次止于指定的 2.6。OpenSpec **17/59**，2.9–2.10 与 S1 整体未验收，S2 未进入；未提交、推送或归档。下方保留此前工作记录及待办。

## 此前接续：S1 / 2.9 raw 局部名与 alias evidence 已验证（2026-10-21，整项未验收）

- [本检查点](openspec/changes/restructure-demand-driven-recovery/results/s1-raw-alias-v1/README.md)贯通 raw LVT/MethodParameters、DebugLocal/SlotEvidence/RenderedName 和 reuse，NameTable 统一最终源码拼写；NameDetails 发布结构化 raw 证据，bulk 权重计入真实缓冲容量。两个 surrogate 旧证据/输入摘要碰撞转绿，不冒称旧完整物理 binding 合并。
- 3×3 alias 查询、6 参数组 18 编译/36 受控 JVM；21 冻结名称仍 13 编译/26 JVM，4 合法根类失败、4 损坏拒绝；136×3、6 编译/12 JVM、真实两类 4 重编与字段 4 编译/8 JVM 保持。未执行真实目标，不声称 reflection/ABI 或完整公开报告迁移通过。
- 40 测试目标 1697 passed / 8 基线 failed / 8 默认 ignored；A1.4 全 11 项保持。R3 产品构建与 R4 单一 tests/class_source.rs 后继/reader 分开绑定，当前 532 源码与 R4 对应；R2 编译遗漏、R3 两项旧丢名断言失败和后继加强编译的迁移原流保留，所有本轮 target 清理。
- schema 8、恢复合同 9、解析合同 3、输入域 v5。下一步继续 2.9 的其余精确消费者和 simple-name 语法；2.10 停止/旧失败/资源门未完成，OpenSpec 17/59，S2 未进入，持续目标 active。未提交、推送或归档。下方为历史记录。

## 本次指定范围：S1 / 2.6 当前构建复核（2026-10-21，v6）

- [复核结果](openspec/changes/restructure-demand-driven-recovery/results/s1-environment-recheck-v6/README.md)：旧构建实际复现 2 个 loader / 7 个身份失败；当前 A1.4 全部 11 项通过，相邻聚焦回归 286 passed / 0 failed / 1 默认 ignored。未发现需要追加的 2.6 产品修复，本轮无产品修改。
- 当前 532 项源码与 raw-class-names R4 构建/源码包/32 个二进制匹配；恢复保留引用该构建已运行的 136×3、6 编译/12 受控 JVM、真实两类 4 重编，核验全部 2791 个证据 pins，本轮未重跑 oracle 或 Cargo，target 已清理。
- 本轮止于用户指定的 2.6；2.9 overlay 与 2.10 仍未验收，OpenSpec 17/59，S2 未进入。下方保留此前类名检查点及后续待办，本轮不推进它们。

## 当前接续：S1 / 2.9 类名检查点已验证（2026-10-21，整项未验收）

- [当前检查点](openspec/changes/restructure-demand-driven-recovery/results/s1-raw-class-names-v1/README.md)：类/包/父类型改为严格 MUTF-8，InnerClasses/caller/组装的物理比较与 static fold descriptor census 保留 raw。四个 supplementary 根类由失败转为完整编译/行为通过；21 名称当前 **13 编译 / 26 次行为通过 / 4 合法根类编译失败 / 4 损坏拒绝**，原成功保持。
- 四个嵌套子类单独查询由 0/4 变为 4/4 编译，未声称行为/ABI 等价。四个 RawDescriptor 根类投影仍失败，同源 ASCII 对照也失败，后续来源/投影阻塞单列，不按 cast/BCI 放行。
- **31 目标 1431 passed / 8 基线已有 failed / 3 默认 ignored**，仅新增两项根库测试，原失败集与 ignored 保持。A1.4 的环境身份八项及 loader/cache 三项全部通过，package-info 原投影保持。
- 本轮实际通过 **136×3** 正文/核心、6 编译/12 受控 JVM、真实两类 4 重编、字段四组/8 JVM；未执行真实包，不声称完整公开报告迁移。当前 R4 的 **532 项 Rust/manifest**、归档和全部 32 个二进制匹配，五个本阶段构建 target 清理；主动中断和 R3 编译错误保留。
- 当前 schema **7**、恢复合同 **8**、解析合同 **3**、输入域 **v4**。下一步按[raw alias 清单](openspec/changes/restructure-demand-driven-recovery/results/s1-raw-class-names-v1/raw-alias-inventory.md)继续 2.9：reader → facade/MethodParameters → DebugLocal/SlotEvidence/RenderedName → reuse → 发布/摘要；另核对剩余精确消费者和 simple-name 语法。2.10 旧失败与资源/停止门保持未完成，OpenSpec **17/59**，S2 未进入，持续目标 active。未提交、推送或归档。

## 此前接续：S1 / 2.6 保持通过，已开始的 raw Field 检查点收尾（2026-10-21）

- [本检查点](openspec/changes/restructure-demand-driven-recovery/results/s1-raw-fields-v1/README.md)在当前 R14 二进制验证 A1.4 全部 **11 项**通过；完整环境身份、defining loader 后继、两个 loader/raw-name memo 与 loader/物理节点均保持。旧 2.6 的 2 loader/7 身份失败复现引用下方 v5 收据，本轮没有再运行一次旧构建，也未追加 2.6 产品修复。
- 持续目标中已开始的 Field 迁移完成当前验证：raw 字段三元组贯穿判断/sidecar/类级校验，声明合法 MUTF-8 拼写不再与使用分裂；三轴碰撞转绿，四组字段 JAR 从 0/4 变为 4/4 编译、8 次隔离 JVM 一致。21 名称由 5 编译增为 9 编译/18 行为通过，4 损坏仍拒绝，**8 合法输入仍不能整类编译**。
- 当前 **31 目标 1429 passed / 8 failed / 3 默认 ignored**。原 14 目标失败集不变；扩大五项失败在本阶段迁移前源码独立构建中实际复现，WA main 的整类失败仍是真实缺口，2.9/2.10 按根因承接，不称为整个工作区全绿。
- 本轮实际执行 **136×3** 正文/核心零差异、6 编译/12 受控 JVM、真实两类 4 重编；未执行真实包，不声称完整公开报告迁移已验收。两个当前构建与 532 项 R14 源码/归档及二进制 pins 匹配，17 个阶段构建 target 均已清理。
- 当前 schema **7**、恢复合同 **7**、解析合同 **3**、输入域 **v4**。2.9/完整 A1.6、2.10 资源/停止门和 S1 整体未验收；下一步仍需类名/嵌套类型、其余精确消费者和 raw alias evidence。OpenSpec **17/59**，S2 未进入，持续目标保持 active。未提交、推送或归档。下方为历史检查点。

## 此前指定范围：S1 / 2.6 当前源码复核（2026-10-21）

- [本次验收](openspec/changes/restructure-demand-driven-recovery/results/s1-environment-recheck-v5/README.md)：修复前构建实际复现 2 个 loader、7 个身份失败；当前 A1.4 全部 11 项通过，相邻聚焦回归共 **286 passed / 0 failed / 1 ignored**。完整有序环境身份、defining loader 后继、两个 loader/name memo 与 loader/物理节点身份均保持。
- 本轮实际重跑 **136×3** 正文/核心零差异、6 次完整类编译/12 次受控 JVM，以及真实 JAR/WAR 两个成功类 4 次重编；未执行真实包。当前 532 项 Rust/manifest、归档、构建前清单和全部二进制 pins 匹配 raw-dynamic R2，target 已清理；本轮复用构建，未追加产品代码。
- 进入本轮前的 2.9 overlay 仍未验收；其 14 目标回归为 1149 passed / 3 failed / 1 ignored，不把工作区说成全绿。当前 schema 6、恢复合同 6、解析合同 3、输入摘要域 v3；下方历史版本只适用于各自冻结检查点。
- 本轮停在用户指定的 2.6；OpenSpec **17/59**，2.9–2.10 未验收，S2 未进入。未提交、推送或归档。持续架构目标保持 active，下方为此前进展。

## 当前接续：S1 / 2.9 raw 调用签名消费已验证（2026-10-21，整项未验收）

- [最新检查点](openspec/changes/restructure-demand-driven-recovery/results/s1-raw-signatures-v1/README.md)：实际复现 raw CallTarget 的展示 descriptor 被重新解析为 `p.\\uD801` 源码类型；调用参数/返回、构造偏移、varargs、bridge/concat/guard、类级参数检查改用 raw descriptor，共享 Java 类型拼写。反例转绿，没有按 fixture/BCI 增加特例。
- 当前 R1 的 14 目标 **1144 passed / 3 项本轮基线已有 failed / 1 ignored**；Java 559、JVM 251、根库 261 全通过。两项源码断言和一项 prepared/direct 的 CallerSupplied/SelectedReads 比较继续保留；实际修复前 raw-members R2 同样失败，不冒称用 2.8 原二进制重现。来源边界不能为比较换绿而删除。
- 当前 CLI 完成 **136×3** 正文/核心零差异、6 编译/12 受控 JVM、真实两类 4 重编。21 名称输入保持原 5 项编译/10 次行为成功、4 损坏拒绝；**12 合法名称输入仍不能整类编译**。本轮不声称完整公开报告迁移已验证，未执行真实 JAR/WAR 目标代码。
- 当前 532 项 Rust/manifest 为 raw-signatures 的 `product-sources-r1`；构建/二进制摘要、原流和清理记录同目录。schema **6**、恢复合同 **5**、解析合同 **3**、输入摘要域 **v3**。本轮 baseline/product target 已精确清理；后面的版本号只适用于各自冻结检查点。
- 下一步按 **2.9** 迁移 Field/DynamicSite/SAM MethodType、剩余 owner/name 精确判断、raw alias evidence 和完整类名称/类型拼写，再完成 A1.6 与 2.10 阶段门。OpenSpec **17/59**，S2 未进入；未提交、推送或归档。持续目标保持 active。

## 此前指定范围：S1 / 2.6 当前构建复核（2026-10-21）

- [本次结果](openspec/changes/restructure-demand-driven-recovery/results/s1-environment-recheck-v4/README.md)：修复前构建再次复现 2 个 loader、7 个环境身份失败；当前构建 A1.4 全部 11 项通过，连同相邻回归共 **286 passed / 0 failed / 1 默认 ignored**。6 个二进制目标、7 次调用，根库只执行 3 项 loader/cache 测试。
- 当前全部 532 项 Rust/manifest 与 raw-members R2 构建的归档/清单逐项相同，二进制 pins 匹配且 target 已清理；本次复用构建，未改产品代码。2.6 无新增确认缺口，未继续 2.9 或进入 S2。OpenSpec 保持 17/59。
- 进入本次前的 raw-members R2 overlay 尚未验收：已有 prepared/direct 比较失败只显示 `input_origin` 不同，未重建基线归因，不能说整个工作区无退化。当前该构建 schema 6、恢复合同 4、解析合同 3、输入摘要域 v3；下方旧检查点版本仅适用于各自冻结源码。完整恢复 oracle 继续引用原 2.6 收据，本次未重跑。未提交、推送或归档。

## 当前接续：S1 / 2.9 raw 声明交接已验证（2026-10-21，整项未验收）

- [本检查点](openspec/changes/restructure-demand-driven-recovery/results/s1-raw-declarations-v1/README.md)：真实复现孤立 surrogate 方法名通过 lossy MethodFacts 校验的误认；MethodFacts/DeclaringClass 保留 raw 名称/descriptor，Engine 同次声明校验按 raw 比较，参数/返回及 generic constructor descriptor 读取 raw，合法 supplementary 类型不再按 UTF-8 错读。仍有符号引用和部分声明消费者待迁移。
- R2 的 12 个目标 **875 passed / 2 既有源码字符串断言 failed / 0 ignored**；根库 261、Java 556 全通过。两项失败仍检查 prepared.method_code，测试与其读取的 JVM 文件均和本次改动前相同，2.10 承接。构建/测试 profile/CLI feature 参数错误及名称分类脚本修正均保留原流，不混入最终通过数。
- 最终 CLI 完成 **136×3** 正文/核心决定零差异、6 次受控编译/12 次隔离 JVM、真实两类 4 次重编。21 名称输入中原 5 项编译/行为保持，4 损坏输入仍 Failed/classfile_decode；12 合法输入仍不能整类编译，raw aliases 未修。没有宣称完整公开报告迁移或 2.9 完成。
- 当前源码为 product-sources-r2 的 532 项；11 root 目标来自 product-build-r2，CLI 与 Java 单元来自 final-build-r2，两次源码逐项相同。schema **5**、恢复合同 **3**、解析合同 **2**、输入摘要域 **v2**。所有本轮 target 已精确清理。OpenSpec **17/59**，2.9–2.10 未勾选，S2 未进入；未提交、推送或归档。
- 下一步仍按 2.9 迁移成员/callee/调用/字段/动态引用的 raw handoff 和精确消费者，补 raw alias evidence 及完整类拼写；不能因私有校验反例通过就结束名称恢复工作。

## 本次指定范围：S1 / 2.6 当前工作区复核（2026-10-21）

- [复核结果](openspec/changes/restructure-demand-driven-recovery/results/s1-environment-recheck-v3/README.md)：冻结旧构建再次复现 2 个 loader 与 7 个环境身份失败；当前 532 项源码的 6 个二进制目标、7 次调用 **51 passed / 0 failed / 0 ignored**，含 A1.4 全部 11 项。根库只运行 3 个 loader/cache 测试，其余为 5 个完整集成目标。
- 2.6 已有实现与验收，本次未发现需要追加的产品修复。两个构建的源码逐项相同、二进制 pin 已核对，target 均已精确清理。完整恢复 oracle 沿用原 2.6 冻结记录，本次未重跑，不冒称全工作区全绿。
- 本次停在用户指定的 2.6 复核；保留此前 2.9 未验收 overlay，未继续其 Java raw handoff。OpenSpec 17/59，2.9–2.10 未验收，S2 未进入。未提交、推送或归档。下方是此前接续记录。

## 当前接续：S1 / 2.9 进行中，reader 与名称/ZIP 边界已验证（2026-10-21）

- 按持续目标继续既定重构；[2.9 进展与原流](openspec/changes/restructure-demand-driven-recovery/results/s1-raw-names-v1/README.md)冻结 21 个名称反例、34 次受控原始 JVM oracle。进入 2.9 时有 8 个合法输入生成类不能编译、4 个 supplementary 类名选不中；5 个别名控制行为一致但 raw 证据丢失，4 个损坏编码已被正文恢复拒绝。分母是受控名称样本，不外推真实包。
- reader R3 已实际 **183 单元 + 2 边界测试通过**，严格 MUTF-8 校验用于容错成员表；合法孤立 surrogate 的 raw/UTF-16 保留。R1 的编译编辑错误、R2 的新增 fixture census 断言失败与 R3 修正原流分开保留。
- Unicode 查询/CLI 文本编码为 MUTF-8，ZIP 查找只将 JVM 名称无损转为 UTF-8，保留物理 entry、raw prefix 与 origin；`PathNameDiffers.path_name` 改用 ArchiveNameBytes。修复前实际选中了 MUTF-8 路径诱饵；R1 三项新反例通过，12 目标 **659 passed / 1 既有取消 failed**。R2 补齐 serde 直接注入 raw 名称的扫描前拒绝，**24 项通过（20 导航 + 4 边界）**。两组真实 javac fixture 的 CLI class/member 选择由旧构建 exit 2 改为 R1/R2 exit 0，每次 1 Header/1 Body/0 IR；这不是整类 Java 恢复验收。
- 当前 532 项 Rust/manifest 对应 navigation-product R2，R1→R2 仅 facade 查询验证和新增负例；R2 CLI 位于 `/private/tmp/jarde-s1-raw-names-navigation-product-r2-bin-20261021/cli-jarde-cli`，摘要与源码包见构建收据。所有本轮 target 已精确清理。
- **下一步仍为 2.9**：迁移 MethodFacts/DeclaringClass/MemberBody、CallTarget/字段/动态引用及 callee handoff 的 raw 名称/descriptor；把同次输入校验与精确匹配切回 raw，LVT/MethodParameters/RenderedName 保留结构化 raw alias evidence；再修源码类型拼写，执行全部 21 名称、136×3 和既有完整类/受控行为门。当前 Java/facade 中的 lossy 精确判断尚未修复，不能靠本次物理查找通过宣布 A1.6 完成。
- OpenSpec **17/59**，2.9–2.10 未勾选，S2 未进入。解析合同目前为 2，恢复合同仍为 2；Java raw handoff 的后续语义变化需同步恢复合同。2.10 两项既有失败/资源门及 3.3 平台 uncertainty 风险保留。未提交、推送或归档。

## 本次接续：按用户指定复核 S1 / 2.6（2026-10-20）

- 用户本次指定 2.6 环境身份、defining loader/cache key，范围停在 S1。[当前源码复核](openspec/changes/restructure-demand-driven-recovery/results/s1-environment-recheck-v2/README.md)从工作区 531 项 Rust/manifest 重新构建，6 个目标 **308 passed / 0 failed / 0 ignored**；其中 A1.4 环境身份 8 项、loader/cache 3 项全部通过。冻结的修复前构建再次复现 2 个 loader 失败与 7 个身份失败。本次未发现需要追加的 2.6 产品修复，没有修改产品代码。
- OpenSpec 保持 **17/59**，保留已验收的 2.7–2.8；未验收 2.9–2.10，S2 未进入。原 2.6 的 136×3 及编译/JVM 保留结果仍以冻结收据为准，本次不冒称重跑整套 oracle。两项既有失败与资源门继续归 2.10。
- 工作区已有未验收的 2.9 前置改动：相对 2.8，reader 的 model/classfile、JVM semantics 三文件变化，新增 raw_name_boundary 测试；原件与 21 组名称基线保留于 `results/s1-raw-names-v1`。reader R3 已完成构建，尚未执行其 R3 测试；不能把 reader 前置改动或本次 A1.4 回归当成 2.9 验收。Java/facade raw 匹配、ZIP 名称编码边界和别名 raw evidence 尚未迁移。
- 本次构建期间源码摘要未变，临时 target 已精确清理；构建计划、源码包、二进制摘要、修复前失败与当前运行原流均在复核目录。未提交、推送或归档。

## 当前接续：架构调整 S1 / 2.1–2.8 已验收（2026-10-20）

用户持续目标为按 OpenSpec 优先推进既定架构重构，本轮接续完成 S1 2.8 的输入来源隔离与私有交接一致性；暂不进入 S2。下方“停止交接”是此前任务的历史记录，其停止指令不覆盖本次新授权；旧 change 的未归档状态仍保留。

- 当前 change：[restructure-demand-driven-recovery](openspec/changes/restructure-demand-driven-recovery/tasks.md)，17/59：S0 九项 + S1 2.1–2.8。S1 2.9–2.10 未验收，S2 未进入，未同步主规格或归档。
- [S1 结果与职责边界](openspec/changes/restructure-demand-driven-recovery/results/s1-core-v1/README.md)：983 passed / 0 failed / 1 既有 ignored；136×3 evidence 正文/核心决定一致，旧 all 的公开报告除 usage 外 136/136 相等。E2 反例 6 编译/12 JVM 对照全部通过；真实 JAR/WAR 两个既有成功类的 4 次重编通过，未执行真实包目标代码。
- [资源问题](openspec/changes/restructure-demand-driven-recovery/results/s1-core-v1/resources.md)：480/480 采集返回，RSS 均通过；同期时间门 2/24、冻结 S0 时间门 5/24 失败，2.10 保持未完成，未放宽阈值。essential 的恢复工作补齐至旧 all 有效路径，新增证明计费与候选峰值如实记录。
- 当前 HEAD 仍为 `30464b0838aa9da4746aad3b1753501231e9ace0`，S0/S1 改动尚未提交。2.1–2.2 的 R7/R8，以及 2.3、2.4 的产品/测试后继均封存在各自结果目录；不能只凭 HEAD 重建当前产品。没有活动 Cargo；本轮临时 target 已清理，测量适配器 `tests/s1_resources.rs` 已移除（源码包内可恢复）。
- [2.3 需求复用验收](openspec/changes/restructure-demand-driven-recovery/results/s1-demand-v1/README.md)：334 项实际回归通过，136×3 对照及 6 编译/12 受控 JVM、真实两类 4 重编通过。R1 产品源码包与 R2 仅测试修正分开绑定；S0/S1 旧结果未改。无活动 Cargo，临时 target 已清理。
- [2.4 状态/停止验收](openspec/changes/restructure-demand-driven-recovery/results/s1-demand-state-v1/README.md)：15 目标 1140 passed / 0 failed / 0 ignored。Header 结果分层、两 memo poll、driver 停止零辅助 Body、类族交接及 pool-rerun 多候选停止均通过；136×3、6 编译/12 隔离 JVM、真实两类 4 重编无退化。R3 产品 + R5 仅测试输入修正 + R6 公共 Header 成本测试后继分开绑定；原失败不改写。
- [2.5 源码目标验收](openspec/changes/restructure-demand-driven-recovery/results/s1-source-target-v1/README.md)：SourceTarget 7/8 与 RuntimeProfile 分开，方法/类/类族/批量、库/CLI 请求与报告贯通，binding schema 2。28 目标 1145 passed / 0 failed / 4 既有 ignored；136×3 正文不变，136 个完整报告经明确字段迁移后等价；源码 7/8 受控编译及 JVM 对照、真实两类重编通过。R1 源码包 + R2 五文件纯测试补丁可重建，产品 CLI 逐字节相同。
- [2.6 环境/loader 验收](openspec/changes/restructure-demand-driven-recovery/results/s1-environment-v1/README.md)：完整有序 domains/providers 进入 EnvironmentIdentity；Java Header 保留 defining loader/definition，两个 memo 按 initiating loader/raw name，节点按 loader/definition 区分，binding schema 3。A1.4 新增 11 项通过；34 目标 1497 passed / 2 既有 failed / 4 既有 ignored，新增退化零。136×3、严格完整报告迁移、6 编译/12 JVM、真实两类 4 重编通过，未执行真实包。
- 两项扩大回归失败已用进入 2.6 前的 520 份源码重建确认：EM11 cap2 新旧都 complete（旧断言失效），预取消 skipped 范围旧新都遗漏（合同缺口）。两项原流保留、由 2.10 承接，未改测试换绿。完整身份复制/报告字节成本亦需 2.10 沿冻结资源协议验证。
- R1 产品与 R2 两文件纯测试后继分开绑定；Root/CLI 大部分回归用 R1，根库最终加强断言与 Java/JVM 使用 R2。不能仅凭 HEAD 重建；结果内有源码包、补丁、构建计划和来源收据。
- [2.7 语义绑定验收](openspec/changes/restructure-demand-driven-recovery/results/s1-semantics-v1/README.md)：schema 4 分开合同 revision/源码实现身份/包版本/major 元数据，绑定实际物理依赖及词法/投影输入。34 目标 1501 passed / 2 既有 failed / 4 既有 ignored；136×3、完整报告及受控 oracle 通过。实际语义变体拒绝旧 expected 且保留正文；normal/test-support 的 binding/逻辑预算一致并匹配独立目录的 R3 CLI。最终 529 项源码包、R3 全部二进制与独立探针收据分别 pin，不能仅凭 HEAD 重建。
- [2.8 输入来源验收](openspec/changes/restructure-demand-driven-recovery/results/s1-trust-v1/README.md)：公开 Java `*Input`/subject 为 caller-supplied，真实消费位置的假构造器和错 facts/subject/runtime 保留预览但 unbound；五种公开入口/callback 无法授予 binding。Engine 私有同次读取检查并在正文提交边界绑定，停止路径保持。41 目标 1532 passed / 2 既有 failed / 4 既有 ignored，136×3、完整报告及编译/JVM 对照通过，530 源码及 R2 二进制固定。
- 下一项为 2.9：raw 名称/descriptor 与 MUTF-8 边界；输入摘要和来源隔离不能修复 lossy 精确判断。本轮完成 2.8 的 A1.6 来源部分，完整 A1.6/S1 尚未验收，S2 未进入。3.3 需复核非 caller uncertainty 的平台契约消费（静态风险，未复现），不与当前身份修复混为完成。2.10 原资源门保持未完成。

## 历史停止交接：四原件验收完成，两项OpenSpec各7/8，尚未归档

用户要求把未完成工作写入handoff并停止，本会话不再推进。代码产品 `8ea98e2e11d64f02092ac200d2b596511dcf5e40` 已通过本机完整验收及[自身CI39278336384](https://github.com/LordCasser/jarde/actions/runs/39278336384)。两项OpenSpec [return leaves](openspec/changes/recover-proved-post-test-loop-return-leaves/tasks.md)与[prefixed retry](openspec/changes/recover-proved-prefixed-catch-retry-loops/tasks.md)各7/8：3.3已接受，3.4未完成；两个目录仍在active changes，尚未移动到archive。主规格已同步2要求/9场景。本次仅提交交接文档和CI证据，不冒称文档后继提交另跑产品CI。

### 接续者先做这些

1. 先拉取GitHub并核对当前main。无需重做两项实现、三方矩阵、native或产品CI；当前剩余是文档、归档及最终交付检查。
2. [自身CI证据包](openspec/changes/recover-proved-prefixed-catch-retry-loops/results/own-ci-root-v1/README.md)已入本次交接：874 payload，tar SHA256 `cd6190ccd2d8d4db922c7b30054ab0f2ec2b6b2fc3c448986039827857c218c1`。root已逐成员复开；额外Luna独立tar复核在用户停止时中断，尚未验收。包内root合同决定、实际复跑报告、原始失败与源码绑定均可查。
3. 复核两个delta仍逐字同步主spec，按OpenSpec归档流程关闭3.4；归档日期使用接续时的实际日期。归档后修正Markdown链接，保留JSON/raw/tar中的历史路径。最后一次OpenSpec strict为归档前390 passed/0 failed；归档后尚未运行，结果数量应实际派生，不能照抄390。
4. 完成文档最终状态、归档后strict与链接检查，再勾选3.4、提交推送，并实时检查HEAD/origin/GitHub main相等、工作区干净、全部15 checkout无target、14辅助worktree detached/clean/为main祖先，以及5333来源仍绑定产品。不要把本次handoff提交或旧产品检查点当成上述最终关闭验收。

本机接续工具在 `/private/tmp/jarde-last-four-closeout-tools-root-v1-20261019/`，包括`audit_delivery.py`与`run_logged.py`；5333来源绑定原件是 `/private/tmp/jarde-last-four-product-binding-root-v1-20261019.json`。两个合同/独立审计的root实际复跑分别在 `/private/tmp/jarde-last-four-ci-contract-review-root-rerun-v3-20261020/` 和 `/private/tmp/jarde-last-four-ci-independent-rerun-root-v1-20261020/`，也已封存在CI包中。私有临时路径不保证跨机器存在，按清单迁移工具副本，不改封存原件。

曾为准备归档临时将3.4勾成完成，并取得8/8的apply/status私有回执；用户要求停止后已恢复3.4未完成，当前实际7/8。`/private/tmp/jarde-last-four-{apply,status}-recover-proved-*-root-v1-20261020/` 的8/8回执仅是中间状态，不代表归档或最终交付完成。未创建`results/final-delivery-root-v1`，也未移动任何change目录。

### 已完成事实与边界

基线和真实IR见[四原件基线](openspec/evidence/java-syntax-2026-10-19/cf09-last-four-original-root-v1/README.md)、[同轮IR](openspec/evidence/java-syntax-2026-10-19/cf09-last-four-ir-root-v1/README.md)。DoWhileBreak2复用终端return叶证书，保留StatementFree既有优先级；BreakInLoop3精确持有未保护lead/单named row/retry/normal DAG，组合现有Straight+LoopBreak，并复用Sites/SSA证明允许循环前构造保存跨try使用，初始化效果不移动。无新CFG/AST或Dex前端。

[最终完整对照](openspec/changes/recover-proved-prefixed-catch-retry-loops/results/final-comparison-root-v1/README.md)：Do2十一例70命令/20VM220CASE；Loop3原件94命令/36VM含实际check；Do3十例94命令/36VM360CASE；独立重试控制105命令/40VM400CASE。8API/default-all稳定，完整生成源码原样重编及成功产物双runtime隔离执行；框架55/52失败单列。Loop3 NYI/@Test注释与当前实际JADX成功分开，Do3无check/外层JUnit断言未运行。

[native完整验收](openspec/changes/recover-proved-prefixed-catch-retry-loops/results/native-accepted-root-v1/README.md)：一次fresh运行396 targets/104批、3856通过/0失败/98既有忽略，111精确门；3954具名结果由实际Running重建。5326 native/3373 build并集5333来源逐Git blob/bytes/mode绑定，默认栈/5GiB/1GiB/每秒守卫与finally clean均接受。runner旧observed_counts错配93批314目标项，不消费原错误归属。

[本产品自身CI与合同审计](openspec/changes/recover-proved-prefixed-catch-retry-loops/results/own-ci-root-v1/README.md)：4 jobs/52 API步骤全部成功；双seed各396 targets、3856通过/0失败/98 ignored、3954具名结果与156精确门，两轮完整Counter相等。跨native比较时统一should_panic展示后缀，并唯一源码绑定一个被GitHub遮蔽的测试名；不声称原始Counter字面相等。官方ZIP有38个独立step成员、缺14个，fuzz CLI原流不完整；原始aggregate与API补充证据均保留，未拼造缺失正文。私有ZIP完整成员门仍失败；root已另按tasks 3.3合同实际重跑接受，不与OpenSpec strict混淆。

13个固定原件中12项已接受，Loop6是Smali/Dex输入边界：JADX实际解码原Smali和生成Dex，Jarde原始输入拒绝、Dex-only ZIP无class候选；无JVM oracle，不以Java控制替代。来源债务：Loop3 writeMore 48/49缺BCI 88，独立控制run 40/41缺BCI 71；两者各自为goto映射缺口，不与旧Loop5 BCI 103推断同根因。Iterator泛型字段呈现和runner observed_counts错配仍独立记录。[成本模型债务](openspec/evidence/java-syntax-2026-10-19/cf09-prefixed-retry-cost-model-debt-root-v1/README.md)单列重复扫描、边集物化轮询及额外visited复制。CF09/71/612保持partial。

## 上一完整产品：Loop4/5 完整验收通过

主线`main`。当前产品`412245d514896b1d0eb9a8fe24388f6a70668637`，由首版恢复a4ccac6b4和最小预算修正412245d51组成。Luna私有实现，root审查、集成及实际对照；无新分支/worktree。

Loop4给自然循环外的闭合break分支DAG建立精确owner证明，复用既有sequence/If/LoopBreak，保留ForHeader。Loop5在精确普通named catch范围内组合prefix+完整While+return-tail；保留effectful latch的append/update。Builder复用已有catch身份，区分handler入口异常绑定与同物理槽普通double，不引入通用槽拆分。外来入口/出口、Call/Return跨owner、异常行漂移、重复/遗失owner及预算/取消/晚期Stop均有拒绝和回滚门。

- [完整三方与定向验收](openspec/changes/recover-named-catch-loop-return-tails/results/focused-and-full-comparison-root-v1/README.md)：两原件各70命令/140raw、8API/24方法structured；各Jarde8及JADX2完整JDK23编译成功、20生成VM同原，10次JDK8框架55/52编译边界保留。所有case及实际check按同profile/runtime对照，default/all、identity和映射稳定。Loop5扩展12例的20VM/240case亦同原件，含实际null/M/g/重复点/零/超大数。root与独立raw审计已接受。
- 最终定向CF12 9、Java核心540、公开API4、Clippy/fmt均通过。首轮native和首版CI39248350582在旧CF12精确预算门失败，因非候选单出口循环被多付完整scan费用；仅前置既有两出口必要条件，保留真实候选收费和取消轮询。旧365@29/130@1测试完全未改。
- 最终fresh workspace：394 targets、103批、3841通过/0失败/98既有忽略，96精确门通过；5299 source pins前后相等，默认栈及5GiB/1GiB守卫未触发，finally target absent。构建3346/native5299来源并集5306均绑定产品Git blob和实际字节/模式。独立raw审计已接受。[本产品自身CI39251504476](https://github.com/LordCasser/jarde/actions/runs/39251504476)四jobs全部成功，52个官方步骤逐一绑定原始ZIP；双seed各394目标、3841通过/0失败/98既有忽略，全部3939具名结果逐项相等，141精确门各一次通过。root和独立审计均接受；[native/自身CI归档](openspec/changes/recover-named-catch-loop-return-tails/results/native-and-own-ci-root-v1/README.md)保留完整原流、工具与来源绑定。
- 既有工具债务：runner的`observed_counts`把编译产物顺序与执行摘要错配，影响316个target项/95批。已有verifier及独立审计都按实际Running头重建394目标和3939具名结果，二者逐项相等；原收据不改，不能直接消费其错误计数表。本轮不改语法范围外的runner。
- 来源覆盖：Loop4 test23/23；Loop5 test59/60，缺BCI103（物理goto14），[独立映射债务](openspec/evidence/java-syntax-2026-10-19/cf09-implicit-loop-transfer-mapping-debt-root-v1/README.md)已记录，不宣称完整来源覆盖，也未推断与旧69/78同根因。

上轮最终CLI：`/private/tmp/jarde-loop45-final-focused-root-v4-20261019/candidate-cli`，64,631,664 bytes、0555、SHA-256 `d29b6fee723d8f59fbecd9fd8be9f15a8ec73de3a90f5a2a29b24ff91e4af607`。临时路径不保证跨机器存在；证据及来源pin进入持久归档，二进制/target不入库。上轮主规格同步2要求/8场景，两项OpenSpec均8/8。此段仅为Loop4/5历史产品检查点。

## 下一步与独立范围

Loop4/5旧轮交付时为9/13；当前四原件状态以上方新检查点为准。CF09、71单元及612文件保持partial，见[JADX账本](openspec/evidence/jadx-feature-inventory-2026-09-27/summary.md)。

四原件静态输入边界历史见[静态输入边界](openspec/evidence/java-syntax-2026-10-19/cf09-breakinloop3-to6-static-scope-root-v1/README.md)；Do2与Loop3两项真实gap已实现并完成验收，Loop6继续单列输入边界。finThrow5/5、nestedFin16/16、Loop2与ComplexIf已验收旧片不重做。

## 工作与资源边界

root独占共享写、Git/网络及Cargo/JDK/JADX/Jarde CLI；Luna只读共享文件，在新私有tmp用Python`-B`准备实现和审计。按OpenSpec派发，root负责架构、失败路径及完整对照验收。

Cargo串行：余量≥5GiB、target≤1GiB、默认Rust stack、每秒进程组守卫、finally精确清理。Cargo期间不共享写、不Git mutation、不网络或手动并行JDK/CLI。shell使用`login:false`。所有完成/clean/远端/worktree结论须由最终实时审计核对，不能沿用历史快照。

交付审计：主线产品与origin/main的同步、工作区清洁及target absent由最终推送后实时复核；此前14个辅助worktree均detached、干净且HEAD为main祖先，未新增分支/worktree，只有main占用分支。历史干净辅助checkout不作为待完成实现。


### 2026-10-25 / AU：继承 bridge 契约与源码 header 再生

[结果](openspec/changes/restructure-demand-driven-recovery/results/s1-bridge-contract-source-v1/README.md)、[验收](openspec/changes/restructure-demand-driven-recovery/results/s1-bridge-contract-source-v1/acceptance-r1.json)、[完整报告](openspec/changes/restructure-demand-driven-recovery/results/s1-bridge-contract-source-v1/semantic-final-r1/summary.json)、[后续边界](openspec/changes/restructure-demand-driven-recovery/results/s1-bridge-contract-source-v1/next-boundaries.md)。

Java source_proof 接管继承 owner 选定后的 interface/superclass header 再生资格，两个 Signature helper 收为内部实现；门面保留现有 resolver、平台 clean-Missing 探测、访问检查、事实供给、停止清空与原子提交。平台 raw owner/name/descriptor/use-kind matcher 放在 clean-Missing 后、认领契约前；不匹配继续原 member resolver 和其他父接口，不新增 provenance bool、DTO、Session 或缓存。

旧引擎完整 JAR 入口已复现 notCompare 被错认 Comparable.compareTo。恢复契约 40→41，schema12/解析4/inputs v9保持。1584 完整 JSON 严格保持；另18条错误平台准入转为拒绝并保留原物理方法，6条合法 Dual 双接口保持两个 bridge 投影。24条变化全部按事先冻结的 proof/text/费用向量核验，其余字段保持。absence每条少 AnalysisSteps6、AttributeBytes8及原 marker 257/275 UTF-8 bytes；Dual每条新增一次Other header及必要 resolver 逻辑费用，费用不充实际 I/O、RSS 或 resolver read 身份。

六项局部门通过：30 native 1183/0/2；1608完整报告；168全文、78声明、136×3核心及408完整JSON/跨evidence outline；AA12/130/3既有字段子集。新合法Dual+Other完整源码按SourceTarget7/8原样重编，3 javac（含独立Driver）/6 VM形成4个同runtime行为对；桥flags/擦除调用/实际CodeSource和loader均通过。AS14源变体/54报告/28行为对经AS→AT→AU两跳完整text/pins核验复用，该项新增JDK0。变异只查询，真实目标未执行。

本轮只改三条生产路径及一个既有测试函数，2735来源同路径、五构建/31产物/11原工具绑定；target清理，908105990bytes只是构建磁盘峰值。collector缺row.engine和core审阅器错指父目录均在成功采集后由独立只读finalizer修正；AS复用脚本的dict/list错误也只重跑审阅。原错误收据保留，成功CLI/JDK命令未重跑，原collector不改写成通过；权威报告为semantic-final-r1。

下一项仅迁 enum anonymous-child 的纯形状政策，复用现有facts引用、pending与Result/refusal，root保留selected read/原始身份/census/预算/stop/递归请求/commit。widening主体已在Java，已有公开class-source入口回归；待补同一数值BCI的跨method与跨loader同名异物理定义组合，不重复搬薄wrapper。Other类级可见性仅记静态待核实债务；当前Dual不是非法访问反例，不混修。验收工具的父producer/报告映射预检债务独立记录，后续必须先确认角色一致再执行产品。

2.9的其余职责/raw handoff与2.10实际read/Body/IR/live/peak/RSS、各类停止/背压/panic、八历史失败当前复核、API/双轨收尾仍待原门验收。2.9/2.10/S1不勾完成，OpenSpec17/59，S2未进入。


### 2026-10-25 / AV：enum child metadata relation 完整阶段迁移

[结果](openspec/changes/restructure-demand-driven-recovery/results/s1-enum-child-relation-source-v1/README.md)、[验收](openspec/changes/restructure-demand-driven-recovery/results/s1-enum-child-relation-source-v1/acceptance-r1.json)、[完整保持](openspec/changes/restructure-demand-driven-recovery/results/s1-enum-child-relation-source-v1/report-retention-r1/summary.json)、[下一项](openspec/changes/restructure-demand-driven-recovery/results/s1-enum-child-relation-source-v1/next-boundaries.md)。

Java source_proof接管selected child的metadata relation阶段：child/enum两侧唯一anonymous InnerClasses行、类型级EnclosingMethod及唯一匹配构造器。输入借用现有reader facts/pools与raw owner/descriptor；返回原anonymous owner列表和借用MemberHeader，复用标准Result/Option/tuple，无新DTO/Session/cache。两条原扫描收费和逐行poll原位保留。root继续selected read、this/super与物理身份、pool/attribute读取、constructor/override Code、use census、递归请求和全组commit；不把原忽略的无效CP行提升成全局错误。

本轮纯职责迁移，recovery41/schema12/resolution4/inputs v9不变，实际Java/facade指纹更新。只有facade/source_proof两生产路径，root减少56行、Java增加85行；0测试函数/永久资产/来源路径新增。六项局部门通过：30 native1183/0/2；1740完整JSON全部严格保持，0语义例外，仅规范已核验的实际engine tuple及usage直属elapsed_millis。168全文、78声明、136×3核心、408完整JSON/outline、AA12/130/3原字段子集均保持。

分母1740=AU1608+旧lowering边界66+现代metadata负例66，两组各6复用控制已包含在AU，不重复计数。首轮误选Corretto8无$values输入，原流保留并明确不计本阶段覆盖；现代控制在新CLI前已按AS实际编译源码/report/input唯一匹配，两个常量体的projection_inputs存在。现代11类单点负例60 exit0/6重复constructor exit4，均保留可解析报告且不再投影；不能从末端报告推断所有私有helper guard均命中，变异只查询、不编译执行。

核心实际6编译/12VM与真实成功两类4重编通过。AS14源变体/54报告/28行为对按三跳，Dual两source targets/12报告/4对按一跳，均以完整源码与过程/pins核验复用；该复用项新增JDK0，不把新增132变异计入oracle。真实目标未执行。

五构建/31二进制/2735同路径来源、11原工具与3adapter绑定，target清理，908243530bytes仅为构建磁盘峰值。实际core-parent工具预检和旧新engine内存比较自检在构建前通过；构建后current producer发现build-plan缺旧模板baseline合同键，补入与原parent和冻结源码一致的四值，构建命令/目标/源/预算不改，原计划保留，原工具不改且成功命令未重跑。后续工具必须覆盖current producer冻结后预检，不能只用parent预检证明另一执行路径。

下一步验证widening完整class-source请求中的跨method同数值BCI与跨loader同名异PhysicalDefinitionId组合；已有普通/平台widening公开入口不重复搬迁。剩余enum group政策受root-private Pending/code candidate类型约束，先设计基于现有facts的边界，不凭空增实体或搬读/停止/commit。raw enum_owner旧UTF8仅为待完整入口复现风险，本轮不混修。2.9剩余职责/raw handoff与2.10实际资源/各类停止/八历史失败/API双轨仍待原门验收；2.9/2.10/S1不勾完成，OpenSpec17/59，S2未进入。

### 2026-10-26 / CE：capture 停止消费者已统一

统一 capture 依赖的实际 IR/原始停止返回通道。Java anonymous capture 调度透传泛型供应方错误；Root 复用同一 DependencyProofFailure，集中实际分析、执行合并及原 Report/diagnostics relay，anonymous/val 的 optional sink 与三个 legacy/strict 包装删除。普通 member、instance、匿名 interface/nested/super 和 double-brace constructor partition 的公开消费者均接收同一通道；源码形状拒绝保持原域，capture certificate 判断未改变。恢复46→47，schema12/解析4/inputs9，无新DTO/Runtime/cache/read。

先在原代码同三测试复现2pass/1fail：48公开控制中8充分预算正例、8后置proof Error及double-brace两路8个Report stop原已通过；其余24个Report stop失败。member/instance原总体停止类型正确但原diag丢失，interface/nested/super另有停止原因被Unsupported覆盖；没有公开假Complete证据。修后16目标482/0/0；48控制全通过，32真实IR Report逐字消费原execution/diag一次，8 proof Error保留AnalysisSteps，均保持已发布源码/方法prefix，停止后actual wrapped-reader calls/bytes、Body decoder及六IR starts为0。原25政策控制保持，CD原6成功/早拒与4 strict报告保留，删除4 legacy可选sink控制，现CD10；不是新增独立程序/holdout。

2469继承全JSON/退出码严格保持，含168枚举与20匿名profile子集，无额外矩阵CLI；408核心全JSON保持，零语义/计费例外。168完整源码/78声明、136×3核心、核心6编译12受控VM、真实两类4重编保持，真实目标未执行。2745源/资产、归档与17当前产物及三正常构建绑定，target清理；baseline-r1切点修正和current-r2静态scope纠正均主动终止，原流/归档/exit -15保留，无native运行。构建磁盘护栏不作S0运行资源证据。

共享capture证明簇已归Java，typed供应/消费闭包现补齐；剩余ordinary/instance方法需求编排、owner census、call-site/bridge与nested/super/double-brace专用资格/暂存事务仍按机制迁移。shared emitter Interrupted、其他read/bridge/sink/资源/API及旧native失败保持开放，第三轮历史六项原JDK8/失败限制不豁免。2.9/2.10未勾选，OpenSpec17/59，S1未整体验收，S0原记录保持，S2未进入。

当前证据：`openspec/changes/restructure-demand-driven-recovery/results/s1-capture-stop-consumers-v1`；canonical producer/current-build-r3，下一组ordinary/instance需求编排，然后owner census及专用匿名策略，见next-boundaries。
