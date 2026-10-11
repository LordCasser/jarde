# CF12 剩余五类上游覆盖审计

审计范围是 `control-flow.md:111` 列出的整数 switch 十类分母，以及本地 JADX 固定测试源码和 CF12 `direct-harness-root-v3` / `full-replay-root-v1` 的真实清单。仅做文件、源码、JSON、脚本和既有记录读取；未运行 JUnit、JDK、Jarde CLI、Cargo 或 Git，也未写 workspace。

## 覆盖结论

CF12 仍是 10 类单元。当前已冻结的真实上游批次覆盖 5 个 JUnit 类、6 个 `@Test` 方法、8 个 class 输入实例、6 个唯一物理 class。两个 Labels 配置各捕获 root + Inner，造成 class 输入实例大于唯一 class 数。full replay 有 6 个 fixture/profile case，各生成原/JADX/Jarde-default/Jarde-all 三种 source row，共 18 rows；不是十类全量重放。

另五个上游 fixture `TestSwitch2`、`TestSwitch3`、`TestSwitch4`、`TestSwitchSimple`、`TestSwitchWithFallThroughCase2` 仍在本地固定 JADX 源码中，但没有进入当前冻结证据：direct harness 的 v3 `CASE` 常量明确只选 `TestSwitch`、`TestSwitchNoDefault`、`TestSwitchLabels`、`TestSwitchFallThrough`、`TestSwitchWithFallThroughCase`；相同的 24 条 `source_files` pin 也只含这五个测试源码。捕获 execution manifest 明确记录 8 个 input `.class`，只来自这五类中的五个类（Labels 的两个 profile 各两件）；full replay 的 `cases` 明确仅有这五类的六个 case row。接受 JSON 还将状态标成 `cf12_complete: false`。仓库 evidence 下没有这五个名字的冻结产物路径；这里的结论主要依赖上面的 runner 输入清单和已验收 manifest，而不是用搜索无结果推断覆盖缺失。

核查的固定源码位置：

- `/Users/lordcasser/workspace/testzone/jadx/jadx-core/src/test/java/jadx/tests/integration/switches/TestSwitch2.java`
- `/Users/lordcasser/workspace/testzone/jadx/jadx-core/src/test/java/jadx/tests/integration/switches/TestSwitch3.java`
- `/Users/lordcasser/workspace/testzone/jadx/jadx-core/src/test/java/jadx/tests/integration/switches/TestSwitch4.java`
- `/Users/lordcasser/workspace/testzone/jadx/jadx-core/src/test/java/jadx/tests/integration/switches/TestSwitchSimple.java`
- `/Users/lordcasser/workspace/testzone/jadx/jadx-core/src/test/java/jadx/tests/integration/switches/TestSwitchWithFallThroughCase2.java`

五份源码均只有一个 JUnit 5 `@Test public void test()`。没有 `@Disabled`、`@NotYetImplemented`、`disableCompile`、`disableConst`、不完整测试标志或同类配置调用；它们继承 `IntegrationTest.init()` 的 `compile = true` 默认值。`IntegrationTest.getClassNode(TestCls.class)` 编译原测试源码并只把目标 `TestCls` 类输入 JADX；每个 JUnit 方法中的 `getClassNode(...).code()` 字符串断言是格式 oracle。`IntegrationTest` 随后编译 JADX 生成源码。若目标原类含 public、非 static、无参 `check()`，框架会先运行原类的 `check()`，并在生成源码成功编译后再运行生成类的 `check()`。这比单独看 `@Test` 方法体多了一层真正的运行 oracle。

## 五个测试的有效 oracle 和依赖

