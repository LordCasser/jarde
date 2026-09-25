## Why

[冻结 Java 8 八路径对照](../../evidence/java-syntax-2026-09-25/mixed-short-circuit-return/analysis.md)表明，`return (a && rhsB()) || rhsC()` 与刚验收的混合字段短路值具有相同的三个测试、两个 `1/0` producer 与唯一栈 Phi，只将消费指令从 `putstatic Z` 换为 BCI 21 `ireturn`。原 class/JADX 的返回值和两次 RHS 调用数八行一致；当前 Jarde 在字段图实现后仍整段引用，完整类缺少返回语句而无法重编。这是消费者证明边界，不需要新的布尔图机制。

## What Changes

- 复用已有 `ShortCircuitValue` 的有界闭合决策图与 `Conditional` 表达式；让候选消费点可为唯一静态 `Z` 字段写入或唯一 `ireturn`。
- 返回分支只在方法描述符声明 `Z`、真实 opcode 为 `ireturn`、同一栈 Phi 仅由该指令消费且逐图测试/生产者证明全通过时提交已有 `Return` 语句。
- 对 `I`/其他描述符、第二消费者、额外入口、异常边、独立效果及预算停止维持整图引用；原字段正例及其负例不变。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`: 对可证明闭合的混合短路布尔值返回，输出逐路径保真、可 Java 8 重编的返回表达式；证明不足时完整拒绝。

## Impact

限定在 `jarde-java::region` 的既有候选消费锚点、`jarde-java::build` 的 SSA/描述符消费证明与既有 `Conditional`/`Return` AST 发射。无需新增公开 AST/IR 或独立重写 pass；字段 `presented` 报告问题与局部变量作用域债务另案处理。
