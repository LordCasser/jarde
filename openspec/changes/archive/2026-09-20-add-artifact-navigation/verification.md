# 验证记录

实现提交见仓库历史（紧邻本文件归档提交之前）。本机门禁如下，CI 结果随后回填。

## 契约实现

四个任务级入口落在库门面（`src/facade.rs`）之上，只读 snapshot：

| 入口 | 证据等级 | 读取内容 |
| --- | --- | --- |
| `Engine::list_class_candidates(&snapshot, &PhysicalScope, budget)` | entry 候选 | 只读范围内的目录，**零 Header** |
| `Engine::list_class_declarations(&snapshot, &PhysicalScope, budget)` | header 确认 | 每个候选一次 `class_headers` 尝试 + 物化 |
| `Engine::list_members(&snapshot, &PhysicalDefinitionId, budget)` | 一次确认读取 | 一次 `class_headers` 尝试 + 被指字节 |
| `Engine::find_targets(&snapshot, &PhysicalScope, &NavigationQuery, budget)` | 命名查找 | 只读取路径声明了该名字的候选 |

公开类型均在 `src/facade.rs` 并随 `pub use facade::*` 导出，每个类型的文档都写明它**不**证明什么：

- `ClassListingItem`（`ClassCandidate` / `Resource`）：某条 raw name 规则命中，不代表「已找到类」、可解码、路径与声明一致；`Resource` 是物理余项，不解析、不解释。
- `ClassDeclarationFacts`（`this_class`/class flags/super/interfaces）：一次读取的 parse 事实，不是 dialect 合法性、解析、verification 或可加载性。
- `ClassNameBinding`（`StandaloneRoot` / `PathNameAgrees` / `PathNameDiffers{path_name}`）：路径声明与类声明的关系；`PathNameAgrees` 只说明「在调用方声明的 container 前缀下，这条 entry 是该名字的一个绑定」，不证明 root、layout 或可加载性，且**不推断 prefix**。
- `MemberBodyEvidence`（`CodeAttribute{content_span}` / `NoCodeAttribute`）：成员的**声明**状态；Body 未被读取、解码或验证，`abstract`/`native` 以 `NoCodeAttribute` 表达，不伪造 Body。
- `MethodItem.identity: PhysicalMethodId`、`FieldItem.identity: PhysicalMemberId`：owner 就是该次读取的物理定义（location、class bytes、variant），调用方无需另行拼装；`index` 是表位置，不是身份。
- `ClassNameQuery` 的 `spelling()` 是请求回显，**绝不**进入身份。
- `NavigationReport` 中空候选 + 已扫描范围是**答案**（Complete），不是失败。

reader 侧新增有限：`classfile::{class_member_facts, ClassMemberFacts, MemberTableStop, MemberTablePhase}`（容错的声明 + 成员表遍历）与 `inspect::{materialize_root, materialize_definition}`（按身份/位置寻址的读取）。

## 关键证据（`tests/navigation.rs`，20 项，全部内存 fixture）

- **1.1 物理包含**：同名同字节出现在 `WEB-INF/classes/p/S.class`（root container, ordinal 0）与显式展开的 `WEB-INF/lib/L.jar` 内（`steps[0].via_raw_name`，ordinal 0）→ 4 个候选、2 个同 `this_class` 者各自保留 container/ordinal 与相同 digest，**两条条目、不合并**。
- **1.2 两种证据等级**：候选列举用量 `class_headers=0, class_bytes=0, code_bytes=0, method_bodies=0`；同一范围确认列举 `class_headers=4, class_bytes=368, attribute_bytes=28`。损坏候选（8 字节 magic）→ `Failed{classfile_decode}`、诊断带 `Location::Entry{span 0..8}`、`Partial` + skipped `[2,3)`、未读取候选出现在 `unconfirmed`。预算停止（`class_headers: 1`）→ `Partial{BudgetExceeded{ClassHeaders}}`。取消 → `Cancelled`、0 条目。路径名与 `this_class` 不一致 → 条目保留 `this_class=p/Real` **同时**给出 `PathNameDiffers{path_name:"p/Wrong"}` 与 `navigation_path_name_mismatch` 诊断，execution 仍 `Complete`（领域发现，不是扫描失败）。
- **2.1 身份交接**：取 ECJ v52 样本 `finallyPath(I)I` 的列举结果，**原样**放入 `MethodAnalysisRequest` → `Engine::analyze_method` 只读一个定义（`reads[0].definition == item.identity.owner`、`DriverMethodBody`、`method_bodies=1`，而列举从未计费 method body）。archive 内的成员同样落在同一物理定义。
- **2.2 隔离与损坏成员**：成员列举用量 `class_headers=1`，`method_bodies/code_bytes/ir_items/ir_edges/analysis_steps/normalization_clones` 全 0，`coverage.runtime_resolution/dynamic_analysis == NotRequested`（三层入口都断言）。成员结构损坏（谎言 `Code` 长度）→ 类列举仍确认该类并在条目上带 `member_table=Some((Methods, 1, "classfile_invalid_attribute_span"))` 与诊断；对该身份做成员列举 → `Partial{Error{...}}`、skipped `[1,2)`、`methods=["first"]`、Body/IR 维度全 0。
- **3.1 friendly 名称**：`dotted("p.S")` 与 `internal("p/S")` 绑定同一 `PhysicalDefinitionId`（ordinal/digest/length 相同，各一次 header），拼写只出现在 `query` 回显。重载：只给名字 → 两个 descriptor 各自身份；给 `(I)I` → 恰好一个；kind 过滤把同名字段挡在方法查询外。空匹配：空候选、无诊断、`Complete`、`navigation_candidates [0,0)` 范围、`class_headers=0`。
- **3.2 歧义与稳定性**：两个 origin 上的 `p.S` 都返回（bytes 91/digest `fe83ef11…` 与 92/`607db9f5…`），`Complete`、无诊断、`class_headers=2`；把任一身份回传就读到该类。重排归档记录不改变 container-relative 身份（`via_raw_name`、ordinal、raw name、digest）与各自读到的内容；跨 artifact 的身份被拒（`definition_snapshot_mismatch`）；不自洽的身份被拒（`definition_class_bytes_mismatch`/`definition_entry_not_found`/`definition_location_mismatch`/`definition_variant_mismatch`）。

