# 单臂分支内前缀循环：root接续入口

当前tasks **5/7**。root实际 validation-build-root-v5 的12命令全部成功，556 passed/0 failed/1 ignored；独立验证重新核24个raw streams和50个产品/测试/canonical pins，结果见 results/validation-acceptance-root-v1.json。八份完整生成类重编运行已成功且自身精确产品 CI 已独立接受；全 BCI 接受仍待独立 for 来源修复。

root在已推送产品404b422e141e200f0eea000f64937becedbad664上实际apply-check/apply method-aware probe，格式化，按5GiB机器/1GiB target守卫构建独立诊断CLI，再对两份冻结javac23 class执行default/probe四调用。真实raw在results/arm-loop-diagnostic-luna-v1，应用/撤销逐字hash在results/probe-application-root-v1，root只读复核接受在results/diagnostic-acceptance-root-v1.json。report.rs/region.rs均已逐字恢复，loop来源产品49pins保持；diagnostic CLI不作为产品CLI。

| 物理方法 | 实际首段 | 返回的循环头 | arm Frame boundary |
| --- | --- | ---: | ---: |
| VariablePostfixLoop/countEmpty(List)I | Straight[6] | 13 | 45 |
| PlainOneArmLoops/prefixWhile(ZI)I | Straight[6] | 8 | 23 |
| PlainOneArmLoops/loopAndTail(ZI)I | Straight[6] | 8 | 26 |
| PlainOneArmLoops/takenArm(ZI)I | Straight[6] | 8 | 23 |

四个外层分支的真实terminal BCI为3，canonical branch块仍为0；start=6，scope/own_loop均None，loop_targets为空。循环头到达时已有非空prefix，因此region_at先返回Straight与next，单臂caller现仅尝试boolean early-return续接，不能接这个首段。noPrefix直接返回已有Loop(header6,next17,gateway_origins=[14])，正常闭合。default/probe完整及方法正文、source map恒同；探针没有恢复新的语法。

动态证据确认design的prefix停止假设；**尚未动态证明**首段之后的loop与可选tail能够闭合。Luna private实现应只复用同一arm Frame下的统一region_at与已有loop proof，准确检查唯一入口出口、完整owner、新visited与父scope，拼接非空Straight prefix、一个普通Loop和可选Straight tail至原join。不得硬编码单块prefix、BCI或假定loopAndTail的loop exit即外层join26；其tail必须单独证明。失败仍按既有整方法拒绝，Stop原样传播，不加Frame/publicIR/pass/通用回滚。

下一步：root读审private patch并在资源守卫下实跑永久正反例及fresh完整类对照。原iterator及Plain双JDK/default-all八份完整源码必须原样重编运行、逐字核raw与所有物理BCI来源；已完成来源片noPrefix@14不能豁免其它可能出现的来源缺口。新缺口须先实证，再独立拆分，不扩大本片。loop来源产品自身CI38045578457运行中，不借其结果完成本片CI。71/612分母与整单元完成数不变。

11:05 UTC root已应用并格式化 private `prefixed-loop-private-luna-v1.patch`；新实现限于原 one-arm caller，沿同一Frame续接已证Straight prefix、一个Loop与可选Straight tail，不新增Frame/IR/pass。已有双臂调用传入entry_source=branch，原地址和loop owner/entry/exit门禁保持；新单臂以prefix最后块为实际entry，依靠canonical边与exact新增owners证明。失败仍整方法quoted-whole，无rollback。

root静态核frozen bytecode CFG leader后从iterator owner测试删除BCI32（它是block22内的aload，而非分支target/后继leader）；instruction origins与block owner分别核。validation-build-root-v1实际fmt通过、CI同范围Clippy因新scope测试初始化写法而exit101；真实raw与failed execution保留。root读审后追加followup，拒绝外层loop header及continue_target，并以struct update构造scope/boundary测试，不调整lint；privatefollowup原hunk因rustfmt展开context而apply-check失败，root按已审相同semantic delta应用，记录patch-application-root-v2。

validation v4使用同一组来源pins与5GiB机器/1GiB target实时守卫，并在Clippy成功后清理check-only metadata、每项测试成功后清理对应test binary。root已启动v4；目前尚无其最终结果，因此尚无新冻结CLI或候选全类接受。BCI42是否存在来源缺口仍必须以新CLI实跑核验，不按静态猜测豁免最终来源门禁。

## 2026-10-10 root 实际构建进展

