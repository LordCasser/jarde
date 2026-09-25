## Context

见 `proposal.md` 与 `../../evidence/java-syntax-2026-09-22/string-switch/analysis.md`。当前 `Region::Switch` 和 `SwitchGroup` 保留整数键，`StmtKind::Switch`/`SwitchArm` 也只打印整数 case。最小审计 class 中，第一个 `lookupswitch` 按 `String.hashCode()` 分桶，`equals` 比较各桶的字面值并写一个整数 discriminator，第二个 `tableswitch` 才进入源码各 arm。jarde 在这个最小输入上两个 switch 均可编译且行为一致，因而这是形态优化，不能牺牲已有正确性。`string-switch/variants/` 的共享标签/穿透扩展输入曾暴露第二级整数 switch 的 `SwitchArmsOverlap` 拒绝；父变更 2c.4 已修复该普通 switch 问题。当前工作树中该冻结 class 的第二级整数 switch 无引用、整类可重编，18 行执行结果与原 class 相同，见 [独立验收](../../evidence/java-syntax-2026-09-24/switch-fallthrough/analysis.md)。它仍未折叠为字符串 switch，不能把普通 switch 的闭合算作本变更完成。

## Goals / Non-Goals

**Goals:** 对同方法中完整可证的降级形状，写出一个 Java 8 `switch (String)`，保留 selector 只求值一次、碰撞比较、null 异常、default、共享标签与 fallthrough，并给所有被折叠指令来源。

**Non-Goals:** 不解码跨类的枚举映射，不把任意 `hashCode`/`equals` 链视为字符串 switch，不恢复 Java 14+ switch expression，不重排原 arm，不推导一般值域或类层级。

## Decisions

1. 在既有 SSA/CFG 与区域构造边界做一个有界的**局部形状认领**，只接受同一方法内的 selector 单次保存、对 `java/lang/String.hashCode()I` 的调用、有限 hash bucket、每个 bucket 中对已知字符串常量的 `String.equals(Object)Z` 判别、唯一整数 discriminator 写入与后继整数 switch 的完整映射。比较调用与常量池身份必须匹配；按 Java UTF-16 code unit 的 `String.hashCode` 检验每个常量确属其 bucket，重复/缺失标签、额外前驱、额外副作用或不支持的字符串拼写均不认领。名称或字节码邻近本身不是证据。与 `enumswitch@1` 的外部表问题不同，这里的所有判据都在一个方法内；不引入跨类依赖解析。
2. 认领结果须共同拥有两个原始 switch 与其中的保存/比较/赋值指令，区域输出再承接原最终 arm。现有 `Region::Switch` 保留原始整数键供普通路径使用；为这个跨两个区域的已证明形状加入一个局部结构变体比伪装成单个原始整数 switch 更忠实。构建层把被证明的字符串常量映射到原 arm，不复制可执行比较或 discriminator 赋值。失败时完全沿用当前两级 Java，不发布半折叠结构。

   当前 `region::recover` 已将 hash 分派与其 join 上的最终整数 switch 分别写成完整、相邻的普通区域，且先执行未认领块与 Code 覆盖检查。因此 2.2 的最小接缝是在该完整区域结果上识别这对区域，再用 2.1 证书校验同一方法的物理 BCI 与入边；预先构造持有两层所有权、沿用最终 `SwitchGroup` 的新局部结构，成功后一次替换这两个相邻区域，失败则不动原结果。`Recovered::claimed` 的块集合不变，结构自身的 `blocks()` 和来源必须同时包含两层，不能仅在 build/emitter 隐藏第一层文本。这样复用现有臂与穿透证明，也避开 JADX 先改 switch 指令、后替换区域的半提交顺序。
3. 共用现有 `StmtKind::Switch`、arm、break/fallthrough 与 emitter。`SwitchArm.keys` 继续保留原始整数键供方法证据和 enum 投影；当前 `enum_labels: Option<Vec<String>>` 已是一个只在完整类证明后才填的展示层 sidecar。字符串投影不要再叠一个可与它同时为 `Some` 的 `string_labels`：将这个 sidecar 收敛为一个封闭的已验证标签类型（枚举常量或字符串字面值），emitter 逐种拼写，普通整数键仍走原路径。区域原始键不强行转成字符串，只有完整形状证明成功才提供字符串标签。这样沿用已有投影接缝，不复制 switch 语句或打印器，也不会出现 enum/string 两套标签同时生效的歧义。
4. 选择器表达式经现有求值位置、绑定与类型合同呈现一次。第一层 `hashCode()` 对 null 抛出的异常由 Java 8 的 `switch (String)` 保留；异常处理边界若不相同则拒绝折叠。将 hash/equals/保存/discriminator/final switch 的真实 BCI 并入同一结构及子表达式来源，默认/all 只改变证据量，不改变正文。每个 SSA/边/标签访问遵守已有预算、取消和深度限制。
5. 无新增库：现有 classfile/SSA/region 已持有形状事实，Java 8 `String.hashCode` 与 `switch` 语义由标准定义。候选字符串若当前常量解码或字面量 emitter 不能逐字符保真，则维持两级输出，不添加通用 Unicode 规范化层。当前 `decode::constant` 经 `lossy` 把孤立 UTF-16 代理项写成 U+FFFD，单看 `ConstantValue::String` 已无法区分它和真正的 U+FFFD；本切片先拒绝含 U+FFFD 的候选。将来若要覆盖该字面量，须从原常量池 UTF-16 单元证明每个标签和 Java 源拼写完全一致，再用那些原单元计算 hash，不能仅放开该拒绝。

