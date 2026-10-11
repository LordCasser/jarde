## Why

固定 JADX TestNestedLoops2 已真实完成29命令双JDK完整三方：九行行为同原，JADX保留外层indexed for，而Jarde输出while。Root同次Reader/SSA诊断确认初始化、更新、phi门通过后，已有ForHeader在条件List.size@6处被独立白名单拒绝；同一条件已被既有test_is_pure接受。

## What Changes

- 让已有ForHeader条件扫描接受既有条件证明支持的非局部写入值生产者，包括调用、字段读取、数组长度/元素读取、取负与数值比较。保持原条件每轮的求值位置、次数、次序和异常路径。
- 保留初始化、phi、更新、局部不变量、所有权及Builder消费约束；不引入另一套purity或loop识别机制。
- 增加固定上游、调用副作用/异常与动态读取控制，核完整源码重编执行、物理来源、Stop与旧拒绝边界。

### Prerequisites and Non-Goals

前片产品b155877ee及自身CI38144212651已接受，关闭文档593fbcc0b已推送并实际clean审计。当前诊断源码已恢复、无target。先完成新控制基线，再实施。泛型List<String>投影退化单列债务；不改Signature、自然环、公开IR/pass、foreach、header赋值/自增或不满足现有证明的loop，不计整个CF11/71单元完成。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：已证明计数循环的条件含既有支持的值表达式时保留for呈现与行为、来源及停止语义。

## Impact

主要是crates/jarde-java/src/region.rs的私有ForHeader准入；继续使用build.rs已有条件、初始化和更新消费。永久固定fixture/API与必要反例测试，语料manifest/census按真实新增测量更新。无公开API、依赖或新层。
