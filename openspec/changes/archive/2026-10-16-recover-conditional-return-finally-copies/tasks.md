# Tasks

## 1. 实际红基线与永久回归

- [x] 1.1 将实际a89新CLI/4915源码、两独立原class、未改source/driver/oracle及完整FY/FX重筛绑定并保存可移交raw；冻结永久fixture和指纹。验收：两原class的唯一row[0,16)→37、50B Code和三cleanup事实分别核实；12命令原类控制及完整生成源码缺return红可复核，四次driver134B exact；FP只新增本片预期输入、不删旧项。原类成功不计为恢复。
- [x] 1.2 在真实Facts/SSA固定现有首拒链及region边界，加入最窄证明、公共库与JSON CLI具名回归并在旧生产先证行为红。验收：具体失败指向两正常出口/第三copy的缺证，非工具或路径故障；记录guard首拒/若后续路径未到不得冒称到达，原FY loopless与完整FY/FX其它失败分别保留。

## 2. 完整证明与结构化交付

- [x] 2.1 实现最小有预算三副本完成证书，证明值保存/唯一consumer、row/handler/三cleanup等价与每个出口的真实continuation；同步边界文档和负控制。验收：真实分析后copy差异、自保护范围、异常对象/返回值误连、额外前驱或消费者不被新证书认领；真实late预算与收费中取消无部分证书/ownership且保留原原因用量，不用入口预取消/退款/改P5快照代替。
- [x] 2.2 复用bounded Region/Try/Return/Finally接缝接入两正常完成，完整认领与原子发射并补来源/兼容测试。验收：保存值在实际try内求得，落穿回真实join，全部物理BCI与方法身份闭合；不完整child/fallback/未声明local保持原拒绝；既有直线/条件清理/共享catch/循环finally与FO控制按自身契约通过，P5 all-features旧计费原断言通过，不放宽其它规则。

## 3. 完整源码与最终产品门

- [x] 3.1 冻结最终CLI，对两腿完整独立report.text原样重编，未改driver只对rebuilt编译并在Java8/23运行。验收：实际10命令全部exit0，4次134B六case oracle逐字一致，return字段3000→99仍返回3000、清理恰一次、原NPE与cleanup覆盖顺序正确；无原class/JAR混入，无生成文本修补/删方法。另保留原完整FY/FX实际重编结果，loopless恢复不伪称其它锚完成。
- [x] 3.2 完成fmt/CI原参数Clippy/指纹/两版OpenSpec strict、MSRV及最终产品自身CI全门禁独审。验收：前片交付后主线重绑定，真实非空命令与日志、full source/CLI/CI exact head一致；默认stack、5GiB free/1GiB target磁盘门及exact clean；双seed完整workspace、workflow全部实际通过，保留失败/skipped，不借a89或前片CI。

## Workflow follow-up

- 当前6/6；六项均已实际验收。产品 `51de58710291cd3a3587f7040eec6f972cb26229` 自身CI38863891656终态全成功及独立日志验收通过（root793/0、独审1894/0）；见results/own-ci-51de-v1。
- 前片已正常归档并交付main `7970ca890`；本片三未发布历史提交已重放该主线。结果文档曾提交5b3efc566，产品CI仍绑定51de；后续文档提交不冒称另一次产品CI。
- 最终native15命令全0、74pass/0fail/4ignore：reader人口1177/5263/525/3451/10、FP、未改P5计费、finally/FO兼容、公开API例子、CI原参数Clippy、MSRV及两版strict364/0通过；完整4922 Git blob/mode与冻结a989 CLI绑定，exact clean后target缺失。
- 最终完整独立类对照10命令全0：两份完整1225B生成Java分别经真javac8/javac23原样重编，未改driver仅对rebuilt编译，Java8/23四次134B oracle exact。独审与portable包逐成员/原raw核验通过，见results/fixed-recovery-v1。
- 六项已完成，继续同步delta、正常archive并核main源码/证据绑定。父 `recover-return-through-effectful-finally` 的FY.noCall、FX.finReturn及其它全锚/JADX验收继续，不将本片归档当父目标完成，不增全局覆盖数字。

## Historical evidence boundaries

- 立项基于a89；规划包12命令是原类控制及完整生成源码缺return红，另外8命令为原完整FY/FX筛查。它们不是新能力成功，见results/planning-baseline-v1。
- 旧生产native先实证public2和CLI1行为红；FP仅新增本片五输入2226→2231。真实probe固定旧straight proof首拒row.end16与推导copy起点25不符，canonical26融合48/49尾部；后续旧门未到，不能冒称验证。旧CLI v2 abab与此前6e075字节不同，另跑4命令完整源码红；见instrumentation.md及results/native-baseline-v1。
- 第一实现轮因旧测试残留已迁移的公开enum字段而编译失败，clean成功；修后WIP private3/public2/CLI1及完整源码10命令成功。随后真实负控制与兼容67/0/3成功；历史四个变更源码均按原manifest逐hash保存，不拿当前源替代。见fixed-recovery-v1包中各阶段。
- 固定产品新增ExtraConsumer控制明确在实际SSA样本上调用新proof并拒绝；未消费保存值与额外consumer是两项不同控制。候选/边界拒绝不冒称深入每条SSA检查，末段预算/取消则由实际收费验证。公开FinallyCompletion保持原形状，额外落穿元数据只在私有Plan。
- 最终原完整FY/FX重筛v2实际8命令：FY5/FX6所有方法保留、四CLI0/四完整javac1，四recompile全部JDK23 --release8（两inputlegs说明原class来源）。FY.loopless已恢复；noCall/finReturn等仍缺return，无rebuilt runtime。v1仅首CLI0后因harness错误期待直接return3000而失败，未到javac；原raw保留，未改生成源码。
- CI collector v1第二次API连接失败后退出，保留raw；collector v2有限重试采同run。采集器失败不等于产品CI失败；本产品终态已由成功v2原日志与独审验收。
