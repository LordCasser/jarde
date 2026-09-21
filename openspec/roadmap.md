# JVM Rust Engine 阶段路线

P0–P5 与分层已按各阶段范围归档：P2 29/29、分层 7/7、P3 12/12、P4 10/10、P5 10/10；主规格现为 **18 份**。2026-09-20 对 `cd6f2f0` 的独立复核确认的两条 P1——嵌套表达式旧值重算（P3-R8）与字段生产者在 fallback 中丢失（P3-R9）——已由 `close-recovery-correctness-gaps`（4/4，`fd0aae8`）关闭，两条反例与正向对照进入永久语料；历史归档记录保留。见 [收尾验证](changes/archive/2026-09-20-close-recovery-correctness-gaps/verification.md)。

相关入口：[OpenSpec 规划入口](README.md)、[技术栈与依赖选型](dependencies.md)、[架构验收与阶段映射](acceptance.md)。

**最新完成判定（复核 `ad0ffa6`，最后生产实现 `5c35cbf`）：T1–T4 约定判据通过，本轮确认关闭。** [第四轮复核](completion-review.md) 重跑针对性与编译执行、debug/release 深链门禁，核对三项归档及 CI；新发现 P1/T5：`append(int)` 消费 char 表达式时丢数值转换，另行处理。CLI 文档预算、类型忠实度等已登记边界保留，不能宣称全部恢复正确。性能专项及并行全量导出仍各 0/22。

历史 R1/P1（名称选择丢失搜索停止）与 R2/P2（类视图顶层漏汇总 body 停止）已由 [preserve-task-operation-stops](changes/archive/2026-09-20-preserve-task-operation-stops/tasks.md)（5/5，`8586356`）关闭。`8586356` 保留为当时冻结的性能比较臂，不代表后续恢复修正已被包含；新的正式候选须在声明缺口和范围后重新冻结。

## 阶段依赖

```text
P0 establish-p0-foundation → P1 p1-query-xref → P2 p2-jvm-ir → P3 p3-java8-recovery → P4 p4-modern-semantics
                                  │                │                 │                    │
                                  └────────────────┴─────────────────┴────────────────────┴──→ P5 p5-measured-optimization
```

`layer-jarde-crates` 与 P2 已全部归档，原 4.2b/4.3b 及 P2 出口不再是待办。六包 workspace 与 P3 只读 IR 交接、恢复门面/CLI 均已存在，声明、按需成员与物理方法映射也已交付；旧恢复缺口 R8/R9 和任务选择/停止传播均已关闭，当前缺口见下表。

`p1-query-xref` 还直接消费 P0 的 snapshot/classfile/contract；`p4-modern-semantics` 需要 P1 的 views/query、P2 的 resolver/IR 和 P3 的 recovery。P5 选择一个或多个已有稳定结果契约作为优化目标，在被选阶段的真实基线稳定后即可进入，不要求先完成 P4；若优化跨阶段，再纳入所有受影响阶段的回归。

## 阶段边界和门槛

| 阶段 | OpenSpec change | 本阶段新增边界 | 主要验收 IDs | 进入条件 | 出口门槛 |
| --- | --- | --- | --- | --- | --- |
| P0 | `establish-p0-foundation` | bounded in-memory snapshot、CLASS/JAR/WAR 顶层 locator、Header/bytecode inspection、公共 coverage/diagnostics | P0 specs 与基础 reader/预算测试 | 架构基线和依赖评估完成后 | 快照不受源文件变化影响；顶层物理 entry、Header、BCI/attribute span 可复核；不宣称 X1/resolver/decompile |
| P1（已归档） | `p1-query-xref` | X0/X1 structural XRef、code/metadata/resource/bootstrap consumers；nested/MR/Boot 的物理/运行视图和分页 | A01–A08、A14、A17、A18 | P0 事实、identity、budget、JSON contract 稳定 | 不构建 CFG/SSA/Java AST；consumer 不漏报；PhysicalView 保留全部 entry；RuntimeView 显式选择；预算/取消永不伪造 Complete |
| P2（已归档） | `p2-jvm-ir` | Demand Resolver、Platform/LoadDomain providers、raw CFG、jsr/ret normalization、CanonicalCFG、Frame/SSA、effects、Conservative fallback | A09、A10、A11、A13、A14、A16、A17 | P1 query/view 及 P0 bytecode facts 可按需读取 | Resolver 状态可区分；IR pass 依赖/invalidation 可检查；单方法不物化无关 Body；verification 与静态分析分开；失败成员隔离 |
| P3（已归档，12/12） | `p3-java8-recovery` | Java 8/历史恢复模式、确定性命名、source map、representation/quality/compile_status/semantic_validation/verification | A09、A10、A12、A13、A16 + 第 20 节恢复矩阵 | P2 IR/Resolver/origin/effect contract 稳定 | 只有满足前置证据才 Structured；TWR/异常/副作用顺序可复核；X1 原始边保留；语料/重编译/受控行为结果只按 profile 记录 |
| P4（已归档，10/10） | `p4-modern-semantics` | module、nestmate、condy、modern concat、record/sealed、RuntimeMatrix、X2/X3、versioned query plugins | A04–A07、A12 + 53–71/preview registry tests | P1 views/query、P2 resolver/IR、P3 recovery status 可独立演进 | release-bound registry 和 preview 诊断；Unknown/OpenWorld 不被猜成唯一目标；Java 8 output conflict 明确；不把结构支持 Java 27 等同完整源码恢复 |
| P5（已归档，10/10） | `p5-measured-optimization` | 仅以实测决定的 query merge、并行、cache/index、退化优化和发布 gate | A14–A18 + 被选目标阶段适用回归 | 被选目标阶段的结果契约和真实基线稳定，不要求 P4 完成 | 每项优化有 direct baseline、完整 cache key、失效/回退、资源和正确性对照；无证据收益保持 disabled；不预设性能数字/新依赖 |

