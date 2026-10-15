# Tasks

## 1. 固定基线与公共回归

- [x] 1.1 固定实际实施基线与DT-29物理JAR，整理旧958a与final2669各9次、共18次CLI观察、cap1/cap2真实栈及独审、宽预算完整root/原Runner双JVM对照的可移交清单；验收：全部bytes/hash与原raw一致，正确区分owner动态证据、source静态边界、v2组装失败、宽预算成功，旧CI不算新fix通过。
- [x] 1.2 接入公共class-source库/JSON CLI的wide/cap1/cap2具名永久回归，并加入真实外部source关系读取停止控制；验收：旧生产因行为断言实际失败且非路径/编译错误，wide真有父owner cast，低cap保留原analysis/ClassHeaders/诊断与可靠constructor前缀，source读点不能被IR快捷路径替代，失败raw保留。

## 2. 最小交接与阶段停止

- [x] 2.1 只将两消费者及必要source/owner读取接入既有typed proof交接、恢复前工厂与返回tuple；验收：公共回归实际通过，cap1首次accessor停止不再进入field，当前正文不产出，普通缺失/歧义与字段/helper来源控制通过；评审正常predicate、reference-overload/global merge/publication准入未扩改，并同步局部边界说明。
- [x] 2.2 复用最窄observer覆盖原Error/原report两渠道和真实proof内取消/期限；验收：初始token clear、实际收费/检查点到达、原原因与全部诊断准确、真实handoff账目及tail证明停止后无body/evidence/收费工作，已准许用量不退款，不用预取消/sleep/假Error替代；记录实际覆盖边界。

## 3. 兼容性与产品集成

- [x] 3.1 在最终候选复用适用字段/accessor身份、flags、缺失歧义及来源控制，执行真实DT-29宽预算完整root嵌套类族与未改Runner的重编译/Java8、Java23对照，并运行同CLI预算矩阵；验收：正确source assembly、无原JAR混入rebuilt classpath，原oracle逐字一致，低cap恢复与类/CLI共同如实停止，保存全部raw/pins及失败历史，不改Java或削弱有效断言。
- [ ] 3.2 对最终产品源码运行当前项目适用格式/Clippy/OpenSpec strict、workspace/MSRV及workflow其他既有门，取得该修复commit自身CI并完成独立复核；验收：命令非空、source/CLI/CI head绑定一致，原失败与未覆盖范围明确、exact clean后target缺失，不借旧c871或其他commit CI完成本片。

## Workflow follow-up

- 产品与适用评审完成后，再同步main specs、正常archive并核最终main源码/证据绑定；规划时0/6表示实现未开始；当前实施状态以下列进展和勾选为准。
- 已授权持续goal按实际证据推进；规划、旧反例、宽预算正例或其他提交CI均不能勾选未来修复任务。

## 实施进展（2026-10-15）

- 已完成1.1、1.2、2.1、2.2、3.1，5/6；基线见results/repro-v1，最终局部门禁及完整源码对照均已独立复核。
- 旧生产原生public4为2通过/2失败；CLI3为1通过/2失败；observer12为5个原有H1通过/7个新增失败，普通缺失/歧义两控制通过。旧失败仅证明首个失败断言，后续channel/header/prefix断言不冒称执行。
- 修复初验12 observer +6 public +3 CLI全部通过；最终focused为18个summary、457通过/0失败/1默认忽略，含全部16个新增具名门。格式、当前CI参数Clippy、OpenSpec1.11/1.14各364/0通过。
- 最终CLI c2db4721a879c47493b537e946b0ff827986e1e58e7b021222df1e66a8c9c3cd，59676240B，0555，绑定4905源；exact clean后target缺失。自身commit CI尚待，不以旧c871 CI完成3.2。

- 最终CLI实际12命令/24raw：完整root含嵌套A/B的原样源码与原Runner重编译，rebuilt classpath无原JAR，Java8/23原始及重编四次均逐字为原oracle；宽预算whole-root文本与旧正例逐字相同。B的wide/cap1/cap2报告真实exit0/4/4，低cap不编译。独审核全部4921运行输入pin和4905源，全部输入未变。该固定Runner只覆盖set(true,false)，不声称所有输入等价或增加全局覆盖数。

- 完整源码实验已独审通过；既有field/accessor身份、flags与来源专项四套21/0/0已补跑并独审，3.1已验收；不把未运行的旧测试作已通过声明。
