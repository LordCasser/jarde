# CF-14 字符串 switch 审计

日期：2026-09-27。此提交只增加审计证据与 inventory 状态，不改生产代码、不写 OpenSpec change。

## 固定来源与测试断言

基线为 `/Users/lordcasser/workspace/testzone/jadx` 的 `2fb1b16386941660fda07e9017285aec40fcb37f`；JADX CLI 为 1.5.6。来源文件 SHA-256 固定记录在 `baseline/jadx-source-sha256.json`：`TestSwitchOverStrings.java` `ba62fa81…aa710f`、`TestSwitchOverStrings3.java` `25c43a94…b9cba521`、`TestSwitchOverStrings5.java` `f2a72285…151c879c`、配套 `TestSwitchOverStrings5.smali` `dc7863e2…d38efa`、`SwitchOverStringVisitor.java` `10037b32…91913`。完整哈希以该 JSON 为准。

`TestSwitchOverStrings.test()` 是强正向样本：源码注释注明 `frewhyh`、`phgafkp`、`ucguedt` 哈希冲突；`.code()` 断言源级字符串标签、分组标签、返回值和无数字 hash 标签，嵌套 `check()` 覆盖已知标签、同 hash 未命中及未知值。JADX `IntegrationTest` 的编译和自动 `check()` 运行使它也有运行时语义验证。

`TestSwitchOverStrings3.test()` 覆盖外层 default 内嵌 String switch；文本断言检查 3 个 case、2 个 default 和 4 个返回，嵌套 `check()` 覆盖 a/b/c/d，IntegrationTest 编译并调用它。它不覆盖任意深度、交错控制流或独立 hash 使用。

`TestSwitchOverStrings5.test()` 来自 Smali issue #2359；测试禁用编译，只检查一个字符串标签及 case/default 计数，并允许 warning。因此该项只证实特定 Smali 形态的输出文本，不证明生成 Java 可编译或语义正确。

固定实现 `SwitchOverStringVisitor` 注释描述 javac 的两级 lowering，并兼顾 D8/R8 将首级改写为 if 或扁平化次级 switch 的情形。实现将模式分类为 `SWITCH_SWITCH`、`IF_SWITCH`、`SINGLE_SWITCH`，验证 hash key 分支中的 `String.equals`、hash literal 与判别值写入，再重映射第二级 switch 标签；合并有完整性校验，无法安全恢复时保留失败路径。其模式识别入口及合并路径见固定 SHA 的实现文件；JADX 当前固定版本对下文嵌套样本能恢复完整源 switch，但负边界会误用未定义 `r0`，故不能据此推断其所有 lowering 均可编译。

## 三方重编译与运行

`replay.py` 从输入 Java 源构建原始 class，分别以 JADX/Jarde 生成完整类，再以 `javac --release 8` 编译，并对可编译产物运行 `java -Xverify:all`。逐行运行结果、编译状态、输入与 class 哈希在 `baseline/summary.json`；BCI 来自对应原始 class 的 `javap.log`。所有样本均由独立临时 Cargo target 构建并在脚本结束清理。

| 样本 | 原始 class | JADX | Jarde | 判定 |
|---|---|---|---|---|
| `StringSwitchAudit`（哈希碰撞、分组 case、selector 单次求值） | 编译、验证、8 行输出 | 编译、验证，8 行逐行相同；一个 String switch，无 hash/equals/整数判别变量泄漏 | 同左；生成 class SHA 与原始 class 完全相同 | 已测形态语义和源码形状均追平 |
| `NestedStringSwitchAudit`（外层 default 中的内层 switch） | 编译、验证，a/b/c/d/null 五行 | 编译、验证，五行相同；保留两级嵌套 String switch | 完整源码编译失败：`missing return statement`；未生成 class，不能运行 | 明确控制流/源码生成差距，详见下节 |
| `ExtraHashUse`（hash 另用于 parity） | 编译、验证，六行 | 源码使用未声明 `r0`，`javac` 失败 | 编译、验证，六行相同；保留 hash/equals 与两个整数 switch | 负边界已正确拒绝折叠，hash 的可观察独立用途未丢失 |