## 设计偏差与本人决定的取舍（已披露）

1. **「复用既有 Header 读取」到不了损坏的成员表**：`noak` 的 `Class::new` 会急切校验整个类结构，成员表损坏的类在**所有**既有 Header 读取下都会失败。因此为实现「损坏成员保留可靠前缀」，reader 增加了一条 raw 声明 + 成员表遍历 `class_member_facts`，并与 `class_facts` 在**全部 43 个已提交 class fixture** 上交叉校验。这是本 change 唯一较大的新增，理由在此。
2. **确认列举使用该容错读取**，因此**不携带** version/dialect 平面、也不接受 `InspectionMode`；那些平面仍由 `Engine::inspect_header` 提供（已在代码与文档写明）。否则成员表损坏的类永远无法被确认，它的容错成员列举也就不可达。
3. **成员表停止不终止类扫描**（类已被确认，条目带 `member_table` 与诊断，execution 保持 `Complete`），而**成员列举**因该停止为 `Partial`；真正终止列举的是**类读取失败**（`Failed`/`Partial`/`Cancelled` + 可靠前缀）。对照 `artifact-views` 要求：「失败、预算耗尽或取消 → 非 Complete」——本条针对的是确认集合的扫描；成员级停止由条目与成员列举如实表达。两者合起来满足 `classfile-inspection` 的「返回可靠前缀、诊断、物理 origin，其他成员继续可用，不返回完整空列表」。
4. **路径名规则是「路径在 `/` 边界上声明了该名字」**，不是精确相等：WAR entry 的 raw name 自带前缀（`WEB-INF/classes/p/S.class`），而推断或声称该前缀被明确排除在范围外。这也是查找的候选规则，因此 WAR 查询可用并把 entry 交回，前缀可见于 raw name 而未被声称为 root。
5. **候选读取之间的取消在黑盒测试中不可达**：token 只在 poll/charge 边界被观察，测试覆盖「取消的范围扫描」（空前缀、`Cancelled`、无 provenance 的诊断）与「带已确认前缀的预算停止」，两者合起来构成前缀 + 非 Complete 的证据。
6. 其他自定取舍：类列举用 `unconfirmed` 列表、条目用 `member_table`（比塞一个索引更忠实）；列举 coverage 合并 reader 的物理范围与一条列举进度范围；成员列举计一次 `class_headers` 尝试（该维度不得少报一次 Header 读取）。

## 门禁

| 门禁 | 结果 |
| --- | --- |
| `cargo fmt --all -- --check` | 通过 |
| `cargo test --test navigation --locked` | 20 passed / 0 failed |
| `cargo test -p jarde-reader --lib --locked` | 142 passed / 0 failed |
| `cargo test --workspace --all-targets --all-features --locked --no-fail-fast` | **1211 passed / 0 failed / 6 ignored**（基线 1187，+20 navigation、+4 reader） |
| `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings` | 通过 |
| `cargo test --test p3_execution_comparison --locked -- --ignored` | 2 passed / 0 failed（18.6 s） |
| `openspec validate --all --strict --no-interactive` | 22 passed / 0 failed（归档前） |
| 实现提交的 CI（`e0c83b6`） | [run 35499237358](https://github.com/LordCasser/jarde/actions/runs/35499237358) **四 job success**：stable（fmt、clippy `-D warnings`、两轮固定 seed 全量测试、JDK 25 oracle、P3 编译执行对照、依赖边界、OpenSpec strict、`git diff --exit-code`）、MSRV 1.88.0、supply chain、fuzz smoke |

未新增 fixture 文件、未调用编译器（全部为 `tests/navigation.rs` 内的内存生成器，并在 `tests/fixtures/README.md` 登记一行），因此 corpus fingerprint 与 `p5_corpus_fingerprint` 未变（5 passed）。

## 边界

- 列举**不解析、不加载、不构建 CFG/SSA/Region/Java AST、不推断 WAR/Boot 布局或 classpath 前缀、不给 MR 选择、不证明可加载性**；示例与 README/支持矩阵已按此声明。
- 顺带记录、本次未改的既有问题：(a) 枚举按发布条目计 `ResultItems`，因此会发布派生条目的列举对同一批记录计费两次（有界、已记录）；(b) `JvmString` 的 `Deserialize` 强制严格 MUTF-8 而读取路径不做该校验，新的 raw 遍历与读取路径保持一致而不是分叉；(c) `inspect_header` 的 `ClassTarget::Entry` 仍要求已完整枚举的 `PhysicalEntry`，且 `container_record` + `read_entry_for_analysis` 对按身份寻址的读取会走两遍 container 目录（3 条目 JAR 计 `ArchiveEntries=5`），均未改动。
