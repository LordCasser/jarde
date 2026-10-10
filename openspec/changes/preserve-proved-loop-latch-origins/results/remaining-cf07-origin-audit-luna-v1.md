# CF-07 剩余来源缺口：`counted@20` 与 `lastIndexOf@25`

**root 最新复核（10:54 UTC）：** 已用冻结 method-aware probe 实际确认下述两个 Region 形状，独立接受文件为 `results/cf07-origin-probe-root-v1/diagnostic-acceptance-root-v2.json`。后文“尚待 probe”是 Luna 静态审计阶段的边界；最新动态事实见末节，不代表来源修复已经实现。

这两个缺口都落在已经结构化、并且整类重编运行成功的候选里；缺的不是 Java 语句，而是没有独立语句的 `goto` 来源。它们不是同一种 transfer：`counted@20` 是 `if` then arm 跳到共同 update 的分支汇合边，`lastIndexOf@25` 才是回到 loop header 的 natural latch。把两者都塞进当前 loop latch 规则会把 owner 记错。

本文只做证据与架构审计，为后续独立工作留边界。它不修改产品、正式 design/spec/tasks，也不扩大当前 loop-latch 接受范围。`cf07-candidate-root-v1` 的 `manifest.json` 当前记录 `candidate-replay-observed`；它声明四个 Jarde profile 的全类编译、运行与原始 oracle 对照成功，但 source-map 汇总仍分别缺 `counted@20` 和 `lastIndexOf@25`。全类成功说明生成的程序行为匹配，不代表物理 instruction 的来源表完整。

## 0x00 证据读取结果

输入是 [CF-07 原始 class 的 `javac23-original/javap.txt`](</Users/lordcasser/workspace/projects/jarde/openspec/changes/preserve-proved-loop-latch-origins/results/cf07-candidate-root-v1/cases/javac23-original/javap.txt>)。四个候选 JSON 是 `javac8/javac23 × default/all` 的 `rendered_profile.document`，集中在 [候选 manifest](</Users/lordcasser/workspace/projects/jarde/openspec/changes/preserve-proved-loop-latch-origins/results/cf07-candidate-root-v1/manifest.json>) 指向的 `cases/javac23-jarde-render/class-source-{default,all}.json` 与 JDK 8 对应文档中。四份 `physical_facts.method_facts` 给出相同结论：

| 方法 | 物理指令 | 候选已映射 | 缺失 | 候选源码中的结构节点 |
|---|---|---|---|---|
| `counted(II)I` | then block `17..20` (`iinc 2, 2`; `goto 27`); else block `23..26`; shared update/latch block `27..30` | `14`、`17`、`23..27`、`30` | `20` | `if` 的 then arm 与后续 update；`while` |
| `lastIndexOf([IIII)I` | header block begins `5` (test at `8`); return arm `19..21`; else block `22..25` (`iinc 4, -1`; `goto 5`) | `8`、`16`、`19..22`、`28..29` | `25` | `if/else` 与包住它的 `while` |

四份报告都把两个方法标为 `structured/java/contains_statements`，`fallbacks=[]`；两处缺失分别记录成 `missing_origin_bcis: [20]`、`[25]`。JDK 23 default/all 文档哈希分别是 `0347a3a4…d9c3ee8c`、`69bf0d03…58e0fe17`，JDK 8 两种 profile 也报告相同的来源集合。default/all 正文和来源表一致。

这些不是只看汇总得到的结论。JDK 23 raw render stdout 保存在 `streams/javac23-jarde-default-render.stdout.raw` 与 `...all...raw`；候选生成源码在 `cases/javac23-jarde-default/LoopCases.java`、`...all/LoopCases.java`。其中 `counted` 输出一个完整 `if/else`、接着一条 `local3 = local3 + 1`，`lastIndexOf` 输出一个 `if/else`，else 里递减，再由外层 while 继续。四个 profile 的 Jarde whole-class compile 和 run 命令在 manifest 的 `commands` 中均为 exit 0；比如 JDK 23 两个 run raw 的 stdout SHA-256 都是 `85487355…7154e41c`，与同 JDK original raw 相等。对应的 four-profile report 证明的是运行与结构观察；source-map 行仍明确缺了这两个 BCI。

## 0x01 `counted@20 → 27`：if 汇合，不是 latch