本地 JADX `2fb1b1638694` 的 `SwitchOverStringVisitor` 展示了值得复用的顺序：在已建好的区域中识别第一层 hash 分桶，对每个 bucket 校验 `String.equals` 字面量及 hash，收集 discriminator，再以第二层 region 的 arm 容器为准组装字符串标签。Jarde 保留这一“先收集两层关系，再使用最终 arm 所有权”的算法，但不直接沿用其可变 region 替换与删除指令方式；本项目需原子发布新的结构及完整来源。具体地，`replaceWithMergedSwitch` 先用 `BlockUtils.replaceInsn` 改写原 switch 指令，随后才调用可能返回 `false` 的 `part1Parent.replaceSubBlock`；失败时前一改写已经发生。Jarde 必须先完成全量证明和新结构构建，再一次性提交，不能让拒绝路径留下已改过的 selector。尤其 `prepareMergedSwitchCases` 会把未由显式整数 key 消费的字符串标签并到 default，并在没有 default 而仍有残项时只记告警；在本项目中这类标签须逐项证明确实沿第二级 default 边执行，没有合法 default 或存在重复键、多余效果就拒绝折叠。JADX 的清理还遍历 discriminator 的整个 code variable 的 SSA 赋值和使用并标记删除；Jarde 的证明必须把删除范围限定在已认领子图，检查其它消费与真实指令效果，不能因同名/同槽位变量而吞掉子图外代码。

测试证据也按实际门槛解读：本地 JADX 的 `TestSwitchOverStrings`、`2`、`3` 对 Java 源输入检查字符串 case；`4`、`5` 是 Smali 输入，均显式 `disableCompilation()` 并允许告警。后两项只说明特定输入能打印 case，不证明输出可按 Java 8 重编或执行等价；本变更的三方验收仍须自己编译、执行同一冻结 class，且合法变体的子图外消费者不能被清理掉。

[合法拒绝反例](../../evidence/java-syntax-2026-09-24/string-switch-negative-fixtures/analysis.md)把这个风险变成可执行门槛：`ExtraHashUse` 的 hash 整数同时供分桶和 `hash & 1` 使用，奇数 hash 输入使后者可观察；JADX 1.5.6 先折叠后清理，留下未声明的 `r0.hashCode()`，完整类 `javac` exit 1，当前 Jarde 两级分派仍与原 class 6/6 行一致。`WrongHashBucket` 把字符串比较放在错误的 hash bucket，JADX/Jarde 都保留整数路径且 5/5 行一致。故“hash 还有独立消费者”和“字面值实际 hash 与桶不匹配”是必须**拒绝折叠**的事实，不是只在清理阶段补一条告警的情形。

[中间 default 冻结样本](../../evidence/java-syntax-2026-09-24/string-switch-variants/analysis.md)给出一个可以比 JADX 更完整、但仍可局部证明的边界：javac 把未匹配字符串的初始 discriminator 设为 `-1`，它走第二级整数 switch 的 default；第二级显式 `case 2` 没有任何字符串比较写入，且与整数 default 共用入口，随后顺序穿透到空串 case。`case 2` 是不可达的空洞 key，不是缺失的字符串标签或未命中初值。JADX 的 `prepareMergedSwitchCases` 要求每个显式整数 key 都能从字符串映射取出，因而拒绝折叠。Jarde 可在完整控制流/SSA 证明初值仅为 `-1`、所有成功比较的写入集合完整且不含 `2`、其它路径不能产生 `2`、该空洞 key 与 default 共用入口、其后穿透关系已由普通 switch 规则证明且没有额外消费者时，忽略不可达的整数标签，仅把 default 路径映成源码 `default`；任一不成立就保留当前可执行的两级分派。不能把“缺字符串映射的整数 key”一律视为 default，也不能像 JADX 的告警路径那样丢标签。

依据：[JLS 8 §14.11](https://docs.oracle.com/javase/specs/jls/se8/html/jls-14.html#jls-14.11)、[Java 8 `String.hashCode`](https://docs.oracle.com/javase/8/docs/api/java/lang/String.html#hashCode--)。

## Risks / Trade-offs

- **误把任意 hash 分派折叠成字符串 switch** → 强制校验实际 String 方法、每个 literal 的 hash、整数目标的全映射、无额外入口及效应；任何缺口保持现有输出。
- **碰撞、null 或 selector 调用次数改变** → 以 `"Aa"`/`"BB"`、null、一次副作用 selector 与抛错 selector 的真实 JVM 对照验收。
- **标签顺序或 fallthrough 改写控制流** → 以最终 switch 的目标/边作为 arm 归属，不按字面值排序重建 arm；共享标签与 fallthrough 单列样本。复用已验收的 2c.4 真实入口链与最终邻接证明，字符串折叠仍须证明两层所有权和每个标签的最终目标；不能单靠重写标签跳过它。
- **新来源归属或工作成本失控** → 两层原始 BCI 全部映射，逐节点计费/轮询，并保留默认/all 正文一致与限额停止回归。
