# P4 验证记录

本文件记录 `p4-modern-semantics` 各片的**实际验证**：命令、数字、证伪与如实边界。与 P2/P3 的记录同口径——**只有实际跑过并复核过的才写在这里**，未做的写在「未完成」里。

## 2026-09-20 1.1：release-bound feature registry（提交 `8620249`）

### 演进方式与既有语义

**新增** `crates/jarde-reader/src/release_registry.rs`（2042 行含测试），`classfile.rs::classify_version` 改为**消费**它；既有 `VersionRuleStatus`/`VersionDialectSupport`/`DialectValidationScope`/`Java8RuntimeCompatibility` **全部保留**。
**理由**：既有枚举回答的是「**本次构建支持到哪**」的档位，而 P4 要的是「**该 release 各自定义什么**」的表；把属性/CP tag/flag/opcode 约束塞进单变体枚举会把两件事混成一个类型，且 release 表还要被 1.2/1.3 按 `(name, location, major)` 查询。

**既有语义逐条对照**（旧字面区间 → 新来源，结果全部相同）：`major<45 → InvalidMajor`（`MINIMUM_MAJOR`）；`major>=56 && minor∉{0,65535} → InvalidModernMinor`（该规则**刻意越过 registry 上限**——它是格式规则，不是某 release 的能力）；`preview = major>=56 && minor==65535`（旧代码只算不暴露，现在可读）；`>71 → FutureRelease`；`53..=71 → StructuralProbeOnly`；`45..=52 → Supported`；`else → StructuralProbeOnly`；`java8 = (45..=51) 或 (52 && minor==0)`；`dialect_validation_scope` 仍 `VersionOnly`。
既有 4 条版本用例（`version_matrix_keeps_rules_dialect_and_java8_profile_orthogonal` 等）**一字未改即通过**。

### registry 形状

每个 release 一条 `ReleaseRecord { major, band, introduced, unregistered }`；`introduced` 记 **CP tag / attribute / flag / opcode** 四类该 release **引入**的约束，每条带 `since` 与 JVMS `source`；查询按「≤ 该 major 的累积」取值，故 `Record@60` 在 71 依然合法而**无须逐条重述**。band 承载支持决策（`DialectValidated{java8_runtime}` / `StructuralProbe` / `StructuralProbeWithPreview{preview}`），**覆盖 45–71 每一条 major、无空洞**（表自身有不变式测试）。

**53–71 的登记项**：53 = `Module`(4.7.25)/`ModulePackages`(4.7.26)/`ModuleMainClass`(4.7.27) + tag `Module`(19)/`Package`(20) + `ACC_MODULE`；55 = `NestHost`(4.7.28)/`NestMembers`(4.7.29) + tag `Dynamic`(17)；60 = `Record`(4.7.30)；61 = `PermittedSubclasses`(4.7.31)；54/56–59/62–71 = 无新增（**62–71 明确记「本 release 无自有条目」，不声称「与 61 完全一致」**）。

**来源诚实（本片最重要的一半）**：每条记录带**非空** `unregistered` 列表（测试强制非空），列出**刻意不主张**的东西——例如 `ACC_STRICT` 一律未登记（含 61 起 JEP 306 的效力变化，JVMS 措辞未确证）、非 JVMS 的 JDK 属性（`ModuleTarget`/`ModuleHashes`/…）未登记、flag 组合规则未登记、历史 tag 2 与保留 tag 13/14 未登记。**`NotRegistered`/`Unregistered` 是「无主张」而非「非法」**：`attribute_diagnostic` 对其返回 `None`。
**理由（写在此处以免被后人读成疏漏）**：一张**看起来完整**的表才是这里的失败模式——它会替我们**没收集过的证据**承诺合法性。

### 四个平面

| 平面 | 落点 |
| --- | --- |
| parse | `HeaderInspection::structural_read`（既有） |
| dialect validation | `VersionCapability::{version_rule, version_dialect_support, dialect_validation_scope, preview_marker, release_registration}` |
| verification | `HeaderInspection::verification`（本层恒 `NotPerformed`） |
| output-level | **新增** `HeaderInspection::output_level: OutputLevelStatus`（本片只有 `NotEvaluated`；与 `verification` 对称，**未评估不得读成通过**） |

### 两个 scenario