validation v4在正确接受for显示后暴露真正的loopAndTail拒绝：region_at到Frame boundary返回Straight与None，调用方错误要求Some(boundary)。root读审并应用单行修正，只接受None且仍验证exact canonical entry→tail→boundary、scope、无侧入口/出口/异常及新增visited精确，双臂门禁不改。v5实际12命令成功；Java lib335、gateway7、root arm-join4、double-jumps7、terminal5、effectful12、reader178、fingerprint5+1 ignored、Boolean3均通过。target峰值701581213 bytes。冻结CLI /private/tmp/jarde-prefixed-loop-cli-v1，SHA472c5f957470671252649da688287895596bb8008342fb5d35e7e51d40d1e18a；metadata SHAfe55d6aa49d1fc7bbe2b56f23e92df998acc6bee69d5f9ba07ccc23eda58d555。准确路径results/candidate-cli-v1.json。

完整类collector v1因私有脚本JDK manifest pin转抄错误在工具链执行前失败；root以两个已SHA绑定的accepted baseline collectors核正确pin，v2随后真实完成16条命令，却在第17条前因机器余量5178011648 bytes低于5GiB停止。该次未完成，不声明候选完整类成功；原始字节分别在candidate-replay-invocation-root-v1/v2及candidate-whole-classes-root-v1/v2。v3已准备逐命令journal，待余量满足后重新完整执行，绝不以旧部分成功补齐。

root随后只cargo clean本仓target，删除3022 files/669.1MiB，冻结CLI保留、target不存在；实际记录results/root-clean-build-v1。其它进程耗盘使完成清理时机器余量仍低于5GiB，不下调已授权下限，不清除其它项目或用户数据。OpenSpec全量strict真实337 passed/0 failed，见results/openspec-strict-root-v1。For投影goto@20及iterator@42尚未在本CLI实跑确认，不据静态审计宣称缺口或豁免全BCI门禁。

## 14:49 UTC root 完整观测及自身 CI 接受

candidate-whole-classes-root-v3 已真实完成57命令，原4/JADX8/Jarde8共20个完整编译运行腿全部成功，八份Jarde完整源码原样生成与重编，exit/stdout/stderr逐字同各自原class。default/all正文及source map恒同、物理origin绑定有效；原形countEmpty包含goto@42且全部BCI完整，Plain ctor/noPrefix完整，prefixWhile/loopAndTail/takenArm四profile各仅缺goto@20。full verifier-v3真实退出1，raw在candidate-verification-invocation-root-v1，不存在full acceptance。root随后实跑verify-observations-only-root-v1退出0，接受文件candidate-whole-classes-observation-acceptance-root-v1.json明确accepted-observations-only/full_bci_acceptance=false，重核28物理方法行及准确12缺口组合，不勾3.1。

精确产品ca43ac74767c4609284f13b251de9d6c349c67a4自己的CI38048944005已捕获并由root实际verify-ci-product-root-v4独立接受：4jobs/52steps、两seed各354结果记录与3383passed/0failed/97ignored；七gateway、两新增region及旧整数11项逐seed通过。50源码/测试/include pins逐一同ca43 Git blob与当前源码、冻结CLI及12命令24raw同本片v5。准确接受文件results/ci-product-v1/acceptance-prefixed-one-arm-loops-root-v4.json，实际调用记录ci-verification-invocation-root-v4。私有v1/v2脚本的动态expected/testnames/不存在字段转抄错误未执行；root v3实跑因freeze没有built_binary字段退出1，原失败raw保留，v4按真实schema核build命令/CLI冻结而成功。共享SHA固定CI helper字节不改，stdout与stderr交错处理仅在本片独立适配器内核准确测试名和summary。

下一步为独立OpenSpec preserve-proved-for-latch-origins（规划4/4、strict有效、tasks1/6），复用现有ForHeader与implicit-tail helper/gateway_origins，不扩大循环识别。3.1依赖新CLI八份完整生成类全部物理BCI独立接受；3.3待最终干净交付，71/612及EM23/CF07整单元统计不变。

15:16 UTC依赖来源门禁完成：preserve-proved-for-latch-origins新CLI已实际执行57命令/20完整类腿、八份Jarde原样完整源码与28方法profile全部BCI独立接受，且CF07新29命令/10完整腿由root v2独立接受scoped observations。新旧正文和既有来源全部保留，只有三方法四profile的derived for@20新增。故本片3.1现关闭，旧v3 full失败不改写。当前6/7，3.3待来源产品d714a6bcc自身CI与最终main干净交付，不能把旧ca43 CI当新来源产品CI。

15:54 UTC依赖来源产品d714a6bcc自身CI38062706229已root独立v2接受4jobs/52steps、双seed3385/0/97与50pins。当前6/7，最后3.3只待本次文档/账本/handoff实际干净main交付，旧ca43自身CI及历史失败不回写。

最终7/7：当前依赖来源已全BCI及自身CI闭合，8784e2f07 clean主线实际交付审计见For/results/delivery-clean-root-v1/execution.json；所有14辅助worktrees clean detached main祖先，仅main无待合入/分支占用，无target。历史ca43/旧CLI失败及observations-only记录仍保留，不冒增整单元。
