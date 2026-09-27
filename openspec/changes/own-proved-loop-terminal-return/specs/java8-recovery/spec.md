## ADDED Requirements

### Requirement: 已证循环内提前返回具有唯一源级归属

当循环体内的条件分支到达一个唯一归属于该循环、立即终止方法的返回叶时，系统 SHALL 将该返回表达式保留在循环体的对应分支，MUST 保留循环条件、更新、正常退出和返回值的执行顺序与来源。系统 MUST NOT 因终止叶不带回边就遗漏它，也 MUST NOT 把共享或无法证明归属的出口误收入循环。

#### Scenario: 倒序查找从循环体提前返回

- **WHEN** `LoopCases.lastIndexOf` 的循环体在命中数组元素时返回当前下标，未命中时递减继续，循环自然退出后返回 -1
- **THEN** Jarde SHALL 输出无 bytecode quote 且可 Java 8 重编的完整类源码；原 class、固定 JADX 和 Jarde 经 `java -Xverify:all` 运行的八行结果 SHALL 一致，返回值、循环更新和数组读取各只执行在原路径

#### Scenario: 已支持的普通循环保持语义

- **WHEN** 同一固定类含 `enabled && i < 10` 的循环头以及含 if/else 更新的计数循环
- **THEN** 系统 MUST 保持两者完整源码可重编和运行结果，允许用语义等价的 `while` 表达原 `for`，不得重复或遗漏更新

#### Scenario: 终止叶归属证据不完整

- **WHEN** 候选返回叶有循环外额外入口、多个不属于同一已证分支的入口、异常/子例程边、不是终止返回、无法追溯返回值，或请求预算/取消停止
- **THEN** 系统 MUST 原子拒绝受影响的方法并保留物理位置与原因，MUST NOT 发布漏掉返回叶或跨未呈现区域使用局部变量的完整源码
