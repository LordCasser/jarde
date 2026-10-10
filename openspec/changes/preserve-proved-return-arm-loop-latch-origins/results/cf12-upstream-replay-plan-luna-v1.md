# CF-12 五个上游测试的重放清单（只读准备）

范围依据本地 JADX `2fb1b16386941660fda07e9017285aec40fcb37f` 对应源码 `/Users/lordcasser/workspace/testzone/jadx/jadx-core/src/test/java/jadx/tests/integration/switches/`，及仓库 `openspec/evidence/java-syntax-2026-09-27/cf12-integer-switch/`、CF-12 账本行和 Jarde `region.rs` / `build.rs` / `ast.rs` / `emit.rs`。五个测试类均逐字阅读。下文不把近似样例当成上游测试通过；没有执行任何测试或工具链。

## 测试实况与基线映射

| 上游测试 / 活动 JUnit 测试 | 当前实际断言、依赖和潜在 oracle | 已有证据覆盖 | 最小完整输入与本项待验 |
| --- | --- | --- | --- |
| `TestSwitch.java` / `test` | 内嵌 `TestCls.test(String)`：取 `length`，循环逐字符 switch：`'.'` 与 `'/'` 合并标签后写 `'_'`；`']'` 写 `'A'`；`'?'` 空操作；default 原字符；循环后转 String。断言只查 `case '/'`、某缩进位置的 break/default、恰有一个 `i++`、总共四个 `break;`。未见 `@Disabled` / `@NotYetImplemented`。无运行断言或 `check()`，潜在 oracle 只是文本结构。 | `IntegerSwitchAudit` 有分组标签、default、break 的近似，但没有 char selector、switch 位于 loop 内、空 case、字符写回 builder 的组合。已有 CF-12 root acceptance 是 `IntegerSwitchAudit` 自定义类重编运行11行，并非这个 `TestCls`。 | 复制完整嵌套类及其外层壳，单独 runner 调 `test` 覆盖字符值：`. / ] ?`、default、空输入；与原/JADX/Jarde 完整类对照。scope 限于整数/字符型 switch 标签、同 target 多标签、空 arm/default 和 loop-body switch 的必要边界；不扩展到 loop latch/出口证明。 |
| `TestSwitchNoDefault.java` / `test` | `TestCls.test(int)` 初始化 `s=null`，四个 case 各写字符串并 break，switch 后 `println(s)`。断言恰四个 break，恰一个 println。无 `@Disabled` / `@NotYetImplemented`，也无调用 `check()` 的执行 oracle。要额外核实未命中时保留 `null`，以及全部四个 case。 | 自定义 `IntegerSwitchAudit.noDefault` 有两个 case、LOW/HIGH 与保留初值 99，重编运行包含若干输入；只覆盖“无 default、join 后续语句”的子形，不覆盖四 case 与 `null`/println。 | 原样类加 runner；输入 1–4 和 miss 值，捕获输出验证 `1`..`4` / `null`。完整来源核各 case 与 join 后 println 恰有一次，避免为未命中路径合成 default。 |
| `TestSwitchLabels.java` / `test`、`testWithDisabledConstReplace` | `TestCls` 含 `CONST_ABC=0xABC`、`CONST_CDE=0xCDE`；嵌套静态 `Inner` 有私有同值 `CONST_CDE_PRIVATE`，其 `f1` 用私有常量作 case 并返回外层常量；外层 `f1` 用 `CONST_ABC` case 并返回 `CONST_CDE`。第一测试检查外层 label/return 用常量名、私有常量名不泄露。第二测试**不是被禁用的测试**：它主动设 `replaceConsts=false`，断言数字 2748/3294 且没有常量名。两者都只作文本断言，没有行为 runner。未见注解禁用。 | `IntegerSwitchAudit.labelConstant` 仅有外层 LOW/HIGH 一条 case；既有类级 root acceptance 确认该单条映射以及方法级仍为数字。`integer_constant_name_tests` 5/5 是 Rust 定向 tests，不等于上游类内外两个方法 + 参数配置测试。 | 可用现有常量字段 fixture 作第一方法的对照，但应补齐原测试完整 nested class（特别是私有同值常量）与两种 `replaceConsts` 配置。类级输出应仅在源级字段候选可证时投影；方法级/关闭替换必须保留数字。运行两种 f1 的命中/未命中行为；源码边界包括私有重复常量不得误选。该项作为 CF-12 的上游输入纳入清单，但本轮实现不得把新增的常量名投影产品混入 switch 结构需求。 |
| `TestSwitchFallThrough.java` / `test` | `TestCls` 有字段 `r`；`test(int)` 初值 `i=10`，case 1 写 1000 后自然 fall-through 至 case 2，再令 `r=i` 并 break；default 写 -1 并 break；switch 后 `r*=2` 并 println。`testWrap` 重置字段后调用 test；`check()` 内三条 assert 验证 1→2000、2→20、miss→-2。JUnit `test` 本身只查 switch、两条赋值和两个 break；源码注释写“correctness checks done in check method”，但该测试没有调用 `check()`，不能据此算活动运行 oracle。未见 `@Disabled` / `@NotYetImplemented`。 | `IntegerSwitchAudit.fallthrough` 真正有 case10→20 连续执行，但初值及表达式不同；baseline runner 的 11 行是否包含该方法行为应按实际 runner源码核对，不能代替这份上游 `TestCls`。既有 evidence 明确仅自定义输入。 | 复制 `TestCls` 完整字段与三个方法；独立 runner 显式调用 `check()`，并直接覆盖 default 和每个 case。核 case1 的自然 fall-through、case2 直接进入、case后公共乘法一次。源代码提示标记/注释不是执行证据。 |
| `TestSwitchWithFallThroughCase.java` / `test` | `TestCls.test(int,boolean,boolean)` 对 `a%4` 分派。case1 写 `>`；当 `a==5 && b` 时内层 `if(c)` 写 `1` 否则 `!c` 并 break；否则自然 fall-through 到 case2。case2 条件写 `2` 后 break；case3 空 break；default 写 `default` 后 break；末尾追加 `;`。`check()` 有四条断言，但 JUnit `test` 未调用它，只查 switch/两个 if 的代码形状。未见禁用或 `@NotYetImplemented`。 | 既有 `IntegerSwitchAudit` 无条件 arm-internal break 与后续 case；不覆盖内层 if 条件提前 break 与 fall-through 并存、空 case3、`a%4` selector。已有 evidence 没有该上游类输出。 | 保留原完整类，runner 显式调用 `check()`，并另覆盖 `a==5,b=true,c=false`、`a==1,b=false/true`、各余数、负数（余数语义）及 default/空分支。区分 case1 条件 break 与 case1→case2 真 fall-through；case3 是直接离开 switch，不是空 fall-through。仅检查构建出的源码和 BCI/edge 所有权，不把分支内继续/外层 loop 纳入。 |

