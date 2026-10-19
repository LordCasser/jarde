## Why

完整原件 TestBreakInLoop5 的三方基线确认：Jarde 的普通 try 只恢复初始化，循环和返回表达式共 11 个 live block 未恢复。真实 IR 表明旧 prefix+loop 候选只接受范围结束处的 goto，而本例的后续块在保护范围内计算表达式、于范围右边界直接 return，不能冒充 goto bridge。

区域恢复后，完整公开 API 又确认同一原件的 catch 绑定缺口：槽2的普通 double 与异常参数共享物理槽，多异常源使 handler124 的入栈值成为自身 Stack0 phi。guard 已接受这种 catch 绑定，Builder 类型兼容性检查却仅排除直接 Caught，误将该头部声明当作 double 的普通赋值。

## What Changes

- 在已有普通 named catch 区域内，恢复有精确证据的初始化前缀、完整循环和单块返回表达式；表达式仍受原 catch 保护，唯一边界外指令为真实末尾 return。
- 在 Loop4 的闭合 break 分支证明基础上，为 Loop5 的循环外侧条件体添加精确命名异常行归属；没有该归属时保持拒绝。
- 保留 handler 独立归属、效果次序、物理异常表和源码映射；候选失败或中止须回滚，旧 goto bridge 与 List census 契约保持不变。
- 复用已恢复 catch 参数的入口身份和既有 handler Stack0 phi 契约，避免 catch 头部绑定污染普通局部变量的类型；保持现有槽身份、作用域和逃逸检查，不新增一般局部变量拆分机制。
- 用完整 Loop5 原件、JADX、Jarde 的完整 API、整类重编和双 VM 对照验收，并回归 Loop4 及已有关键形状。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`: 增加普通命名 catch 中有证据的循环和范围右边界返回表达式恢复。

## Impact

局限 jarde-java 区域证明、Builder 的 catch 绑定类型检查和定向测试，不增加公共 API、AST 类型、依赖或 CFG pass。前置为冻结的 Loop4/5 完整三方基线和真实四原件 IR；实现依赖 recover-proved-loop-break-branch-bodies 的闭合证明，不能以未完成循环进入本候选。非目标为 finally/retry/monitor/嵌套 try、任意跨行范围、多个 tail、多循环串联、一般 List census 扩域及其他待基线用例。仅成功验收两个原件不代表 CF09、71 单元或 612 文件全部完成。
