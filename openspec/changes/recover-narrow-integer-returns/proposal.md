## Why

普通 javac 的 byte/char/short 局部回读目前因局部呈现为 int 而在返回处被拒绝；合法的 ireturn 窄返回变体还让现有字段自增快捷路径输出无法编译的正常正文。返回指令自身已规定窄化，恢复不需要先推导局部取值范围。

## What Changes

- 对已能呈现的整数值，按真实 ireturn 和本方法 B/C/S 返回 descriptor 保留返回时窄化。
- 普通返回、现有同步返回、已有 switch 返回下推和字段自增返回共用该返回位置语义，保持更新效果、异常及真实来源。
- 复用 Cast 和现有类型事实，不改变普通赋值、字段存储或调用实参规则；不精化窄局部类型，也不新增范围求解器。
- boolean 最低位转换、未支持的 stack phi/条件表达式、15条显式 conversion opcode 分别处理。生产实施在共享求值顺序任务之后串行进行。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：恢复已有可呈现整数值在 ireturn 的 B/C/S 隐式窄化，保持来源、预算与保守拒绝契约。

## Impact

主要涉及 jarde-java 的返回位置与已有特殊返回消费路径。SSA 指令已携带 effective opcode，方法事实已提供返回 Type；不需要新的 reader/SSA 模型、AST 节点、恢复 pass 或依赖。证据在 `../../evidence/java-syntax-2026-09-22/numeric-conversions/` 的 `narrow-locals/`、`return-sinks/` 和 `return-sinks-core/`。后者独立去掉尚未支持的 stack phi，以完整类实际执行验收本项，不能把广义三元表达式恢复混入。
