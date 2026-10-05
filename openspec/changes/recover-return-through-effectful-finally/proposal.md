## Why

判别矩阵证明：continue/break 穿副作用 finally、return 穿**空** finally、finally **返回值**形全部恢复（等价分布/归一化机制已存在）；唯独 **try 返回 + finally 落穿带副作用**在所有位置整方法拒绝（循环内 `noCall`/`finReturn` 与无循环 `loopless` 皆拒，"local N crosses a quoted fallback region"）。jadx 以 finally 体复制到各路径解决且行为精确——同族等价法在 jarde 已有先例，缺这一员。

**核心可证性**：Java 语义要求 return 表达式先求值（恰一次）、finally 副作用后执行（[FO 基准](../../evidence/java-syntax-2026-10-05/return-effectful-finally-patrol/fixture/FO.java)：`try{return bump();}finally{i=100;}` 输出 `1/100`）。朴素把 finally 复制到 return 语句之前会错序——正确形是 **temp 绑定**（`t = <expr>; <finally body>; return t;`）或等价可证形式。

取证：[return-effectful-finally-patrol](../../evidence/java-syntax-2026-10-05/return-effectful-finally-patrol/README.md)。

## What Changes

- MVP 先行：**单 return 出口穿过单条副作用 finally**（finally 落穿、无双 return 竞争）——呈现为 temp 绑定形或路径复制形，以 FO 求值序判别测试为验收；
- 插桩先行（实现者改动前回答，落盘 change 目录）：
  - **Q-i**：`loopless`（无循环）被拒的精确触发链——finally 副作用块的哪条结构事实使局部 def-use 切片无法词法绑定（对照空 finally 恢复、continue 分布恢复）；
  - **Q-ii**：与 local-scope 片的诊断门关系（同文本 "local crosses"——同门不同入口则合并实现）；与 finally-return-value / continue 分布等价法的关系（复用哪段呈现）。
- 拒绝边界保持：多 return 出口竞争同一 finally、finally 内含非局部出口（自身 return/break）、求值序不可证（副作用与 return 表达式交织）——维持整方法响亮拒绝。
- 对照零回退：空 finally、continue/break 分布、finally 返回值形逐字节不变。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：扩充可证明的 return 穿副作用 finally 恢复，保持求值序、副作用次序与拒绝边界。

## Impact

主要涉及 `jarde-java` 呈现层（exit/finally 等价分布）与局部 def-use 词法绑定；验收 fixture：[patrol](../../evidence/java-syntax-2026-10-05/return-effectful-finally-patrol/) 的 FY.java（noCall/loopless/emptyFin 锚）+ FX.java（finContinue/finBreak/finReturn）+ FO.java 求值序判别。
