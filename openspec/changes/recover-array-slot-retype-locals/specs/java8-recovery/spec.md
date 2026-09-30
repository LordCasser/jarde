## ADDED Requirements

### Requirement: 不同元素类型的数组槽复用按定义分段声明

系统 SHALL 在同一局部槽被多个不同元素类型的数组定义先后写入、且各定义的读取集合互不相交（无跨段 phi）时，按定义分段呈现：每段以自身元素类型声明，恢复文本可 `javac` 重编且行为与原 class 一致。同类型多定义、单定义与交叠真别名 SHALL 保持现有输出；不可分段形状 SHALL 保持既有拒绝或呈现。

#### Scenario: 两类型复用可重编

- **WHEN** `int[] a = {…}; …; boolean[] f = new boolean[3]; …` 复用同一槽且读取不交叠，三方 Java 8 重编运行
- **THEN** 每段声明类型正确，`java -Xverify:all` 下副作用次序与输出与原 class 逐字一致

#### Scenario: 边界不变

- **WHEN** 同类型多定义、单定义、或跨段交叠读取（真别名/段内 phi）
- **THEN** 输出与本变更前逐字一致

#### Scenario: 数组证书零回退

- **WHEN** 输入覆盖嵌套初始化器、布尔数组、窄存、部分分配等已验收数组形态
- **THEN** 输出逐字不变
