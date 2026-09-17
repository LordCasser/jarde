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

以下是验证缺口，不是已证明的运行时错误。它们由独立的 [harden-p1-validation](../harden-p1-validation/tasks.md) 承接，验证通过后再进入本 tasks 的实现；P2 设计先完成。不把维护代码混进 IR 任务，也不重新打开历史归档。

本轮状态：V1/V2 修复和本地正反例验证已完成，见[验证记录](../harden-p1-validation/verification.md)。维护改动尚未提交/推送，新 CI 配置没有远端执行证据；进入 P2 实现前确认维护提交对应的 CI 结果，不能沿用旧 P1 run 代替。

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

## Risks / Trade-offs

- [Risk] frame/phi/origin 或 jsr 克隆乘法膨胀 → 分配前计费及高扇出/多槽位用例；P1 输入有界不代替 IR 上界证明。
- [Risk] provider 或 loader 不完整 → 状态、coverage 和 open-world 分开，唯一已知候选不证明唯一运行目标。
- [Risk] 库算法调用期间不能立即取消 → 明确规模上限和可观察取消粒度，不承诺硬 deadline。
- [Risk] 过早恢复源码吞掉缺口 → P2 固定 Bytecode 交付，先验证 IR，不引入 Region 或伪造 Java。

## Migration Plan

先独立完成 V1/V2 验证维护，再按 tasks 的基础契约、resolver、raw/legacy CFG、Frame/SSA、产品验收逐片执行。第一轮只做 1.1–1.3，退出时证明 reader、预算、结果模型可用。每片记录命令/结果和只读复核结论，阻塞项修复后以定向反例复验，再开始依赖片。交接列明未完成项和允许修改模块，不一次派发整条管线。

P2 本轮只修改规划与过时的 OpenSpec 阶段上下文，不新增 P2 代码、不勾选本 change 的实现任务。P1 的 harness/CI/deny 修复及实际验证归 harden-p1-validation。CI 状态按具体 commit/run 查询，不追加“记录文档提交自己的 CI”的循环提交。

### 保持拆分的既有债务

重复 unit/Code 物化和 Type-only 二次解码继续归 P5；has_more=true/cursor=null、record descriptor 类别、Entry.span、MR/tree aggregate 优先级和冗余 allow 等分别维护，不顺便清理。P2 通过直接 Header、显式停止状态和 class/BCI origin 避开这些依赖；实际受阻再以最小独立 change 修正，不能悄悄改变 query 契约。
