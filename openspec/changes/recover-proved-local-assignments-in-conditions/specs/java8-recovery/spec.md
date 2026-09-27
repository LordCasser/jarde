## ADDED Requirements

### Requirement: 条件中的已证局部赋值保留执行位置和值

当一个局部变量赋值的结果继续参加同一路条件测试，且物理定义使用、目标类型、作用域与短路路径均已证明时，系统 SHALL 在条件原执行位置恢复可编译的局部赋值表达式，MUST 对右值只求值一次，并保留该局部变量在后续路径的读取。证明不完整时系统 MUST 保留物理引用和拒绝理由，MUST NOT 将赋值无条件提前或把右值重复求值。

#### Scenario: 调用结果赋给局部再比较长度

- **WHEN** `InnerAssignCases.lengthBranch` 仅在第一个短路条件为假时执行 `text.length()`，把其值赋给 `length` 再与 5 比较，并在未早退时返回 `length`
- **THEN** Jarde SHALL 恢复一次条件内赋值和可编译的完整 Java 8 源码；原 class、固定 JADX 与 Jarde 的完整类 SHALL 在 `java -Xverify:all` 下保持七行输出一致

#### Scenario: 字段读取赋给局部再判空和调用

- **WHEN** `InnerAssignCases.assignedAndChecked` 先执行可能写字段的 `call`，只有其返回 false 才读取当前 `field`、赋给局部并对该局部判空与调用 `isEmpty`
- **THEN** Jarde SHALL 保留调用/字段读取/局部赋值的原短路次序，字段值不重复读取，完整类 SHALL 通过 Java 8 重编且三方七行运行结果一致

#### Scenario: 值、作用域或路径证据不闭合

- **WHEN** 复制值的两个用途不分别是一次局部写入及同路径测试、存在额外读取或写入、目标类型/局部作用域不明、赋值跨异常边或请求预算/取消停止
- **THEN** 系统 MUST 原子拒绝受影响的方法并保留 BCI 来源和原因，MUST NOT 发布看似完整但改变副作用、短路次序或局部值的源码
