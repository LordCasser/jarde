## Why

JADX TestArrayFillConstReplace 对照已完成39命令/8完整腿、205闭合文件，原程序、JADX和当前Jarde重编执行全部一致；呈现仍有差距：Jarde把 `new int[] { 127, 129, CONST_INT }` 的末项写成65535。现有同类整数常量候选与AST节点已具备，缺少数组返回的准入和精确名称范围，不需要另建常量解析机制。

## What Changes

- 扩展既有同类整数名称投影，仅恢复普通方法直接返回的一维int数组初始化器中的直接int字面量。
- 复用同轮opaque AST、唯一static final I ConstantValue候选、名称遮蔽拒绝和原子呈现；保留全部物理报告与BCI。
- 用既有emitter回放核名称的确切输出范围，不增加commit阶段常驻segment表，不以字符串搜索定位数组元素。
- 前置条件：静态阶段修复链最终提交6fd51a18自己的完整CI验收（41b8因旧Unicode断言失败，历史证据保留）；当前冻结静态CLI仍用于基线，不能冒称含本片实现。实例数组片先单独提交检查点，再顺序应用本片，分别冻结产品与测试并验收自身CI。
- 不处理跨类/继承/全局常量、long极值格式、所有表达式常量传播、嵌套数组、cast与算术恢复；不改变71单元分母或宣称EM18完成。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`: 一维int数组直接返回中的同类唯一常量名称呈现，含遮蔽/歧义/停止及精确物理来源约束。

## Impact

修改范围为现有 `src/facade.rs` 的AST保留与整数名称投影、`crates/jarde-java/src/report.rs` 的既有transformer和内部use handoff，以及 `emit.rs` 的body-only replay适配器和 `src/class_source.rs` 的既有writer范围适配。公开JSON/schema、依赖、常量候选表、AST种类、commit发射原则不变。字节码中的整数不能证明原源码用了常量名，输出只是已有唯一候选约束下的等值名称呈现；完整源码oracle用于检验本例。
