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

## 当前完整源码矩阵

产品`30afa7c9ab2818f41959f9ea6a8ca6f896714dc1`，冻结CLI SHA256 `05e0272eb8cbc65acb87418cecda6d15571ddaea84c041d30436c1238e60b89d`。4945项源码与native/实际Git blob和mode绑定；叶片正常归档提交`4be31d03bf77880496a9b586f4f8193e3ff0b2fe`已快进交付main，后续父文档不是另一产品CI。

| 两真实编译器来源，各自完整保留方法 | 当前报告 | 整类原样javac与rebuilt-only Java8/23 | 观察范围 |
| --- | --- | --- | --- |
| FY，各5方法 | 全Java/Structured，无fallback | 两次编译0、四次main逐字等原oracle | noCall/loopless/emptyFin固定main，值3000/107/3000及原打印 |
| FX，各6方法 | 全Java/Structured，无fallback | 两次编译0、四次main逐字等原oracle | finContinue/finReturn/finBreak固定main，204/3000/100及fin:1、fin:3 |
| FO，各5方法 | 全Java/Structured，无fallback | 两次编译0、四次main逐字等原oracle | evalThenFin值/字段1/100 |

实际24命令/48raw，6完整类/32方法，不删方法或修文本。相对前一CallReturn CLI，28个非transfer方法记录文本相同，四个continue/break记录按本次授权移除旧handler45 fallback。新纯Python核对32方法/724物理指令实例，所有BCI来源与PhysicalMethodId相符，missing/extra均空；它没有执行Java。原样生成JAR类与经真实WAR枚举/LoadRoot访问的对应类另由归档driver验证，不能扩称其它入口或任意档案。

固定JADX1.5.6实验21命令中19成功、2次预期FX编译失败（undefined th，line56）；FY/FO四完整类编译通过、8次main一致。FX没有rebuilt运行，包名defpackage按实记录；不修JADX文本，不宣称三工具全部可执行一致。原始/本产品/JADX三方是对照记录，外部工具失败不证明本产品正确。

上面最终运行和独审原始材料在[leaf final-runtime-v1](../archive/2026-10-17-recover-loop-transfer-finally-handler/results/final-runtime-v1/README.md)，默认stack、资源与本地29门/120个不同passing FQN、双seed自身CI每seed3680/0/98在该叶片相邻implementation-v1和own-ci-v1。124次本地pass appearances含4次重复FQN，不能写124个不同测试；CI是产品30afa，stable1.99.1与本地1.99.0明确区分。

## 当前仍红的finally返回或显式抛异常

新增[remaining-controls-v1](results/remaining-controls-v1/README.md)保存6命令、12raw、原始类、原样生成文本、物理来源核对和独审。EX.finReturn(I)I（`try{return n;}finally{if(n==0)return -1;}`）前一b4a6/当前05e方法文本逐字同，仍`jre_region_uncovered_blocks`；未重编EX整类、未重建运行，不能沿用旧“if/else已恢复”说法。

FinallyCompletion两真实Java8 class各907B，各有7个Code方法（旧文档写6有误）。完整生成文本各4114B，真实javac8和23各3处missing return：line47/58/69分别对应finallyReturns、finallyThrows、tryThrowsFinallyRuns。normalReturn/mark/构造器/clinit已有结构，其余保留fallback和具体诊断。两次编译都失败，0 rebuilt运行；原source/Runner/215B oracle由历史pins复核未变，但本轮没有新原始JVM replay。原oracle记录normal 1:12、return覆盖4:34、finally异常56、try异常78是历史基准，不计本轮成功。

三种红项中，finallyReturns和finallyThrows的finally自身return/throw超出本父非覆盖型清理边界；`tryThrowsFinallyRuns`的try主动抛原异常、finally正常完成是下一有界调查候选。先核真实Facts/SSA/保护区和异常身份，不能直接复用死handler证明或忽略另外两红方法使整类编译过关。父全锚/对照/全门验收仍按tasks逐项完成，不借新包自动勾选。
