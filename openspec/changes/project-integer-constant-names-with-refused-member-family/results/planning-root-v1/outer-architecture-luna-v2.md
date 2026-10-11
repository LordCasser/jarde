# CF12 Labels：Inner → outer 常量名（v2，只读修订）

本文修订 `/private/tmp/jarde-cf12-outer-constant-architecture-luna-v1.md`。v1 保留作历史稿；它把已证明的 member family 当成真实 replay 的可用前提，并误判了 3294 的 JADX global 重复情况。以下方案按真实冻结输入改写，仍是独立后续债务，不属于 conditional-switch 当前 gate。

## 先看真实冻结 request：当前没有 owner 内容

检查了 change 的完整回放记录：

- `openspec/changes/recover-proved-conditional-switch-fallthrough/results/complete-source-root-v1/execution.json` 中，Labels root report 的 render record 为 index 1037/1041 邻近记录，Inner 为 1036（default）和 1040（all）；两类 argv 都是 `class-source --input <一个 .class> --class ... --policy single-class`。Inner 输入路径是 `.../capture/TestSwitchLabels.test/input/TestSwitchLabels$TestCls$Inner.class`，root 输入路径是另一个 `.../TestSwitchLabels$TestCls.class`。每个 request 的内容并没有同时带这两个 class。
- 两份 Inner report 的 `view.scope` 都是 `snapshot_all`，但 snapshot/location 是 `standalone_root`，class bytes 长度 744；recovery artifact binding 的 `environment.content` 只有该 class 的一个 digest，runtime root 也是 `standalone_class`。Root report 同理只有 root 的一个 digest（长度 753）。`SnapshotAll` 对 standalone CLASS 只表示该 class 自身，不会把同目录旁边的 class 自动并入。
- 两边 `member_family` 均为 `{"state":"refused","child":null,"reason":"selected root has an InnerClasses self row naming an outer class"}`。无论拒绝是否与单类 scope 有关，这两份真实报告都没有 Prepared relation/child。
- Inner class 的 `InnerClasses` self row 可以给出外层类名和 nested name/access flags；它不携带 outer 字段表。`return 2748` 在字节码中是常量值，既没有 field reference，也没有指出某个字段声明。只凭这些 child bytes，不能证明 outer 有 `CONST_ABC`、它是 `int static final`、它自己的 `ConstantValue` 是 2748，或这个名字可作为 owner-qualified Java source reference。因此当前 request 无权把字面量改成 `CONST_ABC`。

上面的 report 事实来自已保存 JSON/raw，不是根据 fixture 的 Java 原文猜测。本文没有运行任何命令或工具链。

## 最窄输入闭环：给 request 一个含两个 class 的已选内容

常量来源必须从输入范围本身闭合。可复用现有 class-source/API，没有必要把工作押在 family projection 上：

1. CLI 路径可用一个已有 ZIP/JAR artifact 作为 `--input`，其中至少包含 exact `TestSwitchLabels$TestCls.class` 和 `TestSwitchLabels$TestCls$Inner.class`，然后以 `PlainJar` 环境在该 snapshot 中选中 Inner。这样一个 selected snapshot 的 ZIP entry scope 与 runtime root 都含两个物理 class。用当前类名/physical-definition 导航规则选择 Inner 和 outer，不再分别传两个 `single-class` class files。
2. 库 API 也可把两个 standalone `ArtifactSnapshot` 放进 `content`，选 Inner 的 snapshot 作为 request physical target，并用现有 `EnvironmentPolicy::ExplicitClasspath` 指定两个 `LoadRoot::StandaloneClass`。该方式可保留每类独立 snapshot 身份，但 CLI 当前 `--input` 是单一 artifact 参数，不能假装 CLI 自动加载相邻目录文件。
3. `ArtifactInput::Path` 当前打开一个 path 为一份 byte snapshot，再按内容分类为 standalone CLASS 或 ZIP；本地 Reader/CLI 文档把输入限定为 archive 或 standalone class，并没有“把普通目录递归打包成 snapshot”的路径。若 fixture 当前是 class 目录，应使用现有构建/测试产物提供的 JAR，或 API 的多 snapshot + ExplicitClasspath 形状；不要把原始目录路径当作已支持的 reader 输入，也不要新增产品工具。

