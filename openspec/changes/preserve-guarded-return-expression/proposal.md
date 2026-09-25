## Why

`p3_sync_return` 当前把已证明的 `synchronized (this) { return this.n; }` 写成块内 `int saved0 = this.n; return saved0;`。虽然这个样本行为等价，合成局部违反现有同步返回的源码形态要求；原因是值放置规划把该同步语句自身拥有的 `monitorexit` 当作需跨越的独立效果。

## What Changes

- 在同步 Guard 已证明返回值于退出监视器前计算、且 AST 把 `return` 放在同一 Guard 内时，使该 Guard 自己的退出指令不迫使值绑定为合成局部。
- 真实独立语句、未证明的退出路径或额外异常效果仍保留当前保存或拒绝；用可执行正反例区分。
- 维持原始 BCI 来源、预算与停止语义，不改一般值放置和循环恢复。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：已证明的同步块返回表达式保持原求值位置与直接源码拼写；不能因 Guard 自有清理指令生成无必要的局部。

## Impact

影响 `jarde-java` 的 Guard 所有权和值放置判定、现有同步返回回归及 Java 8 执行对照；不增加依赖、公开 IR 或新恢复阶段。`present-proved-java-structure` 的同步返回要求继续作为既有边界，本变更只处理规划器与该要求的交互。