`counted` 的 loop header block 从 BCI 6 开始，条件分支在 BCI 8。body 里 BCI 14 测试 `local3 == 7`：then block 从 BCI 17 开始，先更新累计值，再以 BCI 20 无条件跳到 27；else block 从 23 执行乘法后落到 27。共同块 27 更新 loop index，块尾 BCI 30 再回到 header 6。因此 JVM CFG 的 then block 是 `17..20`，`20 → 27` 是 branch arm 绕过 else 并到达共享 update 的 transfer；natural back edge 是 `30 → 6`。候选 map 已把 30 作为 derived origin 放在 `while` 整体 span 上，但 20 没有来源。它不能由 `implicit_tail_latch_origin` 认领：20 的目标不是 header，且它不是 loop 的 natural latch。

Java 不需要 `goto 20` 语句。它是 then arm 结束并汇入 `if` join 的隐藏控制边，最贴切的来源 owner 是 BCI 14 的 `if` statement：父语句 origin 保持 `Direct(14)`，再把 20 作为 `Derived(20)` 加在同一个 `if` span。现有 `build.rs` 已经对“空 arm 只含一个到 join 的 goto”这样做：`Region::If` 的 branch BCI 作 primary；空 arm 有 transfer 时，把 transfer BCI 加入该 `If` 的 `OriginSet`（[build.rs `Region::If` 分支](</Users/lordcasser/workspace/projects/jarde/crates/jarde-java/src/build.rs:17440>)，尤其 17489–17518）。本例 then arm 不为空，因为 BCI 17 已发出赋值，所以现有 `statements.is_empty()` 门槛跳过了这条 transfer。JDK 23 JSON 正好显示 `if` span primary 为 14、derived 为空；17 的赋值和 27 的 update 各自有直接来源。

这条来源可以复用 `OriginSet::plus_derived` 和 branch statement 的已有来源模型，不需要新增 provenance 类型、emitter pass 或通用 CFG 扫描。当前 `Region::If` 携带 `join` 和两条 arm，但没有单独携带已证明的 hidden-transfer BCI（[region.rs `Region::If`](</Users/lordcasser/workspace/projects/jarde/crates/jarde-java/src/region.rs:432>)）。若不想让 `build.rs` 重复推导 CFG，最小表示调整是在 `If` region 上携带局部、已证明的 gateway origin 列表，并让 `build.rs` 按现有 `OriginSet` 派生到 if span。也可以由 Builder 使用已有 canonical edges、arm blocks 和 `join` 复核这一个严格形状；那会把控制流证明复制到 AST 构建层，审查时要确认不会出现两个判定源。不要把 BCI 20 挂到 loop 的 `gateway_origins`：`Region::Loop.gateway_origins` 的定义是“由 loop 结构而非 statement 表示的 transfer”（[region.rs Loop 字段](</Users/lordcasser/workspace/projects/jarde/crates/jarde-java/src/region.rs:534>)），这里的文字 owner 是 `if`。

最小准入证书应限于：一个已建立的 `If` 有 `Some(join)`；某一 arm 的最后一个真实指令是已解码 unconditional `Transfer`；该终端指令的唯一 normal successor 就是这个 `If` 的 join；终端 transfer block 由该 arm 唯一拥有，位于相同 method/path 和当前 frame scope；branch 与 arm 组成的 region 已精确覆盖；没有 exceptional/外部边被当成该 arm 的 join transfer。block 可以同时含 BCI 17 的业务指令和终端 BCI 20，前者仍由原 assignment segment 直接标记，只有尾部 goto 派生到 `if`。这里需要按 SSA/CFG 的 terminal instruction 证明，不可要求整个 block 只有 goto，也不能因 block 中出现一个 goto 就映射。

## 0x02 `lastIndexOf@25 → 5`：自然回边藏在 If 的 else 尾部

`lastIndexOf` 的 loop header block 从 BCI 5 开始，header test 是 BCI 8。body 的唯一明显控制结构是 BCI 16 的比较：命中路径执行 BCI 19–21 并 return；未命中路径所在 block 从 BCI 22 开始，执行 `iinc` 后以 BCI 25 `goto 5` 结束。也就是说 `22..25` 是回到 header 5 的真实 loop latch block；被遗漏的原因不是目标不可判断，而是候选 region 的 loop body 尾项是整个 `If`，不是顶层 `Region::Straight`。

