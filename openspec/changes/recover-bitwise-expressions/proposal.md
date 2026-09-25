## Why

独立审计确认int/long与boolean共用的六个位运算指令尚未恢复：211项边界/效果对照和56项类型传播对照中，原class与JADX一致，jarde完整输出均无法编译。补齐该基础表达式不需要新恢复pass，但必须同时处理整数与boolean类型，避免生成JVM允许而Java无法表示的混合操作数表达式。

## What Changes

- 恢复已证明可表示的`& ^ |`表达式，覆盖int/long、boolean、嵌套组合、局部传递、已支持的返回/实参/条件消费。
- 保留真实操作数类型、非短路求值、调用与异常顺序、最终求值位置、括号及物理来源；混合boolean/int等不可表示形状明确引用。
- 固定267项正面执行基线及8项合法混合descriptor负例，增加消费/旧值/预算测试，由root用独立完整类执行验收。

前置：现有frame/SSA、局部类型决策、统一表达式来源/预算和调用消费机制。数值比较、throw、final字段与instanceof正在改动相邻文件，生产按root移交串行实施，不覆盖其代码。

非目标：移位、控制流0/1汇合、短路逻辑合并、复合赋值与自增、将异或模式优化成`~`/`!`、一般类型求解或区域扩张。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：在既有表达式范围加入类型忠实的整数位运算及非短路boolean逻辑表达式。

## Impact

受影响的是jarde-java的操作事实/解码、既有Binary运算符及类型、build的值消费与共享boolean证明、现有局部类型工作队列和统一emitter优先级；必要时只补现有纯值区域名单。无新crate、外部依赖、缓存或全图索引，Engine/CLI保持现有入口与报告平面。本proposal仅完成规划起点，未声明实现已经存在。