## 变更使用约束

- 每个阶段保持独立 OpenSpec change；`proposal.md` 的 capabilities 必须与 `specs/<capability>/spec.md` 一一对应。
- P0 的顶层 locator 与有界内存快照不在 P1 重做；P1 才增加 nested/MR/Boot 的递归候选、布局和 RuntimeView。
- 各阶段只有实际完成并验证的任务才能勾选；P0 状态以归档 tasks/verification 与主规格为准，P1 以归档记录为准，P2 以归档记录为准，P3–P5 以各自归档记录为准；后续修正以当前 active change 的 tasks 与最新复核为准。本文描述依赖和门槛，不把规划完成解释为实现完成。
- 每个 change 的实施阶段归档前先执行对应 strict validation，再按架构验收 IDs 和真实语料验证；有行为 delta 时 archive 默认同步主 specs，不使用 `--skip-specs`。
- 优先复用 P0 已评估的 noak、rawzip、flate2 rust_backend、blake3、serde、thiserror、clap；后续库、持久 index 或并发方案先依 [选型准入](dependencies.md) 评估。JVM/JADX 不进入生产核心运行依赖，只可作为受控测试 oracle。

## 当前执行顺序（2026-09-21 完成裁决后）

| 顺序 | 范围 | 完成条件 |
| --- | --- | --- |
| 已关闭 | 停止传播、指定递归 abort、`inverse32` 二元分组、Produced/content 读法 | 分别由 `8586356`、`4f62e26`、`445a277` 及契约归档交付；不推广为任意输入安全或完整源码恢复 |
| 原反例已关闭 | 调用 receiver 分组（`fa6dc6e`） | `(a + b).substring(1)` 保持分组，输入 `("a","bc")` 得 `bc`；不重开该反例 |
| 原反例已关闭 | boolean 返回/调用条件、就地局部声明（`5a8c36a`） | 原样例通过；后续 T2/T3 由以下独立归档关闭，字面量臂等类型忠实度仍有限 |
| 原反例已关闭 | 数组类型拼写（`66bd2d0`） | `byte[]`、引用数组及多维数组原样例通过；既有记录边界保留，本轮未确认新的正常数组声明回归 |
| 已关闭 | 深拼接（`5c35cbf` 实现，见其归档验证） | 当前 debug 的 2048 次 append 必须返回产物或明确停止报告，不得 SIGABRT；同时验证构建、发射和释放，不用增大线程栈替代界限。debug/release 分别记录 |
| 已关闭 | 整数二元比较回归（`f4d1044` 实现，见其归档验证） | `1 == n`、`0 < n`、`1 < n` 保持整数文本，覆盖常量左右位置与比较方向；boolean 原反例仍通过 |
| 已关闭 | 局部类型一次性决定（`2895bc4` 实现，见其归档验证） | 已声明 boolean 局部跨分支复制后仍按一致证据定型，或可靠拒绝；不得生成 `int local3` 再赋入 boolean 的矛盾文本 |
| 已关闭 / T4 | concat 数值前缀与列明转换形状（`5c35cbf`，见归档验证） | `(1,2)` 得 `"12!"`，单段加法仍得 `"3!"`；boolean/null/Object、求值与异常顺序对照通过，不代表所有参数转换已完整 |
| 1 / T5 | `append(int)` 消费 char 的转换，独立后续 | `value(char c)` 的 `append((int)c)` 输入 `'A'` 必须得 `"65!"`，首段/后段都覆盖；当前是 `"A!"` / `"xA"`。保留 Concat 片段表示，在片段内落实目标转换，证据不足拒绝，不混入 bulk |
| 独立债务 | CLI 停止报告的文档预算 | 请求停止与最终报告共用 output_bytes，部分发射中止只能 exit 2/stderr；库清理和可交付 CLI 停止已通过，文档交付能力未闭环 |
| 核实 | 历史 clone/toString 报错 | 逐条保存 opcode、owner、descriptor 和包装器上下文；`ArrayUtils.removeElement` 是静态 helper 正向对照，不作为丢 receiver 反例 |
| 5 | 重新冻结 benchmark 候选 | 保留历史版本身份；新候选登记 T1–T4 关闭、T5 与其它已知边界、构建与输入摘要，不把不同版本/构建配置结果相互替代 |
| 6 | `optimize-demand-workloads`，0/22 | 调查继续；正式对照先 G0 与 O1 证据复核；已确认的 W6a 全量并行交给下述子 change，其余候选独立处置 |
| 6a | [add-parallel-bulk-recovery](changes/add-parallel-bulk-recovery/tasks.md)，0/22 | 类准备复用 → 串行流式 bulk → 共享总账和类间并行 → CLI → 真实 1/2/4/6 worker 全量门禁；发布前验证普通 worker 栈和同一正确性基线 |