当前 `implicit_tail_latch_origin` 只看 `body.last()` 是否 `Straight`，再验证其最后 block 是 natural loop 唯一 latch、最后指令是 `goto/goto_w`、唯一 successor 是 header、canonical outgoing edge 只有一条 normal edge（[region.rs `implicit_tail_latch_origin`](</Users/lordcasser/workspace/projects/jarde/crates/jarde-java/src/region.rs:10102>)；header-tested path 调用位置在 [region.rs](</Users/lordcasser/workspace/projects/jarde/crates/jarde-java/src/region.rs:10401>)）。这条 gate 对现有顶层尾块 BCI 30 保持窄，且解释了为什么 25 没被看到。当前 `loop_body_sequence` 将每个 region 作为有序 body 项累加；本例一个 if 就结束循环体（[region.rs](</Users/lordcasser/workspace/projects/jarde/crates/jarde-java/src/region.rs:11714>)）。来源目标应是 `while` statement span，所以复用 `Region::Loop.gateway_origins` → `build.rs` `OriginSet::plus_derived` 即可（[build.rs](</Users/lordcasser/workspace/projects/jarde/crates/jarde-java/src/build.rs:18067>)），不需要改 `emit.rs`。

局部扩展不应递归搜索整个 loop body 里的任意 `goto`。一个可控形状是：loop body 的最后且唯一 continuation region 为 `If`（或者严格序列中最后一个 region 为它）；一条 arm 以唯一 natural latch block 收尾、末条指令为回到当前 header 的无条件 transfer；另一条 arm 由已证明的 loop-exit/terminal return 结束；If 后无同层 loop body suffix；自然 loop 的完整 blocks 被 region tree 精确拥有一次；latch 唯一且非 irreducible；transfer 没有异常边/额外 successor；当前 frame scope 与 `loop_targets` 未把它归属到外层 loop。满足后仅把该 BCI 追加到现有 `gateway_origins`，并保留头部 test origin。出现两个 latch、尾部还有普通 body、transfer 到内层/外层其他 target、arm 不闭合或 owner 重叠时，不加来源。

## 0x03 Region 观察与 JADX 对照的边界

四份 CLI JSON 的 `evidence.requested.kinds` 只有 `source_map`，`region_details` 是 `not_requested`；所以冻结结果能直接证明 physical BCI、来源缺失和输出 AST 形态，但没有保存运行时 `Region` Debug 树。按 javap 中 branch target、fall-through 与 terminal transfer 重新划分，JVM basic block 边界为 `counted` then `17..20` / else `23..26` / join `27..30`，以及 `lastIndexOf` else `22..25`；精确的 Jarde canonical `Region` 树和 owner 仍需实际 probe，不把推导的 Region 结构写成已观察值。

JADX 源码参考固定为本地 checkout `2fb1b16386941660fda07e9017285aec40fcb37f`（roadmap 说明它是 `v1.5.6-26`），不等同于对照运行时安装的 JADX 1.5.6。源码 `LoopRegionMaker.process` 从 `LoopInfo` 选 loop exit/header，递归调用 `RegionMaker.makeRegion(loopBody)` 并在最后 `insertContinue`；`insertContinue` 只在 synthetic loop-end predecessor 等 dominator/exit 条件满足时注入 continue（[LoopRegionMaker.java](</Users/lordcasser/workspace/testzone/jadx/jadx-core/src/main/java/jadx/core/dex/visitors/regions/maker/LoopRegionMaker.java:80>)、[continue 规则](</Users/lordcasser/workspace/testzone/jadx/jadx-core/src/main/java/jadx/core/dex/visitors/regions/maker/LoopRegionMaker.java:739>)）。`IfRegionMaker.process` 选择 if out block、递归构造 then/else region，并处理合并和 edge instruction（[IfRegionMaker.java](</Users/lordcasser/workspace/testzone/jadx/jadx-core/src/main/java/jadx/core/dex/visitors/regions/maker/IfRegionMaker.java:58>)）。这说明 JADX 的 Region 模型也在结构边界消化低级 edge；它的可变图/合成指令是算法参考，不能替代 Jarde 的物理 owner、normal-edge 与 method 来源证明。实际四腿输出来自 baseline 记录的已安装 JADX 1.5.6，不应把 2fb 源码行当作该 release CLI 的输出证据。

## 0x04 最小复现与负边界