## 公共实现路径与复用结论

当前 Jarde 有正式 `StmtKind::Switch` / `SwitchArm`（`ast.rs` 约 878、949 行）；`SwitchArm.fall_through` 是显式语义。构造路径在 `region.rs::switch_region`（约 12670–12850 行）：同 target 合并 key，独立探测 arm 间 fall-through，按 target 排序，并要求每条 fall-through 对应相邻 case；不唯一/重叠时产生 `SwitchArmsOverlap` fallback。emitter 在 `build.rs` 的 `Region::Switch` 分支（约 17750 行起）创建 arms，并只在完整 switch join 且末 block 的最后操作是 Transfer、存在 canonical Normal edge 时把 goto 作为派生 break 来源；真正 fall-through arm 不添 break。源码输出由 `emit.rs` switch arm 输出逻辑负责（约 1120 行起），依据 arm 的 fall_through 与正文是否能正常完成来决定 break。此路径已经覆盖了基础整数 switch 的若干语义，应先对上游输入复放确认，不另立新 IR 或普遍 CFG 规则。

现有 CF-12 evidence 的 `IntegerSwitchAudit.java` 只有四个方法：分组标签、真 fall-through、无 default、单条常量标签/return。原/JADX/Jarde 的原始类/源码和 javap、运行输出齐备；root acceptance 声明 `javac --release 8` 与 `-Xverify:all` 的 11 行一致。它能作为通路 smoke fixture，不能视为五个 TestCls 的已冻结 baseline。当前基线中没有这五个上游 `TestCls` 的已验收原/JADX/Jarde 完整源、独立运行记录或逐 BCI map；计划应先为每一类原样取证，不能仅将现成 Audit 的行为输出复制过去。

## 五个子验收边界

1. `TestSwitch`: case key/合并标签/default、空 arm、break、loop-body 组合全部有来源；完整类运行覆盖所有字符形；不得扩查 loop latch/continue 语义。
2. `TestSwitchNoDefault`: 不构造虚假 default；未命中保持原初值；四个 case、公共 join 语句和 break 各有唯一来源。
3. `TestSwitchLabels`: 两个 `f1` 及两个 replaceConsts 配置都重放；数字模式不能受类级投影串扰；准确比较私有同值 case 的拒绝理由，不混入常量名功能实现。
4. `TestSwitchFallThrough`: 明确证明 case1 到 case2 的 normal 路径是 fall-through，且公共尾部只执行一次；显式执行原 test class 的 `check()` 作为行为 oracle。
5. `TestSwitchWithFallThroughCase`: 证明条件 break、真正的跨 arm fall-through、空 case、default 与 switch 后尾部各自来源唯一；显式执行 `check()` 并增加其未覆盖分支；不得将 case 内条件流误当作 CF-13 的 loop transfer。

每项都要保存固定原 class、固定版本 JADX 输出、Jarde 完整输出、运行对照和每条物理 switch/case/join/transfer 的来源映射；首先核原始 JADX 测试的断言到底覆盖了什么。上游测试文件数是 5，但现有已验收完整输入目前只有自定义 Audit 一类，因此有 **5 个上游测试输入均未作为各自完整类完成三方重放**；其中若干语义形状被 Audit 近似覆盖，不能折算成已验收上游测试数。唯一显式配置差异是 Labels 类的第二个活动测试（replaceConsts=false），并非 JUnit disabled。CF-13、当前 CF-07、String switch、以及本次常量名产品实现均排除在此验收工作边界之外。
