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

## 2026-09-20 2.1：RuntimeMatrix、模块/loader policy 与 MR/layout 选择（提交 `c2676d4`）

### 归属与形状

**归属 `jarde-reader`**（`crates/jarde-reader/src/runtime_matrix.rs`；`src/facade.rs` 加 `Engine::runtime_matrix` 一行委派）。理由：矩阵的全部输入都是 reader 自有事实（一次物理枚举、`multi_release` 选择、`view` 的 profile/LoadDomain）；放到 reader 之上要么每 profile 重扫（违反决策 2），要么另写一份 MR 选择（明确禁止）——`multi_release::select` 的重扫逻辑在 reader 内部，只有这里能真正共享。jvm 的 `ResolutionState` 属 2.2，且 jvm 依赖 reader（分层不允许反向）。

**输入** `RuntimeMatrixRequest { physical, profiles: Vec<RuntimeProfile>（1..=8 且互不相同）, domains, requester }`。
**每 view**（`RuntimeMatrixProfile`）：`view`、`policies`、`layout`、`loader`、`definitions`、`containers`、`coverage`/`execution`/`diagnostics`/`usage`。`definitions` 按**运行时名**聚合，每 `RuntimeDefinitionOrigin` 含 container、logical path、**选择函数** `rule` 与**每条物理 entry**（decision/compliance/root/on_path）；`candidates()/unselected()/selected()` **由同一份 decision 派生**，不会与选择函数互相矛盾。
**选择函数**（纯投影）：`HighestReleaseAtOrBelowTarget{release,target}`、`BaseSelected{reason}`、`Ambiguous{entries}`、`NoSelection{reason}`、`Unknown{reason}`——自定义/未知策略保持既有 `unsupported_policy` 语义，**绝不呈现为某个具体选择**。

### 共享扫描（决策 2 的核心）——**父级独立复核**

- **结构**：`multi_release::select` 拆为 `physical_evidence()`（唯一枚举）+ `select_over_physical()`（在既有证据上选择），`select` = 两者，**行为不变**（p1 MR 29 条、golden 6 条、query model 5 条**一字未改即通过**）。父级读码确认：**`physical_evidence` 调用在每 profile 循环之外**，循环内的 `select_over_physical` 读**零** archive 字节。
- **证据是测得的恒等式**（不是写死的数字）：`matrix 总成本 == 3×扫描 + Σ每profile == 三个 standalone 运行的总和`；`scan_times_three_plus_profiles(&matrix) == baseline`；`rescan_projection().archive_entries == shared.archive_entries * 3`。实测平坦 jar：矩阵 `archive_entries=34` vs 三次独立 `42`（省下**两次未发生的枚举** = 8 条目录记录）；WAR 树：`11/331` vs `27/717`（另省 **386 字节**）。每 profile 的 220 字节是 compliance probe **自己的**重读，报告**如实分开呈现**。
- 注：`scan.physical_scans == 1` 是结构性事实（一处调用点），**测量的部分是上述恒等式**——父级在此点明，以免后人把它读成计数器结论。

### 选择函数压缩的判据

签名 = 每个 `(binary_name, container)` 的**选中结果**（`Selected`/`Undecided`/`NoSelection`/`Unknown`），**不含规则文本**（否则 target 会让同一答案看起来不同）。可证区间来自同一份扫描：target release 作自变量时，分段点**正好是枚举到的 versioned release**，故 `[r, 下一 release-1]`；`PolicyDisabled`/`ManifestNotActive`/`NoVersionedCandidate` → `[0,∞)`；`TargetBelowNine` → `[0,8]`；`ReleaseAboveTarget{nearest}` → `[9,nearest-1]`；`Ambiguous`/`NoSelection`/`Unknown` → **无区间**。合并 = 签名相等 **且** 各成员区间之交包含所有成员 release；报告区间即该交集。
实测：8/11/17 → **三组单例**（未塌缩）；11/18/21/25 → 18/21/25 合并；**反例**：两个 `Custom{id}` profile 答案相同但无区间 → 两组 `NotProvable`。

### module/loader policy 的边界（如实）

**能表达**：`ModuleMode::ClassPath`；`ParentFirst`/`ChildFirst` 可产生跨 loader 的显式根次序；`LoadRoot::Snapshot/ArtifactTree` 覆盖判定按 origin 链；`External` 根不覆盖本快照容器。
**不能表达**：`ModulePath`/`Hybrid`/`Custom`/`Unknown` 一律 `Unsupported{detail}`（与 `jarde-jvm::providers`/`environment` 的拒绝一致，矩阵把它作为**数据**报告并**放弃顺序主张**）；具名模块可读性（`requires/exports` 只被测量以便游标前进）；`Custom`/`Unknown` delegation、父 loader 未提供、`external_override`/`runtime_transformation = Unknown` → 顺序 `Undetermined`、候选 `RootAttribution::Undetermined`（**不给位次**）。**layout**：改前 `LayoutMode` 全仓库**无消费者**；矩阵把树行走器已发布的 `LayoutNode` 按模式投影为层 + 在路径容器 + 运行时名，`Custom`/`Unknown` → `Unsupported`、`on_path = None`。

### A06/A07 用例

- **A06**：Java 8 → `BaseSelected{TargetBelowNine{8}}`；11 与 17 各选自己的版本化条目；**三个答案同时在矩阵里**（断言 answers == [base, v11, v17]）；`physical_entries().len()==4`、每 view 列出全部 4 条、每定义列全部 3 条候选、`unselected().len()==2`。Manifest：未激活 + 有版本条目 → base 规则 `ManifestNotActive` + `RuntimeMatrixVersionedEntriesInactive`（**条目仍列出**）；manifest 激活但缺 public 前驱 → 选中条目 `NonConformant` + `RuntimeMatrixSelectedEntryNonConformant`（Error）+ 转发的 `MultiReleasePublicPredecessorMissing`。
- **A07**：`p/Dup.class` **两个 origin**（WAR 根容器 vs 嵌套 `WEB-INF/lib/a.jar`），ordinal/origin 不同；`roots=[Snapshot]` → `Ambiguous{position:0, containers:2}`；`roots=[ArtifactTree{嵌套}, Snapshot]` → `Ordered{position:0}`；**parent-first 子 loader + parent Snapshot → 回到 Ambiguous，child-first → Ordered**（声明真的改变答案）；`Custom` delegation → `Undetermined`、`search` 空、候选 `Undetermined`，**选择仍然给出**。

### 父级独立复核与证据

- 全量 **1040 passed / 0 failed / 3 ignored**（1029 + 11，全部为新文件）；`p4_runtime_matrix` 11 passed；fmt/clippy 1.98.1 干净；`openspec validate --all --strict` 14 passed；两个 CI example exit 0；分层三包中 `jarde-java` **0** 次；锁文件两条 exit 0（未触及依赖边）。
- **既有断言零改动**（父级核：删除行中 `assert` 计数 **0**）。
- **父级独立证伪**：把每个 view 的 release 换成常量 17（即**塌缩成单一答案**）→ `three_profiles_select_different_entries_and_keep_every_physical_entry` **恰好 1 红**，其余 10 绿——证明「不塌缩」这条断言**承重**。
- **实现者三组证伪**：① 塌缩 → 红（`left: HighestReleaseAtOrBelowTarget{release:17,target:17}` / `right: BaseSelected{TargetBelowNine{target:8}}`）；② 不列未选择 entries → 4 红；③ 不可证相等却合并 → `agreement_without_a_proven_interval_is_never_merged` 红（`left: 1 / right: 2`）。副本 130 文件清单只报一个差异文件。
- **CI**：`c2676d4` → 见下。

### fixture

**未新增 fixture 文件**（沿用 `tests/fixtures/README.md` 记录的**内存生成器**约定：`divergent_jar()` 与冻结的 P1 生成器同形——base/v11/v17 的 `p/Join.class`、major 52/55/61、manifest 字节相同、STORE；`war_bytes()` 沿用 `nested_fixture` 的 `p/Dup.class` 置入 `WEB-INF/lib/*.jar`，另加类目录层与一条不在路径上的条目；未激活 manifest 与缺前驱两个 jar）。
**未产出/未尝试（如实）**：`WEB-INF/classes/` 前缀之下的 MR version 目录（既有 MR 层按 `META-INF/versions/` 识别，矩阵只剥离树行走器发布的层、**不发明**）；具名模块顺序；兄弟 loader 并存请求。

### 未完成（如实）