先用现有冻结 class 作为两个正例。正例 A 是 `counted`：断言 `if` 语句 span 有 primary BCI 14 和 `Derived(20)`；while span仍有 primary 8 与 `Derived(30)`；BCI 17、23、27 保持各自的 direct anchors。正例 B 是 `lastIndexOf`：断言 while span primary 8 新增 `Derived(25)`，BCI 22 的递减和 BCI 16 的 if 仍保留原直接来源；default/all 两 profile 对方法文本与 map 必须相同，整类 source/runtime 对照仍与 oracle 字节一致。

负例 A 可以从 `counted` 派生一个 class fixture：让 then arm 的 terminal goto 改到并非该 `If.join` 的有效目标块，并更新对应 stack-map frame；保持其它指令可验证。期待该 BCI 不出现在 if/loop derived origins 中，不能凭“这看起来像 if 结束”去映射。负例 B 从 `lastIndexOf` 构造两个不同 source block 回到同一 header 的 class（例如在循环中增加一个独立 `continue` 路径并用 `javap` 确认形成第二个 latch）；期待唯一-latch gate 拒绝把任一个 transfer 作为 loop 的单一隐藏 tail 来源。两个 negative 都应看完整 owner/CFG，不因 source text 能编译而弱化证明。

## 0x05 还需要 root 实际 probe 的事实

当前私有 `arm-loop-diagnostic-luna-v1.patch` 的 probe 会在 `report.rs` 打印 method begin/end，在 `region.rs` 记录 `loop-region-return` 和每个 `one-arm-return` 的 branch、start、join、regions、region BCIs、frame boundary/scope/loop targets。root 对同一 frozen class 各跑默认和 probe 后，需要从 raw stderr 确认：

1. 输出 method identity 确实分别是 `cf07/LoopCases/counted(II)I` 与 `cf07/LoopCases/lastIndexOf([IIII)I`，并有完整 recover begin/end；
2. `counted` 的 branch 14 arm tree 是否把 20 放在 then arm terminal，join 是否正是 27，块 owner/next 是否闭合到 update；`30` 仍是自然 latch；
3. `lastIndexOf` 的 loop header/exit、branch 16 两条 arm 的 Region/next、含 25 的尾块与 loop owner；确认 25 是该 loop 的唯一自然 latch，非 nested/outer target；
4. probe 与 default 的 stdout、exit、生成源码逐字一致；stderr 的变化只来自 `JRE_ARM_LOOP_PROBE` diagnostic 行。

在这些 raw facts 到位前，内部块归属是从 javap/输出和现有算法推导的假设。后续实现要按实际 region tree 收紧局部规则；如果形状不同，先重审，不把新的 branch-tail/latch 证据顺手纳入当前已验收的 `andWhile@15`、`counted@30` patch。

## 0x06 root 实际方法探针

root 对 frozen javac23 原 class 用独立诊断 CLI 实际执行 default/probe 两调用，均 exit 0，未编译或修改产品。`counted(II)I` 返回 Loop(header6, exit33)，body 为 If(branch terminal14，then Straight[17]，else Straight[23]，join27) 后接 Straight[27]，gateway_origins=[30]；`lastIndexOf([IIII)I` 返回 Loop(header5, exit28)，唯一 body 是 If(branch terminal16，then Straight[19]，else Straight[22]，join=None)，gateway_origins为空。其 raw 位于 `cf07-origin-probe-root-v1/1.stderr.raw`，完整方法 begin/end 包围各自 loop return，不借别的方法行推断归属。

首次 capture 要求整个 JSON 字节相同，因 elapsed_millis 不同而实际失败，原 execution 的 failed/AssertionError 保留。root 独立 verifier-v1 误读 command streams 字段失败也保留；v2 实际 exit 0，重新核 argv 两次 exit/guard、全部 raw hashes、冻结诊断 CLI/class hash、当前17产品 pins，并递归确认 JSON 仅10条 elapsed_millis 路径不同；所有非耗时内容（包括完整源码、source map、physical methods 和结果）恒同，default stderr为空且 probe stderr仅诊断行。准确接受为 diagnostic-only，不要求 wall-clock usage 相同，也不据此宣布完整 BCI 或新来源恢复。

后续应分别创建 if 汇合来源与尾部 If latch 来源 change；动态事实补齐了 Region 形状，canonical edge/SSA terminal/唯一 natural latch 与 owner 门禁仍须在各自实现和正负用例中核验。不能把 lastIndexOf 的 join=None 偷换成已闭合 If join；不得扩当前已验收顶层 Straight latch 片。