- **preview / future**：`inspect_header(Forensic)` 保留可读结构，同时 `structural_read=Complete`、`verification=NotPerformed`、`output_level=NotEvaluated`、dialect 明确 `UnsupportedPreview`/`FutureRelease`、`release_registration=UnregisteredFutureRelease`；strict 拒收；**未登记 release 上任何 registry 查询返回 `UnregisteredRelease`/空集/`None`**。
- **版本/位置不符**：`Record`@55、`PermittedSubclasses`@60、`Module`@52、`NestHost`/`NestMembers`@54 各给对应诊断；`NestMembers`@MethodInfo → `classfile_attribute_location_not_applicable`；flag 与 opcode 同理。诊断只含属性/flag/opcode 名、release 门槛、位置名与 JVMS 引用，**无 provenance、不含任何来自工件的类名/方法名/路径**（用**逐字固定串**钉住）。

### 父级独立复核

- 全量 **1006 passed / 0 failed / 3 ignored**（990 + 16：registry 单测 9 + classfile 单测 3 + `tests/p4_feature_registry.rs` 4）；fmt/clippy 1.98.1 干净；`openspec validate --all --strict` **14 passed**。
- **既有断言零改动**：父级用 `git diff -U0 | grep '^-' | grep -c assert` 核为 **0**。
- **父级独立探针（已删）**核对 registry 的核心诚实性，**非空洞**：
  ```
  major=71  registration=Registered                 Record@71 → Legal (since 60, JVMS 4.7.30)
  major=72  registration=UnregisteredFutureRelease  Record@72 → UnregisteredRelease
  major=100 registration=UnregisteredFutureRelease  invokedynamic@100 → UnregisteredRelease
  major=59  —                                       Record@59 → VersionNotApplicable
  major=44  registration=UnregisteredBelowMinimum   rule=InvalidMajor
  sanity: 71 有记录、Record@71 合法、Record@59 版本不适用
  ```
  即**未登记 release 绝不借用上限 release 的能力**（72/100 的两次查询都返回 `UnregisteredRelease`），而 71/59 等已登记 release 正常作答——探针**不是**在到处返回 `None`。
- **实现者三组证伪**：① `>71` 返回已登记 capability → 8 单测 + 2 集成红；② 未登记 release 的 dialect 报 `Supported` → 5 单测 + 1 集成红；③ `verification` 由 `NotPerformed` 改 `Performed` → 4 单测 + 2 集成红。三组均在 `/tmp` 副本（294 文件逐一校验）执行，仓库未被触碰。
- **CI**：`8620249` → run 35467145921，四 job success。

### 与 1.2 的边界（本片**没有**做）

- **没有**读取或结构化 module/record/sealed/nestmate 事实（属 1.2）。
- **没有**把 attribute/flag 合法性接到任何**读路径**上：registry 的规则是**按需查询 + 诊断**，由 1.2（事实读取）与 1.3（golden diagnostics）接线。负向用例 `header_inspection_leaves_attribute_and_flag_legality_to_the_fact_passes` 钉住这条界（**不要**把合法性塞回 `inspect_header`）。
- **没有**改 `dialect_validation_scope`、**没有**给 `output_level` 做出 `NotEvaluated` 以外的变体、**没有**新建 fixture（1.3 的职责）。

### 已知保守处（如实）

62–71 只登记「无自有条目」；`ACC_STRICT` 未登记；CP tag 条目统一引用 `JVMS 4.4`（`CONSTANT_Dynamic` 引入后子节号在版本间移动，故引用节而非子节号）；opcode 只登记 `invokedynamic`/`jsr`/`ret` 与三个保留 opcode。

## 2026-09-20 1.2：现代结构 facts、condy 图与 output-level 冲突（提交 `3df96d1`）

### 事实的形状（`crates/jarde-reader/src/modern.rs`）