## 全量导出架构（2026-09-21，规划完成、实现未开始）

多线程已是明确需求，不再以“尚无并发宿主”为理由暂缓本场景。[设计](changes/add-parallel-bulk-recovery/design.md) 与 [bulk 契约](changes/add-parallel-bulk-recovery/specs/bulk-recovery/spec.md) 定义：一次打开输入、按类共享准备、类间有界 worker、按方法有序交付，共用总预算与有界保留。CLI 目标默认 `--jobs auto`，单线程是同一操作的 `--jobs 1` 配置；数值容量和启用依据仍需实施门禁确认。

子 change 唯一拥有 O2 恢复路径、O4 bulk、O7 类间并行和必需的 O5/O8，父专项继续测量归因；查询分页、预热、跨请求 single-flight 和完整类源码各自独立。首版导出方法 JSONL，不能宣称等于 jadx 的完整 Java 工程。性能判据是包含准备、编码及输出关闭的实测总时间，不能用 warm p50 推导整包追平。主 specs 在真实实现验收归档前不提升为已交付能力。

## 已关闭的恢复正确性收尾

| 顺序 | 范围 | 结果 |
| --- | --- | --- |
| 已完成基线 | P0–P5 与分层归档；18 份主规格 | 已有交付和门禁保留，不重做建包、SSA、声明、accessor 接线或优化选型 |
| 1.1 / P3-R8 | 嵌套表达式的实际求值上下文 | 已关闭：`nestedLocal` 可靠降级并指名被拒的 load 与消费点，`nestedPlain` 仍写出整段 Java；把递归改回 producer BCI 的变异使回归变红 |
| 1.2 / P3-R9 | fallback 中可观察生产者与 origin | 已关闭：被拒 cast 前的 `getstatic`/`getfield` 与字段链进入 quote，并有 BCI 与物理方法映射；字段记录不再虚报 presented |
| 2.1 | 永久受控语料、独立行为与库/CLI | 已关闭：两个 fixture 及提交的 baseline driver、`tests/p3_eval_context.rs`、comparison 的精确 quote 集合与库/CLI 逐字段比较都在门禁内 |
| 2.2 | 修正后固定提交门禁与完成确认 | 已关闭：`fd0aae8` 上全量 1122/0/5、clippy `-D warnings`、两条显式编译执行对照、benchmark smoke、strict 与 fixed-commit CI 通过；delta 已并入主规格并归档 |

[close-recovery-correctness-gaps](changes/archive/2026-09-20-close-recovery-correctness-gaps/tasks.md) 已归档（4/4，`fd0aae8`）。门禁数字与 CI 结果见 [收尾验证](changes/archive/2026-09-20-close-recovery-correctness-gaps/verification.md)。新反例此前在 `cd6f2f0` 基线上成立，因此历史 CI 不构成本次验收：验收以固定提交自身的门禁为准。