**2.2**：declaration resolution/dispatch 候选查询（缺失依赖、default conflict、open-world）；`ResolutionState::Ambiguous` 等解析态留在 resolver，矩阵只暴露它需要的物理 origin/候选；兄弟/多请求 loader 的解析序。**2.3**：有界 X3 patterns。**3.x**：plugin、文档、矩阵与最终门禁。**未做**：矩阵的 CLI 出口（`query` 未扩展）。

## 2026-09-20 2.2：X2 三态、缺失依赖与 default conflict（提交 `2472887`）

### 本片修掉了 A11 的一个**真实违反**

**规格句**（A11）：`Base.foo 声明、Sub CP owner | P2，P4 深度扩展 | symbolic 原 owner；宽候选/继承扩展后 resolves_to；**缺失依赖不能变否定**。`
**实际缺陷**：层级行走**读不到**某个父类时，解析仍发布 `ResolutionState::Missing`——而 `Missing` 的含义是「**搜索覆盖了全部、什么都没找到**」，由一个**没有覆盖全部**的搜索给出。**7 条既有断言**（`p2_closure` 1 + `p2_members` 5 + `p2_resolution` 1）把同一主张编码下来，这正是它能存活的原因。

### 扩展而非平行模型

**既有**（未动语义）：`ResolutionState` 七态、`DispatchReport{candidates, open_world}`、`OpenWorldEvidence`、`ReadReason`。
**本片补的**（全是既有实体的扩展）：
- `ResolutionState` **加一个变体** `UnresolvedDependency`；
- `ResolutionReport` **加一个平面** `unresolved_dependencies: Vec<UnresolvedDependency>`，`UnresolvedDependency { name, loader, reason: ReadReason, declared_by: Option<JvmBytes>, gap: DependencyGap }`，`DependencyGap { Missing, Ambiguous, Cyclic }`；
- `WalkGaps` 由**三个名字数组**改为**记录表**（`HierarchyGap { name, loader, demand, declared_by }`）；`Search.unread: u64` → `Search.gaps`；`MemberOutcome.hierarchy_complete: bool` → `MemberOutcome.unread: WalkGaps` + `hierarchy_complete()`；`DispatchOutcome.unread_branches: bool` → `unread: WalkGaps`；
- 抽出 `read_reason(HeaderDemand)`，使 `reads` 与 `unresolved_dependencies` 对同一条边用**同一个公开词汇**（消除双份映射）。

**关键**：名字因此**第一次以结构化形式**发布（此前只出现在诊断**文字**里）。

### 三态在类型上分开

| 面 | 类型 | 语义 |
| --- | --- | --- |
| declaration resolution | `state: Option<ResolutionState>` | **仅当每个进入的分支都被读过**才允许 `Missing`（`member_planes` 以 `unread.is_empty()` 守卫） |
| possible dispatch | `dispatch: Option<DispatchReport>` | `candidates` 是**已知候选**，**无「唯一目标」字段** |
| open-world / unknown | `Option<OpenWorldEvidence>` + `open_world: bool` + `unresolved_dependencies` | 事实与名字并列，**不是 confidence 数字** |

### A11 的核心断言与**对照**（父级独立复核）

`tests/p4_x2_states.rs::a_missing_dependency_is_stated_by_name_and_never_as_the_negative_answer`：
- 缺陷面：`p/Orphan extends p/Absent` 且 `p/Absent` 不在快照 → `UnresolvedDependency`，**并 `assert_ne!(state, Some(Missing))`**，且**整结构体钉住** `{name: p/Absent, loader: app, reason: ParentChain, declared_by: p/Orphan, gap: Missing}`，`resolved.is_none()`；
- **对照（这是关键）**：同形状但父类**在场** → `Some(Missing)` **且 `unresolved_dependencies.is_empty()`**。即「每个分支都读过时，否定正是答案」——**证明修法不是「一律不说 Missing」**。

### default conflict

**沿用既有语义，不新增概念**：声明解析面 `IncompatibleClassChange` + 诊断 `resolution_default_conflict`，由 `maximally_specific`（JVMS 5.4.3.3 两步）在解析期判定。**理由**（既有文档的理由，本片复核并写进断言）：JVMS 8 把该失败放在 invocation selection，本片**刻意提前一步**，好过让调用方看到一次「沉默任选」；链接错误既不是 open-world 事实、也不是候选集合，故**不进 dispatch 面**。
**避免过度报告**（各一条，均断言**不含** `resolution_default_conflict`）：① 子接口的 default **覆写**父接口的 → `Resolved` 到子接口；② 类自带 `m` → `Resolved` 到类（类链先命中）。冲突侧：两条**不相交**接口 + 类未覆写 → ICCE、`resolved` 空、`dispatch` 空。

### open-world（`OpenWorld dispatch` scenario）

- 候选逐条带事实；范围**命名它读不到的类**（1 候选 + `MissingDependency` + `open_world` + `i/Gone`/`HierarchyClosure`/`declared_by p/Sub2`）；
- **链外 loader** → `UnknownLoader`、`open_world=true`、无缺口名；
- **范围位置解析到范围外** → 0 候选、`open_world=true`、`CoverageState::Partial`（**是「未搜」而不是「排除」**）；
- 「**不声称唯一目标**」在**编译期**钉住：测试对 `DispatchReport`（3 字段）/`DispatchCandidate`（2 字段）**穷尽解构**——**加一个运行时目标字段就编译失败**。

### 查询出口与分层

**不加 relationship**：状态经既有 `Engine::resolve_symbol`（X2 公开入口）读取。`crates/jarde-query/**` **一字未改**，`QueryRelation` 集合不变。
**schema 变化**：新增键 `unresolved_dependencies` + 新增 `state` 取值 `unresolved_dependency` → **JSON 层加字段**（向后兼容读取），**Rust 层破坏性**（穷尽 match 需补臂，`tests/p2_resolution.rs::resolution_state_code` 正是这样把它抓出来）。新平面像 `reads` 一样是**证据、不计费**（`ResultItems` 未变，既有预算断言零改动）。
**分层**：改动只在 `crates/jarde-jvm` + facade 文档；12 条 `cargo tree` closure 检查全绿。

### 与 2.1 RuntimeMatrix 的关系：**独立，不接矩阵**

一个 `ResolutionRequest` 只命名**恰好一个** `RuntimeView`，报告用 `environment_identity.runtime` 声明它属于哪个 view；2.2 的状态是**单个 view 的属性**。故多 profile 的正确接法是「矩阵选出 view → 每个 view 跑一次请求」；**反向依赖（resolution 去问矩阵）才是分层倒置**。用例 `the_states_of_one_request_belong_to_the_one_view_it_names` 在 8/17 两个 release 下断言「答案命名它自己的 view」。

### 既有断言的修正（7 处，**全部加强、无一处放宽**）

| 位置 | 原 | 新 |
| --- | --- | --- |
| `p2_closure.rs:617` | `Some(Missing)` | `UnresolvedDependency` + **整条依赖钉住** + `resolved.is_none()` |
| `p2_members.rs:1220` | `Some(Missing)` | `UnresolvedDependency` + `{p/Absent, ParentChain, declared_by p/Orphan, Missing}` |
| `p2_members.rs:1281` | `Some(Missing)` | `UnresolvedDependency` + `gap: Ambiguous` |
| `p2_members.rs:1344`（field） | `Some(Missing)` | `UnresolvedDependency` + `gap: Cyclic` |
| `p2_members.rs:1376`（method） | `Some(Missing)` | 同上，且与 field 报告**相同**的依赖列表 |
| `p2_members.rs:3339` | `Some(Missing)` | `UnresolvedDependency` + `{p/Self, parent, ParentChain, Self, Cyclic}` |
| `p2_resolution.rs::resolution_state_code` | 7 臂 | +1 臂（**编译期强制**） |

**父级独立复核**：`tests/` 的 diff 为 **+100/−7**（净增）；5 条被删的 `assert_eq!` **全部**被更长的整结构体断言取代，且每处都**新增**了「不得是否定」或「依赖必须被点名」的断言。其余 1035 条既有断言一字未改。

### 父级独立复核与证据

- 全量 **1048 passed / 0 failed / 3 ignored**（1040 + 8，全部为 `tests/p4_x2_states.rs`）；该文件 8 passed；fmt/clippy 1.98.1 干净；`openspec validate --all --strict` 14 passed；两个 CI example exit 0；分层 12 条 closure 全绿；锁文件两条 exit 0（未触及依赖边）。
- **实现者三组证伪**：① `member_planes` 恢复无条件 `Missing` → A11 核心用例红（`left: Some(Missing) / right: Some(UnresolvedDependency)`）；② dispatch 去掉证据与 open-world → 2 红；③ `Members::extends` 恒 false（去 maximally-specific 规则 2）→ 不冲突用例红（`left: Some(IncompatibleClassChange) / right: Some(Resolved)`）。副本三个源文件校验 OK。
- **CI**：`2472887` → 见下。

