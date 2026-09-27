## ADDED Requirements

### Requirement: Complete class source projects multiple proved enum switches in one method

当完整 class-source 恢复在同一方法中找到多个 Java 8 enum remap-array switch 站点，且同一 helper 中候选所选的全部不同 table 恰好封闭覆盖其 `<clinit>` 表初始化时，系统 SHALL 对 helper `<clinit>` 进行一次完整联合证明，并继续逐表验证物理 enum 定义及常量映射。联合证明 MUST 将每条物理指令及 handler 恰好归属一个表初始化组或唯一最终 return；每组 MUST 具有实际 BCI 顺序上的 values/arraylength/int[]/putstatic 结构、逐常量 enum-field/ordinal/int-store 和独立 `NoSuchFieldError` handler。候选之外的额外表、未知操作、额外副作用、缺失或重复映射、或无法精确归属的 CFG/SSA 路径 SHALL 拒绝。联合证明不扩展候选发现或依赖读取。

当所有站点和选中表均证明成功时，系统 SHALL 将所有站点应用于同一份已恢复方法 AST，并作为一个原子结果投影。如果任一站点的证明、grouped AST 发射或预算执行失败/停止，完整方法 SHALL 保留原整数 table switch；输出 MUST NOT 只投影该方法内的部分站点。系统 MUST 保持每个站点的 selector、case 映射、求值顺序、null 行为和命中分支副作用。

#### Scenario: Two independently mapped enum selectors

- **WHEN** 一个完整 Java 8 方法先对 `Count` enum switch、再对 `Animal` enum switch，且两张 helper table 的映射都分别得到证明
- **THEN** 完整 class source SHALL 输出两个 enum selector 及其映射标签，完整源码族 SHALL 可由 `javac --release 8` 编译，并在 `java -Xverify:all` 下对所有测试值和 null selector 与原 class 行为一致

#### Scenario: Closed shared helper initialization

- **WHEN** 一个 helper `<clinit>` 连续初始化同一方法候选引用的多张 enum 表，每个初始化组均符合已证明的 javac remap-array 形状
- **THEN** 一次完整联合扫描 SHALL 为每张表分别生成映射，且所有物理指令、独立 handler 与唯一 return 均恰好归属一个组

#### Scenario: One site in the method cannot be proved

- **WHEN** 一个方法有多个 enum switch 站点，但其中一张 table 或 enum 映射缺失、歧义或未获证明
- **THEN** 整个方法 SHALL 保留整数 table selector 和整数 case 标签，且证明报告 SHALL 保留该站点的拒绝原因

#### Scenario: Shared helper has an unselected table or unknown operation

- **WHEN** 同一 helper `<clinit>` 还初始化候选未选的表，或包含不能归属任一选中表的操作
- **THEN** 联合证明 SHALL 拒绝整个同方法投影，且不得根据字段名、声明顺序或遗漏操作推断映射

#### Scenario: Grouped emission stops

- **WHEN** grouped AST 匹配、输出预算或取消在检查部分站点后停止
- **THEN** 方法 MUST NOT 发布任何 enum 标签编辑，且每个候选的证明或停止结果 SHALL 可审计