## Benchmark 复核后的四个 change（2026-09-20）

[benchmark 复核](benchmark-review.md) 把实测观察收敛成四个各自闭环的 change；建议顺序与当前状态如下，逐项验证后归档。

| 顺序 | Change | 状态 |
| --- | --- | --- |
| 1 | [expose-recovery-content](changes/archive/2026-09-20-expose-recovery-content/proposal.md) | 已归档（4/4，`7d095ce`）：产物内容三值分类，评测分母可对账 |
| 2 | [carry-declaring-class-evidence](changes/archive/2026-09-20-carry-declaring-class-evidence/proposal.md) | 已归档（4/4，`618de49`）：同源 driver class name/flags 交接，声明 form 由本次读取决定 |
| 3 | [bound-container-lookup](changes/archive/2026-09-20-bound-container-lookup/proposal.md) | 已归档（8/8，`b22ea04`）：定向 container 访问、raw-name 定位表、有界 backing 复用 |
| 4 | [bind-prefixed-load-roots](changes/archive/2026-09-20-bind-prefixed-load-roots/proposal.md) | 已归档（6/6，`fbcf06b`）：container+raw prefix 的加载位置模型、候选 `this_class` 绑定、CLI 树枚举闭环 |

它们与已归档的正确性收尾相互独立：R8/R9 不因这些 change 重开，这些 change 也不把基准分布当作修复后的实测。

## 易用性三阶段（ASC 任务链）

| 顺序 | Change | 状态 |
| --- | --- | --- |
| 1 | [add-artifact-navigation](changes/archive/2026-09-20-add-artifact-navigation/proposal.md) | 已归档（8/8，`e0c83b6`）：候选/确认两级类列举、成员列举与 `PhysicalMethodId` 交接、歧义候选 |
| 2 | [add-task-oriented-operations](changes/archive/2026-09-20-add-task-oriented-operations/proposal.md) | 已归档（11/11，`2428752`）：库内目标选择/阶段调度/有界预算/三种环境策略、类视图与引用组织、恢复呈现顺序 |
| 3 | [add-task-oriented-cli](changes/archive/2026-09-20-add-task-oriented-cli/proposal.md) | 已归档（9/9，`85828c4`）：五个薄子命令、text/JSON 同源、诊断分离、四态退出状态 |

三者依赖顺序固定，现已按当时验证范围归档；R1/R2 的独立修正也已关闭，不回写历史归档为失败。显式 artifact-tree 发现和 prefix root 已交付，仍未实施的是自动 WAR/Boot layout policy，不应将两者合并成“树发现未实现”。

## 分开处理的后续范围

| 范围 | 当前边界 | 何时继续 |
| --- | --- | --- |
| 声明与源码覆盖 | driver 类名/access flags（含 ACC_INTERFACE）已同源交接；MethodParameters、InnerClasses 的完整恢复消费及完整类级源码仍未交付 | 宣称对应命名/类级源码覆盖前单独验收；不重复实施已归档的声明交接 |
| 异常图策略 | handler 入口作为根的语义取舍仍待裁决，当前可靠 fallback 保留 | 改 canonical 策略前单独验收 Frame/SSA/异常次序，不能靠恢复层隐藏 |
| 现代输出与适配 | P4 是结构/推断能力，恢复 OutputLevel 仍 Java8；新增 P4 入口没有 CLI/JSON 面 | 需要相应产品能力时单独设计与验收，不把结构支持当源码恢复 |
| 优化与资源 | CP/Header + container facts cache 默认 off；entries/retained_bytes 双限、满则拒绝；更高层 key、index/parallel/merged 未实现，规模语料/阈值未定 | 由性能专项复核实际工作负载、归因和资源代价；未实现可选路径不构成 A15/A18 当前阻塞 |
| 局部解析与隔离 | class_view 已共享字节/成员列举，body 解码仍重建类 reader；成员表损坏可能连带拒绝前面的方法体 | 单独评估结构/locator 复用与损坏隔离，不混入当前停止传播修正 |

这些边界限制可声明的产品覆盖；它们不能被总括成“完整 JVM/Java 恢复已实现”，也不能无差别塞进一个正确性修复。A15/A18 按已实现路径通过，usage 节省允许但资源/取消仍逐项对照；不存在的路径为不适用。归档阶段、当前正确性与后续覆盖分别记录。
