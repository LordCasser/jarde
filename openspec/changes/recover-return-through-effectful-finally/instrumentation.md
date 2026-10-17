# 父级实际取证与当前矩阵

本记录补齐历史Q-i/Q-ii，并区分当前源码恢复、有限运行观察、静态证明与未解决形。原巡查Java/输出保持历史原样，不把同名“local crosses”诊断当成相同原因。

## 历史Q-i：loopless的真实首拒

[conditional叶片instrumentation](../archive/2026-10-16-recover-conditional-return-finally-copies/instrumentation.md)保留旧Facts/CFG/SSA探针。唯一catch-all为`[0,16)→37`；旧线性后缀选择normal_return34、return_load31、cleanup25/26/29，实际首拒为row.end16不等于cleanup起点25。后续边检查当次未执行，不能声称已触达。真实块26同时含48/49尾部，没有独立block48；只改cleanup候选仍须证明通往26的落穿出口。

最终叶片用受限Continuation::Tail分别呈现return与fallthrough，保存字段返回值后再执行cleanup。独立`LooplessFinally`双编译器456B fixture与真实class反例/停止/公库/CLI测试在该归档；永久证明测试`conditional_return_proves_three_copies_and_the_saved_field_value`。这不是finally自身return，也不是后续循环局部iinc缺口。

## Q-ii：两种循环返回分别证明

[NoCall设计](../archive/2026-10-16-recover-local-increment-loop-finally/design.md)：两腿62B Code/36指令；自然循环10/16/40、latch40，rows `[22,34)→46`、`[46,48)→46`。54更新/57回跳融合于块40；旧Region没有terminal return，return28在原循环scope外。正常cleanup34/40读cphi，handler48读含normal34新值与自身反馈的另一phi；不能凭slot同号把它们当同值。证书核全保护操作nonthrowing、保存返回值链与handler46→load51→throw53，且局部授权唯一已证返回块。永久测试`local_increment_loop_finally_proves_actual_carry_and_saved_return`，fixture `tests/fixtures/recover-local-increment-loop-finally`。

[CallReturn设计](../archive/2026-10-16-recover-call-return-loop-finally/design.md)：两腿127B Code/60指令、7块8边，rows `[17,30)→88`、`[88,90)→88`，header7/latch59，return23原先在loop scope外。c0@25真实may_throw且受row0保护；这里不能借用NoCall不抛前提。30/59/90三份cleanup分别分配StringBuilder，按各自NewSite/dup/构造器初始化、append链与真实SSA consumers核对应；保存调用结果后执行cleanup，异常store88→load116→throw118保持原对象。永久测试`call_return_loop_finally_proves_three_fresh_cleanup_graphs_and_saved_call`，fixture `tests/fixtures/recover-call-return-loop-finally`。纯c0在driver中不能直接观察次数/时点，受保护try异常也未被该driver触达；这些责任来自静态证书和来源，不虚报动态覆盖。

## FO与循环转移的边界

[FO归档](../archive/2026-10-16-recover-static-field-prefix-return/results/fixed-recovery-v1/README.md)保存`1/100`完整类对照，永久测试`static_prefix_proves_two_real_copied_new_values_and_rejects_unwritable_declarations`核同一可写static int字段、iadd/dup新值及唯一write/return消费者；这不泛化为任意字段或调用。

[transfer归档](../archive/2026-10-17-recover-loop-transfer-finally-handler/design.md)核两个方法61B/35指令，continue31→53、break31→59；rows `[22,28)→45`、`[34,39)→45`、`[45,47)→45`。保守canonical异常边16/34/45→45保留，实际throw-sites仅5/19/52且handlers均空，所有保护操作nonthrowing。证明handler保存/重抛链和phi后，Builder成功才原子移除fallback45并独立认领该块。真实AST是For13，派生来源含42/56/45/47/50/52，不冒充gateway。永久正证明`loop_transfer_dead_handler_proves_actual_nonthrowing_ranges_and_distinct_targets`；原baseline refusal probe不计正证明。fixture `tests/fixtures/recover-loop-transfer-finally-handler`的11观察driver只证明实际输入结果，不声称动态触发不可达handler。

## 历史30afa完整源码矩阵