| Fixture | 测试中的静态源码断言 | 实际 check / 运行 oracle | helper 和关键范围 |
|---|---|---|---|
| `TestSwitch2.test` | 目标 `TestCls` 输出恰有 4 个 `break;` 和 4 个 `return;`；另一个期望 2 个 return 的断言被注释并带有 `TODO: remove redundant returns`，不是 oracle。 | 类没有 `check()`；JUnit 不验证字段状态或执行 action 序列。 | JDK 算术/`Math.abs`，字段读写。测试夹具源码行 18–52，断言 56–64。全量语义 replay 若要执行，需要 probe 对 `test(int)` 输入与字段终态做原 class/生成 class 对比；单靠该测试没有既存运行输入 oracle。 |
| `TestSwitch3.test` | 恰有 3 个 `break;`、0 个显式 `return;`。 | `check()` 调 `test(1/2/3/4/10)`，检查字段 `i` 依次为 `1/2/2/5/5`；IntegrationTest 自动对原 class 与可编译的生成 class 各运行。 | `check()` 内静态 `JadxAssertions.assertThat(i)`（继承官方 AssertJ/JADX test helper）。源码行 14–40，断言 44–50。switch 含两个直接返回 arm 和 default break，检查“不生成 return”的重构结果。 |
| `TestSwitch4.test` | 恰有一个 `switch (`、3 个 `case `，且代码不含 `break`。 | `check()` 调 `parse("123",0,3)` 与 `parse("a=1234",2,4)`，断言 123/1234；自动在原类与生成类运行。 | `@SuppressWarnings({"FallThrough", "unused"})` 只压 javac 警告；case 4→3→2 物理 fall-through、char[] 累加，目标 helper 使用静态 `JadxAssertions`。源码行 12–29，断言 32–39。 |
| `TestSwitchSimple.test` | 恰有 5 个 `break;`；保留一次 `System.out.println(s);` 与一次打印 `"Not Reach"`；保留 `switch (a % 4) {`，排除多余括号形式 `switch ((a % 4)) {`。 | 类没有 `check()`；JUnit 不执行 `test(int)`，因此字符串行为没有原始 check oracle。 | 普通 String 局部、五个 break 与一个不可达 case 4。源码行 12–32，断言 35–44。 |
| `TestSwitchWithFallThroughCase2.test` | 输出恰含一次 `switch (a % 4) {`、`if (a == 5 && b) {`、`if (b) {`。 | `check()` 的五个结果：`(5,true,true) -> ">1+-"`、`(1,true,true) -> ">2+-"`、`(3,true,true) -> "+-"`、`(16,true,true) -> "default+-"`、`(-1,true,true) -> "-"`。IntegrationTest 自动运行原类和可编译生成类。 | 条件 case-1 break：`a==5 && b` 内再分 `c`，否则 fall through 到 case 2；case 3 与 default 各 break。`@SuppressWarnings("fallthrough")`。check 使用 `org.assertj.core.api.Assertions.assertThat`，静态代码断言使用 `JadxAssertions`。源码行 14–52、56–63。 |

这里的 `disableCompile`/`disableConst` 问题需区分**上游配置**与候选 CLI profiles：这五个上游 fixture 本身未禁编译、未禁常量替换。现有完整回放的 `default`/`all` 是 Jarde evidence profile，不是上游 `disableConst` 开关；Labels 的 `testWithDisabledConstReplace` 才是单独的上游 replace-constants 负控制，本次不要把它当成其他 fixture 的配置。

## harness 复用与最小追加方案

优先在新版本私有产物中沿用现有三步，不覆盖 `root-v3` 的原始脚本/结果：

