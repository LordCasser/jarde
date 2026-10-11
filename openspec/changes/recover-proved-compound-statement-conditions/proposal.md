## Why

真实 TestSwitch2 的直接 case-target 汇合点证明通过后，普通条件恢复仍在 `ArmsDoNotMeet@78` 拒绝。其四个测试共享后续条件与语句出口，现有单测试 `Region::If` 无法承载先合并条件、再唯一消费两侧语句的结果；本地 JADX 的嵌套条件合并提供了可参考的算法方向。

## What Changes

- 对可证明、单入口、有限且可按短路规则归约的复合语句条件，先核对完整物理控制流，再合并条件并沿用普通 arm 恢复；共享 payload、return 和后续语句各拥有一次。
- 对真实Switch2中普通If包围已证明CompoundIf、二者共享一个void return续接的有限形状，复用既有terminal continuation步骤，完整验证该return的所有canonical入边与唯一归属后消费一次；不通用化boolean/Phi返回证明。
- 保留每个测试的真实边方向、求值顺序、条件表达式来源、预算和取消行为；未证明的形状保持有界拒绝与完整来源解释。
- 以前置 continuation 候选及真实 Switch2 为组合验收锚，并用有副作用、浮点 NaN、外部入口及 scope 边界对抗验证。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`: 增加受限复合语句条件的结构恢复、单次物理归属、来源和原子 Stop 契约。

## Impact

影响 `jarde-java` 的 region 证明与 AST 条件构造，以及读取这些结构的内部消费者。沿用 canonical CFG、SSA、现有 AST/来源/预算，不新增依赖、公开请求参数或 IR/Frame schema。前置 `recover-proved-switch-continuations` 尚未整体完成，两个变更可以共同验证，但必须分别记录证据；不将候选 join 的成功视为完整 Switch2 成功。

不扩展一般不可归约图、循环条件、异常/子例程边、ternary 条件、赋值内联或任意语句 DAG，不处理独立常量名 gate，也不据此宣称整个 CF12 或 71 单元完成。用户已授权本项目测试样例的编译和执行。
