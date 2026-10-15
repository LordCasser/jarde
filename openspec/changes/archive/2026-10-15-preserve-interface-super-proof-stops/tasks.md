# Tasks

## 1. Baseline and public regression

- [x] 1.1 在选定实现基线上复核 `results/repro-v1/manifest.json` 的最小测试、三个 fixture 和私有组合身份；保存新基线 source pins，明确旧宽预算 1 pass/深度零 1 fail 不等于 primary 644 或新 fix 实跑（验收：manifest 逐项 hash/bytes 一致，新基线与历史来源差异有记录）。
- [x] 1.2 将最小反例接入适当公共 integration 测试，保留真实 `Engine::recover_target`、`defaultCall()I`、宽预算成功限定调用和 depth0 的原 DependencyDepth 断言；在修复前保留真实行为失败及编译/raw，不以 prove-only helper 或普遍拒绝替代（验收：每条 exact 命令实际 1 test，wide 真 Complete/接口 super，negative 因行为断言变红）。

## 2. Minimal stop handoff

- [x] 2.1 在既有 report 停止构造周围增加最小跨 crate 入口，按原 Error 或 resolver 原报告初始化 execution、诊断、Stopped/not_produced 和空 text/source map；保持 evidence pending 与 artifact 未产出语义（验收：对 counted、DependencyDepth、ElapsedMillis、Cancelled 的映射测试逐字段核对，非 counted 不改报 OutputBytes/IrItems，无新预算维度/全局状态/依赖）。
- [x] 2.2 将接口证明预算/取消停止接到该停止路径，保留同次 analysis，普通空证明仍走原保守恢复；同步修正包装注释，说明真实控制流（验收：公共 depth0 变为 Partial/DependencyDepth 且有停止，wide 保持 Complete 和原合法限定调用；合法请求得到报告而非裸 Error，文档与分支一致）。
- [x] 2.3 固定停止点 usage 并验证之后不运行正文、后续证明或证据物化，不重跑/重新收费分析；让 class-source 继续合并该成员停止并保留允许的 raw/兄弟前缀（验收：公共方法和 class-source 控制证明无停止后的 recovery 工作/收费，报告与 presentation 原因一致，不以事后改标签通过）。

## 3. In-flight stopping controls

- [x] 3.1 复用 caller-owned observer 或最小测试边界，在真实接口 proof Charge 中置取消 token；记录初始 clear、Charge 实际发生与后续 checkpoint（验收：公共请求 Cancelled、analysis 保留、无恢复产物或停止后工作；预取消/sleep/prove-only 不计该场景，raw 和观察量保存）。
- [x] 3.2 增加抵达 proof 后实际期限停止控制及计数型元数据/分析预算控制，保留各自原原因与可用位置（验收：公共报告非 Complete、期限不变为正文输出预算/Cancelled，counted 维度准确；证据证明首次停止在接口证明而非入口/IR0）。
- [x] 3.3 组合早证明停止与本会在后续触发的其它限制，检查原最早停止不被后续恢复重写；同步记录限额、usage 与原因映射边界（验收：公共报告保留原 DependencyDepth/实际预算原因，停止点后无新增 recovery 费用，已获准收费不要求退款）。

## 4. Existing source and refusal contracts

- [x] 4.1 重跑接口合法限定调用、class-super/private/构造器普通调用，以及 missing/ambiguous/redundant/abstract 和 receiver/argument effect 来源控制（验收：合法结果保持既有字段/文本及来源；完整读取的普通拒绝不被标为预算停止，方法与 BCI 可定位；仅 elapsed 可不同，异常新增差异说明归因）。
- [x] 4.2 复用 special-dispatch/qualified-super 的完整生成源、正确 helper/runner 和受控 JDK 做既有编译执行对照；核对必要 public/CLI stop 消费与 class-source 汇总（验收：全类源编译/运行真实成功，拒绝源依既有拒绝契约核验；实际输入/编译源/raw/pins 齐，单段正文或 verifier 成功不代替全源门）。

## 5. Final integration evidence

- [x] 5.1 在最终产品源码上执行本项目当时适用的格式、Clippy、workspace 测试、OpenSpec strict 及 CI 配置规定的其他既有门禁，绑定确切源/argv/raw 并保留所有失败历史（验收：适用门禁实际成功、测试数非零且范围准确，不派生额外无关门槛，不借用旧 644 或私有反例的结果）。
- [x] 5.2 完成 fix 产品提交并取得该确切 commit 自身 CI 成功，整理本片最终结果、未覆盖范围、迁移说明及清理收据后再核对 tasks（验收：产品源与 CI head/原始结果一致，工作区范围清晰；需要清理的私有 Cargo 最终严格 clean 成功且 target absent，完成结论不提前）。

## Workflow follow-up

- 原规划阶段为0/12且不实施；2026-10-15根已采用规划并进入用户持续goal的实现阶段，1.1/1.2实际完成。OpenSpec校验与历史私有复现不代替后续产品验收。
- 实现及产品自身 CI 验收后再进入归档工作流；相邻 snapshot/direct-parent/reference 包装债务继续独立跟踪。

## 本地实施进展（2026-10-15）

- 10/12已完成；5.1/5.2仍开放，等待最终组合产品源码的完整workspace与自身CI，未把class-view e8c CI计为接口修复验收。
- 公共6项与真实proof observer5项均通过；随后boxed同源受影响门813通过/0失败/1默认忽略，含受控JDK23显式执行的完整special-dispatch一项。双strict366/0为当时规格状态。
- 原验收脚本aa47…未改，8组完整类/接口源（4种合法情形×两debug模式）编译运行，与原Runner共12行输出一致；16组拒绝控制保留原断言。178命令/356raw、40输入和144保留产物均有pins。
- CLI采用实际可覆盖的ClassHeaders=2：方法和class-source均exit4、原ClassHeaders诊断保留；depth为库高水维度，不是CLI预算覆盖项，先前无效CLI请求exit2排除产品反例。
- 原始depth0产品失败、初版测试错误/编译错误、report-only原因断言失败、Clippy大Err、缺JDK PATH和录制器失败均保留；范围与逐次修正见results/root-local-v1。

## 最终产品验收（2026-10-15）

- c8718ae自身CI38692921405已成功：4jobs/52steps；两seed各367suites、3615/0/98，总7230/0/196。105具名回归逐seed通过，MSRV1.88、supply-chain、fuzz、显式JDK/full-class及该head的strict365/0均真实完成；root848项与独立1644项审计无错误。
- 最终产品源4902项与冻结CLI2669绑定以原receipt为准：准确CLI字节59674128、SHA256 `2669b2064747f16ff883788bdf31811b67c5ed3da2da3c1676eed6e99eec04ac`，所有源仍匹配focused-v5，exact clean后target缺失。后续只改规划、证据和规格，不冒作再次产品运行。
- 最终补包已独立逐字节审核；本片实现任务全部完成。此前本地段落的pending/开放数是历史时点，未覆盖失败记录；本片完成不等于全项目或下一direct-parent片完成。
