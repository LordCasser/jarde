# CF12 conditional replay 计数拒绝审计

## 结论

首个拒绝位于 `TestSwitchWithFallThroughCase.test` 中 `check()V` 的嵌套 method-report usage `ir_edges`。这是 class-source 共用 `Budget` 的累计快照：`check()` 本身仍是同一个直线方法；前一物理方法 `test(IZZ)Ljava/lang/String;` 从 ExplanationOnly/fallback 改为结构化 switch 后，Builder 多做了一次完整 canonical-edge 表检查，新增的 17 个 `ir_edges` 累计到 `check()`。现有 candidate 与 typed baseline 报告显示该偏移前后均稳定为 +17，且 `check()` 自己没有再增加偏移。最窄解释是新语法恢复引发的、可解释且应被精确计费的额外 Builder 工作，不是同一 `check()` 的 CFG/IR 事实改变。

不应移除 verifier 对资源计数的约束，也不应允许任意 `ir_edges` 差值。应保留只针对已锚定目标方法、并向后续累计快照传播的精确 offset；复跑前确认 verifier 版本确实含该 offset 规则，并把增量与真实 CFG edge scan 的调用逐项核对。若这 17 不能由具体 Builder 扫描解释，就应继续拒绝，不能只凭输出恢复成功来放宽。

## 输入身份与比较面

- 被拒绝的 invocation stderr：`/private/tmp/jarde-conditional-replay-verifier-invocation-root-v1/stderr.raw`。首行指出字段路径为 `...TestSwitchWithFallThroughCase$TestCls...check()V...$.methods[0].outcome.report.execution.usage/ir_edges`，拒绝信息是 `nonpermitted resource counter changed`。
- 当前 conditional replay：[`execution.json`](/Users/lordcasser/workspace/projects/jarde/openspec/changes/recover-proved-conditional-switch-fallthrough/results/complete-source-root-v1/execution.json)，报告在 `results/complete-source-root-v1/reports/TestSwitchWithFallThroughCase.test/{jdk8,jdk23}/{default,all}/TestSwitchWithFallThroughCase$TestCls.json`。
- 已接受的 typed comparator：[`execution.json`](/Users/lordcasser/workspace/projects/jarde/openspec/changes/recover-proved-local-source-types/results/complete-source-root-v4/execution.json)，报告在 `results/complete-source-root-v4/reports/TestSwitchWithFallThroughCase.test/jdk23/{default,all}/TestSwitchWithFallThroughCase$TestCls.json`。conditional execution 固定绑定此 v4 execution SHA `27afad91dc4e5c426d90f6ff4253b4244541490161752b759d1f5ada19847ed5` 及 acceptance SHA `d9e336b8b6eb33b76c0d866eb83040540586eba9df5ba629311f2371f577812f`。
- 三边 method owner 的 class digest/snapshot均为 `92a103230cd6cbb8ccc535ca33a838ad3c91265b1b784d5ad5a69db824b17edd`。对应 frozen input SHA-256 是 `5347758d128465aa1889769ac2a18448f060682a7f6567b27f8d98e3499fc034`。比较对象是同一物理 class；typed replay只记录 JDK23，conditional replay记录 JDK8、JDK23。

## 精确观测

执行报告的两个 profile 均显示相同变化。`test(IZZ)Ljava/lang/String;` 的 presentation 从完整 fallback 改为结构化 switch；`check()V` 的 source text 与 source map 保持相同。计数如下：

| 快照 | typed v4 | conditional v1 | 增量 |
|---|---:|---:|---:|
| `test` report `ir_edges` | 194 | 211 | +17 |
| 后续 `check` report `ir_edges` | 230 | 247 | +17 |
| `check` 的 analysis `ir_edges` | 230 | 247 | +17 |
| class total `ir_edges` | 230 | 247 | +17 |
| `test` analysis `ir_edges` | 177 | 177 | 0 |

`check()` 的自身报告 usage 和 analysis usage都为 230→247；因此这些值是在分析前一方法后携带进来的 class-scope cumulative counter，而不是 `check()` 新构造了 17 条 CFG edge。`<init>()V` 的 report usage 两边都为 2。

其它锚定增量也符合“前一个方法的 recovery 变了，后续计数继续累计”：`test` 与后续 `check` 的 `analysis_steps` 均增加 311，`ir_items` 在 default profile 增加 212、all profile 增加 211，`output_bytes` 增加 469；`check` 自己文本、source map、region shape没有改变。它们是已恢复语法导致的额外 proof/build/output 计费，不应用来解释独立的 `ir_edges`，但验证器应对已经确定的目标增量精确比较。

## Region / build 路径

typed v4 的 `test` report 是 `ExplanationOnly`，fallbacks 为 `jre_region_loop`、`jre_region_switch_arms_overlap`、`jre_region_uncovered_blocks`，对应记录为：loop block 117；switch overlap region 从 BCI 0 覆盖 `[0, 32, 59, 63, 67, 92, 121]`；以及 uncovered `[146, 149, 171]`。完整 switch 没被结构化，因而源码只保留 fallback 说明。

