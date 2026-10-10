## Why

已独立验收的新 CF07 双 JDK 完整类对照中，counted(II)I 能生成和执行正确的 while/if，但物理 goto@20→27 没有来源。它是内层 If 的 then arm 汇合跳转，现有 Builder 只保留空 arm 的隐式 goto 来源；应在现有 If 来源构造中补齐已证明的非空 arm 尾部跳转。

## What Changes

- 先诊断实际 Region::If join、末尾 Straight、canonical 全出边与所有权；不把静态预计形状当已证明结果。
- 在既有 If/OriginSet 路径保留精确指向该 If join 的末尾 goto/goto_w，派生来源覆盖完整 If 语句，保留条件、外层循环及所有旧来源与正文。
- 增加真实边/末尾反例及 Stop 回归，新 CLI 做完整源码三方重编运行和物理 BCI 独立验收。
- 前置来源产品 d714a6bcc 自身 CI 必须接受后才应用本片生产改动。lastIndexOf@25、ForHeader 扩展、conditional-value 折叠及通用来源扫描是明确非目标。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`: 已证明非空 If arm 的末尾汇合跳转保留准确物理来源。

## Impact

限 jarde-java 既有 Builder::region 的普通 If 来源路径、现有永久测试和完整 CF07 证据。复用 Region::If.join、SSA/canonical/operations、OriginSet 与 budget，不增加公共 IR 字段、Frame、pass、依赖、CLI 选项或通用回滚机制。71 单元/612 测试文件分母与 CF07 整单元状态保持，不能用本窄片代表 nested else 回跳已覆盖。
