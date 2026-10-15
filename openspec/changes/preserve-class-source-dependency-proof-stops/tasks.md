# Tasks

## 1. Baseline and public regressions

- [ ] 1.1 固定实际实施源码/构建/fixture pins，核对 results 副本与原始 manifest；确认狭义接口停止计划尚未实施或其最窄构造已经真实可用，记录本片选用入口且不复制完整报告初始化。验收：基线清单、API 评审与本片范围差异可复核，三处静态邻居债务未混入。
- [ ] 1.2 将固定 H1 JAR 接入永久公共 class-source 测试，保存 wide 与 caps1/2/8/14/15 的本地真实结果，分别绑定前置/后置语义及真实物理 main。验收：测试实际非零执行，宽预算有语句、低cap正常预算停止；修正前 cap8/cap15 对契约的真实失败原始流保留，不调用 GDB、不以 helper-only 代替公共入口。

## 2. Snapshot prebody stop handoff

- [ ] 2.1 在两条限定消费路径可复用的最窄交接中保留原 Error 或 resolver 非 Complete 中承载 request-ending 预算/取消/期限的 execution/diagnostics，普通 None 仍独立。验收：实际依赖读取的 request-ending 非 Complete report 与直接 Err 两渠道均有测试，原诊断/维度未遗失，不解析字符串、重 charge/poll 或编造预算字段；附局部源码评审记录。
- [ ] 2.2 接入 snapshot 证明前置停止及最小 stopped 构造，保留本次 analysis/callee/tuple 所有权与可靠兄弟前缀。验收：公共 H1 cap8 回归得到 Stopped/not_produced、原 ClassHeaders/诊断、类非 Complete；原宽预算正文/来源不变，停止后正文/optional evidence/类投影观察量不再增加，并更新边界说明。
- [ ] 2.3 增加前置证明真实收费内确定性取消、实际期限检查点、非 counted 原维度与第一停止原因控制。验收：初始 token clear、真实 Charge/checkpoint 已到达，Cancelled/ElapsedMillis/其他实际先发生原因逐字段正确，正文未运行；不以预取消、sleep或假 Error 替代，并保存明确未覆盖边界。

## 3. Double-brace postbody stop handoff

- [ ] 3.1 接入 double-brace selected-read 与其实际调用 child 报告中的 request-ending 预算/取消/期限停止，保留已提交正文，只汇总 class execution/diagnostics。验收：公共 H1 cap15 回归 class Partial/原 ClassHeaders，而 main 与兄弟逐方法结果、正文/source maps 保留；不调用 not_produced factory 清空产物，更新阶段说明。
- [ ] 3.2 在 double-brace 第一停止后结束 anonymous-interface 等后继投影，添加真实收费内取消/实际期限与先因控制。验收：class Cancelled/原期限语义、已交付前缀仍在，后继阶段/费用无新增，通过无后续工作/重 poll 保留本片原因且全局 strongest-priority 不变；报告发布受限时 aggregate/coverage 与缺失 record 分别核验。
- [ ] 3.3 运行原 snapshot widening 与 double-brace 合法、缺失/歧义/不适用/固定算法界拒绝来源控制。验收：普通无证明不变 Stop，value-level Unsupported/decode 不完整仍按原隔离/汇总，宽预算保持已有正文/拒绝与 origin/effect，必要完整生成源使用正确 Runner 编译执行；单方法片段不冒整类，控制清单与结果写入本片。

## 4. Integration and final product evidence

- [ ] 4.1 用最终候选实际构建运行库与 JSON CLI 的 six-cap 公共矩阵及两渠道组合回归。验收：库/CLI同源，前后体状态和 content 分离，所有未完整命令退出4、普通完整拒绝不误标停，limits/usage/原诊断可复核；精确 argv、退出、raw/hash 与输入/完整编译源码 pins 留存，不以历史六次观察计作修复通过。
- [ ] 4.2 对最终源码执行当前项目适用的格式、Clippy、workspace/MSRV、OpenSpec strict 与必要完整生成源等既有门禁，覆盖本片新增永久回归。验收：以真实实施基线 workflow 为准、各执行非空且 source pins 一致，保留失败/修复历史与适用范围，不增无关门槛、不以旧局部/矩阵或其他提交 CI 替代。
- [ ] 4.3 将本片修复提交为可定位产品 commit，取得该最终源码自身项目 CI 成功并核验绑定；整理最终源码/测试/原始证据 manifest 与完成说明。验收：CI 对应实际 fix commit 和适用 workflow，pending/未跑/失败明确记录，尚未完成不得勾选；相邻债务及未覆盖范围保持，实施证据不覆盖历史 probe。

## Workflow follow-up

- 实施与适用评审完成后，另行按 archive 工作流同步规格、归档并复核；规划完成不计实现完成。
