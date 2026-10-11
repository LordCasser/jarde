# CF12：Labels Inner → outer 常量名（只读架构分析）

## 结论

这是一个独立、窄的 class-source family 投影债务，不属于 conditional switch fallthrough 当前 gate。最小实现方向是：在 root 已经选中且证明的 direct-member family 内，把 root 自己已通过现有常量候选筛选的字段，作为 child 方法 integer projection 的额外候选；只有 outer 候选在 family 可见范围内按值唯一、relation 与两个物理定义都仍成立、目标 child body 与其 AST 满足现有投影准入时，才改写 child 的 return literal。派生 source span 要用现有 Field + MethodPoint 锚分别指向 outer 字段与 Inner.f1 的原始 BCI。未闭合或不唯一时继续写数字。

不需要引入新的全局常量索引、类路径搜索器、访问控制框架或 AST 机制。已有 root/child 选定报告、family relation、同类 integer candidate、AST rewrite 和 derived anchors 可组成最窄闭环；需要的是 family 内跨物理 owner 的候选传递与 source spelling。这个结论是基于现有边界的方案判断，尚未实施或测试。

## 冻结案例的实际事实

JADX fixture `/Users/lordcasser/workspace/testzone/jadx/jadx-core/src/test/java/jadx/tests/integration/switches/TestSwitchLabels.java` 定义：

- `TestCls.CONST_ABC` 是 `public static final int`，值 `0xABC = 2748`。
- `TestCls.CONST_CDE` 是 `public static final int`，值 `0xCDE = 3294`。
- `TestCls.Inner.CONST_CDE_PRIVATE` 是 `private static final int`，值同为 `0xCDE = 3294`。
- `Inner.f1(I)I` 的 switch case 用 private 常量，return 用 outer `CONST_ABC`。
- 上游 test 只要求默认输出包含 outer 名、包含 `.CONST_ABC`，且不含 `case CONST_CDE_PRIVATE`；禁用常量替换的控制要求数值 `2748`/`3294`。

本机上游 raw output `/Users/lordcasser/workspace/projects/jarde/openspec/evidence/java-syntax-2026-10-11/cf12-upstream-java-root-v1/harness-v3-java/test-stdout.raw` 的 nested source 片段实际写成 `case 3294:` 和 `return TestSwitchLabels$TestCls.CONST_ABC;`。这说明当前上游 fixture 对该 exact class/source mode 的观察结果是：3294 不被 private 或 outer 常量名替换，2748 被替换并显式限定 outer owner。该 raw 与 fixture assertion 是观察事实；它不证明所有 JADX class nesting/rendering 模式都用同一种名字。

## JADX 实际常量查找和拼写路径

`CollectConstValues`（`jadx-core/src/main/java/jadx/core/dex/visitors/prepare/CollectConstValues.java`）在 usage 信息已可用后遍历类字段。候选必须为 static、final、有 `CONSTANT_VALUE`，且 `getUseIn()` 为空；public 字段送入 global store，其他可见性（private/protected/package）存入其 declaring class 的 local store。它是 whole-root 常量收集，但替换策略受后续 `ConstStorage` 限定。

`ConstStorage`（`jadx-core/src/main/java/jadx/core/dex/info/ConstStorage.java`）按常量值建映射，并对同一个值保留 duplicate 标记、移除唯一映射。public 字段的值在 global map；非 public 字段在 owning `ClassNode`。查询先查当前类，再沿 `parentClass` 链查 enclosing classes；它不向下搜索子类，也不是任意包、继承层次或 classpath 范围的 Java 可访问性计算。global 仅在该次 literal lookup 允许 `searchGlobal` 时参与。若找到 local candidate，而同一值还出现在 global，local 结果也会被拒绝，以免不同 owner 之间出现不确定选择。

`getConstFieldByLiteralArg` 按 literal 类型和数值大小决定是否允许 global search。int 的阈值是绝对值 100；2748 和 3294 都走 global search。因此在本 fixture 中：

- 2748 在 outer public global store 中只有 `CONST_ABC`，可选中 outer field；
- 3294 同时有 outer public `CONST_CDE` 与 Inner private `CONST_CDE_PRIVATE`。global 对该值重复，且 local 选择也会因 global 同值而拒绝，所以 literal 保留为数字。这与上述 raw 中 `case 3294` 一致。

这一 duplicate 规则是按数值而非字段名筛选，且 public 候选范围跨整个输入 root；若同一 root 的其他 public constant 也等于 2748，JADX 也可能不替换。上面的单一 `TestSwitchLabels` fixture 事实不能推导跨 APK 的具体候选集合。

替换和限定名分属两步：`ModVisitor` 把被选中的常量用 `SGET`/field ref 替换（switch key 也经过同一字段引用）；`InsnGen.makeStaticFieldAccess` 对 declaring class 调 `ClassGen.useClass`。`ClassGen` 根据 alias、内外类关系、imports 和 collision 决定 owner spelling；不能只凭 `$`/`.` 做字符串替换。对这个 fixture，已采到的 render 输出证明结果为 `TestSwitchLabels$TestCls.CONST_ABC`。

## jarde 当前能力与边界