| 事实 | 类型要点 | origin 回指 |
| --- | --- | --- |
| record components | `RecordFacts { name_index, span, content_span, components: Vec<RecordComponentFacts>, rule }`；每组件含 `name_index`/`name`/`descriptor_index`/`descriptor` + `attributes: Vec<NestedAttributeFact>`（复用 P1 的嵌套 shell 类型） | 组件的 `attribute_name_index`、整套 entry 的 class-file 范围、CP 索引 |
| sealed | `PermittedSubclassesFacts { …, permitted, rule }` | 同上 + 每个名字的 `CONSTANT_Class` 展开 |
| condy 图 | `CondyGraph { nodes, edges, use_sites, cycles, bootstrap_entries, dynamic_tag, budget }`；节点身份 `CondyNodeRef::{ConstantPool{index}, Bootstrap{index}}` | 节点带 CP `span`；边带 `argument_index` |
| modern concat | `ConcatSite { constant_pool_index, span, name, descriptor, bootstrap_index, strategy, factory_owner/name/descriptor, recipe, tag_rule }` | CP 索引 + span + bootstrap 索引 |

**modern concat 归 `jarde-reader`**（结构事实）：只读 CP + `BootstrapMethods` + 句柄 `MethodRef`（reader 已有的三个结构），**不求值 recipe、不解析句柄、不调用工厂**；站点 BCI 归**字节码读取**，join key 是 **CP 索引**——故既不新造也不丢 BCI（真实 `ConcatSample.class` 断言 `concat[i] → CP#13/#17 → BCI 5/2` 全链路）。**与 P3 的 `concat@1` 的关系**：P3 那条是**呈现**（StringBuilder 链），本片是**事实**；模块文档明确写了二者不互相取代。

### A05：condy 图（父级独立复核）

- **行走是显式栈**（`let mut stack = vec![Step::Enter{..}]; while let Some(step) = stack.pop()`）而**非递归**——**结构性**地保证病理图不能消耗宿主栈；**父级直接读代码确认**。
- **visited**：重复节点只记一条 `CondyReach{repeat: Some(序号)}`、**不下钻**；`on_path_ids` + `Step::Exit` 区分「环」与「菱形共享」。手工 fixture 的环 `condy4 → bsm3 → condy5 → bsm4 → condy4` 被记为**事实**而非跟随。
- **预算**（全部走既有 `CountedBudgetDimension`，**未新增维度**）：`IrItems`=节点、`IrEdges`=边、`AnalysisSteps`=每次访问、`DependencyDepth`=深度；**图维度命中 = 停走 + 记录 `stopped`**（其他维度按错误上抛）。实测（手工 fixture）：`nodes=12, edges=15, steps=35, max_depth=6, cycles=2, use_sites=5, is_complete=true`；**共享证据**：两条入口到同一 leaf 的 `via` 在分叉点之后**序号完全相同**（`edges` 只存一份）。四种维度各有命中用例（`ir_edges=3 → stopped=Edges{3}` 且**已走前缀仍发布**）。
- **不执行 bootstrap（关键证据）**：fixture 的句柄是 `REF_invokeStatic p/NoSuchFactory.boom:(...)CallSite`——**本仓库没有任何 class 声明该成员**；断言图完整、句柄是带 span 的 `MethodHandle` 节点、**5 条边只记引用**、且 `edges.iter().all(|e| e.kind != BootstrapHandle || e.to == handle)`（**走到句柄即止，从不跟随它命名的成员**）。若代码试图解析/加载/调用句柄，该用例会失败或 panic。
- 另有用例：indy 句柄的 `reference_index` 指向 `CONSTANT_Utf8` → 只出 `classfile_concat_handle_unresolved`（Info）与「无 concat 事实」，无 panic、无解析。

### output-level 冲突（spec 的 `Record and sealed declarations` scenario）

`OutputLevelStatus` 新增 `Representable { level }` 与 **`Conflict { level, conflicts }`**；`ModernFeature::{RecordComponents, PermittedSubclasses, StringConcat}`、`OutputLevelConflict { feature, level, since, source, origin }`、`ModernOrigin::{ClassAttribute{..}, ConstantPoolEntry{index,span}}`。
**`Conflict` 即「冲突 + fallback」**：冲突项列出每个 feature 的 **origin** 与**决定它的 release 规则**（`since` 60/61/51，`source` 取自 registry），而被判冲突的事实**原样保留**（record 组件、sealed 名单、两个 concat 站点都还在）——**不做等价降级**（真实 `RecordSample`/`SealedSample`/`ConcatSample` 分别 1/1/2 条冲突）。Java 8 目标类（`ConcatJava8.class`）= `Representable`。
`HeaderInspection::output_level` **仍**只由 header 平面产 `NotEvaluated`（负向用例 `the_header_plane_still_evaluates_no_output_level_and_applies_no_attribute_rule` 钉住）。

