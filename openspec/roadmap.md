# JVM Rust Engine 阶段路线

P0–P5 与分层已按各阶段范围归档：P2 29/29、分层 7/7、P3 12/12、P4 10/10、P5 10/10；主规格现为 **18 份**。2026-09-20 对 `cd6f2f0` 的独立复核确认的两条 P1——嵌套表达式旧值重算（P3-R8）与字段生产者在 fallback 中丢失（P3-R9）——已由 `close-recovery-correctness-gaps`（4/4，`fd0aae8`）关闭，两条反例与正向对照进入永久语料；历史归档记录保留。见 [收尾验证](changes/archive/2026-09-20-close-recovery-correctness-gaps/verification.md)。

相关入口：[OpenSpec 规划入口](README.md)、[技术栈与依赖选型](dependencies.md)、[架构验收与阶段映射](acceptance.md)。

## 阶段依赖

```text
P0 establish-p0-foundation → P1 p1-query-xref → P2 p2-jvm-ir → P3 p3-java8-recovery → P4 p4-modern-semantics
                                  │                │                 │                    │
                                  └────────────────┴─────────────────┴────────────────────┴──→ P5 p5-measured-optimization
```

`layer-jarde-crates` 与 P2 已全部归档，原 4.2b/4.3b 及 P2 出口不再是待办。六包 workspace 与 P3 只读 IR 交接、恢复门面/CLI 均已存在，声明、按需成员与物理方法映射也已交付；当前仅收尾新发现的恢复正确性缺口。

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

## 当前执行顺序（2026-09-20 收尾完成）

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

## 分开处理的后续范围

| 范围 | 当前边界 | 何时继续 |
| --- | --- | --- |
| 声明与源码覆盖 | MethodParameters、类级 InnerClasses/ACC_INTERFACE 尚未进入恢复载荷；仍是方法体产物 | 宣称对应命名/类级源码覆盖前，单独交付事实与语料；不混入 R8/R9 |
| 异常图策略 | handler 入口作为根的语义取舍仍待裁决，当前可靠 fallback 保留 | 改 canonical 策略前单独验收 Frame/SSA/异常次序，不能靠恢复层隐藏 |
| 现代输出与适配 | P4 是结构/推断能力，恢复 OutputLevel 仍 Java8；新增 P4 入口没有 CLI/JSON 面 | 需要相应产品能力时单独设计与验收，不把结构支持当源码恢复 |
| 优化与资源 | CP/Header cache 默认 off；容量按 entry 数；更高层 key、index/parallel/merged 未实现，规模语料/阈值未定 | 有代表性成本证据和明确资源权重/收益时再评估；未实现路径不构成 A15/A18 当前阻塞 |

这些边界限制可声明的产品覆盖；它们不能被总括成“完整 JVM/Java 恢复已实现”，也不能无差别塞进一个正确性修复。A15/A18 按已实现路径通过，usage 节省允许但资源/取消仍逐项对照；不存在的路径为不适用。归档阶段、当前正确性与后续覆盖分别记录。