- Class-source request 由 `ClassSourceRequest { class, environment }` 指定。所读内容来自该 request 选中的 physical definition 和 environment，不是任意源码或工程扫描。字段 initializer 仅由字段自身的 `ConstantValue` 属性决定（`src/class_source.rs::declared_constant_value`）；该路径不会从方法使用处反向推断 initializer。
- 每个 `ClassSourceReport` 保留自己的 physical `fields` 和 `methods`。在 member family 准备时 child 仍是独立 `ClassSourceReport`；root report 另存 relation、child report、capture/calls 与 projection verdict。family text 可被投影，但 child 的 physical text、method recovery、method-local source maps 保持独立。`ClassSourceMemberRelation` 同时保留 root/child `PhysicalDefinitionId` 与匹配 InnerClasses 行得出的 `simple_name`/flags；这是可复用的 owner/source-family 证明输入。
- 当前 `ProvedIntegerConstant` 明确是“same-read, same-class”。`facade.rs` 仅从当前 read 的字段收集候选：必须有可写 declaration、`static final`、descriptor `I`、恰有一个 `ConstantValue`、合法 Java 字段标识符；并在 class/member 表完整时剔除同名和同值重复候选。缺少完整结构或请求已停时清空候选。它没有读取 sibling/outer 的字段当候选。
- `jarde-java::report::project_class_source_integer_constants` 从本 method 的完整 AST 中匹配 int switch labels；只有 int-returning switch arm 的唯一直接 return 可同时投影返回值。它依赖节点呈现类型、节点 BCI 属于该 method 且出现在 instruction BCI 集合。非 switch 的现有 int/array 投影形状也有各自限制。
- facade 仅在 class report 完整、method 是 structured Java、`ContainsStatements`、无 fallbacks，且存在 retained AST 时运行 integer projection。最后要求文本 span 唯一且和 AST/body 投影一致。
- 每个现有 `IntegerConstantName` 的派生范围都有两个 physical anchors：`Field { field, index }` 与 `MethodPoint { method, bci }`。Field identity 可用于指向 root owner 的物理字段；method anchor 可指向 Inner.f1。projection 类型已有 `IntegerConstantName`，因此不必增加新的 derived kind 或身份概念。
- Family 只在 relation/capture/calls 等前置证明满足时进入相应 Prepared 形态，最终 `projection` 仍可 Refused；普通 physical reports 不因一次 source fold 失败而丢弃。outer-name 投影应服从这条 all-or-nothing 源文本投影边界，不能将“关系已选中”当作自动可访问或自动可拼写的授权。

## 最窄后续方案

把任务限定在以下一条路径：root owner 是 `TestSwitchLabels$TestCls`，selected direct child 是 `TestSwitchLabels$TestCls$Inner`，child method 是 `f1(I)I`，case 继续保持数字 3294，只有 direct switch return 的 2748 尝试映射到 root `CONST_ABC`。

1. 复用 family 准备过程中已经保留的 root/child physical reports 和 relation；候选仍必须来自 root 当前 class read 自己的 `integer_constant_candidates`，不是额外扫描 environment，也不重新解析未选中的 class。
2. 复用已有的 value uniqueness、合法字段名、字段声明完整与 method AST/BCl 校验。跨 owner 时只为本次 child method附加 root candidate；不要让其他 root、兄弟、外层祖先或任意环境成员成为候选。对 3294，root 本身唯一 `CONST_CDE` 仍与 child private `CONST_CDE_PRIVATE` 冲突；最保守规则是 family scope 内按值合并冲突检查，使其维持数字。这里是建议的保守规则；JADX 现有 whole-root public/local duplicate policy 与 jarde family-local候选范围不等价，不应宣称完全复刻其全输入 duplicate 扫描。
3. 生成 `TestCls.CONST_ABC` 这样的显式 owner-qualified spelling，owner 名只从已证明 relation 的 `simple_name`/所选 source path与 root declaration name构造/校验；任何无法证明 Java source path、name collision 或名字不唯一的情况都拒绝投影并留数字。若 family writer所处文本命名约定需要更完整的包/顶层名字，则沿现有 source path 规则选择，不能裸把 binary `$` 改 `.`。
4. 将修改范围定位到 root assembled text 中 child method 的 return span；derived projection 同时锚定 root 的 `CONST_ABC` physical `Field` identity/index 与 child `f1` 的 method identity、return BCI。现有 child physical recovery text/map仍写 `return 2748;`，和其他 projection一样只改变 family assembled source。
5. 只在 relation 和 root/child class/member completeness 足以保证两个物理候选与 method body均来自此次选中内容时投影。任一字段读取、child recovery、ownership、unique-name/duplicate 或 span replay条件失败，保留现有 family refusal/physical text，避免半边 rewrite。

这项工作是否要实现完整的 Java field visibility resolver，答案是当前 exact case 不需要：`CONST_ABC` 是 public static final，并且通过 direct member family 在 enclosing root 中呈现，现有 selected relation 已为来源提供窄证据。将来若扩到 package/protected/private、继承字段、任意外层深度或不呈现的 owner，则需要另行定义可见性和名字冲突范围；不要在本 debt 里暗中扩成全 Java 名称解析器。

## 独立债务记录与验证边界

后续 OpenSpec/task 建议名：**CF12 Labels nested method can name a selected enclosing-class integer constant**。只覆盖上面的 `Inner.f1(I)I` return 2748，另把同类 3294 的 duplicate 保留数字作为负控制。永久测试至少检查 assembled family text 的限定 outer name、`IntegerConstantName` 两个真实 physical anchors、child physical body/map 仍为 literal，以及 case 3294 仍不误命名；再加未选 owner/duplicate or source-name ambiguity 的拒绝控制。具体 fixtures 应复用已有已冻结 class/evidence，不在当前 fallthrough gate 中更改其测试。

截至本审查，我没有改工作树、没有运行编译器或测试。本文是基于所读源码、fixture assertions 和已保存上游 raw 的设计分析，不是实现或 gate 通过证据。