产品`30afa7c9ab2818f41959f9ea6a8ca6f896714dc1`，冻结CLI SHA256 `05e0272eb8cbc65acb87418cecda6d15571ddaea84c041d30436c1238e60b89d`。4945项源码与native/实际Git blob和mode绑定；叶片正常归档提交`4be31d03bf77880496a9b586f4f8193e3ff0b2fe`已快进交付main，后续父文档不是另一产品CI。

| 两真实编译器来源，各自完整保留方法 | 当前报告 | 整类原样javac与rebuilt-only Java8/23 | 观察范围 |
| --- | --- | --- | --- |
| FY，各5方法 | 全Java/Structured，无fallback | 两次编译0、四次main逐字等原oracle | noCall/loopless/emptyFin固定main，值3000/107/3000及原打印 |
| FX，各6方法 | 全Java/Structured，无fallback | 两次编译0、四次main逐字等原oracle | finContinue/finReturn/finBreak固定main，204/3000/100及fin:1、fin:3 |
| FO，各5方法 | 全Java/Structured，无fallback | 两次编译0、四次main逐字等原oracle | evalThenFin值/字段1/100 |

实际24命令/48raw，6完整类/32方法，不删方法或修文本。相对前一CallReturn CLI，28个非transfer方法记录文本相同，四个continue/break记录按本次授权移除旧handler45 fallback。新纯Python核对32方法/724物理指令实例，所有BCI来源与PhysicalMethodId相符，missing/extra均空；它没有执行Java。原样生成JAR类与经真实WAR枚举/LoadRoot访问的对应类另由归档driver验证，不能扩称其它入口或任意档案。

固定JADX1.5.6实验21命令中19成功、2次预期FX编译失败（undefined th，line56）；FY/FO四完整类编译通过、8次main一致。FX没有rebuilt运行，包名defpackage按实记录；不修JADX文本，不宣称三工具全部可执行一致。原始/本产品/JADX三方是对照记录，外部工具失败不证明本产品正确。

上面最终运行和独审原始材料在[leaf final-runtime-v1](../archive/2026-10-17-recover-loop-transfer-finally-handler/results/final-runtime-v1/README.md)，默认stack、资源与本地29门/120个不同passing FQN、双seed自身CI每seed3680/0/98在该叶片相邻implementation-v1和own-ci-v1。124次本地pass appearances含4次重复FQN，不能写124个不同测试；CI是产品30afa，stable1.99.1与本地1.99.0明确区分。

## 历史30afa阶段的finally返回或显式抛异常

新增[remaining-controls-v1](results/remaining-controls-v1/README.md)保存6命令、12raw、原始类、原样生成文本、物理来源核对和独审。EX.finReturn(I)I（`try{return n;}finally{if(n==0)return -1;}`）前一b4a6/当前05e方法文本逐字同，仍`jre_region_uncovered_blocks`；未重编EX整类、未重建运行，不能沿用旧“if/else已恢复”说法。

FinallyCompletion两真实Java8 class各907B，各有7个Code方法（旧文档写6有误）。完整生成文本各4114B，真实javac8和23各3处missing return：line47/58/69分别对应finallyReturns、finallyThrows、tryThrowsFinallyRuns。normalReturn/mark/构造器/clinit已有结构，其余保留fallback和具体诊断。两次编译都失败，0 rebuilt运行；原source/Runner/215B oracle由历史pins复核未变，但本轮没有新原始JVM replay。原oracle记录normal 1:12、return覆盖4:34、finally异常56、try异常78是历史基准，不计本轮成功。

三种红项中，finallyReturns和finallyThrows的finally自身return/throw超出本父非覆盖型清理边界；`tryThrowsFinallyRuns`的try主动抛原异常、finally正常完成是下一有界调查候选。先核真实Facts/SSA/保护区和异常身份，不能直接复用死handler证明或忽略另外两红方法使整类编译过关。父全锚/对照/全门验收仍按tasks逐项完成，不借新包自动勾选。

## 历史范围纠正：EX的其它非覆盖型正向欠账

本轮完整EX的actual当前JSON（remaining-controls包`return-controls/run-v1/2.stdout.raw`）还保留finThrow与nestedFin，二者均quality=fallback。finThrow诊断含两次jre_guard_finally_copy、uncovered blocks及未呈现字段；nestedFin含finally_copy、uncovered及未呈现字段。它们不能只因不在固定FY/FX/FO名单就排除出父的其它未证明正向形。