即便两个 class 在同一个 JAR 中，仍不要求 `member_family` 成功。真实 Reports 的 Refused 是一个要遵守的边界；而 `TestCls` 自身的 self row 还指出 `TestSwitchLabels` 是其 enclosing owner。新方案不依赖把 `TestSwitchLabels`、`TestCls`、Inner 整个家族都 fold 成一个 nested source unit，也不把“同包 / 同 jar / self-row 名称相同”单独当成字段证明。

## 有界 outer 读取和复用点

目标仍严格是 `TestSwitchLabels$TestCls$Inner.f1(I)I` 中返回 2748 的那处 int literal。建议由现有 class-source 请求在同一已选择内容/environment 中，执行一条明确的 owner 读取路径：

1. Inner 的 selected bytes/typed `InnerClasses` facts 提供一个 raw outer class-name 候选。再在同一 selected snapshot/environment 以现有 class navigation 解析该精确 raw 名称；必须唯一并得到 `PhysicalDefinitionId`。可用同一份 archive 的 `Engine::enumerate`/现有 class listing/`ClassRef::Definition` 流程，不按磁盘目录猜 classpath。若无法唯一选择该 owner，拒绝 cross-class name projection。
2. 将目标范围限定为这个 exact owner class，而不是扫描所有 public 常量或整个 classpath。读取 outer 自己的 class header/InnerClasses facts，核对其 row 是否把 selected Inner 作为直接 member（raw child name、outer owner/name、flags 与 child self row 一致）；然后只读取 outer 的 field table，并对候选 field 使用其自身的 `ConstantValue` 属性。此处必须看到真实 outer 字段，literal 相等不是身份。
3. 最简单的既有 public operation 可对同一个 archive/snapshot 再调用 `Engine::class_source` 选择 outer，让它复用当前 `ClassSourceField` declaration 和 `ProvedIntegerConstant` 生成逻辑；但这会额外分析 outer 的所有有 body methods，并可能得到 outer 自身的 family Refused，这些均不提供此功能所需的证明。更窄的实现是在现有 facade/class-source 字段读取位置复用 field-table / `declared_constant_value` / identifier / unique-candidate 规则，只保留此次 outer read 的字段事实；这是内部 bounded read，不新增 public API、全局索引或新的 CLI。若工程实现坚持只用 public API，full outer class-source 是可行的正确性基线，但要显式接受其额外 body 分析预算。
4. 对 `CONST_ABC` 的 outer 候选沿用现有字段准入的核心条件：descriptor `I`、唯一且实际可读的 `ConstantValue`、值 2748、Java 标识符名、外层 field identity 完整。候选范围只含唯一选中的直接 owner 的字段，不复制 JADX 对整个 dex/root 做 public-global 搜索。若同一 owner 有多个同值字段或字段身份/属性读取不完整，保持 2748 数字。
5. 生成 owner-qualified 的名字，owner spelling 直接使用已选 `PhysicalDefinitionId` 对应 class file 的 raw binary name并按当前 top-level class-source spelling 规则写出（`/` → `.`、保留 `$`），field name 用自己的 raw bytes 验证为 Java identifier。例如该包内形状为 `TestSwitchLabels$TestCls.CONST_ABC`。不把 `$` 无证明地改成 canonical nested `.` 路径，不猜 imports，也不需要借用拒绝的 member family `simple_name` 来组名字。
6. 投影 span 留在 Inner 的 class-source output，复用 integer-constant name derived projection 结构：`Field` anchor 指向 outer 的 `PhysicalMemberId` 与物理 field-table index；`MethodPoint` anchor 指向 Inner `f1(I)I` 的 `PhysicalMethodId` 和 return literal 的原始 BCI。现有 identity 类型含 owner 的 physical definition；无需新 anchor 类型。Inner 的 method physical text/source map 仍记录 `return 2748;`，外层字段只作为 source-only derived name 的锚。