### 未完成（如实）

`DeclarationRefReport` **没有**新增 `unresolved_dependencies` 平面（它既有 `unresolved_candidates` 计数器 + 逐候选诊断本就**不把**缺依赖的候选当排除；本片改了 `undecided_reason` 使诊断文字点名读不到的类）——若要结构化列表，那是对 **2.4** 自身契约的加法。
**留给复核的契约选择**：歧义分支与环分支目前与「名字不存在」**共用同一状态** `UnresolvedDependency`，区分由 `gap: Ambiguous|Cyclic` 承载（理由：`Ambiguous` 在本仓库已专指「同一选择位上多条成员声明不可区分」，而分支未读意味着成员**根本没被搜到**，合并会丢事实）。若要拆成多个状态，属公开契约决定。
**2.3**：X3 有界常量传播/`pattern_inferred_target`/动态 `Unknown` 本片一行未动；2.2 只处理 X2 事实，**不写 X1 原始边**。

## 2026-09-20 2.3：有界 X3 reflection/ServiceLoader（提交 `886f814`）

### 登记的模式（`crates/jarde-jvm/src/reflection.rs` 的 `PATTERNS` 表）

照 `release_registry.rs` 的「登记 + `not_claimed`（测试强制非空）+ `source`」风格，每条记**五要素**（API overload / 常量输入 / 传播范围 / loader 假设 / rule version）+ `support`；`RuleVersion` 是 jvm **自有**类型（jvm 不许依赖 `jarde-java`）。

**登记并可推断**：`class-for-name`（`forName(String)`，`Argument(0)`，loader = CallerDefiningLoader，**可查**）；`class-for-name-loader`（`forName(String,Z,ClassLoader)`，ExplicitLoaderArgument，**不查任何 order**）；`class-get-method`/`-declared-method`/`-field`/`-declared-field`（成员名 = `Argument(0)`、类 = `Receiver` 类字面量，**两个输入都必须可证**）；`methodhandles-find-static/-virtual/-special/-getter/-setter`；`service-loader-load`（`load(Class)`，loader = **ThreadContextLoader**）与 `service-loader-load-loader`。

**如实登记为做不到的 4 条**（`PatternSupport::Unsupported` + reason）：`methodhandles-find-constructor`（成员名由 API 固定为 `<init>`，`MethodType` 不传播，名字级推断只会复述类参数且不校验）、`class-get-declared-methods`、`class-get-constructors`、`class-get-declared-fields`（枚举型 overload 无名字常量可读）。
**未登记的 overload 不是「不支持」而是「不主张」**——不产站点记录（`getSuperclass` 有专测）。

### 落点与理由

**`jarde-jvm` 的新模块 `reflection.rs`**，出口 `Engine::reflection_patterns`（与 `resolver`/`ir`/`environment` 同一交叉方式）。理由：传播必须是**本方法 SSA**（SSA/frames/canonical 全在 jvm），loader 假设需要 jvm 私有的 `ResolutionEnvironment`/`HeaderClosure`，而「名字解析到哪个定义」只能走 2.2 的 closure；放进 xref 扫描既拿不到值流也拿不到 loader，且 `jarde-query`/`jarde-jvm` 不许反向依赖。
**照 2.2 的纪律**：**不加 `QueryRelation`**（会造出 jvm→query 的语义倒置）、证据平面**不计费**、**复用 2.2 的记录类型**（`UnresolvedDependency`/`ReadReason::{PatternScan,PatternTarget}`/`DependencyGap`）与 2.5 的 `enumerate_range`/`covered_by_scope`（改成 `pub(crate)`，diff 只有可见性）。

### 有界传播（**复用而非重写**）

每个含已登记 overload 的**方法体**跑**一次既有 `analyze_method_ir`**（`AnalysisStage::ALL`）——站点、指令、pool、SSA **全部来自那一次 run 自己的 decode**，**没有第二次解码**。类级先用 header pool 预筛（`names_a_pattern`），不命中的类**一个 body 都不读**。
**常量源**：`ldc/ldc_w`（String/Class）与 `getstatic`（**本类**声明、`ConstantValue` 为 `Ljava/lang/String;`，经 `attribute_facts` 从同一次读的字节解出）。
**有界**：`replaced_by` 链 ≤8、别名链（store→load）≤8 步，**不跨方法**、不追 callee 返回值；**预算 = run 自己的既有维度**（`MethodBodies`/`ClassBytes`/`CodeBytes`/`AttributeBytes`/`IrItems`/`IrEdges`/`AnalysisSteps`），站点 code 用 `budget_exceeded_<dimension>`。
**假设如实标注**：`ldc` 源 `assumption = None`（方法自身字节的常量不会变）；`getstatic` 源带文本假设「`ConstantValue` 是初始值，而静态字段是进程状态——**本快照不持有的写者未被排除**」；loader 假设逐条带 `statement`（caller → 可查；thread-context/explicit → **明说不查 order**）。

### 五要素如何被断言 / `Unknown` 判据

`PatternInference { rule, rule_version, constant_input, target, propagation, loader, resolution }` 与 `ReflectionSite` 都是 `#[serde(deny_unknown_fields)]` 且被测试**穷尽解构**（**加字段即编译不过**）——「不能缺要素」是**编译期**性质。JSON 拼写钉在 **`pattern_inferred_target`**（规格原词）。
`Unknown` 判据：目标名或 owner 的值**不是本片可证的常量** → 按 SSA 定义给 `DynamicInput{input, origin}`（`Entry` 参数/`this`、`Merge`、`Caught`、`Instruction{bci,opcode}` 其它形状、`NoDefinition`、`ChainBeyondBound`）；body 解码存在但 run 停了 → `AnalysisStopped`；登记为 unsupported → `PatternNotSupported`。**不产任意 confidence 数字**。

### 边界情形的判据（各带理由与测试）

- **常量名但类不在快照** → **inferred** + `resolution = NotInSnapshot{loader}` + `unresolved_dependencies` 一条（`ReadReason::PatternTarget`、`gap = Missing`）。判据：**推断是「调用点要什么」，快照只能给「它有什么」**——两者分开发布；**读不到名字不是「类不存在」**。对照：类在场时同一站点 `Resolved` 且依赖列表为空。
- **通配 `p/*`** → inferred + `NotInSnapshot`（`*` 是合法 identifier 字节，JVMS 4.2.1，只是没人声明）；**非法名 `p/.Hidden`** → **inferred 但 `NotDemanded`**，字节原样发布，**不搜索 order、也不记缺依赖**——把畸形串当「缺失的类」是错的。
- **`ServiceLoader.load` 的接口类不在快照** → inferred + **`NotDemanded`**，**连缺依赖都不记**（规则声明的 loader 是 thread context，本请求没有可走的顺序）。

### 「不执行」的证据

`coverage.dynamic_analysis == NotRequested`（有断言）；代码路径只有 reader facts + 本方法 SSA——无 bootstrap/反射调用/launcher/JNI/网络；`getDeclaredMethod("foo")` 在目标类**不声明** `foo` 时**仍给名字级推断**（不求值、不解码目标）；同一 fixture **去掉目标类**后 Member/ServiceLoader 站点的 `target`/`constant_input` **逐字段相同**；类不在快照**不 panic**；`Class.forName` 的目标**从不被 decode 或 member-resolve**。

### 与 X1/X2 的可区分性

`ReflectionSite` 是**独立类型**（穷尽解构钉住，加 X1 字段即测试编译失败），状态名与 X1 的 `derivation/certainty` 不共用；**未改 `QueryRelation`、未改 `XrefDerivation`、未向 X1 边流写入任何东西**。X2 的字段一个未改，只**新增**两个只读证据/读取语义词（`ReadReason::{PatternScan,PatternTarget}` 与 `HeaderDemand` 同名变体），X2 既有测试 233/233 全绿。

### 父级独立复核与证据