已封EX.class是本轮真实输入，原EX.java不在该轮inputseal，故源码描述只作定位：finThrow为try内分支抛错或返回、finally println；nestedFin为try算术返回、finally内再嵌套finally打印。物理Code/异常表和实际SSA/Facts需分别核实，不能从当前源码文本假定相同。尚未对EX完整生成源码重编或运行；不报告它们新行为成功。

最后三项曾按收窄后的固定fixture文字得到足够证据、短暂勾成7/7并生成status/apply收据；随后root重审完整EX发现原父范围协调过窄，在任何规格同步或归档前纠正。2.1与最终3.1/3.2恢复待办，已通过的固定对照2.3保留，当前4/7。原7/7与窄范围assessment不删除，均为历史阶段输入，不是父最终验收。下一片必须分清try主动抛错、嵌套cleanup与finally自身非局部完成，保持所有原完整类和红项。

规格边界决定：本父2.1继续覆盖具有try返回路径的EX.finThrow/nestedFin，并新增对应窄验收scenario。FinallyCompletion.tryThrowsFinallyRuns没有正常return路径，保留为正向相邻欠账但转后续独立变更先定义规格；当前不把它当负控制，也不把它的实现算成本父return场景已授权完成。


## 最终4c6852组合产品与父级验收

产品 `4c6852ac8c4ab9ce88f0716f4d2bd702e15684c5` 已推送；冻结CLI 62697056B、SHA256 `4811eea2d7f52d4c1b818c86acc1d961c91e3d7fe9d5adaa2c53131f84140f8e`。以下均为root实际执行和独立审读；本文件前述30afa矩阵与4/7范围纠正是历史，不替代本节证据。

| 实际范围 | 结果 | 收据（nested叶片results） |
| --- | --- | --- |
| FY/FX/FO两编译器完整6类/32方法 | 32方法全Java/Structured、文本同旧基线，完整BCI/映射/身份接受；6原样编译/12 rebuilt-only main逐字等固定历史原oracle | parent-fixed-matrix-root-v1 |
| nestedFin独立双编译器类及完整封存EX | 12原样编译、24 rebuilt-only运行、432case全匹配；20方法配置全BCI、EX六方法完整，非nested文本同旧基线 | final-matrix-root-v1 |
| 同一最终CLI的finThrow回归 | 4配置/8原样编译/16 rebuilt-only运行、144case全匹配 | finthrow-regression-final-cli-root-v1 |
| FinallyCompletion双907B完整class/default-all | 7方法保留；4份4114B源码、8原样编译均仅三处missing return，无runtime | finally-completion-refusal-controls-root-v1 |
| 父29本地门 | 131 passing appearances/127不同合格FQN、双strict365/0、MSRV1.88、CI同款Clippy；分两轮实际完成，初次直接cargo +toolchain启动失败保留 | parent-final-gates-root-v1 |
| native完整workspace双seed | 各376targets/98批/3691通过/0失败/98忽略，完整4933来源和inventory及每target计数相同；404raw核验、默认栈/资源守卫/最终exact clean，target不存在 | workspace-seed1-root-v1、workspace-seed2-root-v1 |
| 产品自身CI39042611463 | 4jobs/52 API steps，两seed各376targets/3691/0/98，14项关键门各一次通过；MSRV/JDK额外对照/层级/supply chain/fuzz通过 | own-product-ci-root-v1 |

JADX父矩阵仍21命令19成功/2 FX编译失败、0 FX runtime；nested另6编译成功/12运行在out异常时首错IOIO，原始失败保留。原JADX outer catch包住正常cleanup的做法会重复执行清理，因此实现复用Java嵌套finally语义并以四份out的真实无handler归属证明，不照搬其结构。

完整EX现已原样编译成功，但finReturn仍是相同fallback，不冒称恢复finally自身return；FinallyCompletion的finallyReturns/finallyThrows仍拒绝，tryThrowsFinallyRuns仍是下一独立正向欠账。该欠账只读备忘见results/exception-only-next-analysis-root-v1，尚未做实际Facts/SSA probe或选择新机制。

29门的两轮执行和MSRV启动失败不拼接成未发生的一轮；中文javac诊断的原英文分类器失败亦保留，root只修只读接受器，不重写生成源码。产品CI属于4c6852，后续文档提交只按生产源码恒同绑定。最终父文档更新后两版本strict365/0，父7/7、nested16/16，见tasks及nested的final-delivery-root-v1；不增加71/612总账，也不自动声称本父或两叶片已同步归档。