### registry 接线与 `inspect_header` 的边界

接线在**事实读取路径** `modern_facts`：遍历 class-level shell → `attribute_placement(name, ClassFile, major)` → **`Legal` 才读内容**并作为**版本证据**挂在事实上；`VersionNotApplicable`/`LocationNotApplicable` **不读内容**且报 registry 自己的诊断；`NotRegistered`/`UnregisteredRelease` **不主张**（不读内容、不造诊断、不进事实）。
证据：`Record@60` 合法（`since 60`/`JVMS 4.7.30`，与 `javap` 的 `#40` 同序号）；`Record@59` → 诊断 + `record=None` + **`AttributeBytes` 低于合法读**（内容确实没读）。**`inspect_header` 一字未动**（1.1 的负向用例仍绿；本片另证 header 诊断里无任何 `classfile_attribute_*`）。

### fixture（`tests/fixtures/p4-modern/`，`javac 23.0.1`）

11 个 `.class`（52/60/61），SHA-256 与字节数记入该目录 README。**父级独立复核**：用提交的源文件按 README 的命令**重新编译**，11 个输出**与提交的 `.class` 逐字节相同**（`RecordSample`/`Marker`/`SealedSample`/`Alpha`/`Beta`/`NestSample`+`$Inner`/`ConcatSample`/`module-info`/`p4/sample/Provider`/`ConcatJava8`）。
**产不出（如实记录）**：`javac 23.0.1` 在 8/16/17 三个 release 上**一个 `CONSTANT_Dynamic` 都不产**（`javap -v` 全量扫过 11 个输出 = 0），故 condy 图 fixture 在测试内**手工组装**（与 P1 `p1_xref_bootstrap.rs` 同一约定）；也没有 `makeConcat`（两种 `+` 形状都编成 `makeConcatWithConstants`）。

### 父级独立复核与证据

- 全量 **1023 passed / 0 failed / 3 ignored**（1006 + 17，全部为 `tests/p4_modern_facts.rs` 新增；**无一条既有用例变弱**）；fmt/clippy 1.98.1 干净；`openspec validate --all --strict` 14 passed；两个 CI example exit 0；分层三包中 `jarde-java` **0** 次；**未触及依赖边**。
- **fixture 可复现性**：父级独立重编译 → 11/11 逐字节相同（见上）。
- **实现者三组证伪**：① 去掉边预算记账 / 去掉环记账 → 相应用例红（`cycles left: 0 right: 2`），且**以断言失败终止、不靠栈溢出**；② Java 8 不报冲突（恒 `Representable`）→ 冲突用例红；③ registry 接线失效（版本不适用也当合法去读）→ 3 条红。
- **CI**：`3df96d1` → run 35468561216，四 job success。

### 被修正的既有断言（1 条，未放宽）

`classfile.rs::repository_class_fixtures_validate_without_false_target_rejections`：`(26, 116, 44, 86, 8)` → **`(37, 138, 44, 86, 8)`**（fixtures/bodies 两项）——该测试遍历 `tests/fixtures/**` 下**全部** `.class`，本片新增 11 个样本与 22 个 body。**覆盖未变**：仍逐一 `class_facts` + `method_code_facts` 并要求 body `Complete`；handler/branch/`jsr` 三项计数（44/86/8）**不变**，正说明没有既有 body 退化解码。

### 未完成（应属 1.3 或 2.x）

**1.3**：flag/opcode 合法性**尚未接线**（registry 已登记，读路径未消费）；**无 golden 诊断文件**；`CONSTANT_Dynamic` tag 只给证据不给诊断（1.1 无 tag 类诊断产生器）。
**2.x**：`OutputLevelStatus` 目前只有 `Java8`；`ModernFeature` 刻意只列设计决策 5 点名的三项（condy/nestmate/module 的输出级别答案由后续切片决定）。
**边界（已在模块文档声明）**：只查 class-level 位置（`FileInfo`/`MethodInfo`/`Code`/`MethodParameter` 位置的现代属性不产生 placement 行）。

