## ADDED Requirements

### Requirement: SAM adaptation proves wrapper-primitive slot pairs

lambda/方法引用站点的适配证明 SHALL 在 impl 方法类型与 instantiated 方法类型 arity 一致、且每个对应槽位为「恒等 / 已证明的 Object 检查或上溯 / 基本类型与其唯一包装类的装箱或拆箱」之一时接受该站点，箭头文本按既有规则呈现。数组构造器引用（impl 为合成分配方法，逐槽与 instantiated 对应）SHALL 同样被接受并被呈现为 `T[]::new` 形式的箭头。配对不可证明时 MUST 保持既有拒绝并保留原方法体呈现。

#### Scenario: Primitive implementation behind a boxed SAM

- **WHEN** `Supplier<Integer>` 绑定 `this::supply`（impl `()I`），另一站点以 `Function<String, Integer>` 使用方法引用，方法体串联多个此类站点
- **THEN** 该方法 SHALL 恢复为含这些箭头调用点的语句，MUST NOT 因包装类/基本类型槽位配对整方法引用

#### Scenario: Array constructor reference

- **WHEN** `Function<Integer, int[]>` 绑定 `int[]::new`，实现为编译器生成的合成分配方法
- **THEN** 使用点 SHALL 呈现 `int[]::new` 箭头，合成分配方法保留在类文本中并标注已被使用点呈现

#### Scenario: Unprovable pairs keep the refusal

- **WHEN** impl 与 instantiated 的槽位对不是上述任一类（arity 不同、`String`↔`int`、静态性不匹配、impl 读不到）
- **THEN** 站点 SHALL 保持既有拒绝码与引用呈现，MUST NOT 发明适配或丢掉 impl 的效果

#### Scenario: Execution through the adapted SAM

- **WHEN** 重编译后的恢复文本经 SAM 调用传入边界值（含 Integer.MIN_VALUE、null 接收者路径）与原 class 同输入运行
- **THEN** 返回值与异常 SHALL 一致
