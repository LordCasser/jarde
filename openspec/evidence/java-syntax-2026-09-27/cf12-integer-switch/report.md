# CF-12：整数 switch 源码恢复审计

本轮只记录证据，不改生产代码、不新增 OpenSpec。基线是固定 JADX checkout `/Users/lordcasser/workspace/testzone/jadx`，HEAD `2fb1b16386941660fda07e9017285aec40fcb37f`；`replay.py` 钉住五个代表测试、关键实现及自动 check runner 的文件哈希。该范围共有 10 个账本测试；这里只静态审阅清单代表项，并用独立 Java 8 样本做三方完整类编译和运行，不声称已运行 JADX 的整个 Gradle 测试集。

## 测试断言与算法边界

- `TestSwitch.java` 断言字符 case、default、break 数和循环递增，但没有 `check()`；JADX 的 `IntegrationTest.runChecks` 会编译反编译输出，且发现公开 `check()` 时运行它。因此此项给出源码形状和编译证据，没有自己提供运行语义断言。
- `TestSwitchNoDefault.java` 只断言 4 个 break 和 switch 后的 `println`，没有 `check()`；它要求保留缺省 selector 路径，但未直接验证该路径的值。
- `TestSwitchLabels.java` 对默认常量替换及 `replaceConsts=false` 分别断言 `CONST_ABC`/数值两种源码输出，也覆盖私有嵌套字段不能被错误命名；没有 `check()`。
- `TestSwitchFallThrough.java` 断言 switch 与分支语句形状；嵌套 `check()` 覆盖 case 1 落入 case 2、case 2 本身及 default 的结果。`IntegrationTest` 会对反编译类执行该自动检查。
- `TestSwitchWithFallThroughCase.java` 断言 switch 和两层条件结构；其嵌套 `check()` 覆盖显式 break 与 fall-through 进入下一 case 的结果，并由同一自动检查机制执行。

`SwitchRegionMaker.process` 先按 target block 聚合 keys，再由 `addCases` 按区域顺序生成 case。它使用 dominance frontier 检测 fall-through；检测到顺序异常时尝试 `reOrderSwitchCases`，仍异常便明确输出警告 “Can't fix incorrect switch cases order, some code will duplicate”，随后放弃 fall-through 合并。这是算法已知限制：不可恢复排序可能重复代码。当前 Java 编译器样本没有触发该保守分支。`RegionGen.makeSwitch` 输出 region 中已有的 labels 和 case body；实现没有承诺任意 CFG 都保持单一、无重复源语句。

## Java 8 三方回放

固定输入源码与 runner 在 `input/`，完整反编译源码、原 class 和重编 class、编译/运行日志与 javap 在 `baseline/`。运行 `python3 openspec/evidence/java-syntax-2026-09-27/cf12-integer-switch/replay.py` 可重做。脚本确认 JADX checkout revision 与相关源文件 SHA，使用独立临时 Cargo target，并在构建 Jarde 后删除该 target。

原源码、JADX 完整类（只剥除默认包 CLI 添加的 `package defpackage;`）、Jarde 完整类分别通过 `javac --release 8 -g:none`，再以 `java -Xverify:all` 执行同一 runner。11 个 selector 的逐行输出完全相同：覆盖匹配/未匹配标签、同目标多标签、显式 fall-through 和没有 default 的保值路径。JADX 重编类 SHA-256 `b631927bf0557b89de102f37f2d4d6662642b58ec851059458302e1f48697a71`；Jarde 重编类 SHA-256 `28e29e1d2449ded82992e1b78e43412ea54c2d60dad23cc998a384585b216c3f`；原 class SHA-256 `b1a6be1a8c0c3ecc517cae946e298711f1367b5735c683c69d4812c015f9857b`。字节不同符合源码重编预期，验收依据是三份都能编译、验证并产生相同输出。

| 方法与结构 | 原 class 的 BCI 证据 | 恢复结果 |
|---|---|---|
| `grouped(I)I` 多个 label 共享目标 | BCI 1 `lookupswitch`: `1→36`, `2→39`, `7→36`, default→42 | 原/JADX/Jarde 均输出匹配 case 1 和 7 返回 11，命中行为一致。 |
| `fallthrough(I)I` | BCI 3 `lookupswitch`: `10→36`, `20→39`, `30→46`; case 10 在 BCI 36 `iinc`, 顺序落入 BCI 39；case 20 在 BCI 43 跳出到 BCI 49 | 三者都保留 10 落入 20 的语义，无额外 break。 |
| `noDefault(I)I` | BCI 4 `lookupswitch`: `4→32`, `8→39`, default→43（源码 switch 没有 default，default edge 到 join） | 三者都保留初值 99；缺省路径运行结果相同。 |
| `labelConstant(I)I` | 字段 `LOW`: flags `0x0018` (`static final`)，`ConstantValue: int 2748`; `HIGH`: flags `0x0018`，`ConstantValue: int 3294`; BCI 1 `lookupswitch` key 2748→20; BCI 20 `sipush 3294`, BCI 23 `ireturn` | 原源码/JADX 用 `case LOW` 与 `return HIGH`；Jarde 输出 `case 2748` 与 `return 3294`。运行相同，但源码常量名未恢复。 |

## 判定

CF-12 的控制流核心在这个代表样本上已追平：合并标签、无 default、源顺序 fall-through 都能完整编译并产生相同运行结果。发现一个窄的源码质量差距，而非语义/控制流错误：Jarde 丢失 `ConstantValue` 对应的字段名，未像 JADX 输出 `case LOW` / `return HIGH`。

JADX 常量恢复路径可在本地源码直接核对：`jadx-core/src/main/java/jadx/core/dex/visitors/ModVisitor.java` 的 switch visitor 调用 `replaceConstKeys`，其以 `parentClass.getConstField(keys[k])` 将匹配整数 key 替换为字段引用；`jadx-core/src/main/java/jadx/core/codegen/RegionGen.java` 的 `addCaseKey` 对 `FieldInfo` 调用 `useField`，最终通过字段 alias 输出标签，并用字段 `ConstantValue` 注释数值。JADX 测试 `TestSwitchLabels.test` 与 `testWithDisabledConstReplace` 对此有正反断言。Jarde 当前 `crates/jarde-java/src/emit.rs::switch_key` 只依据 key 数值和 selector 是否呈现为 `char` 格式化 label；整数 key 输出十进制数，符合本次观测。

该差距不改变控制流，也不影响编译或运行。建议后续将它作为独立的“整数 switch 常量字段别名恢复”窄任务；不要因此把 CF-12 标为控制流不支持。此报告没有实现该任务。

主线复核：root 用合入 CF-06/CF-08 后的 CLI（SHA-256 `95295d3e1688077b9cde0620739178525fb95b77084425540452b79623b791d1`）重新恢复同一原 class，完整源码 SHA-256 仍为 `ad3777d4808c25d7e601806f191deb10fe34edc419316e40ee62b115e83dae8a`，与本目录归档逐字一致；再次 Java 8 重编并以 `-Xverify:all` 运行同一 runner，11 行逐字一致。数值 case/return 的质量差距仍存在。
