## ADDED Requirements

### Requirement: Proved user static suffix of an enum initializer

当 Java 8 enum 的完整常量构造与 `$VALUES` 前缀后带有用户 `<clinit>` 后缀时，系统 SHALL 只在常量组、静态字段初始化及全部后缀语句的顺序和来源都被同次证据完整证明后，输出可编译的 enum 常量声明及等价静态初始化代码。首个有界切片覆盖：一个已准确声明的 Map 字段由无参构造建立，随后对 `values()` 的增强 for-each 按顺序把每个常量的 `name()` 和该常量实例写入 Map。源恢复 MUST 保持 enum 实例初始化、Map 创建、遍历和每次写入的顺序与次数；未解释指令、未知副作用、控制流/表不完整或预算停止时 MUST 原子拒绝整个 enum 组投影，并保留物理字段与 `<clinit>` 来源。

#### Scenario: A proved map suffix follows the enum constant prefix
- **WHEN** 完整 enum 前缀后出现本要求规定的 Map 创建和 `values()` 增强 for-each 写入，且原 class 每项常量恰写一次
- **THEN** 完整输出 SHALL 以正常 enum 常量语法声明常量，在正确初始化位置建立 Map 并执行循环；Java 8 consumer 编译、`-Xverify:all` 运行后 Map 大小及每个键到原 enum singleton 的引用都与原 class 相同

#### Scenario: An unknown or incomplete suffix refuses the whole enum projection
- **WHEN** suffix 含本切片以外的调用/写入、不同 loop 形状、额外入口/汇合、不能追溯的数组值或不完整 Code/AST/预算
- **THEN** 系统 MUST NOT 去掉后缀或仅隐藏 enum 字段来伪造 enum 常量；整个 enum 投影保持拒绝，物理常量字段、原始 `<clinit>` 和拒绝/停止事实 SHALL 可查

#### Scenario: A standard enum prefix remains projected
- **WHEN** enum 完整 initializer 仅有标准常量与 `$VALUES` 设置并正常结束
- **THEN** 输出 SHALL 保留常量顺序且不制造用户 static block；现有无参数和已支持构造参数 enum 的行为保持不变