## 2026-09-20 1.3：非法 fixture、golden diagnostics 与 flag/opcode 接线（提交 `6ed448a`）

### golden 机制（照 `p2_golden` 逐字段同形）

`tests/p4_golden.rs` + `tests/fixtures/p4-golden/{illegal-modern,version-boundaries,legal-modern}.json`：
- **固定回放表**（`replay_list()` 字面量）与每个 JSON 自带的 `replays` 数组**互相比对**——删/改名/加条目即使其余能回放也会失败；
- 每条记录 `fixture` + `source`（provenance 文本，**断言相等**）+ `bytes` + **`blake3`**；回放时**按其具名生成器重建**并断言两者；
- `expect.header` = 整个 header 平面投影（`structural_read`、`version_capability` 逐字、`verification`、`output_level`、诊断 `{code,severity,message}`，逐字段 `==`）；`expect.modern` = 事实侧投影（`attributes[]` 的 name/placement/read、`record_read`、`permitted_subclasses_read`、`concat_sites`、`dynamic_tag`、`output_level`、`conflict_features`、诊断）；
- `expect.strict` = `{error: <code>}` 或 `{read: "complete"}`；被接受的类**另证两种模式给出相同平面**；文件是**静态**的（无测试写它）。
- **与 P2 的两处有意差异**：条目**另钉 strict 结论与事实平面投影**；JSON 里的 `illegal` 标志被**断言必须与其所在文件一致**（`file != "legal-modern.json"`），使条目无法靠改单侧绕过平面不变式。

### 非法 fixture 与可重放性

**未提交新的 `.class` 字节**；两种机制都**逐字节可重放**：

| 生成器 | 形状 | 可重放证据 |
| --- | --- | --- |
| `version_patched(base, major, minor)` | 已提交 `javac` 输出，只补 4..6/6..8 两处版本字段 | 基类的 SHA-256 在 `tests/fixtures/p4-modern/README.md`，结果由 blake3+长度钉住 |
| `jsr_class(52)` | `jsr +4; return; astore_0; ret 0` | blake3 `d63099a9…`，141 字节 |
| `invokedynamic_class(50)` | `invokedynamic` + 置好的 `CONSTANT_InvokeDynamic` + 一个 `CONSTANT_MethodHandle` 的 `BootstrapMethods` | blake3 `2e59c5be…` |
| `nest_members_in_a_method` | class 级 `NestMembers` 放进 `method_info` | blake3 `ad5dd8bd…` |
| `constant_value_in_a_method` | 仅属 `field_info` 的 `ConstantValue` 放进 `method_info` | blake3 `a9f2b7f1…` |
| `acc_module_class(52)` | class 访问标志字 `0x0021 \| 0x8000` | blake3 `b0c3e1e7…` |

**为何用派生而非再提交一份字节**：补丁是**可执行且带摘要断言**的（比同一份字节的第二份拷贝更强，后者会漂移），而其余每个字节都可证仍是已提交的编译器输出。

### 接线落点与 `inspect_header` 边界

`inspect_header` **一字未动**（1.1 的负向用例仍绿）；所有新诊断产在 `modern.rs::modern_facts`（事实路径），在 registry 诊断块内、先于该 pass 自身的结构诊断。成员属性经新 `member_attribute_diagnostics` 判位置；**opcode 合法性在事实路径**，opcode 取自字节码读取器自己的游标（新 `pub(crate) fn classfile::method_body_opcodes`——重复实现解码器不是选项）；**flag 只陈述 release 主张**（`rule.since > major` 且该规则确实声明该结构）；**tag 不新造 code**（`CondyGraph::dynamic_tag` 即已发布的证据）。

### 两处**必须上报**的发现

**发现 1（阻塞点，已绕开）**：把 `classfile_opcode_forbidden` 加到 `inspect_method_bytecode` 会**改动 P2 已冻结的回放**（`tests/p2_golden.rs:492` 的 `illegal-version-jsr-in-52`），故 opcode 合法性改接在**事实路径**；字节码路径**仍不对 release opcode 发言**。

