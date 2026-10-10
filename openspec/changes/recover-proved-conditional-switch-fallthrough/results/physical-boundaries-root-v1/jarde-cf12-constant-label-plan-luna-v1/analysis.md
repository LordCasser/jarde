# CF12 constant-label and nested-root analysis

只读分析。没有运行 Git、Cargo、JDK、JADX 或 CLI，没有改仓库。当前 conditional-switch 片与产品 CI 不在本分析范围。

## 当前状态和契约

71 账本 `openspec/evidence/jadx-feature-inventory-2026-09-27/summary.md:84` 把 CF-12 标为“部分已测”；它记载早期 `IntegerSwitchAudit` 首片通过，但明确不代表其他 lowering 已追平。真实上游基线 `openspec/evidence/java-syntax-2026-10-11/cf12-upstream-java-root-v1/README.md:9-10,29` 又确认：`TestSwitchLabels.test` 的 default/all 运行输出与原 class 一致，但 Jarde 两个呈现的常量仍是数字；当前同类字段已在报告中，投影被 `member_family Refused` 门跳过。文档另把 `Inner` 读取外类 `CONST_ABC` 明确划到跨类绑定范围。

本地 JADX 实际测试 `/Users/lordcasser/workspace/testzone/jadx/jadx-core/src/test/java/jadx/tests/integration/switches/TestSwitchLabels.java:10-42` 是有效且直接的文字契约：`TestCls.f1` 需要 `case CONST_ABC` / `return CONST_CDE;`；`Inner.f1` 要求输出中有外类 `.CONST_ABC`，且不得把重复值的私有 `CONST_CDE_PRIVATE` 写成 case 名。真实完整重编与运行的观察已在 README 记录，因此这不是凭测试字符串推定运行语义。Jarde 生成的 root / `Inner` 基线分别可见于 `render-root-v1/TestSwitchLabels.test/TestSwitchLabels$TestCls/jarde-default.java` 和 `$Inner/jarde-default.java`：前者准确声明 `CONST_ABC=2748`、`CONST_CDE=3294`，但根 `f1` 仍写 `case 2748`、`return 3294`；后者保留 `case 3294`、`return 2748`，且作为独立完整 class 呈现。故运行契约满足，JADX 的同类命名契约仍未满足。

## JADX 名称替换依赖什么

JADX 的实现依赖已收集的**字段身份和查找作用域**，随后才把字面值变成字段引用：

- `CollectConstValues.getFieldConstValue`（`jadx-core/src/main/java/jadx/core/dex/visitors/prepare/CollectConstValues.java:49-66`）只收 `static final` 且具有 `ConstantValue` 的字段，并要求 `getUseIn()` 为空，避免把仍由真实字段读取的字段误当成已内联常量。收集时传入具体 `FieldNode`，不只保留数值。
- `ConstStorage`（`jadx-core/src/main/java/jadx/core/dex/info/ConstStorage.java:36-48,75-80,96-125`）按字段所属 `ClassNode` 分开保存非 public 常量，public 常量进入全局表；同一作用域里同数值多个字段会删掉唯一映射并标记 duplicate。查找同时看当前 class/父类链、全局候选和重复值：若非 public 局部值和 public 全局值都存在，`getConstField` 返回空而保留 literal。`getConstFieldByLiteralArg` 还使用 literal 的 primitive type 和阈值来选择搜索范围（160-188）。
- `ModVisitor.replaceConstKeys`（`jadx-core/src/main/java/jadx/core/dex/visitors/ModVisitor.java:240-249`）对每个 `SwitchInsn` key 用当前父类查得 `IFieldInfoRef`，成功后存入 switch key 并记录 field usage；普通 literal 用 `getConstFieldByLiteralArg`，然后由 source emitter 根据 `FieldInfo` 的 owner/name 输出必要的类限定名。常量替换由 `replaceConsts` 控制。

针对该 fixture，根 `TestCls` 自身有且只有一份 `2748 -> CONST_ABC` 和 `3294 -> CONST_CDE`。根方法中的两个 named uses 有本类声明和常量值支撑。`Inner` 又有 private `CONST_CDE_PRIVATE=3294`；它和外部 public `CONST_CDE=3294` 值相同但 owner 与访问范围不同。因此 JADX 对 `Inner` 的 key 保留 `3294`，不猜 private 或外部字段；对返回值 `2748` 才写外部 `TestSwitchLabels$TestCls.CONST_ABC`。把跨 owner 查找简化成“全局按数字找一个字段”会违反这里的实测行为。

## Jarde 已有事实与真正缺口

Jarde 已有足以完成**同类**命名的事实，且不需要新 IR 类型或 resolver：

