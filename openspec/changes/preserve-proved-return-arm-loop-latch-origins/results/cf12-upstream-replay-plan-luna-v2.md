# CF-12 上游 harness 自动检查调用链更正（只读）

这是对 `jarde-cf12-upstream-replay-plan-v1.md` 中两处“JUnit 未显式调用 `check()`，所以没有 runtime oracle”的归因更正。v1 保留作为未执行的旧稿；本稿只按已读源码解释测试运行时会触发什么，**没有实际运行 JADX JUnit、原 class 或 decompiled class**。

## Harness 调用链

JADX `IntegrationTest` 源码显示：

- `@BeforeEach init()` 在 `IntegrationTest.java:134-161` 设 `compile = true`，初始化 `JadxArgs`；没有配置禁用 compile 的测试类调用 `disableCompilation()`。
- `getClassNode(Class<?> clazz)`（约 187-195 行）先 `compileClass(clazz)`，随后 `getClassNodeFromFiles`。后者（约 217-231 行）解析类并调用 `decompileAndCheck(cls)`。
- `decompileAndCheck`（约 291-305 行）执行 `cls.decompile()`、`runChecks(clsList)`。`runChecks`（312-315 行）依次 `checkCode`、`compileClassNode`、`runAutoCheck`。
- `compileClassNode`（约 520-534 行）在 `compile==true` 时编译反编译输出。`runAutoCheck`（432-440 行）先反射源类 `check()`；若找到且反编译编译打开，则自动调用反编译类的同名 `check()`。
- `runSourceAutoCheck`（445-477 行）对公开、非静态、void、零参 `check` 建实例并执行，限时 5 秒；没有 `check` 方法则返回 false。`runDecompiledAutoCheck`（479-484 行）反射执行反编译类同一方法。
- `JadxClassNodeAssertions` 的 `code()`（约 25-34 行）只读已解码文本、不会单独触发 checks；但这些测试在链式 assertion 之前调用 `getClassNode`，完整检查已由上述流程自动执行。显式 helper `reloadCode`/`runDecompiledAutoCheck`（约 45-65 行）是额外入口，本批测试没有调用它们。

据此，v1 对 `TestSwitchFallThrough.test`、`TestSwitchWithFallThroughCase.test` 的“没有 runtime oracle”是错误的：按 harness 源码，两者的源 `TestCls.check()` 会自动执行；编译成功后反编译 `TestCls.check()` 也会自动执行。JADX 源码注释“correctness checks done in check method”与 harness 一致。只有在实际执行JUnit并取得结果后，才能报告它们确实通过；静态链路不等于本次运行证据。

## 五个测试逐项的 auto-check 归属

| 测试 | 调用 `getClassNode` 的活动 JUnit 测试 | 源 check 自动执行 | 反编译 check 按源码条件执行 | 配置/输入计数 |
| --- | --- | --- | --- | --- |
| `TestSwitch` | `test` | 无：`TestCls` 无公开实例 `void check()` | 不触发：source check 不存在，且未调用 `forceDecompiledCheck()` | 单一源 fixture / 默认 args。循环中 switch 语义由文本断言覆盖，活动测试无运行 check。 |
| `TestSwitchNoDefault` | `test` | 无：`TestCls` 无 check | 不触发 | 单一 fixture / 默认 args；文本断言，不存在自动行为 runner。 |
| `TestSwitchLabels` | `test` 与 `testWithDisabledConstReplace` | 无：`TestCls` 无 check | 不触发 | 同一完整源文件、两个不同 JUnit 配置执行。`JadxArgs.java:135` 默认 `replaceConsts=true`；第二测试在调用 `getClassNode` 前设置 false（`TestSwitchLabels.java` 对应测试体）。因此分母应记**1个源码/原始class输入，2个JADX配置输出**，不能当作两个独立 fixture。`testWithDisabledConstReplace` 是方法名，不是禁用标注；源码未导入/使用 `@Disabled`、`@NotYetImplemented`。 |
| `TestSwitchFallThrough` | `test` | 有：`TestCls.check()` 的 3 个 AssertJ 行为断言按 `runAutoCheck→runSourceAutoCheck` 自动执行 | 有：source check 返回 true 且 compile 默认 true，`runAutoCheck` 会调用已编译的 decompiled `check()` | 单一源 fixture / 默认 args。JUnit 另有文本断言；动态覆盖包含 1/2/miss 三路径。静态源码只能证明 harness 会尝试执行，不能替代实际执行结果。 |
| `TestSwitchWithFallThroughCase` | `test` | 有：`TestCls.check()` 四个行为断言自动执行 | 有：同上；源 check 存在且 compile 默认 true | 单一源 fixture / 默认 args。check 覆盖 `a=5,b=true,c=true` 的条件 break、`a=1,b=true,c=true` 的 fall-through、`a=3` 空 case、`a=0` default；未覆盖 `c=false`、case2 的 `b=false`、负数余数等边缘，需额外 replay runner 覆盖。 |

以上五类源文件均通过 `getClassNode(TestCls.class)` 编译源文件，再只将匹配的 TestCls class 送进 JADX；外层测试壳不是目标产品 class。Labels 的嵌套 `Inner` 会由源文件一并编译；基线 replay 应保留目标 TestCls 与 Inner 字节码，不遗漏私有同值常量 case。

## 当前基线能说明什么

仓库 `cf12-integer-switch` 的 `IntegerSwitchAudit.java` 是一个自建 fixture，不是上述五个 `TestCls` 的逐字输入。已有原/JADX/Jarde class、完整源码、运行输出、javap 与 root acceptance，明确覆盖分组 key、无 default、一个 fall-through、一个常量 label/direct return；它对 TestSwitchFallThrough 的语义近似可作已有证据，但没有同一个上游类的源/JADX/Jarde 三方运行。TestSwitchNoDefault、TestSwitch、TestSwitchWithFallThroughCase、TestSwitchLabels 也没有各自的完整类验收包。Labels 的 `replaceConsts=false` 是第二个必须保留的配置 leg，已有数字模式只代表自建类里一个 label 方法，不覆盖上游嵌套私有字段的完整类。

因此应准确计数为：**5 个上游源 fixture；6 个上游 JUnit 方法执行（Labels 同 fixture 有默认/关闭替换两次）；5 个测试类均没有各自的完整三方冻结 replay**。已验证的 Audit 近似覆盖不能折算成这些 fixture 的 accepted count。两个含 `check()` 的类有明确自动运行 oracle 的 harness 路径，另外三个只有文本 assertion；Labels 有两份配置输出而非两份源输入。

下一阶段 replay 应保存每个类的原始编译 class、JADX 输出、Jarde 完整输出及运行日志。`TestSwitchFallThrough` 与 `TestSwitchWithFallThroughCase` 必须记录源 `check()` 和 decompiled `check()` 的实际执行结果；可额外增加边界输入，但不要把新增输入伪称为 JADX 原测试断言。`TestSwitch` 与 `TestSwitchNoDefault` 应用独立 runner 添加行为路径，因为源码测试本身没有 check。Labels 两配置要分别验：默认替换下正确常量引用、关闭替换下纯数字；不要从第二配置推断常量字段投影实现。

实现范围仍仅是 CF-12 整数/字符 switch 的 key 分组、顺序、default、fall-through、break 与 join。`TestSwitch` 的 loop 只是包围上下文，本片不审 CF-13 loop exit/latch；不纳入当前 CF-07；不把 `recover-proved-integer-constant-names` 的产品投影扩成 switch 结构机制。