**发现 2（registry 缺口，实现者自查发现）**：**逐位解析 flag 是不成立的**。其第一版把每个置位解析为已登记名，跑**真实字节**时在 `RecordSample.class` 上产生 **8 条假诊断**（例如对 class、其字段与方法都报 `ACC_FINAL (0x0010) is registered only for MethodParameters`）——**registry 记的是每个 release 「引入」了哪些 flag，不是任一结构的 flag 词表**，故某位置上的 `LocationNotApplicable` **不能读成「非法」**。`ACC_RECORD` 同形（既无规则，又与 `ACC_SYNTHETIC` 共用一个位）。**后果**：**不在任何地方**做 flag **位置**主张；知道名字的调用方直接查 `flag_placement`/`flag_diagnostic`。由 `a_flag_is_answered_by_name_and_never_inferred_from_a_bit` 钉住，并写在 `modern.rs::flag_diagnostics` 的文档里。**扩表成全位置词表，还是维持按名驱动，是 1.1/2.x 的决定，不是本片的。**

**父级独立复核该发现**：对 `tests/fixtures/p4-modern/` 的**全部 11 个真实类**跑 `class_facts` + `modern_facts` → **flag 诊断 0 条、诊断总数 0 条**（探针已删）。即「逐位解析」的假诊断**确实不在交付版本里**。

### 「不冒充成功」的用例

`an_illegal_class_file_is_read_and_its_planes_stay_separate`——对**每一条**非法条目（9 条）逐项断言：`structural_read == Complete`（**诊断不是读失败**）、`verification == NotPerformed`（**无 verifier 跑过**）、`output_level == NotEvaluated`、dialect 平面与记录一致（类型 + JSON 双向）、诊断码（header ∪ modern）非空且全为 `classfile_*`，并另断言 `validated_releases >= 1` 且 `probed_releases >= 1`——**最后两个计数才是要点**：本 build **校验**的 release（52 = `Supported`）**照样可以带 release 规则违规**（`ACC_MODULE` 位），而它只**探测**的 release（59/60/61）**照样被完整读出**。另有 `a_refused_placement_is_a_row_and_not_a_fact`（出行、`read=false`、无组件）与合法对照。

### 实际诊断（取自已提交的 golden）

`classfile_attribute_version_not_applicable`（`Record`@59、`PermittedSubclasses`@60）、`classfile_attribute_location_not_applicable`（`NestMembers`@method_info、`ConstantValue`@method_info）、`classfile_flag_version_not_applicable`（`ACC_MODULE`@52）、`classfile_opcode_forbidden`（`jsr`/`ret`@52）、`classfile_opcode_version_not_applicable`（`invokedynamic`@50）、header 侧 `classfile_future_release`@72（结构仍 `complete`）、`classfile_invalid_modern_minor_version`@60.7、`classfile_version_structural_probe_only`/`classfile_java8_runtime_rejected`。**每条消息都点名属性/flag/opcode、release 门槛与 JVMS 引用**。

### 父级独立复核与证据

- 全量 **1029 passed / 0 failed / 3 ignored**（1023 + 6 新用例）；`p4_golden` 6 passed；fmt/clippy 1.98.1 干净；`openspec validate --all --strict` 14 passed；两个 CI example exit 0；分层三包中 `jarde-java` **0** 次。
- **既有断言零改动**（父级核：删除行中 `assert` 计数 **0**；唯一被替换的是 `modern.rs` 的一行文档注释）。
- **实现者三组证伪**：① 位置拒绝改 `Err` → 3 红（含对「诊断即读失败」的证伪）；② `release()` 把 major 夹到 71 → 2 红（`future_release` vs `structural_probe_only` 等）；③ `inspect_header` 报 `verification: Performed` → 3 红。副本 323/323 文件校验无差异。
- **CI**：`6ed448a` → 见下。

### 未完成（应属 2.x/3.x）

**flag 位置主张与完整 flag 词表**（registry 表决定，见发现 2）；`MethodParameters` 的**参数 flag** 与 module/nest/annotation 成员属性的**内容**仍归各自 pass；`method_body_opcodes` **重解析 class 并每 body 计一次 `CodeBytes`**（不发布事实故不计 `ClassBytes`/`ResultItems`）——成本/记账口径可能要在 2.x 的事实接线时重看，且它**容错**（解不出的 body 不陈述 opcode 而非让读取失败）；事实侧 `output_level` 目前只问 `Java8`（`OutputLevel` 现仅一个变体），多档输出问题属 **2.1/3.3**。
