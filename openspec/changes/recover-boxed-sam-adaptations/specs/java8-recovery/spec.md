## ADDED Requirements

### Requirement: SAM adaptation proves wrapper-primitive slot pairs

lambda/方法引用站点的适配证明 SHALL 对擦除 SAM、instantiated 与 impl 三种方法类型的参数和返回逐槽证明两段转换；每段只能为「恒等 / 已证明的 Object 检查或上溯 / 基本类型与其唯一包装类的装箱或拆箱」。捕获值的精确类型门 MUST 保持。数组构造器引用的 impl 若为合成方法，SHALL 从同次 BSM 的精确句柄经既有按需成员读取取得 Code，且仅在同次物理成员 Code 完整证明为单次目标数组分配并返回后呈现为 `T[]::new`；名称或 synthetic 标志 MUST NOT 代替 Code 证明。类源码若省略该合成方法声明，MUST 先证明选定输入范围内没有未投影的其它使用，同时保留其物理方法报告。候选读取、用途证明或 Code 不可证明时 MUST 拒绝数组引用；独立适配仍已证明的站点 MAY 保留调用原合成方法的普通 lambda 文本及物理声明，不得输出同名冲突的半投影。

#### Scenario: Primitive implementation behind a boxed SAM

- **WHEN** `Supplier<Integer>` 绑定 `this::supply`（impl `()I`），另一站点以 `Function<String, Integer>` 使用方法引用，方法体串联多个此类站点
- **THEN** 该方法 SHALL 恢复为含这些箭头调用点的语句，MUST NOT 因包装类/基本类型槽位配对整方法引用

#### Scenario: Array constructor reference

- **WHEN** `Function<Integer, int[]>` 绑定 `int[]::new`，实现为编译器生成的合成分配方法
- **THEN** 使用点 SHALL 呈现 `int[]::new` 箭头，合成分配方法的物理报告 SHALL 保留；已证明只由此站点使用的 helper 在类源码中 SHALL 原子省略其物理声明，并通过 Java 8 重编译，不得与重新生成的合成方法冲突

#### Scenario: Unprovable pairs keep the refusal

- **WHEN** 擦除 SAM→instantiated→impl 的任一槽位对不是上述任一类（arity 不同、`String`↔`int`、包装类不对应、静态性不匹配），数组 impl 的候选/Code 读不到或含额外效果，或类级用途证明发现其它引用或停止
- **THEN** 未证明的配对 SHALL 保持原有引用；仅数组投影未证明时 MAY 使用独立已证的普通 lambda 调用并保留物理成员，MUST NOT 发明适配、丢掉 impl 的效果或发表会发生 helper 名称冲突的类源码

#### Scenario: Execution through the adapted SAM

- **WHEN** 重编译后的恢复文本经 SAM 调用传入边界值（含 Integer.MIN_VALUE、null 接收者路径）与原 class 同输入运行
- **THEN** 返回值与异常 SHALL 一致