1. **Direct upstream capture。** 从现有 `jarde-cf12-direct-harness-root-v3.py` 做独立新版本，`CASE` 追加五个类名，`SOURCE` 因此新增五个原测试源码，保留原来的官方 JUnit SDK、`IntegrationTest`、`JadxInternalAccess`、capture extension、`TEST_INPUT_PLUGIN=java`、进程组磁盘守卫和原有输入全部不动。预期清单形状是 10 个测试 class、11 个 JUnit method（Labels 自身有两个 method），但测试 pass/fail 数要以实际新运行记录为准。capture observer 会按原 IntegrationTest 的真实 target 输入捕获 class；三个带 `check()` 的源检查和可编译生成后检查由框架自动执行，无须额外模拟。
2. **Full-class source/render。** 沿用 render runner 对 `capture/*/input/*.class` 的遍历，不需要增加 fixture 分支；它会对新输入做实际 `javap`、Jarde default/all 的 `single-class` 完整报告。生成全新的 render 路径并 pin 新 CLI/JDK/manifest，不改历史 24-command evidence。捕获目录共增加五个新唯一目标类；Labels 两 profile 的重复类仍按真实 bytes identity 去重。
3. **完整重编运行。** full replay runner 本身也是 capture-driven，但 `Cf12RuntimeProbe` 的 case switch 只有当前五种类名，遗漏类会落入 `default: AssertionError`。新 runner 至少新增五个显式 case 分支：`TestSwitch3`/`TestSwitch4`/`TestSwitchWithFallThroughCase2` 可优先调用其原有 `check()`；`TestSwitchSimple` 用 `test(int)` 覆盖正/负 remainder 与 default 并逐字节比较 stdout/stderr；`TestSwitch2` 对公开的 `test(int)` 安排状态可控的 action 输入（覆盖 0、1/6、2、5及 `action=261` 这一 `action & 255 == 5` 且原始值大于 10 的路径），逐步比较六个实例字段的终态。FallThroughCase2 可先用原有五值 check，若目标是覆盖到该完整输入域，再沿用已有条件 fall-through 的 `a=-5..9 × b/c` 小矩阵；矩阵是附加观测，不替换上游 check。原/JADX/default/all 全部来自同一捕获 class 与完整生成 source，助手类继续从 target class 输出目录排除。

新增 runner/manifest 需使用新版本路径与独立验收记录。旧 `verify-observations-root-v6.py` 对源数 24、JUnit 总数 6、capture class 数 8、render 命令数 24、replay row/command 数 18/39 和 429-file closed inventory 都有精确旧常量，不能直接拿它验新分母；应从它复制出新版本，把核验量从新执行 manifest/文件集精确固定，不降低旧 evidence 的验证要求。最终只能在这五类的新 source/check、原/JADX/Jarde完整类编译和运行 raw 都实际记录并按真实 pass/fail 分类后，才把它们记入CF12新增覆盖；不预设 Jarde 能通过。

## 依据位置

- CF12 十类原分母：`/Users/lordcasser/workspace/projects/jarde/openspec/evidence/jadx-feature-inventory-2026-09-27/control-flow.md:111`。
- 当前 direct CASE/SOURCE 和实际的源编译、测试选择、守卫：`/Users/lordcasser/workspace/projects/jarde/openspec/evidence/java-syntax-2026-10-11/cf12-upstream-java-root-v1/jarde-cf12-direct-harness-root-v3.py:7`。
- capture-driven complete-source replay 及其按 fixture 类名 dispatch 的 probe：`/Users/lordcasser/workspace/projects/jarde/openspec/evidence/java-syntax-2026-10-11/cf12-upstream-java-root-v1/jarde-cf12-full-replay-root-v1.py:1`、`.../Cf12RuntimeProbe.java`。
- 真实 `check()` 的自动执行与 `compile = true`：`/Users/lordcasser/workspace/testzone/jadx/jadx-core/src/test/java/jadx/tests/api/IntegrationTest.java:98`、`:138`、`:282`、`:314`、`:435`、`:537`。
- 当前验收分母和未关闭标志：`/Users/lordcasser/workspace/projects/jarde/openspec/evidence/java-syntax-2026-10-11/cf12-upstream-java-root-v1/observation-acceptance-root-v6.json`；源码/捕获的明确现存清单见同目录 `harness-v3-java/execution.json`、`full-replay-root-v1/execution.json` 与 `render-root-v1/execution.json`。