现有 `ClassSourceReport.integer_constant_candidates` 是当前 read/current class 的 private same-class 候选，不能把本次 cross-owner candidate 假装成 Inner 自有字段。实现可在 facade 局部把两类 proof 输入分开，再共用 AST 对 int literal 的 rewrite/BCI/span 检查与最后 anchor 写入。不要污染原 Inner 字段列表，也不要在物理 recovery report 中替换数值。

## 3294：不照抄 JADX 的 root-wide 冲突规则

 fixture 有三个相关事实：outer public `CONST_CDE=3294`、Inner private `CONST_CDE_PRIVATE=3294`，以及上述 `CONST_ABC=2748`。JADX `ConstStorage` 的事实应精确描述如下：public outer `CONST_CDE` 是 global store 对 3294 的唯一映射；Inner private 字段属于 Inner 的 local class store。查 `3294` 的 int literal 时 magnitude 阈值允许 global search。`getConstField` 看见 `foundInGlobal=true` 后，如果当前/enclosing local store 另有同值字段，就返回 null。故 local/private 与 global/public 冲突导致该 literal 未命名；不是 global ValueStorage 里有两个 public duplicates。上游保存输出正是 `case 3294:`，return 输出 `TestSwitchLabels$TestCls.CONST_ABC;`。

这只是 JADX 的全 root 常量查找策略事实，不是 Java field visibility 定理，也不该复制成 jarde 新的全输入查找机制。当前 Jarde Inner 单类 report 的 `fields` 已包含自己的 private `CONST_CDE_PRIVATE = 3294`；本次 report 当前 `integer_constant_projections` 仍空，但若该 same-class candidate 被现有 AST/structured/content/fallback/span 规则接受，方法在自身类内写 `CONST_CDE_PRIVATE` 是合法的。不能为了匹配 JADX 的“不命名”外观，而让 outer 同值候选压掉一个合法且绑定到 Inner 自己物理字段的 same-class projection。

建议的 candidate 层次是：先完全维持现有 same-class integer candidate 和投影准入；对目标 literal 如果现有 same-class 唯一候选被选中，保留该字段/name 与其 same-class `Field` anchor。仅当没有 same-class accepted candidate 时，才考虑这次 exact owner 的合格 candidate，并显式写 owner qualifier。若本地同值字段候选因 duplicate/shape 等现有规则被剔除，首版可保守不以 outer 候选绕过该拒绝而留数字；这条是最窄安全策略，不是 Java 语义必需。不要实现 JADX 的 numerical magnitude thresholds、resource priority 或全-root public duplicate map。

由此，限定的 2748 能使用唯一 outer `CONST_ABC`；3294 则按 Inner 当前本地候选的既有规则决定，可能继续是数字，也可能成为 `CONST_CDE_PRIVATE`，不会被新 outer 搜索改名为 `TestSwitchLabels$TestCls.CONST_CDE`。永久测试只应锁定自己声明的安全规则：指定 outer `CONST_ABC` 锚定、相同 owner/child physical identities、无误用 outer 值覆盖本地 same-class候选；不要把 JADX 当前数字 case 当成必须复刻的实现约束。

## 独立 debt 的验证边界

后续债务可以命名为“CF12 Labels: project a literal through one selected outer field”。只新增/调整 class-source 用例输入形状，让 Inner 与 `TestCls` 同处一个被 request 明确选择的 ZIP/JAR snapshot（或显式双 standalone roots）；验证 Inner request 成功读取唯一 owner 的字段 ConstantValue、只将 return 2748 投影为 qualified `CONST_ABC`、derived anchors 分别落在两份物理 definition、child physical body仍是数字。再验证 owner 缺失/重名、outer row 不回指 Inner、字段缺失/非 int/ConstantValue 不等于 2748/outer 同值字段不唯一时保留 literal。

`CONST_CDE_PRIVATE=3294` 是独立 same-class projection 控制，不要作为 cross-owner duplicate 测试去强行保持数字；若现有 AST 路径选择其名，anchor 必须是 Inner 自己的字段。member-family `Prepared` 与 assembled nested source 不属于这个 debt 的前置条件，family 仍可 `Refused`。这项工作应另开范围并在真实 selected-content/environment 与物理 identities 下验收。
