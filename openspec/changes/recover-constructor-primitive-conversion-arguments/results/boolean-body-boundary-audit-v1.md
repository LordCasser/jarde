# Boolean constructor-body 拒绝边界

本记录对应 `root-boolean-context-v1` 的真实单测输出。永久记录位于：

- `openspec/changes/recover-constructor-primitive-conversion-arguments/results/root-boolean-context-v1/result.json`
- `openspec/changes/recover-constructor-primitive-conversion-arguments/results/root-boolean-context-v1/stdout`
- `openspec/changes/recover-constructor-primitive-conversion-arguments/results/root-boolean-context-v1/stderr`

其中 `stderr` 保存了首次 source-map 断言失败时的完整正文与 segment origin（SHA256：`9a4b9e7ca18e0299555529a96a98b40452eea4376f363f0de0f4372f99c0ef64`）。正文明确拒绝 `i2b@5`，消息原文为：

> the primitive conversion at BCI 5 expects `int` but its operand is presented as `boolean`, which does not meet the opcode's source category

实际 fallback 正文只输出 `return@9` 的主 origin，并把 `i2b@5` 列作 derived origin；source map 因而应断言 BCI 9 与 5，而不应要求未渲染的构造过程 `new@0`、`dup@3`、`invokespecial@6` 各自成为文本 segment。

测试仍保留 NewRecord 的结构坐标 `head=0, dup=3, argument=5, constructor=6`，以及直接 SSA 链：`iload_0@4` 读取 entry `Local(0)`，`i2b@5` 读取由 BCI 4 定义的值。新断言只覆盖渲染结果：完整方法是 `Fallback/Mixed/NotJava`，文本包含精确的源类别拒绝理由，source map 对应 5 与 9。

root 采集的 `jre_new_sites` diagnostic 表述为 `1 presented as new, 0 refused`，虽然完整正文因构造器参数转换拒绝而 fallback。`NewRecord.presented` 及该 diagnostic 对“Site 结构已识别”和“构造表达式已成功写入 Java 正文”的边界命名不够清楚，容易被读成正文呈现成功。这是既有证据/diagnostic 语义债务，单独拆片处理；本次不改 API、diagnostic 或产品代码。
