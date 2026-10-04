## ADDED Requirements

### Requirement: 数组初始化器元素 SHALL 按赋值兼容判据接受

当数组初始化器的组件类型来自字节码事实（`anewarray`/`multianewarray` 的组件名）且某元素的呈现类型**可赋值给**该组件类型（同型，或组件是元素呈现类型的超类/超接口）时，系统 SHALL 接受该初始化器并按元素自身类型呈现元素（数组按组件类型）——与一切合法 Java 数组初始化的赋值兼容语义一致。

元素呈现类型**不能**赋值给组件类型（降向赋值、无关节类型）时 SHALL 保持既有拒绝。组件与元素类型 SHALL 均来自既有字节码事实（reader 的组件名、元素构造调用描述符），SHALL NOT 引入 LUB 计算或类型推断。同构数组（元素与组件同型）的既有呈现 SHALL 逐字不变。

#### Scenario: 异构装箱数组恢复

- **WHEN** `static Number cov(){ List<? extends Number> ln = Arrays.asList(1, 2L); return ln.get(0); }`（`anewarray Number` + `Integer.valueOf`/`Long.valueOf` 元素）经 `class-source` 呈现
- **THEN** 初始化器按赋值兼容接受（元素呈现 `Integer.valueOf(1)`/`Long.valueOf(2L)`、数组按组件类型）；整类渲染源集 `javac --release 8` exit 0、`main` 输出与原 class 一致

#### Scenario: 非法赋值方向仍拒绝

- **WHEN** 元素呈现类型不能赋值给组件类型（如组件 `String` 而元素为 `Integer` 的混淆/手工产物）
- **THEN** 保持既有响亮拒绝——判据是赋值兼容，不是无条件放宽

#### Scenario: 同构数组零回退

- **WHEN** 同构装箱数组（`Arrays.asList("a","b")`，元素与组件同型）经呈现
- **THEN** 既有呈现逐字不变
