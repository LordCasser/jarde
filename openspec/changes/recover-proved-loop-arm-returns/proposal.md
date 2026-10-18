## Why

固定71单元/612文件中的CF-11原始`TestNestedLoops5`在八个公开配置均无法恢复`testFor()I`，javac23整类重编因缺返回失败；同输入JADX四格成功且隔离运行同原oracle。实际首拒表明内层循环和return叶已经证明，但条件臂消费者仍按纯自然环所有权拒绝，需补齐现有证据的消费契约。

## What Changes

- 让已闭合的循环条件臂保留同次证明的终止return叶，区分普通join、循环latch与方法完成；原始内层早返回不得按常量不可达推理删除。
- 复用现有返回叶、ForHeader和隐式latch来源证明，核自然环与返回叶的精确所有者、唯一入口/普通出口、完整canonical边及来源。
- 保持未证明的共享叶、异常/clone边、额外入口/出口、错误latch及真实Stop的拒绝；不新增公共AST、pass、solver或依赖。
- 冻结完整原始类及A/B参数化补充，进行原/JADX/Jarde全源码、完整方法、重编运行与全部物理BCI对照；已有失败和工具边界分别保留。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`: 条件臂内已证普通循环可以消费其准确的终止返回叶，同时保留join/latch与整方法所有权和来源不变量。

## Impact

生产修改限于`jarde-java`的局部region证明与既有latch消费者；新增Reader驱动的私有/公开回归测试、固定class/Java输入及必要指纹和reader普查。前置为已接受的755fd产品、原始28命令基线、42命令整类矩阵和八请求真实首拒；补充已采集65命令/130raw、16Jarde/8JADX，独立审查和全源码重编尚待，不能计通过。root负责架构/工具链/对照验收，Luna按OpenSpec准备私有候选。

非目标为任意不可约图、throw/finally/handler新组合、返回叶识别规则放宽、任意循环串接与强制外层for美化。外层当前ForHeader=None本身不是缺陷；正确while可接受。71/612与CF-11整体仍保持部分已测，不因本片增计完成。
