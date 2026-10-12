# 字段拼接与 Error 实参的架构审计

二轮 Luna 私有审计按原字节保留，root核四组 retry-v2 实际报告的 physical class身份、字段70/Builder57–76/Error53–82 拒绝和源码省略。报告字节与digest见 manifest；实际报告已在父项 [311条目证据包](../../../recover-loop-body-try-catch/results/candidate-focused-and-comparison-root-v1/README.md) 归档。这里没有新的 Cargo/JDK 执行，不作为实现通过。

**采用 design.md 的 root 方案。** `audit-luna-v1.md` 和 v2前半段的“必须新增 nonvolatile this metadata”已被第二轮审计推翻；v2后半段的 Revised audit 才是修正结论。字段身份直接复用现有 field::Plan，volatile/null本身不是不恢复的理由。原位单读与异常保护范围仍须逐项证明。

第二轮新增风险是 Object 转换可晚于后续字段求值。已读官方 [OpenJDK讨论](https://mail.openjdk.org/pipermail/compiler-dev/2021-September/017911.html)，它记录 inline/indy 在有副作用 operand 上的转换时机差异；该历史讨论不能代替本机JDK8/23新执行证据。root选择复用现有AST在新许可链的对应片段显式调用 valueOf(Object)，之后用双JDK对照验收，不因只支持primitive的四组v2通过就跳过Object控制。

未计费的旧ordinary concat扫描、旧调用/结果多读判定与static常量读取/初始化的呈现问题独立登记；本次所有新增值流与覆盖检查必须计费，不允许借旧扫描器逃避预算。
