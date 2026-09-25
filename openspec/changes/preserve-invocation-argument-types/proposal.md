## Why

调用参数沿用赋值/返回的隐式转换规则，导致重编译源码重新选择其它重载：已实测 Object、char→int、null、数组、装箱、多参数、构造器、byte/short 常量及方法引用错值。外部泛型返回的擦除类型与参数同为 Object 时仍会错选，因此仅比较呈现类型相等不足以保护目标。

## What Changes

- 调用与构造器参数按池目标描述符保留必要静态类型；赋值、字段写入和返回的隐式转换规则保持各自语义。
- 使用已有转换表达式固定数值类型、范围内 byte/short 常量、null、Object 上溯、同型泛型调用及已证明的函数式目标类型。
- 对无证据的引用类型关系保留调用缺口，禁止为了匹配 descriptor 合成可能新增 CCE 的任意检查。已运行的合法 JVM 接口反例证明该限制是必要的。
- 从自写 fixture 生成真实文本后重编译执行，以原 class 为基线，记录 jadx 也会错选的例子。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：新增调用参数静态类型、重载选择和无法证明转换时的拒绝要求。

## Impact

前置条件为已解码的调用描述符、既有 Cast/Type、函数式工厂 descriptor 和调用生产者保留。本 change 在 cast 生产改动交接后实施，主要影响 `jarde-java` 参数构造、必要的函数式表达式类型事实与对应回归；旧 2c.29 对调用参数的宽泛去 cast 裁决由本项修正。

非目标：类型层级或泛型 resolver、加载目标依赖/方法体、bridge 的源码声明冲突、lambda synthetic 名称管理、静态限定符恢复和方法引用内部目标的全面重载重建。无新 crate、pass、AST 类别或生产依赖。不将测试过的样例等价夸大为任意 class 的验证结论。