- 全量 **1065 passed / 0 failed / 3 ignored**（1048 + 17）；`p4_x3_patterns` 10 passed；fmt/clippy 1.98.1 干净；`openspec validate --all --strict` 14 passed；两个 CI example exit 0；分层三包中 `jarde-java` **0** 次；锁文件两条 exit 0（未触及依赖边）。
- **既有断言零改动**（父级核：`tests/` 与 `crates/` 的删除行中 `assert` 计数 **0**；8 行删除全是可见性/`pub use` 换行）。
- **父级独立证伪**：让**不可证的输入变成猜测**（把 `Entry`/`Phi` 一律当常量返回）→ **恰好 1 红**：`a_dynamic_input_is_unknown_and_never_a_guess`，其余 9 绿——证明规格「**不把 Unknown 变成猜测**」这条**承重**。
- **实现者三组证伪**：① 同上（另 5 条也红）；② 传播无界 + 停止不记前缀 → **恰好 1 红**（`a_budget_stop_is_reported_and_never_guessed`）；③ inferred 不带假设（`statement: ""` + 空 `RuleVersion`）→ **恰好 1 红**（五要素用例）。副本校验：编辑前 214 OK，编辑后 213 OK/1 FAILED，仓库事后 214 OK。
- **CI**：`886f814` → 见下。

### 未完成（如实）

`PatternTargetState::Ambiguous`、`DynamicOrigin::{Caught, ChainBeyondBound, NoDefinition}`、环境被拒 → `Analysis = NotPerformed` 的 X3 路径、「listing 截断」诊断路径、`resolution_pattern_body_not_analysable` 分支**均无 fixture**；`ServiceLoader.stream`、`Class.getConstructor` 等 overload **未登记（=不主张）**。
**属 3.x**：把 `PATTERNS` 暴露为版本化 descriptor（3.1）、只读 plugin fixture（3.2）、现代支持矩阵与 A04–A07/A12（3.3）、X3 的 CLI/JSON 出口（本片只有 facade 入口）。

## 2026-09-20 3.1 + 3.2：plugin descriptor、未注册=Unsupported 与只读 framework fixture（提交 `9b37ea8`）

### descriptor 与注册表（3.1）

`crates/jarde-query/src/plugin.rs`：`PluginRule { id, rule: PluginRuleVersion(name@version), input: PluginInputCategory, config_path: PluginConfigPath{prefix,declares}, schema: PluginOutputSchema{name,version,fields}, evidence, coverage, budget: PluginBudget{dimension,statement}, source, isolation, not_claimed, support }`。
**注册表风格照 2.3 的 `PATTERNS` 与 `release_registry`**：`PLUGINS` 常量表 + `plugins()` + `plugin_for(id, version)`（**id 与 version 双维度字节精确**）+ `PluginSupport::{Performed, Unsupported{reason}}`；`not_claimed`/`source`/`isolation` **非空由单测钉住**。两条登记：`service-loader-registrations@1`（Performed）与 `spring-factories@1`（**登记但不执行**，带 reason）。

**未注册 = Unsupported 的专测**（`an_unregistered_configuration_is_unsupported_and_never_an_empty_answer`）**区分三种拒绝**：id 未注册 → `plugin_rule_not_registered`；**版本**未注册 → `plugin_rule_version_not_registered`（消息点名本 registry 持有的版本）；已注册但未执行 → `plugin_rule_unsupported`（带登记 reason）。三者均 `items: []` + `coverage == not_requested()` + 请求级 `PluginAnalysis::NotPerformed{code}`。
**对照**：只含注释的 services entry → `Performed` + 0 item + coverage `CompleteWithinSchema` 且 scanned **点名读过的 entry**——即「**声明为空**」才是 0 item，**拒绝不是**。
**父级独立证伪**：让 `plugin_for` 对未注册配置**回退到第一条已登记规则**（即抹掉「未注册」与「跑过且没找到」的区别）→ **恰好 1 红**：`an_unregistered_configuration_is_unsupported_and_never_an_empty_answer`，其余 4 绿。

### 落点与理由

**`crates/jarde-query/src/plugin.rs`**：plugin 的输入就是 archive resource entry，即 query 层自己的资源消费面；放 `jarde-jvm` 会倒置分层（jvm→query），放 `jarde-reader` 则把「规则登记 + 独立 coverage + 计费纪律」塞进事实层。
**格式所有者复用（仅可见性）**：`xref/resource.rs` 的 `SERVICES_PREFIX`/`service_key`/`service_providers` 与 `xref/mod.rs` 的 `escaped_raw_name`/`to_u64` 提升为 `pub(crate)`，plugin 用**同一个** matcher 与**同一个**解析器读同一批字节（**无第二份格式实现，行为零改动**）；descriptor 声明的 prefix 与共享常量比对，不一致则 **fail-closed 不读任何 entry**。
**facade**：`Engine::plugins(...)` 一行委派；`src/lib.rs` **只导出产品类型与 `plugins`/`plugin_for`/`PLUGIN_TRUST_DOMAIN`**（不导出 module path；`plugin::execute` 与 `query::execute` 一样留在下面）。

### 约定选择与 fixture（3.2）

