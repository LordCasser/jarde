## Why

固定71单元/612文件的CF-11原始TestSequentialLoops在当前产品八公开格仍为fallback，完整生成源码16次重编均缺return；JADX六次可用重编和12次双VM运行同原oracle。真实Reader诊断确认首循环头Store5是必须每轮执行的语句，当前条件表达式消费者因StatementFree拒绝，需要消费既有Endless与If表示来保留其位置。

## What Changes

- 对已证明唯一入口/出口、循环头一条条件分支及单一直线回边的自然循环，将头部语句留在现有while(true)内，并以现有If/LoopBreak退出；纯条件while/for契约保持原样。
- 先核完整canonical边、同一clone身份、物理分支/回跳、SSA与精确所有者，证明完成后一次提交claim；不通过试走吞Stop或遗漏后续顺序循环。
- 以原始完整类和调用效果补充核执行次数、顺序、异常和同实例状态，原/JADX/Jarde完整源码原样重编及全物理BCI来源验收。
- 增加Reader驱动边界与真实停止测试；不新增公共IR、pass、全局solver或依赖。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`: 闭合的单出口循环可以把条件前必须执行的语句保留在循环体内，正常退出后继续后续循环与return。

## Impact

主要影响jarde-java局部region消费者与现有If/Endless builder连接；增补测试、固定Java/class及必要指纹/reader普查。前置为e70产品、Seq1原始及完整源码基线和当前八API恒同重放；临时Reader诊断已实际执行、独立审计并恢复源码，补充完整基线亦已root接受，冻结记录见results。

本片不承诺任意分支循环体、多出口/嵌套/异常表循环或新的条件赋值表达式算法；这些保留原证明和失败。正确while(true)可接受，无需美化成源文件同形循环。CF11及71/612整体计数不因本片增计完成，root负责架构与工具链验收，Luna准备私有候选。
