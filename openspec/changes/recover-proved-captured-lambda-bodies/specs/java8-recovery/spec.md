## ADDED Requirements

### Requirement: Recompilable captured lambda source with preserved capture timing

当 Java 8 动态调用站点准确表示一个同类编译器生成的 lambda 体，且系统能证明其创建时捕获值、调用时参数与完整方法体之间的映射时，类级源码 SHALL 在 lambda 体中呈现该计算。只有完整类证据证明该合成方法的全部引用都属于已成功投影的站点时，系统 SHALL 原子省略其源码声明。源码 SHALL 保持捕获表达式在函数对象创建时求值、lambda 体在调用时求值的语义；不能证明时 MUST 保留物理成员及可定位的拒绝信息，不得发布部分内联或部分省略。

#### Scenario: One stable primitive parameter is captured

- **WHEN** 一个无额外效果的 Java 8 站点在创建时捕获一个未被重新赋值的 `int` 参数，lambda 调用时接收一个 `int` 参数，且同类私有 synthetic helper 的完整直线算术体与二者顺序精确对应、全部引用均已核对
- **THEN** 类级源码把原计算写在 lambda 体内并省略 helper 声明；原始、固定 JADX 和 Jarde 的完整源码与同一消费端均能以 `javac --release 8` 重编，并在 `java -Xverify:all` 下产生相同结果

#### Scenario: Receiver and primitive parameter are captured

- **WHEN** 站点在创建时准确捕获当前 `this` 与一个未被重新赋值的 `int` 参数，同类私有 synthetic 实例 helper 的 receiver/参数、完整直线体和所有引用均可证明
- **THEN** 类级源码保留对同一 receiver 的实例调用及捕获值，lambda 体只在调用时执行该计算；完整源码重编和验证运行结果与原始类一致，物理 helper 仍可独立查询

#### Scenario: Capture value or use is not stable

- **WHEN** 捕获值来自可能产生效果或会在 lambda 调用时重新求取的表达式，参数存在不满足源级有效 final 的写入，捕获顺序/receiver 不匹配，或 helper 还有普通调用或未知句柄引用
- **THEN** 系统 MUST NOT 为该候选省略 helper，MUST NOT 将创建时表达式移入 lambda 体重复求值，并保留拒绝来源

#### Scenario: Proof stops before class-level commit

- **WHEN** 站点、捕获来源、helper 方法体或全类引用清点因预算、取消、不完整读取或不支持的指令而未完成
- **THEN** 系统 MUST NOT 发布只完成部分捕获映射、内联或 helper 省略的类级投影；报告 MUST 保留实际停止原因和已经读取的物理来源
