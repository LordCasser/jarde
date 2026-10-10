# 71 单元账本下一批候选（只读 triage）

依据当前 `openspec/evidence/jadx-feature-inventory-2026-09-27/summary.md`（71 单元、612 个 Java 测试文件）及仓库根 `handoff.md`。handoff 明确要求先完成当前 If 自身 CI/交付，再处理 CF-07 `lastIndexOf@25`；本清单不抢占这些工作，也不把单个已通过样例算作整单元完成。以下三项分属 switch、成员类声明、字段更新，可由不同实现/验收腿并行准备；每项完成仍需覆盖账本单元自身分母。

| 顺序 / 单元 | 已有 change 与基线可用性 | JADX 测试和算法入口 | 剩余验收与清晰边界 |
| --- | --- | --- | --- |
| 1. CF-12 整数 switch | 已有 `recover-proved-integer-constant-names`，其已交付的是常量名投影窄片；基线在 `openspec/evidence/java-syntax-2026-09-27/cf12-integer-switch/`，含原始 class、JADX/Jarde 源、javap、运行记录及 root acceptance，可直接复核，无需重造输入。账本状态为“部分已测”；此前 11 行三方运行只覆盖一个完整样本，不代表五个上游测试均闭合。 | `TestSwitch.java`、`TestSwitchNoDefault.java`、`TestSwitchLabels.java`、`TestSwitchFallThrough.java`、`TestSwitchWithFallThroughCase.java`。算法参照 JADX `SwitchRegionMaker.process/addCases`、`SwitchRegion`、`RegionGen.makeSwitch`。 | 按五个固定测试分别核对活动断言、物理 case/落空顺序及完整类 Java 8 重编运行；重点保留“相邻 case 可合并”与真实 fall-through 的区分，以及缺省分支和常量名负边界。现有验收不能替代其余 lowering 的逐项来源证据。不要顺带做 CF-13 的 switch/loop 出口或当前 CF-07 的循环来源。 |
| 2. DT-02 静态成员类 | 账本为“冻结差距已修复，单元待扩验”。已有 `static-member-basic` 与 `static-member-basic-fixed` 对照；Leaf、根类和 `$` 命名顶级 `Named$Top` 的输入/原/JADX/Jarde 输出与编译运行记录实际存在。已验窄形 `static class Leaf`、`Leaf make()`、`new Leaf()`，跨类/多子类等仍未覆盖。 | `inner/TestInnerClass2.java`；生产参考 `dex/visitors/ClassModifier.java`、`codegen/ClassGen.java`。 | 从固定上游测试断言出发扩到多个成员类型、真实跨类使用点及名称边界。继续证明成员关系来自 class 元数据，不因 `$` 字符串而把独立顶级类并入。当前单个 Leaf 基线能用作校准锚，但尚无账本全测试的完整冻结投影/负例包；先确定测试分母和真实活动断言，再建最小增量 fixture。 |
| 3. EM-23 字段更新 | 账本为“部分已测、首片质量差距已修复”。已有 `spell-proved-field-unit-updates`、`recover-int-field-multiply-updates` 等窄 change；`em23-field-updates` 中完整 Java 8 原/JADX/Jarde 输入、输出和运行基线在本地可用。整数单位更新与嵌套接收者乘法已单独验收；当前 handoff/账本均明确这些窄片不增加 EM-23 整项完成数。 | `arith/TestArith.java`、`arith/TestFieldIncrement.java`、`arith/TestFieldIncrement2.java`、`variables/TestVariablesDefinitions2.java`。算法参考 JADX `PrepareForCodeGen` 的 `ARITH_ONEARG` 与 `InsnGen` 的 `ARITH` 分支。 | 逐项核实实际启用的字段前/后缀、复合更新、局部值流、接收者副作用次数；`TestArith` 中精确 `+=`/`++` 断言标为 `@NotYetImplemented`，不得按通过或有效正例计入。字符串 `+=` 属 EM-27，条件局部自增及循环控制分别受既有 EM-23/CF 单元边界约束；不要把 CF-07 或 outer TryLoop 来源债务带进来。 |

选择理由：这三项都有可读的上游测试和本地固定输入，且当前不是 handoff 指定的活动 CF-07/If 工作；它们分别落在 switch lowering、成员类型识别、字段表达式上，能拆开准备证据与代码审查。优先级是启动顺序建议，不表示对整体剩余工作的完整排名；尤其 DT-02 只有一项上游测试、EM-23 有未完成断言，必须先校准有效分母。CF-13、CF-18、CF-15 未列入本批：前者与循环出口相邻，后两者容易和异常区/外层 TryLoop、TWR 的独立债务交叠；不应为追求表面并行把它们混为一片。

本文件是私有规划记录；没有运行工具链、Git、JADX、CLI 或 OpenSpec 命令，也没有改动仓库文件。
