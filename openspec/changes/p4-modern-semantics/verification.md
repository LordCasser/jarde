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