正样本覆盖碰撞、碰撞未命中、多个字符串共享分支、未知值、selector 一次求值和 null 的原有 NPE 行为。负边界是有效 Java 8 源码，但不能把“JADX 失败”当作其算法规格；这里的用途是证明 lowering 折叠必须保留仍被后续代码消费的 hash 值，而 Jarde 当前保守输出满足语义。

## 嵌套样本的首个拒绝点与 BCI 归属

`NestedStringSwitchAudit.choose(String)` 的外层 String lowering：BCI 0 加载参数；BCI 5 调用 `String.hashCode()`；外层 hash `lookupswitch` 位于 BCI 8，key 97 分支到 BCI 28（`equals("a")` 后写 slot 4），default 到 BCI 39。外层判别 `lookupswitch` 位于 BCI 39：key 0 到 BCI 60 返回 1，default 到 BCI 62。BCI 62 是 Java 外层 `default` arm，随后初始化内层判别 slot 4（BCI 64–65），内层 hash 调用在 BCI 68，内层 hash `lookupswitch` 在 BCI 71：key 98 到 BCI 96，key 99 到 BCI 111，default 到 BCI 123。`b` 分支在 BCI 105–106 写内层判别 slot 4 为 0；`c` 分支在 BCI 120–121 写为 1。

内层最后一级判别 switch 本身位于 BCI 125（BCI 123 加载 slot 4）；其 0、1、default 目标分别是 BCI 152、154、156，对应返回 2、3、4。区域详情文件 `baseline/NestedStringSwitchAudit.region-details.json` 给出实际所有权：从 BCI 0 开始的结构化 `switch` region 拥有 `[0,28,37,39,60,62,96,105,111,120]`；另一个 `jre_region_uncovered_blocks` fallback region 从 BCI 123 开始并拥有 `[123,152,154,156]`。因而内层 case 写入留在结构化 region，而最终 dispatcher 与返回目标被切到外侧 fallback；Jarde 生成的 Java 只有该方法的 bytecode 注释和拒绝说明，没有可编译方法体。

首个可见拒绝发生在 `region.rs` recover tail 的 uncovered-block 扫描（约 1337–1357 行）：结构化走访完成后，BCI 123/152/154/156 尚无 region owner，于是首个 fallback 从 BCI 123 接管这些 live blocks，诊断 `4 live block(s) are reachable only through edges the normal-flow view leaves out`。随后 declaration planner 在 `build.rs` 检查 slot 4 时发现它跨越 quoted fallback，输出 `local 4 crosses a quoted fallback region`。后者是前述区域断裂的后果，不是首要修复点；也不应先规划 slot 4 声明来掩盖不完整的控制流 region。

“normal-flow leaves out”是诊断的原文，但目前 region-detail 输出没有列出导致失联的具体 canonical edge。`normal_flow.rs` 明确只纳入 canonical `Normal`/`Return`，排除 `Exception` 与 `Call`；其说明还明确普通 switch target 属于 `Normal`。因此证据支持的结论是：这四个 block 在结构化使用的 normal-flow 投影里不可达，不能据现有输出断言是哪种边被排除，更不能说投影按设计忽略了 switch target。样本没有显式异常处理器或 jsr 子程序；根因定位应继续核对 canonical CFG 中 BCI 123 及 BCI 125 targets 的边种类/连通性。可独立实现的窄切片是：让这组内层最终 dispatcher 和三个目标在外层 default 的同一结构化 region 中获得完整所有权，并保留内层 String switch；随后再验证 slot 局部绑定。不要把问题扩成所有 String-switch lowering 或 declaration planner 改造。

## 额外 hash 用途负边界的字节码

`ExtraHashUse.run(String)` 在 BCI 5–9 算出 `hashCode`，BCI 10–13 立即消费其低位并写 parity；判别值初始化在 BCI 14–15，首个 hash switch 在 BCI 18（2112→44、96354→74、default→86）。最终判别 switch 位于 BCI 88；返回路径在 BCI 141–145 将结果与 parity 相加。hash 值在 BCI 18 的常量分派之后仍由前面的 parity 消费，不能被 String-switch 合并器当作仅供 lowering 使用的临时值删除。Jarde 保留这段表达式并通过原/Jarde 六行对照；JADX 的 `r0` 编译错误是其输出缺陷，不影响本审计对负边界的定义。