- `src/facade.rs:7149-7321` 从同一次 physical class read 读取字段的 `ConstantValue`，仅为可呈现声明的 `static final int`、恰有一个属性、合法 Java 字段名构造 `ProvedIntegerConstant { field: FieldItem, name, value }`；全字段表完整性、同类重复字段名和重复数值继续在 7324-7344 拒绝。字段身份仍在候选内。
- 同文件 `project_class_source_integer_constant_names`（19745 起）取同一完整 class-source recovery 保存的 `ClassSourceMethodAst`，调用 `jarde-java::report::project_class_source_integer_constants`；该函数只把 presented `int` 的 switch keys 和符合其窄形状的直接 int return 改为 `IntegerConstantName`，保留物理 method report。使用点 BCI 与物理 Field anchor 写入 `integer_constant_projections`。`ClassSourceAstExpressionShape`（`crates/jarde-java/src/report.rs:1492`）是轻量分类枚举；仓库没有名为 `SourceExpression` 的通用表达式实体。实际投影复用同次 rich AST，而非重新反编译或建一个源表达式层。
- `ClassSourceReport` 同时保留 `fields`、`integer_constant_candidates` 和 `member_family`。family scan 的身份来自 class 自己的 `InnerClasses` 属性。对于被选作 standalone root 的 `TestSwitchLabels$TestCls`，`scan_family_root`（`src/member_inner.rs:2100-2109`）看到该 class 的 typed self row 指向 `TestSwitchLabels`，按“一次只输出一个 top-level source unit”返回 `FamilyRootScan::Refused`。这解释为什么不是常量候选缺失。
- 当前唯一明确阻塞是 `src/facade.rs:2170-2183`：常量名投影要求 `member_family == Absent`（且其他几个 family channel absent）；而 `TestSwitchLabels` 的根是 `Refused`。真实基线 README 已把这个门确认为原因。

**最小闭环无需新机制**：只在普通 root 的独立方法投影通道允许该家族保持 `Refused` 的 standalone 呈现继续尝试；不更改 Refused 的 family 语义，也不尝试把 child 拼进 root。现有 `source_text_with_method_projections`（`src/class_source.rs:12945-12984`）会先要求普通 source writer 重建文本与当前 root `report.text` 完全相等，再原子重建 staged methods 并核对派生 spans；因此这个路径能只改 root 方法文字并保留现有 family refusal。现有候选和 use-site 证书已经包含同类物理 Field 与 MethodPoint anchors。这个最小改动可补根 `f1` 两处名字，且保留 `Inner` 的数字。它**不会**满足 `Inner.f1` 的 `.CONST_ABC`；那需要在 Inner 的呈现输入中证实 parent field identity、owner source spelling 和合法可见性，是明确排除的跨 class 常量 resolver 工作。

## 替换 literal 的拒绝边界

值相等只允许在候选集已由同一物理类的唯一 compile-time constant 证明后作为定位手段。下列情形若不拒绝会输出错误或误导的 field name：

- 一个类中两个 `ConstantValue int` 字段同值，或者同名字段身份不唯一。Jarde 当前按 value 和 name 去重后不投影；此时 numeric literal 没有唯一源码别名。
- 相同值落在不同 owner/access scope，例如本样例的 `Inner.CONST_CDE_PRIVATE` 与外部 `TestCls.CONST_CDE`。不能仅按整数跨类选字段；对 `Inner` 当前保留 `3294` 才与 JADX 断言一致。
- 只有 `static final` 声明但没有该字段的 `ConstantValue`，或同值来自 `<clinit>`/运行期初始化。它不是已证明的 Java 常量表达式，引用它可能改变初始化与执行时机。现有 candidate 资格已排除此类字段。
- 字段名被方法参数或局部变量占用。当前 AST 扫描把这些名字加入 `occupied`，阻止生成会被遮蔽的裸字段引用；不可移除此保护。
- 某个 `int` 是合成表索引/映射值，却恰好与普通常量相同。按数值替换虽可能仍通过一个固定输入的运行，却会把内部编码误呈为源码常量含义。若真实方法中的 switch 不是用户常量域且不能由现有结构证书区分，应保留数字；不以此扩张本片。

JADX 对 duplicate 值保留 literal 的规则和 `TestSwitchLabels` 中 private/public 同值断言，是跨作用域负例。Jarde 候选的全表唯一性、ConstantValue 门和 AST 的 name-shadow 门已覆盖相应同类风险。

## 下一步必要 fixture

只需要一个真实冻结输入：沿用 `TestSwitchLabels.test`（`cf12-upstream-java-root-v1` 已保存完整两 class 和完整运行对照），在 standalone root 允许该既有 projection 后核 `TestCls.f1` 变成 `case CONST_ABC` / `return CONST_CDE;`，派生 ranges 分别锚定根物理字段和原 BCI；`Inner` 仍不得写 `case CONST_CDE_PRIVATE`，外类 `.CONST_ABC` 仍保持当前 numeric 并记为 resolver 边界。这个单 fixture 同时覆盖 family-Refused 触发条件、唯一同类命名、跨类重名负例和 exactly-anchored source output。`testWithDisabledConstReplace` 是 JADX 自身关闭选项的 oracle；Jarde 当前没有对应开关，不能当作 Jarde 的第二项验收。无需为名字新增 fixture/实体，除非该窄改动在原 fixture 上暴露新的失败事实。

## 判定

原始 TestSwitchLabels 上游契约有效；Jarde 当前整体运行行为已一致，但没有满足其 source-name assertions。对 standalone nested root 的同类常量缺口，最小修复落在现有 class-source caller gate，复用已有 `integer_constant_candidates`、同次 AST 和 field/method anchors，不需要新机制。`Inner -> outer CONST_ABC` 属不同身份/作用域的绑定问题，本计划不处理，也不能用它宣称整 CF12 完成。
