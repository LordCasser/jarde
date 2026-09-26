## ADDED Requirements

### Requirement: SAM adaptation proves wrapper-primitive slot pairs

lambda/方法引用站点的适配证明 SHALL 对擦除 SAM、instantiated 与 impl 三种方法类型的参数和返回逐槽证明两段转换；每段只能为「恒等 / 已证明的 Object 检查或上溯 / 基本类型与其唯一包装类的装箱或拆箱」。捕获值的精确类型门 MUST 保持。数组构造器引用的 impl 若为合成方法，SHALL 仅在同次物理成员 Code 完整证明为单次目标数组分配并返回后呈现为 `T[]::new`；名称或 synthetic 标志 MUST NOT 代替 Code 证明。配对或 Code 不可证明时 MUST 保持既有拒绝并保留原方法体呈现。

#### Scenario: Primitive implementation behind a boxed SAM

- **WHEN** `Supplier<Integer>` 绑定 `this::supply`（impl `()I`），另一站点以 `Function<String, Integer>` 使用方法引用，方法体串联多个此类站点
- **THEN** 该方法 SHALL 恢复为含这些箭头调用点的语句，MUST NOT 因包装类/基本类型槽位配对整方法引用

#### Scenario: Array constructor reference

- **WHEN** `Function<Integer, int[]>` 绑定 `int[]::new`，实现为编译器生成的合成分配方法
- **THEN** 使用点 SHALL 呈现 `int[]::new` 箭头，合成分配方法的物理报告 SHALL 保留；类源码中保留该方法时 SHALL 通过 Java 8 重编译，不得与重新生成的合成方法冲突

#### Scenario: Unprovable pairs keep the refusal

- **WHEN** 擦除 SAM→instantiated→impl 的任一槽位对不是上述任一类（arity 不同、`String`↔`int`、包装类不对应、静态性不匹配），或数组 impl 的 Code 读不到/含额外效果
- **THEN** 站点 SHALL 保持既有拒绝码与引用呈现，MUST NOT 发明适配或丢掉 impl 的效果

#### Scenario: Execution through the adapted SAM

- **WHEN** 重编译后的恢复文本经 SAM 调用传入边界值（含 Integer.MIN_VALUE、null 接收者路径）与原 class 同输入运行
- **THEN** 返回值与异常 SHALL 一致
