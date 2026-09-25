## Why

[额外入口与异常边的两个当前拒绝控制](../../evidence/java-syntax-2026-09-25/field-presentation-contract/analysis.md)只发射 `@bytecode` 注释，`putstatic result:Z` BCI 30 没有成为字段语句；`field@1` 的 rule-detail 却写 `presented=true`，摘要也称字段已呈现。原 `ChainExtraBoundary.class` 已被后续短路恢复正确呈现，应作为 true 对照。原因仍是 `field::Plan::claimed` 在 Region/AST 构建前完成，报告直接把 claim 数量当最终输出数量。字节码来源命中不能证明 Java 字段操作已写出，因为引用注释也携带同一 BCI。

## What Changes

- 保留现有字段身份/形状 claim 供 Builder、构造候选和命名证明使用；在最终已提交的 Java AST 上另判定每条 claimed 字段指令是否确实呈现。
- `FieldRecord.presented`、未呈现理由和全方法字段摘要读取这一最终判定；被 Region/Builder 引用压掉的 claimed 字段与 `field@1` 自身证明失败保持不同理由。
- 覆盖普通读写、嵌套表达式、field++/++field 两条现有表示、构造器前缀、`<clinit>`、RuleDetails 的选择/范围/预算，不从 source map、全文字符串或 `claim` 本身推断最终呈现。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`: 字段规则的呈现记录和摘要 SHALL 与本次方法最终提交的 Java 字段操作一致；引用字节码的来源只说明可追溯，不算呈现。

## Impact

只涉及 `jarde-java::build` 完成后的私有呈现判定、`field::Plan` 物化与 `report` 摘要。无公开 AST/IR/pass 改动；不改变字段恢复或 Region 的结构选择。旧 `p3_eval_context` 中“源码包含成员名或 source map 命中即可算呈现”的弱断言需按新契约收紧，不能把本任务扩成短路图或词法作用域修复。
