## Context

见 [proposal](proposal.md)。`RecoveryOutcome` 区分 Produced 与 Stopped；`StmtKind::Fallback` 发射注释与 BCI 引用，仍参与 `Builder::push` 的 `program.statements` 计数。因此既不能用非空 text，也不能直接暴露这个已有计数作为“有 Java 语句”。benchmark 的 `compare2.py::statements` 是过滤 token 的启发式，不能作为引擎契约。

## Goals / Non-Goals

**Goals:** 给调用方一个稳定、无需解析 Java 文本的内容信号，并说明它能证明什么。

**Non-Goals:** 不计算语义恢复百分比，不新增逐指令证明状态，不把 content 合并进 quality 或 execution。

## Decisions

### 1. 只加一个内容分类

`RecoveryReport.content` 使用闭合 enum：`NotProduced`、`ExplanationOnly`、`ContainsStatements`，JSON 为 snake_case。Stopped 对应 NotProduced；Produced 且交付结构中没有非 fallback 语句为 ExplanationOnly；存在实际发射的非 fallback 语句为 ContainsStatements。不另加 produced-text 布尔、计数或第二套 quality。

非 fallback 语句包括声明、赋值、调用、constructor call、return 和控制流语句。`if (arg0) {}` 包含条件求值，算 ContainsStatements，但内部未恢复的内容仍必须带 fallback，quality 不因此提高。`return;` 也是语句；wrapper、成员说明、花括号、理由和 BCI 注释不是。字段只说明内容，不叫 `recovered` 或 `semantically_complete`。

### 2. 从提交成功的结构导出

在现有 AST/emitter 路径累积是否发射了非 fallback 语句，并仅随最终产物提交到 report。预算或取消导致构建/发射失败时，即使此前已形成 AST，也返回 NotProduced，text/source map 继续遵循当前停止契约。分类不重扫输入、不构建第二份 AST、不采用正则；使用既有遍历和预算，避免额外无界递归。

选择这一位置而非 `program.statements > 0`，因为旧计数包含 fallback；选择结构而非去注释后判文本，是为了避免字符串里的 `//`、转义和排版变化影响分类。

### 3. 评测分母与语义证据分开

汇总列出总请求、有效带 Code 请求、Produced、ContainsStatements、ExplanationOnly、Stopped；Produced 的两个内容类别之和可对账。没有 Code 的方法单列适用状态，不在语句覆盖分母里伪装失败。诊断出现次数、受影响方法数、fallback 区域数分别统计，多诊断不相加成互斥失败率。

文本/调用/字符串相似度各自记录有效 pair 数与排除原因，不能把 10,977 个文本 pair 当作 6,132 个调用 pair 的分母。`<init>/<clinit>` 被 jadx 折叠的缺项单列，未知匹配不自动算正确或错误。只对受控、可编译的 Java fixture 做行为对照；Mixed 以原始操作、effect、origin 完整性验收。

### 4. 复用与层次

复用私有 AST、emitter、report 和 serde。没有缺失的通用 parser 能力，不新增 tree-sitter/regex 等库；无需重新做依赖许可及维护选型。内容分类不是 dialect validation、runtime selection、编译或 verification 的新入口。CLI 直接序列化库结果，不自行分析 text。

## Risks / Trade-offs

- ContainsStatements 被再次误读为恢复成功 → 文档与例子同时展示“空分支 + fallback”和 return-only，明确不能替代 quality/验证。
- 在 emitter 成功之前写入 content → 故障注入覆盖 output budget 与取消，要求停止报告始终 NotProduced。
- 新旧测量分类发生差异 → 保存历史启发式定义；新报告标记分类版本并列出差异样例，不回填旧数字。

## Migration Plan

增加字段并一次性更新报告构造、CLI golden 与示例调用方；旧 Produced 方法保留其原本的产物存在语义，不作为兼容伪装改变含义。规划新增字段，实际实现后再同步主规格。与 R8/R9 的 AST 变化在同一候选上回归，不能使用该字段关闭正确性缺陷。
