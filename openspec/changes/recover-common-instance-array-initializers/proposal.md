## Why

JADX TestArrayInitField对应的实例数组仍以构造器赋值呈现。完整fresh基线已确认Jarde语义正确、JADX可提升共同初值，但不同构造器RHS被JADX错误合并；现在应在已有完整AST和class-source原子投影内补共同前缀证明，避免照搬软比较。

## What Changes

- 普通类所有构造器均直接super、且具备相同连续实例数组初始化前缀时，将数组初值一次提升至字段声明，从各派生构造器正文只移除对应赋值。
- 完整比较数组类型/长度/元素序、字面值与same-class static call的完整目标和参数，禁止仅按opcode/方法名/生成文本认定RHS相同。
- this委托、参数/this读、不同RHS、遗漏/重复写、异常scope、不安全字段顺序或任何未闭合构造器均保留既有呈现；保持原物理方法与来源、现有预算/取消和整组提交。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`: 从全部已闭合direct-super构造器的共同数组前缀恢复实例字段初值。

## Impact

`crates/jarde-java/src/report.rs` 的opaque同轮AST适配器、`src/facade.rs` 的class-source装配、现有字段/方法投影与集成测试；复用既有AST/emitter/field@1/init@1/call targets，无新IR/pass/全局类型框架/依赖/公开报告schema。若物理字段claim尚未保留，仅在构造器同轮opaque sidecar带必要已证事实，并逐项计费。

前置：instance-field-init-next的原2/2/Jarde2/2和JADX4编译成功但4语义失败基线已root独立验收，当前25c5测试修复提交自己的CI待验，不能提前借其门禁。非目标：constructor graph/this链提升、标量常量变量/ConstantValue、参考类型协变数组/嵌套多维、更广表达式、字段重排、try范围搬移、其它JADX错误修补，或EM18整单元完成。
