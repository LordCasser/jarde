## Why

reader已经逐位保留float/double常量，但Java层尚未表达它们，导致常量返回、浮点阈值比较及重载调用失败。36项完整类基线、536项有限值拼写实验及特殊NaN池变体说明：补齐忠实常量足够，但不能把所有NaN归一为同一值。

## What Changes

- 恢复有限float/double常量，包括有符号零、次正规值、边界和保留类型的实参；支持正负无穷及标准正quiet NaN。
- 精确保留原始bits和float/double静态类型，沿用算术、取负、比较、局部及调用消费和既有来源/预算路径。
- 对不能以本项无额外调用的常量表达式精确表示的NaN保留明确引用，禁止归一化符号或payload、添加运行时bit-conversion调用后冒称原常量语义。
- 复用先行求值顺序修复的非final有名值保存，阻止javac把真实JVM浮点操作折叠成不同NaN位模式；不新增第二套临时变量机制。
- 以原始class、JADX、实际jarde完整输出重编译执行作三方对照，并为特殊NaN记录各自的位模式差异。

前置：先验收`preserve-deferred-value-order`的局部绑定与位置证明；已有reader的原始bits、frame/SSA、表达式类型、数值比较、统一emitter及产物提交契约。相邻语法点的生产文件按root交接串行修改。

非目标：任意NaN payload的运行时构造、浮点常量折叠优化、转换指令、一般数值求值器、静态字段ConstantValue声明恢复或新区域模式。现有字段声明缺口独立记录。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：加入类型及位模式忠实的浮点常量表达式和不可精确表示时的明确边界。

## Impact

影响jarde-java的常量操作事实/解码、有限浮点AST叶子、结果类型、既有常量值构造及统一emitter；必要的遍历和纯值证明随原路径适配。不新增crate、依赖、pass、解析器、缓存或外部运行器，Engine/CLI入口和结果平面保持既有结构。本项当前只完成规划，未实现。
