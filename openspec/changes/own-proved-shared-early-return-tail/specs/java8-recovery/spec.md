## ADDED Requirements

### Requirement: 已证早退分支的共享前向尾部只恢复一次

当条件分支的两条路径可以提前返回，也可以汇入同一段有可观察效果的正常前向尾部时，系统 SHALL 仅在尾部的入口、正常前驱、分支覆盖、物理来源及求值顺序均已证明后，把该尾部恢复为一次 Java 语句序列。完整类源码 MUST 能以 Java 8 重编；原 class 与恢复源码的返回值和可观察效果 MUST 一致。证据不足时 MUST 按既有拒绝契约保留受影响方法的 bytecode 与 origin，MUST NOT 复制尾部、副作用或虚构返回。

#### Scenario: 两侧均可早退或抵达一次共享尾部

- **WHEN** `nested(boolean,int,int)` 的两侧分别可在 BCI 12、22 返回 `false`，或经 BCI 8、18 汇入 BCI 24 的一次 `hits++` 和 `return true`，且所有边界和来源证明闭合
- **THEN** Jarde SHALL 恢复单一尾部；原 class、固定 JADX、Jarde 的完整 Java 8 类源码 SHALL 重编并以 `java -Xverify:all` 运行相同的 15 行输出，包括各次 `hits` 值

#### Scenario: 共享尾部证据不完整

- **WHEN** 候选尾部有额外入口、回边、异常边、交叉所有权、无法证明的效果顺序或预算/停止条件未闭合
- **THEN** 系统 MUST 原子拒绝受影响的方法并保留可定位的物理 BCI 与来源；MUST NOT 将同一尾部放入多个分支，也不得发布看似完整却缺少返回的恢复声明

#### Scenario: 独立早退与普通 else-if 不回退

- **WHEN** 方法没有共享尾部，只含独立早退守卫或已有可拼写的 else-if 链
- **THEN** 系统 MUST 保留已有返回与条件求值行为，以及已恢复的 else-if 结构，不得为了共享尾部候选而新增重复归属或拒绝
