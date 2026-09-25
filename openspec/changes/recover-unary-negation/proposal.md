## Why

Java 的 `return -x` 在当前恢复层中因 `ineg/lneg/fneg/dneg` 被收成 `Other` 而退成字节码引用。已有 `present-proved-java-structure` 的 2c.11 登记了此问题；本 change 单独承接该任务，补齐可独立验收的一元值表达式，避免与区域、类型转换和循环的其它缺口混合。

## What Changes

- 四种取负指令在操作数可呈现时写成一元 `-`，保留真实的求值位置、数值类型与来源。
- 嵌套负号与复合操作数保持分组，不得写成 `--x` 或用 `0 - x` 替代。
- 自写 Java fixture 经 javac、jadx 与 jarde 三方对照；重编译并比较整数边界、浮点负零、NaN 分类、调用次数及异常路径。
- 无法呈现操作数时仍保留引用，且不能丢掉被延期呈现的生产者。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：新增一元数值取负的呈现、分组和受控执行验收要求。

## Impact

前置条件是当前的 Frame/SSA、类型化 AST、值消费构造和有界 emitter 已存在。实现仅涉及 `jarde-java` 的事实解码、值构造、AST、发射及其直接消费者，和对应 fixture/test。无新 crate、pass、registry、生产依赖或宿主协议；不改变 JVM 指令语义。

非目标：浮点常量恢复、一般转换、位运算、三元/phi、循环新形状和自增。取负被这些尚未恢复的外层结构引用时，不宣称整个方法已恢复。
