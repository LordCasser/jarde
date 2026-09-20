# JVM Rust Engine 阶段路线

P0/P1、P2（29/29）和分层（7/7）均已归档，P2/分层归档提交为 `7a5f994`。P3 已交付并归档（**12/12**，归档提交 `250fe1f`）。已有 Java 方法体、if/loop/switch、lambda 及部分拼接/bridge/accessor 呈现，仍受公开入口和语义边界约束；P4/P5 未实施。 见 [当前恢复复核](changes/archive/2026-09-20-p3-java8-recovery/verification.md#review-2026-09-19-recovery)；历史完成项保留，新问题单列修正。

相关入口：[OpenSpec 规划入口](README.md)、[技术栈与依赖选型](dependencies.md)、[架构验收与阶段映射](acceptance.md)。

## 阶段依赖

```text
P0 establish-p0-foundation → P1 p1-query-xref → P2 p2-jvm-ir → P3 p3-java8-recovery → P4 p4-modern-semantics
                                  │                │                 │                    │
                                  └────────────────┴─────────────────┴────────────────────┴──→ P5 p5-measured-optimization
```

`layer-jarde-crates` 与 P2 已全部归档，原 4.2b/4.3b 及 P2 出口不再是待办。六包 workspace 与 P3 只读 IR 交接、恢复门面/CLI 均已存在，继续工作在现有恢复层收敛正确性与公开契约。

`p1-query-xref` 还直接消费 P0 的 snapshot/classfile/contract；`p4-modern-semantics` 需要 P1 的 views/query、P2 的 resolver/IR 和 P3 的 recovery。P5 选择一个或多个已有稳定结果契约作为优化目标，在被选阶段的真实基线稳定后即可进入，不要求先完成 P4；若优化跨阶段，再纳入所有受影响阶段的回归。

## 阶段边界和门槛

| 阶段 | OpenSpec change | 本阶段新增边界 | 主要验收 IDs | 进入条件 | 出口门槛 |
| --- | --- | --- | --- | --- | --- |
| P0 | `establish-p0-foundation` | bounded in-memory snapshot、CLASS/JAR/WAR 顶层 locator、Header/bytecode inspection、公共 coverage/diagnostics | P0 specs 与基础 reader/预算测试 | 架构基线和依赖评估完成后 | 快照不受源文件变化影响；顶层物理 entry、Header、BCI/attribute span 可复核；不宣称 X1/resolver/decompile |
| P1（已归档） | `p1-query-xref` | X0/X1 structural XRef、code/metadata/resource/bootstrap consumers；nested/MR/Boot 的物理/运行视图和分页 | A01–A08、A14、A17、A18 | P0 事实、identity、budget、JSON contract 稳定 | 不构建 CFG/SSA/Java AST；consumer 不漏报；PhysicalView 保留全部 entry；RuntimeView 显式选择；预算/取消永不伪造 Complete |
| P2（已归档） | `p2-jvm-ir` | Demand Resolver、Platform/LoadDomain providers、raw CFG、jsr/ret normalization、CanonicalCFG、Frame/SSA、effects、Conservative fallback | A09、A10、A11、A13、A14、A16、A17 | P1 query/view 及 P0 bytecode facts 可按需读取 | Resolver 状态可区分；IR pass 依赖/invalidation 可检查；单方法不物化无关 Body；verification 与静态分析分开；失败成员隔离 |
| P3（已归档，12/12） | `p3-java8-recovery` | Java 8/历史恢复模式、确定性命名、source map、representation/quality/compile_status/semantic_validation/verification | A09、A10、A12、A13、A16 + 第 20 节恢复矩阵 | P2 IR/Resolver/origin/effect contract 稳定 | 只有满足前置证据才 Structured；TWR/异常/副作用顺序可复核；X1 原始边保留；语料/重编译/受控行为结果只按 profile 记录 |
| P4 | `p4-modern-semantics` | module、nestmate、condy、modern concat、record/sealed、RuntimeMatrix、X2/X3、versioned query plugins | A04–A07、A12 + 53–71/preview registry tests | P1 views/query、P2 resolver/IR、P3 recovery status 可独立演进 | release-bound registry 和 preview 诊断；Unknown/OpenWorld 不被猜成唯一目标；Java 8 output conflict 明确；不把结构支持 Java 27 等同完整源码恢复 |
| P5 | `p5-measured-optimization` | 仅以实测决定的 query merge、并行、cache/index、退化优化和发布 gate | A14–A18 + 被选目标阶段适用回归 | 被选目标阶段的结果契约和真实基线稳定，不要求 P4 完成 | 每项优化有 direct baseline、完整 cache key、失效/回退、资源和正确性对照；无证据收益保持 disabled；不预设性能数字/新依赖 |

## 变更使用约束

- 每个阶段保持独立 OpenSpec change；`proposal.md` 的 capabilities 必须与 `specs/<capability>/spec.md` 一一对应。
- P0 的顶层 locator 与有界内存快照不在 P1 重做；P1 才增加 nested/MR/Boot 的递归候选、布局和 RuntimeView。
- 各阶段只有实际完成并验证的任务才能勾选；P0 状态以归档 tasks/verification 与主规格为准，P1 以归档记录为准，P2 以归档记录为准，P3–P5 以 active change 的 `tasks.md` 与最新复核为准。本文描述依赖和门槛，不把规划完成解释为实现完成。
- 每个 change 的实施阶段归档前先执行对应 strict validation，再按架构验收 IDs 和真实语料验证；有行为 delta 时 archive 默认同步主 specs，不使用 `--skip-specs`。
- 优先复用 P0 已评估的 noak、rawzip、flate2 rust_backend、blake3、serde、thiserror、clap；后续库、持久 index 或并发方案先依 [选型准入](dependencies.md) 评估。JVM/JADX 不进入生产核心运行依赖，只可作为受控测试 oracle。

## 当前执行顺序（2026-09-19 恢复复核）

| 顺序 | 范围 | 交接门槛 |
| --- | --- | --- |
| 已完成 | P2 29/29、分层 7/7、P3 12/12 均已归档（`250fe1f`） | 保留历史记录；P3 出口门禁已全过，其三项能力 delta 已并入 `openspec/specs/`，不重做建包/交接/CLI/选型 |
| 1.3d | 基础值物化、调用求值与 fallback 依赖 | `return x++` 返回旧值；被拒 cast 不丢生产者调用/BCI；简单调用仍只执行一次 |
| 3.1 | 同次方法声明、参数/receiver、合法作用域 | then/else/合流后的变量可见性、slot 复用、category-2 与无 debug 实测 |
| 3.2 | 物理方法身份的 source map；按需 accessor 证据与门面接线 | 真正呈现字段表达式后 X1 两边仍在；相同 BCI 的 caller/callee 可区分；必要 callee 计费、无关 Body 为零 |
| 2.3 → 2.4 | inner/enum/构造器等模式 → TWR/monitor/finally | 依赖修正后的值、作用域、origin 与异常/effect；不在每条语法糖里补一套值流规则 |
| 3.3 → 3.4 | 独立重编译/行为语料与发布 | 先纳入本轮固定反例；消除 elapsed 假红；库/CLI/主规格/支持矩阵、两个 workspace 门禁和精确 CI 同步 |

当前公开 `recover_method` 返回方法体文本和段表，编译/语义验证尚未执行；不能把 Java/Structured 当成完整源码或已证明等价。2.2 的 accessor 成功主要在低层 API，公开交接仍属 3.1/3.2。P4 已按该边界交付并归档（10/10，`88416ab`）；P5 已按该边界交付并归档（10/10，`2b491ce`），只针对已有测量基线，未吞入正确性修复。