选 **`META-INF/services/<service-interface>`**：① 它正是 2.3 明确留下的声明面（`service-loader-load` 的 `not_claimed` 写着「no `META-INF/services` resource is opened」）——两条规则**互补不重叠**；② 语法由 `java.util.ServiceLoader` javadoc 规定，真实可验证；③ 通用结构面已对同一批字节有 target-driven 视角，**分隔可测而非纸面**。
fixture **测试内生成**（`tests/p4_plugins.rs::archive()`，`rawzip` writer，同其他 P4 套件）：4 个 entry = manifest、`META-INF/services/com.example.Service`（124 字节，sha256 `75a4a087…e225c`，含注释/空行/续行/**不存在的类**）、`com/example/Main.class`（major 52，一个 `native` 方法）、`docs/readme.txt`。**未新增 fixture 文件**，故无文件 SHA。

### 不改变 P1/P2 原始事实（**父级独立复核**）

`the_plugin_plane_leaves_the_structural_planes_own_answer_untouched`：同一 snapshot 上 plugin **前后**各跑一次 X1 `QueryReport` 与 `EnumerationReport`，**serde JSON 逐字段相同**——而 **plugin 确实跑了**（`returned_items == 3`）。**两者缺一不可**：一份没被动过的报告在「声称动它的东西根本没跑」时不证明任何事。
**分隔的表达**：① plugin item 带 `rule`/`rule_version`，X1 item 带 `relation/source/target/...` 且**证据属主断言** `evidence.attribute == Some(ArchiveNameBytes(b"com.example.Service"))`、`via` 空，另加三个身份串**均不出现在结构面**；② plugin 报告**无** `relation`/`consumers`，且 item 的 JSON 键集合 **== descriptor 声明的 `schema.fields`**（形状归 schema 而非结构面）；③ 选择规则不同：X1 对未提及的名字 → 0 item，plugin 从同批字节给出 3 条声明；④ 类型层面 plugin 平面**无** `QueryRelation`/`XrefItem`/X1 edge。

### 预算生效（`Plugin sees bounded input` scenario）

descriptor 声明 `budget.dimension = result_items`（1 item 1 单位，**发布前**计费），另由读取维度（`archive_entries`/`entry_bytes`/`read_bytes`）与 `elapsed_millis` 界定。专测 `a_budget_refusal_is_a_stop_with_the_range_it_read` 覆盖五种：
**(a)** `result_items` 不足 → 发布 2 条、`has_more`、`Skipped{BudgetExceeded{ResultItems}}`、coverage `Partial` 且 **scanned 点名读过的 entry**、诊断 `budget_exceeded_result_items`；
**(a2)** 同限下第二条规则 → `NotRequested{plugin_request_stopped}` 且 `coverage == not_requested()`；
**(b)** `entry_bytes = 0` → 0 item、`Skipped{EntryBytes}`、scanned 空、skipped 点名该 entry、诊断带 `Location::Entry` provenance；
**(c)** `elapsed_millis = 0` → 列举即被拒，报告 `Partial{BudgetExceeded{ElapsedMillis}}`、规则 `has_more` + `Partial`；
**(d)** **独立结构查询同一 snapshot 用自己的预算 → `Complete` + 1 item + `CompleteWithinSchema`**（即「**主查询仍能完成其独立结构结果**」）。

### 不执行（证据，非声明）

`nothing_is_executed_and_a_claim_that_names_an_absent_class_is_only_a_name`：**先证明归档里真有可解码类**（`inspect_header(ClassTarget::Entry(main), Strict)` 成功且含 `native` 方法）、manifest 真写着 `Main-Class`/`Launcher-Agent-Class` 与 `Class-Path: https://example.invalid/absent.jar`。
plugin 只发布 3 个**配置里拼写的名字**（含归档里**不存在**的 `com.example.Absent`），且该请求自身 usage：`class_bytes`/`class_headers`/`method_bodies`/`attribute_bytes`/`code_bytes`/`analysis_steps`/`ir_items`/`ir_edges`/`normalization_clones`/`output_bytes` **全 0**，`entry_bytes=124`（只物化配置），`archive_entries=4+2`；报告 `Complete`（**若真启动 launcher，缺失类会是失败**）；item 中**不含** agent/launcher 名或 URL；`dynamic_analysis` coverage `NotRequested`。URL 由**结构面**作为 literal 发布（其既有行为），**plugin 从不读 manifest**。

### 隔离立场（写在何处）

`PLUGIN_TRUST_DOMAIN`（plugin.rs）："…a Rust function is **not a sandbox**: an untrusted extension must run in a **separate process** or in **Wasm**…under its own change"。每条 descriptor 的 `isolation` 都引用它（单测断言相等 + 文本含 `not a sandbox`/`separate process`/`Wasm`），并在模块文档与 `Engine::plugins` 文档重复；经 `jarde::PLUGIN_TRUST_DOMAIN` 导出。**未**实现进程/Wasm，**也未**把 trait 说成沙箱。

### 实现者自查中的一处**加强**

原以 `serde_json::to_string(report).contains("service-loader-registrations")` 判「插件身份不污染核心平面」，实测**抓不到**——`XrefEvidence.attribute` 在 JSON 中以**字节数组**拼写，字符串搜索看不见。改为语义断言 `evidence.attribute == Some(ArchiveNameBytes(b"com.example.Service"))`（+ `via` 空 + 三个身份串文本检查），并用证伪 ②b 验证它会变红。**方向是加强，未放宽。**

### 证据

全量 **1074 passed / 0 failed / 3 ignored**（1065 + 9 = 4 条 plugin 单测 + 5 条集成）；`p4_plugins` 5 passed；fmt/clippy 1.98.1 干净；`openspec validate --all --strict` 14 passed；两个 CI example exit 0；分层三包中 `jarde-java` **0** 次；**既有断言零改动**（`tests/` 只新增文件）；**未触及依赖边**（故 fuzz 锁两条按纪律 3 不适用，`git status` 对 `Cargo.toml`/`Cargo.lock` 为空）。
**实现者三组证伪**：① 未注册改「空结果」→ 专测红（`left: Performed / right: NotPerformed`）；②a 让框架字符串进通用 scanner（`target_matches → true`）→ X1 逐字段用例红（`7 vs 1`）；②b 给 X1 item 打 plugin 戳 → 同上红（`ArchiveNameBytes([112,108,117,103,105,110,…])`）；③ 忽略预算继续发布 → 预算用例红（`3 vs 2`）。**父级独立证伪**见上（未注册回退）。
**CI**：`9b37ea8` → 见下。

### 未完成（如实，属 3.3/3.4）

**CLI 出口与退出码**（`crates/jarde-cli` 一字未动）；**现代支持矩阵、Java 8 output conflict、source map、A04–A07/A12 与 adversarial tests**（3.3）；`openspec/**` 未改。
**本片如实记录的缺口**：`PhysicalScope::ArtifactTree` 嵌套树 scope 答 `NotRequested{plugin_scope_not_registered}`（**有专测，非静默等同 `SnapshotAll`**）；`PluginRuleAnalysis::Skipped{Error{code}}`/`Cancelled` 分支**无 fixture**（只经预算维度覆盖）；descriptor 的 `input` 类别**目前只有一个取值**（`ArchiveResourceEntry`），class 事实类输入未登记；**未实现进程/Wasm 隔离**（立场已写明，属另立 change）。

### 一处流程注记（父级）

实现者报告「共享 Skill 入口不可用」并列出其检查过的位置——它检查的是**仓内**路径与 `../../skills`，而本宿主该 Skill 位于 `~/.grow/skills/software-engineering/SKILL.md`（**父级已确认存在并读毕**）。**教训**：派单时应把 Skill 的**绝对路径**写进 brief。本片的验证面与汇报结构因此完全依父级 brief 而定，未受影响；后续派单已加入该路径。

## 2026-09-20 3.3：A04–A07/A12 逐行落证据、现代矩阵、output conflict 与对抗测试（提交 `bc2d205`）

### A04–A07/A12 逐行证据（**本片的核心**）

| 行 | P4 侧证据（实跑全绿） | 增量判定 |
| --- | --- | --- |
| **A04** | `p4_modern_facts::a_bootstrap_handle_that_names_nothing_is_never_resolved`（句柄指向**本仓库不存在**的类，图仍完整且断言**所有** `BootstrapHandle` 边 `edge.to == handle`——「走到句柄即止，从不跟随它命名的成员」）；`an_indy_site_whose_handle_is_not_a_member_gets_no_concat_fact`；`a_class_whose_invokedynamic_sites_are_not_concat_reports_no_concat_fact`；`a_real_concat_site_is_a_reader_fact_and_joins_to_the_bci_of_its_instruction` | **有增量，但只在事实平面**：`implementation handle 和创建/调用的区分；未知最终目标` 在 condy/concat 这条「任意 bootstrap」路径上被**证据化**。P4 **未**给 lambda 加语义（那仍是 P3）。**若把该列读成要求恢复语义，则 P4 无增量**——口径边界，**未虚报** |
| **A05** | `nodes.len()==12`/`edges.len()==15`（「每个节点一次，无论多少路径到达」）、`steps==35`/`max_depth==6`、`cycles.len()==2`、共享节点之后两入口 `via[..]` **逐元素相等**而各自前缀深度不同（6 vs 4）；四条预算用例 + 环用例 | 逐条对上（visited/cycle、边数/深度预算、共享节点与 via 路径） |
| **A06** | 三 profile 三答案并存 + `physical_entries().len()==4` + 每定义 3 候选 + `unselected().len()==2`；manifest 条件诊断（Warning，版本条目**仍列出**）；不合规选择（`NonConformant` + Error + 转发 `MultiReleasePublicPredecessorMissing`）；`matrix == 3×扫描 + Σ每 profile == 三个 standalone` | 逐条对上 |
| **A07** | `p/Dup.class` **两个 origin**（WAR 根 vs 嵌套 `WEB-INF/lib/a.jar`）、ordinal/origin 不同；`roots=[Snapshot]`→`Ambiguous`；`ArtifactTree{嵌套}+Snapshot`→`Ordered`；**parent-first 回到 Ambiguous、child-first → Ordered**；`Custom`→`Undetermined` + `search` 空 | 逐条对上 |
| **A12** | **P4 无增量（如实）**：P4 未触碰恢复层——**父级独立核实**：`grep ModernFacts\|modern_facts\|ModernOrigin\|OutputLevelStatus` 在 `crates/jarde-java/src/` 与 `tests/` 中**零命中**；`Engine::recover_method` 输出与 P3 相同；plugin item 不带 X1 边与 source map 锚；1.2 的 concat 事实**不产生文本**。P3 侧证据本片实跑 4 passed | **无增量**（「现代恢复」= 把 concat 站点呈现为 `+`、record/sealed 呈现为源码，**不在 P4**；触发条件已写进矩阵） |

### 现代支持矩阵（`docs/support-matrix.md`，8 处逐字替换全部匹配成功）

1. 首行复核边界 → 2026-09-20 + 三个归档提交 + **P4 已实现** + 指向新章节；
2. `Classfile 53–71` 行 → 追加 registry（45–71 逐 release 登记引入约束）+ 事实路径 + **未登记 = 无主张**；
3. `Preview` 行 → 追加 `PreviewRule` 登记与 `classfile_invalid_modern_minor_version`（结构仍 complete）；
4. `Future major` 行 → 追加 `UnregisteredRelease`（**无主张**）+ `classfile_future_release`；
5. **新增小节**「现代（53–71）能力与 P4 新增入口」：**五平面表**（parse/X1/resolution/decompile-quality/output-level）+ RuntimeMatrix + plugin + X3 + 「**source map 无 P4 增量**」+ 逐条「未做」；
6–8. `README.md`：新增 P4 段、路线句改为「完成 P4 的 3.4 与 P5」、并**修正一处 P3 时代遗留的矛盾句**（原写「门面尚未提供参数/receiver/debug」，与 P3 3.1/3.2 的实现和 support-matrix 冲突）。

**逐字替换全部成功，无一处找不到目标串。**

### Java 8 output conflict 与 source map 的判断（**判断有依据，非偏好**）

- **不加更多 output level**。理由（代码事实）：`assess_output_level` 的谓词是「**Java 8 没有等价物**」，**不是** `since > level`——concat 的 `since` 是 `invokedynamic` **tag** 规则 **51**（比 Java 8 还老），按档位比较会把 concat **误判为可表示**；而 record/sealed 在 11/17 下确实可表示。唯一消费者是 P3 的 Java 8 恢复层，**给无消费者的档位产判定就是「报告一个没人产出的结论」**。
- **做的机制改动**（`modern.rs` +15/−0，**行为不变**）：把 level 变成**穷尽 match** 的显式表态点——**新增 `OutputLevel` 变体会编译失败**，必须在该处给出新档自己的谓词；并新增 `a_construct_no_pass_judges_gets_no_output_level_verdict` 钉住「module/nestmate/condy 不产输出级别结论」。**触发条件**（出现第二个目标档）已写进矩阵。
- **source map：无 P4 增量**（如实写进矩阵）。P4 不产被呈现的文本；concat 事实自带 `ModernOrigin`；plugin item 的 entry 证据属另一平面。当前锚点仍是 P3 3.2 的 `Origin`/`OriginMember::MethodPoint`；**触发条件**：恢复层开始把 concat 呈现成文本。

### 对抗测试（**实跑输出**，Tier 3 口径）

| 面 | 结果 |
| --- | --- |
| ZIP bomb / 超大 entry | **4 passed / 0 failed**。探明：P0 语料已含**真实 bomb 形状**——central directory 谎报 uncompressed size 为 1、实际流展开 `READ_CHUNK+1` → `BudgetExceeded{EntryBytes}` 且**不返回字节**（`declared` 与 `understated` 两侧都断言） |
| condy 图（预算/环/深度/不解析） | **7 passed / 0 failed**（未退化） |
| 缺失依赖 / open-world | **4 passed / 0 failed** |
| 不可约 CFG | **1 passed / 0 failed** |
| 取消压力（workspace 过滤 `cancel`） | **40 passed / 0 failed** |
| 非法现代形状（golden） | **6 passed / 0 failed**（含 `an_illegal_class_file_is_read_and_its_planes_stay_separate`） |

**无回归。**

### 父级独立复核与证据

- 全量 **1075 passed / 0 failed / 3 ignored**（1074 + 1）；fmt/clippy 1.98.1 干净；`openspec validate --all --strict` **14 passed**（父级复跑）；两个 CI example exit 0；分层三包中 `jarde-java` **0** 次；**未触及依赖边**。
- **A12「无增量」由父级独立核实**（见上表：`jarde-java` 对现代事实**零引用**）。
- **测试断言零改动**（`git diff --numstat -- tests/` = **+65/−0**，删除行含 `assert` 计数 **0**）；`modern.rs` 的 +15/−0 是文档注释 + 穷尽 match，**无行为变化**。
- **实现者两组证伪**：① 让 record 在 Java 8 下**不报冲突**（伪等价降级）→ **2 红**（含 golden 的 `left: "representable" / right: "conflict", ["record_components"]`）；② 让矩阵**塌缩**（release 固定 17）→ **3 红**。副本 216 文件清单校验、仓库事后 216 OK。
- **CI**：`bc2d205` → 见下。

### 未完成（如实，属 3.4）

**3.4**：全量门禁数字与「**结构支持 vs 源码恢复独立状态**」的结论。**本片给出的可用表述**：结构支持（parse/registry/事实/矩阵/X2/X3/plugin）已到 3.1–3.2 且**未**触及恢复层；恢复层（decompile-quality、source map）**仍是 P3 状态**，`OutputLevel` 只有 Java 8，因此 A04/A12 的「现代恢复」增量**尚未发生**（触发条件：恢复层开始消费 `ModernFacts`）。
**前序遗留（本片未变、仍如实记录）**：CLI 出口（三个新入口只有库/门面出口）；plugin 的 `ArtifactTree` 嵌套 scope 答 `NotRequested`；X3 若干分支与 plugin `Skipped{Error}`/`Cancelled` 无 fixture；`MethodParameters` 未读；类级事实不在载荷；flag 具体位置不主张。
**小风险（父级记录）**：矩阵/README 新增的章节锚点按 GitHub 规则推导，**未在渲染器中验证**。

## 2026-09-20 3.4：最终门禁与「结构支持 vs 源码恢复」的独立状态

**P4 至此 4/4。** 本节由 3.4 执行者写入（**尚未提交**；本片的改动面只有本文件与 `tasks.md`）。

### 最终门禁（全部本机实跑，记录精确数字）

| 项 | 命令 | 结果 |
| --- | --- | --- |
| 全量测试 | `cargo test --workspace --all-targets --all-features --locked --no-fail-fast` | **1075 passed / 0 failed / 3 ignored**，**57** 个 test binary，exit 0（与基线逐字相同） |
| 可重放对照 | `cargo test --test p3_execution_comparison --locked -- --ignored` | **2 passed / 0 failed / 0 ignored**，17.66s，exit 0 |
| 格式 | `cargo fmt --all -- --check` | exit 0（干净） |
| lint | `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings` | exit 0（干净；rustc 1.98.1 / clippy 0.1.98） |
| MSRV | `cargo +1.88.0 check --workspace --all-targets --all-features --locked` | `Finished`，exit 0 |
| 锁文件（fuzz） | `cargo metadata --manifest-path fuzz/Cargo.toml --locked` | exit 0 |
| supply-chain（根） | `cargo deny --manifest-path Cargo.toml --workspace --locked --config deny.toml check` | `advisories ok, bans ok, licenses ok, sources ok`，exit 0 |
| supply-chain（fuzz） | `cargo deny --manifest-path fuzz/Cargo.toml --workspace --locked --config deny.toml check` | `advisories ok, bans ok, licenses ok, sources ok`，exit 0 |
| fuzz workspace | `cd fuzz && cargo test --locked` | **21 passed / 0 failed / 0 ignored**，exit 0 |
| OpenSpec strict | `openspec validate --all --strict --no-interactive`（`@fission-ai/openspec@1.11.0`） | **14 passed / 0 failed（14 items）**，exit 0 |
| 两个 CI example | `cargo run --example inspect_class_header \| resolve_and_analyze --locked -- tests/fixtures/historical/ecj-4.6.1/v52/HistoricalControlFlow.class` | exit 0 / exit 0 |
| 分层 | `cargo tree -p jarde-reader \| jarde-query \| jarde-jvm` 中各含 `jarde-java` 次数 | **0 / 0 / 0** |
| 工作区 | `git diff --check` | 干净，exit 0 |

**规定时长 fuzz 冒烟（三个 target 各 26s，语料用 `/tmp` scratch 副本——`fuzz/corpus/` 三份 `cp -R` 到 `/tmp/p4-corpus/`，**未写 tracked 语料**）**：

| target | execs | exec/s | `slowest_unit_time_sec` | crash / timeout | exit |
| --- | --- | --- | --- | --- | --- |
| `query` | **218,962** | 8,421 | 0 | 无 | 0 |
| `artifact_tree` | **370,066** | 14,233 | 0 | 无 | 0 |
| `method_analysis` | **292,724** | 11,258 | 0 | 无 | 0 |

冒烟后：`fuzz/artifacts/{query,artifact_tree,method_analysis}` **各 0 个文件**（无 crash 产物）；`git status --porcelain fuzz/corpus/` **为空**。

**3 ignored 的构成（逐条点名）**：

| # | 用例 | 原因 |
| --- | --- | --- |
| 1 | `tests/jvm_bytecode_oracle.rs::jdk25_instruction_boundaries_match_public_bytecode_inspection` | `#[ignore = "requires JDK 25 Class-File API oracle"]`——本机无该 oracle |
| 2 | `tests/p3_execution_comparison.rs::the_corpus_is_read_the_same_way_by_every_legal_flag_set` | 需 PATH 上有 JDK：它用 `javac --release 8` 编译自己生成的 wrapper 并运行 |
| 3 | `tests/p3_execution_comparison.rs::the_p3_findings_are_replayed_by_compiling_and_executing_the_bodies` | 同上 |

后两条**在本机实跑通过**（见上表第 2 行），故 3 ignored **不是**未验证项；第 1 条是**环境缺口**（需 JDK 25 Class-File API oracle），本机未运行。

### 「结构支持 vs 源码恢复」的独立状态（**本片核心交付**）

两个面**互不推导**，各自给出可核实的依据。

#### 结构支持面：已到 3.1–3.2，且**未触及恢复层**

**依据（本片重跑的零引用 grep，逐字输出）**：

```
$ grep -rn "ModernFacts\|modern_facts\|ModernOrigin\|OutputLevelStatus" crates/jarde-java/
$ echo $?
1
$ find crates/jarde-java -name '*.rs' | wc -l
27
```

即：`crates/jarde-java/` 下 **27 个 `.rs` 文件**对 `ModernFacts` / `modern_facts` / `ModernOrigin` / `OutputLevelStatus` **零命中**（grep 退出码 **1**）。补充一条同向证据：`grep -rn "modern\|Modern" crates/jarde-java/src/` **同样零命中**——不是只差四个名字，而是整个「现代」词汇都不在该 crate 里。

**方向性**：`crates/jarde-java/Cargo.toml` 依赖 `jarde-reader` 与 `jarde-jvm`，故反向引用**会被 cargo 在解析期拒绝**（P3 3.4 已实测 `error: cyclic package dependency`）。结构面在分层上是恢复层的**上游**。

P4 的结构面落点（本片未改，仅登记）：`crates/jarde-reader/src/{release_registry,modern,runtime_matrix}.rs`、`crates/jarde-jvm/src/reflection.rs`、`crates/jarde-query/src/plugin.rs`。

#### 恢复面：**仍是 P3 状态**

| 事实 | 位置 | 当前值 |
| --- | --- | --- |
| `OutputLevel` **只有一个变体** | `crates/jarde-reader/src/classfile.rs:144` | `Java8`（无第二档） |
| `OutputLevel::` 的**唯一**非测试消费者 | `crates/jarde-reader/src/modern.rs:484` | `OutputLevel::Java8 => {}`——3.3 加的**穷尽 match 空臂**，**无行为** |
| `Origin` 仍是 P3 3.2 的形状 | `crates/jarde-java/src/source_map.rs:79` | `{ bci: u32, method: Option<Box<PhysicalMethodId>>, cp: Option<u16>, provenance: Provenance }` |

结论：**P4 未改 `jarde-java` 的任何行为**，decompile-quality 与 source map 两个平面与 P3 相同。

#### 因此 A04/A12 的「现代恢复」增量**尚未发生**

- **A12**：「现代恢复」指把 concat 站点呈现为 `+`、把 record/sealed 呈现为源码。P4 只把这些**读成事实**，**一行文本都没产**（1.2 的量身证据：concat 事实自带 `ModernOrigin`，但无渲染器消费它）。
- **A04**：`implementation handle 和创建/调用的区分；未知最终目标` 在 condy/concat 这条**任意 bootstrap** 路径上被证据化——**但只在事实平面**。若把该行读成要求恢复语义，则 **P4 无增量**（3.3 已按此口径写进矩阵）。
- **触发条件（写死在这里）**：**恢复层开始消费 `ModernFacts`**。可观察的最小形状有两条，任一条出现即触发：
  1. `OutputLevel` 出现**第二个变体**（此时 `modern.rs:484` 的穷尽 match **会编译失败**，强制给出该档自己的谓词——这是 3.3 留的机制，不是约定）；
  2. `crates/jarde-java/**` 出现对 `ModernFacts`/`modern_facts` 的任何引用（本片那两条 grep 由零变非零）。
  **未触发即不得声称现代恢复**。

#### 三个新入口**各自报告什么平面**（读其报告类型得出，不套用恢复层词汇）

| 入口 | 报告类型 | 报告的平面 | **不**报告的平面 |
| --- | --- | --- | --- |
| `Engine::runtime_matrix` | `RuntimeMatrix`（`runtime_matrix.rs:71`） | `physical: MultiReleasePhysicalEvidence`、`domain_graph`、`profiles`、`groups`、`coverage: Coverage`(=`CoverageDimension`/`CoverageState`)、`execution: ExecutionReport`、`scan: ScanAccounting`、`usage: UsageSnapshot`、**`verification: crate::classfile::VerificationStatus`**（`runtime_matrix.rs:770` 硬编码 `NotPerformed`） | 无 `compile_status`、无 `semantic_validation`、无 `output_level`、无任何被呈现的文本 |
| `Engine::plugins` | `PluginReport`（`plugin.rs:472`） | `physical: PhysicalView`、`analysis: PluginAnalysis`(`Performed`/`NotPerformed{code}`)、`rules[].analysis: PluginRuleAnalysis` + `rules[].coverage: Coverage`、`execution`、`diagnostics` | **完全没有** verification 面 |
| `Engine::reflection_patterns` | `ReflectionPatternReport`（`reflection.rs:934`） | `environment_identity`、`environment_problems`、`scope: PhysicalScope`、`analysis: ResolutionAnalysis`（**2.2 的解析面词汇**）、`sites`、`unresolved_dependencies`、`reads: Vec<HeaderRead>`、`coverage`、`execution`、`diagnostics` | **完全没有** verification 面 |

**关键区分**：`compile_status` / `semantic_validation` / `quality` / `syntax_status` / `representation` 是 **jvm IR 平面**的词汇，定义在 `crates/jarde-jvm/src/ir.rs`（`RecoveredMethod` 的字段，见 `ir.rs:279`/`ir.rs:283`），即 **P3 恢复入口**的输出。**三个新入口一个都不填充这些字段。**

`runtime_matrix` 那一个 `verification` 字段是**唯一**近似项，且它的类型是**header 平面自己的** `classfile::VerificationStatus`（不是 IR 平面类型），值恒 `NotPerformed`。**不得**把它读成「矩阵做了验证」或「矩阵恢复了源码」。

**一句话口径**：**「结构读得出」≠「源码恢复」**。P4 的交付是**事实与证据**（parse / registry / 现代事实 / RuntimeMatrix / X2 / X3 / plugin），恢复层**原封不动**。

### 证伪（副本 + `shasum -a 256` 校验；仓库事后逐字节相同）

方法：`rsync -a --exclude .git --exclude target` 建 `/tmp/p4-falsify`，独立 `CARGO_TARGET_DIR=/tmp/p4-falsify-target`，编辑前把四个受改文件存入 `/tmp/p4-pristine` 并记 SHA-256；每次还原后用 `shasum -a 256` 逐文件比对。**未用** `git checkout`/`restore`/`stash`/`reset`。

| # | 篡改 | 预期 | 实际 |
| --- | --- | --- | --- |
| **A** | `runtime_matrix.rs:770`：`RuntimeMatrix.verification` 由 `NotPerformed` **改 `Performed`**（一个 P4 新入口用**仓库自己的既有词汇**声称验证/恢复面成功） | 若有守卫必须红 | **全量 1075 passed / 0 failed / 3 ignored（57 binary，exit 0）——与基线逐字相同，零红** |
| **C1（对照）** | `classfile.rs:484`：**header 平面**的同一个 `verification` 字段改 `Performed` | 红 | **红**：`p4_feature_registry` **2 passed / 2 failed**，exit 101（`a_preview_marker_is_reported_without_becoming_dialect_support`、`preview_and_unregistered_releases_keep_a_readable_structure_and_claim_nothing`）；`p4_golden` 6 passed |
| **B** | 在 **P4 新文件** `crates/jarde-reader/src/runtime_matrix.rs` 注入 `#[cfg(any())] use jarde_java::RecoveryReport;`（**编译通过**） | 若在 A17 守卫范围内必须红 | **绿**：`physical_entry_modules_do_not_reference_the_recovery_layer` **1 passed** |
| **C2（对照）** | 同一注入放进 **`crates/jarde-query/src/query.rs`**（A17 守卫范围内，置于内层文档注释**之后**，`cargo build -p jarde-query` **exit 0**） | 红 | **红**：`left: ["crates/jarde-query/src/query.rs: jarde_java::, RecoveryReport"] / right: []`，exit 101 |

**发现 1（无守卫，如实报告）**：**`RuntimeMatrix.verification` 没有任何断言守卫。** A 组把该字段改成 `Performed` 后，**整个 workspace 的 1075 条测试全绿**。C1 对照证明**不是方法无效**：header 平面的同一字段一改就红（2 条集成红）——即 P4 自己引入了**不对称**：header 平面的 verification 有守卫，矩阵的没有。

**为何需要守卫（不是「不需要」，故不写理由而是给建议）**：该字段虽是 `runtime_matrix.rs:770` 的硬编码常量、当前无输入能翻转它，但**这正是 1.1 给 header 平面加负向用例的理由**（`header_inspection_leaves_attribute_and_flag_legality_to_the_fact_passes` 钉住同一类边界），且该字段的类型是**跨平面共享**的 `classfile::VerificationStatus`——将来把矩阵接到任何 verifier 上，改动**不会**被任何测试拦下。
**建议补的守卫（最小、无需新 fixture）**：在 `tests/p4_runtime_matrix.rs` 加一条负向断言，钉住 `matrix.verification == VerificationStatus::NotPerformed`；同理可对 `PluginReport`/`ReflectionPatternReport` 断言其**不含** verification / compile / semantic 面（这两个类型当前**根本没有**这些字段，属编译期保证，故只需一条注释说明，不必造断言）。**本片不代为新增该守卫**（不在允许修改面内），仅如实上报。

**发现 2（守卫覆盖缺口）**：**A17「物理入口不得引用恢复层」的源码守卫，其受守卫集合只有 `crates/jarde-query/src/query.rs` + `crates/jarde-query/src/xref/**`**（见 `tests/p2_contracts.rs::guarded_sources` 与 `A17_QUERY_MODULE`/`A17_XREF_DIRECTORY` 两个常量）。**P4 新增的五个模块——`crates/jarde-reader/src/{release_registry,modern,runtime_matrix}.rs`、`crates/jarde-jvm/src/reflection.rs`、`crates/jarde-query/src/plugin.rs`——全部落在该集合之外。** B 组注入**编译通过**且守卫**照常通过**，C2 对照证明同一注入放在范围内的 `query.rs` 上立刻红。**即 P4 把守卫的盲区从 0 个模块扩到了 5 个模块**，而守卫的既定用途正是「将来重排时仍被复核」（P3 3.4 原话）。**注**：分层仍有 cargo 的循环依赖拒绝兜底，但那拦不住不构成依赖边的文本引用（B 组正是这种）。

**流程注记（如实）**：C2 的**第一次**尝试是**无效对照**——注入点落在 `query.rs` 的内层文档注释 `//!` **之前**，触发 `E0753: expected outer doc comment`，exit 101 是**编译失败**而非守卫命中。改为置于文档注释之后（`cargo build -p jarde-query` exit 0）才得到上表的有效结果。**这条按无效对照上报，不计入证据。**

**副本校验**：A/B/C1/C2 每次还原后逐文件 `shasum -a 256` 比对——`runtime_matrix.rs` `7c0c16c9…`、`plugin.rs` `452ae41b…`、`classfile.rs` `7c1db3a1…`、`query.rs` `0d95b7ad…` **全部 OK**（`RESTORE_BYTE_EXACT_OK`）。仓库事后 `git status --porcelain` 只有本片写入前的两处既有改动（`JVM_Rust_Engine_Final_Architecture.md`、`fuzz/README.md`，**非本片所为**）。

### 既有断言：**零改动、零变弱**

本片**未触碰** `crates/**`、`src/**`、`tests/**`、`fuzz/**`、`docs/**` 与 `Cargo.toml`/`Cargo.lock`（全量 1075 条与基线逐字相同即为佐证）。写入面只有 `openspec/changes/p4-modern-semantics/verification.md` 与 `tasks.md`。

### 未完成（如实收口，逐条：能力缺口 or 有意边界）

见 `tasks.md` 的 3.4 子注——该处列出 P4 全部片的「未完成」汇总及每条的性质判定。

## 2026-09-20 3.4 后续：两个守卫缺口已关闭（提交 `11be3f4`）

3.4 的证伪自查发现两个**守卫缺口**（两者都不是实现缺陷，而是**覆盖**缺口）。本片只改 `tests/**`（**父级核：`git diff --stat -- crates/ src/` 为空**），1077 passed / 0 failed / 3 ignored（**+2**）。

### 缺口 1：`RuntimeMatrix.verification` 无断言

**已核实**（3.4 自查 + 父级独立复现）：把 `runtime_matrix.rs` 的 `NotPerformed` 改 `Performed` → **全量 1075 全绿、零红**；而**同一字段在 header 平面**一改就红。即 P4 引入一处**不对称**。
**修法**：`tests/p4_runtime_matrix.rs` 新增 2 条**负向**断言——`the_matrix_never_claims_verification`（顺利路径）与 `a_matrix_that_reports_an_error_never_claims_verification`（带 `Error` 诊断的路径）。**两条都先断言这次运行真的答了东西**（`profiles.len()==3` / 存在 Error 诊断），使 `NotPerformed` **不是「因为放弃才为真」**。
**为何两个场景够**：`Err` 的请求级拒绝**根本不产生 `RuntimeMatrix`**（无字段可断言），故「有报告但状态不干净」的最强现实场景就是 `NonConformant`。**不做更多铺开**。
**父级独立证伪**：改 `Performed` → `p4_runtime_matrix` **11 passed / 2 failed**（恰好这两条，`left: Performed / right: NotPerformed`），其余同批全绿；还原后校验 OK。

### 缺口 2：A17 源码守卫的**覆盖范围**

**已核实**：守卫的受守卫集合只有 `query.rs` 与 `xref/**`；P4 新增的五个模块**全在其外**——把 `use jarde_java::RecoveryReport;` 注入 `runtime_matrix.rs` 守卫**绿**（漏过），注入 `query.rs` **红**（命中）。**盲区从 0 个模块扩到 5 个。**

**范围判断（宁可少纳入，**已实测**）**：**只**纳入 `crates/jarde-query/src/plugin.rs`（同属 query 层、同属 X1 物理面，报告 `PhysicalView`）；**不纳入** `jarde-reader/src/{release_registry,modern,runtime_matrix}.rs` 与 `jarde-jvm/src/reflection.rs`。
**理由（不是推断，是实测）**：A17 的原意是「**X1 物理入口**不得引用 P2/恢复层」，而 reader/jvm 是这些类型的**所有者**。临时把两个身份加进受守卫集后实跑：`reflection.rs` 会因 `crate::environment`/`crate::resolver`/`crate::ir`/`AnalysisStage`/`ResolutionEnvironment`/**`UnresolvedDependency`** 等**合法导入**被报——规则会变成「禁止导入任何分析面」，**改变断言原意**；`runtime_matrix.rs` 仅因散文里一处 `jarde_jvm::providers` 就被报（module-path 半边**注释也算**）。且这两层对恢复层**结构上不可达**（同一被 cargo 拒绝的依赖环），纳入只扩大政策、不增加可观测的越界。
**父级独立证伪（纳入侧）**：往 `plugin.rs` 注入 `#[cfg(any())] use jarde_java::RecoveryReport;` → **两条 A17 守卫双双红**，输出点名 `crates/jarde-query/src/plugin.rs: jarde_java::, RecoveryReport`；还原后校验 OK。
**未纳入侧仍绿是刻意的**，已在 `A17_PLUGIN_MODULE` 的常量文档里写明，并说明 **cargo 的循环依赖拒绝才是那两层的兜底**。

### 纳入方式与非空洞性

`GuardedModule { identity, candidates }` → `A17_ADDED_MODULES` → 在 `guarded_sources` 枚举；`A17_GUARDED_FILES` 6→7；`assert_single_layout` 泛化为「每个已解析文件都与 xref 目录同布局」（**半迁移树照旧失败**）；非空洞循环改为 `A17_EXPECTED_MODULES ∪ A17_ADDED_MODULES`；**尺寸下限仍只属于六个 P1 文件**（`plugin.rs` 不带）。沙箱两套布局各新增 `added_module_reference` 用例，并补上新增模块的干净桩。
**非空洞性检查全部保持**：派生恢复 token 非空（≥60/≥50、锚点名逐条存在、`OriginSet` 被正确减去）、每个派生名必须被同一 matcher 命中、阳性对照 `src/facade.rs` 仍被命中且不在受守卫集、每个 identity 都解析成功且出现在枚举、总数 = 7。
**承重证伪（实现者）**：把枚举退回修复前形态（`A17_ADDED_MODULES` 空、计数 6）→ **唯一**失败是 `the_a17_guard_detects_rewritten_references_and_added_files`：`case added_module_reference …: the guard must flag exactly ["plugin.rs"], got []`——**证明新用例会在旧行为上失败**。

### 既有断言：**无一处放宽**

删除行 27 行逐条复核：`A17_GUARDED_FILES` 6→7（**收紧**）；`assert_single_layout` 由「query ↔ xref」改为「**每个**已解析文件 ↔ xref」的循环（**同谓词、更多比较**）；枚举循环改为 `chain(A17_ADDED_MODULES)`（**要求更多**）；沙箱干净副本占位扩展到所有恒受守卫身份；其余为注释/文档重写。**没有任何 `assert` 的期望值被改写、删除或放宽**——唯一被替换的 `assert_eq!` 由同一谓词的**更广循环**取代。

### 证据

全量 **1077 passed / 0 failed / 3 ignored**（+2，均在 `tests/p4_runtime_matrix.rs`；`p2_contracts` 仍 30 条）；fmt/clippy 1.98.1 干净；`openspec validate --all --strict` 14 passed；两个 CI example exit 0；分层三包中 `jarde-java` **0** 次；`git diff --check` 干净；**生产代码零改动**（父级复核 `git diff --stat -- crates/ src/` 为空）；锁文件两条 exit 0。
**CI**：`11be3f4` → run 35476894723，四 job success。
**本片未做独立 review**（实现者自查自验 + 父级独立证伪；两条发现本身**未经第二方复核**）。
