# recover-proved-switch-continuations

## Why

冻结的 CF12 证据已覆盖全部十个上游 switch 测试样例；已验收的 remaining-five 回放仍记录 `TestSwitchWithFallThroughCase2.test(IZZ)Ljava/lang/String;` 和 `TestSwitch2.test(I)V` 在 Jarde 中整类编译失败。FT2 当前确认在 `ArmsDoNotMeet@0` 拒绝；从内部汇合点 175 延续到外层 if 汇合点 197 仍是待诊断假说。TestSwitch2 的共享续接与有向无环路径解释，也要等观察到实际首个拒绝位置后才能确认。整类编译失败是本变更的输入，不足以证明任一结构可以安全输出。

## What Changes

- 先观察两个方法的实际拒绝路径；只有诊断证实后，才为已证明的形状扩展现有 switch 续接消费路径。FT2 的 175 到 197 仍是假说；TestSwitch2 复用现有 `prove_switch_fallthroughs`，只补当前缺少的候选汇合点与结果见证。
- 仅在唯一、有界的有限无环路径中扩展既有证明：路径到达候选汇合点或由物理指令证明的 return/throw 终端。只有完整控制流边与 block 归属证明直接 case target 可以作为汇合点时才接受它。
- 保持结构化输出、block 归属、canonical 控制流边闭合、源码来源位置和预算/Stop 行为与现有 Java 8 恢复流程一致；无法支持的形状继续保守拒绝。
- 用冻结的整类输入验证两个结构，再对全部十个 CF12 测试样例做一次全新冻结的完整比较。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：只有物理路径和 block 归属得到证明后，才恢复这两种受限 switch 续接结构。

## Impact

预期实现局限于现有 `jarde-java` 的 switch/region 恢复逻辑及私有测试，沿用已有 `Region`、顺序结构、switch arm、终端结构、canonical 控制流图和预算机制。不增加 `Region` 类型、公开 API、通用图框架、JVM IR/SSA 类型或跨类常量名行为。

本工作要等 conditional-switch 变更自身通过 CI 验收并完成干净交付后开始。root 负责产品代码应用、完整冻结 CLI 验证、CI 和干净交付。两个证明形状可由 Luna 分别准备私有候选实现；root 审查后串行合并到同一个 `region.rs`，不建分支或 worktree。

现有冻结 Jarde 失败不是正向样例。JADX 的 TestSwitch4 检查失败（结果为 `2234` 而非 `1234`）以及 FallThroughCase2 的重复代码警告，只作为对照；后者的完整运行结果与原类匹配，前者没有通过运行检查，不能混淆两者。这些记录，不能作为本变更或 71-unit ledger 的完成依据。嵌套常量名 gate 仍独立排队。