conditional v1 的同一物理方法报告为 Structured Java，区域记录 switch `0` 覆盖 `[0,32,59,63,67,92,117,121,146,149]`，另有 straight region `171`；无 fallbacks。输出写出 switch labels、case fallthrough 和内部 `if`，包括每支的 `break`，使这一差异与本 change 的目标语法一致。物理 class/method 身份没有变化。

对应 region 路径在 [`region.rs`](/Users/lordcasser/workspace/projects/jarde/crates/jarde-java/src/region.rs:13420) 的 `switch_fallthroughs` / `prove_switch_fallthroughs`：它先用完整 canonical edge rows建立入/出邻接，再检查 dispatch 与 NormalFlowView 的完整 outgoing set、case arm DAG/唯一前进落点、terminal `Return|Throw` 和 case-entry owners。该证明在这段函数中逐边 charge `AnalysisSteps`，没有 `IrEdges` charge。因此不能把 +17 归到新 fallthrough probe直接扫描上。

Java Builder 的 `nonempty_if_join_transfer` 在 [`build.rs`](/Users/lordcasser/workspace/projects/jarde/crates/jarde-java/src/build.rs:9034) 对一个非空 If arm的 terminal `goto` 做来源归属证明。通过检查 terminal 指令确为 `Operation::Transfer` 后，它先以 `canonical.edges().len()` charge `IrEdges`，再要求该 terminal 的完整 outgoing row恰好是唯一 Normal edge并指向 If join。调用点在 If 构建路径 [`build.rs`](/Users/lordcasser/workspace/projects/jarde/crates/jarde-java/src/build.rs:17846)。旧 fallback 不暴露这些 nested If arms给正常 statement build；新 switch tree使更多真实 If arm进入该来源检查。若目标 `test()` 的 canonical edge table 长 17，且新树比旧树多触发一次成功/失败前检查，该 +17 与代码中的单次整表 charge精确吻合。当前 JSON 不序列化 `CanonicalCfg.edges()`，所以“该方法 edge table确为 17、恰多一次此调用”是最窄且强的来源假说，仍需用现有 IR observer/reader 对目标方法确认，不能仅由 report JSON宣称为已证事实。

代码中另一处 `IrEdges` charge是 `consumer_block_precedes`（`build.rs` 约 11792 行），为跨块 postfix snapshot 检查邻接；`test(IZZ)String` 的目标差异是 switch/if结构，不应未经证据把 +17 归给该路径。

## Verifier 线索与最窄修复/验证建议

当前磁盘上的 [`verify-conditional-replay.py`](/Users/lordcasser/workspace/projects/jarde/openspec/changes/recover-proved-conditional-switch-fallthrough/results/verify-conditional-replay.py:397) 已包含窄 offset：目标 `test(IZZ)String` 为 `analysis_steps +311, ir_edges +17, ir_items +(default 212/all 211), output_bytes +469`；循环随后把同一个 offset传给每个较晚 method snapshot和 class total。也就是说当前代码的契约已经表达“target method允许精确变化，check()不得新增任何额外成本”。目标 v1 JSON实际值按该表能通过 counter arithmetic。

但所给 stderr 的 Python traceback行号指向与当前脚本不一致的位置（stderr文件时间为 07:41，当前 verifier脚本时间为 07:42；当前 402 行是注释），因此日志来自脚本后来改写前的一版，或当时运行的 verifier副本不等于当前文件。不能据这份旧错误推断当前脚本仍会以同一原因失败，也不应改动 counter tolerance。最窄的下一步是：root用已经冻结的当前执行 JSON和当前 pinned verifier源码重放“只读验证”调用；如再拒绝，保存新 stderr并以新行号/字段名继续定位。执行前先只读核对 execution SHA、report raw stdout SHA与磁盘 JSON一致，避免把不同 run的报告混配。

若当前 verifier确实仍在 check `ir_edges` 处失败，优先核对失败 invocation实际加载的 verifier副本和它的 `cumulative_offset` 是否在 `test` method key命中后传播到了 `check`；不修改 baseline、不降低报告完整性条件。然后用现有真实 API为目标 `test()` 取 canonical edge count，并确认 Builder 多出的恰为一次 `nonempty_if_join_transfer` 全边扫描。可通过临时 observer读取已有 `CanonicalCfg`、RecoveryRequest与完整真实 region tree做离线计数；不增加产品 hook/豁免机制。若实际 edge count不是 17或扫描次数不是 +1，应查明其它 `IrEdges` charge来源；在归因闭合前保持验证失败。

## 限制

本审计只读 JSON、stderr、当前源码和 verifier。没有运行 Cargo、JDK、CLI、Git或 replay；没有修改工作树。结论解释现有记录并提出下一次验证的最小定位方式，不替代 root 对当前 invocation的复验。
