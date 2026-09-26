## ADDED Requirements

### Requirement: Proved anonymous superclass constructor arguments

类源码视图 SHALL 在无捕获匿名子类的准确物理身份、唯一直接返回分配点、完整构造过程、父类构造器目标以及全部待呈现方法正文均得到证明时，把使用点写成带完整匿名体的 `new Base(args...) { ... }`。源级实参 MUST 与物理调用点一一对应，并保持原有类型、从左到右求值、副作用、异常顺序和选中的父类构造器重载。物理子类及其方法的独立报告 MUST 仍可查询。证明不完整、身份有第二用途或预算/取消停止时 MUST 保留物理构造形式，不得发布半个匿名体或提前改变父类/实参拼写。

#### Scenario: Ordered effects and selected overload

- **WHEN** 一个直接返回的匿名类只在一个字节码位置创建，调用点依次求值两个有副作用的 `int` 实参，唯一无字段写入的子类构造器按同序把它们转发给准确的父类 `(II)V` 构造器，父类另有 `(IJ)V` 重载，匿名体方法均完整
- **THEN** 使用点呈现 `new Base(next(), next()) { ... }`，不再调用物理 `$1` 构造器；Java 8 重编后使用 `(II)V` 对应行为并与原 class、JADX 的执行输出同为 `13:2`

#### Scenario: Capture or constructor effect is not a source argument

- **WHEN** 匿名构造器的任一参数用于捕获字段写入、外层实例保存或其他效果，或者不能证明每个调用点参数准确且按序进入所选父类构造器
- **THEN** 使用点保持物理构造形式，不得把捕获值误写成父类构造实参，也不得移动构造器效果

#### Scenario: Physical identity has another use

- **WHEN** 同一物理匿名子类在另一 BCI 再次分配、被所选输入的另一类引用，或这些使用无法完整排除
- **THEN** 全部相关使用点保持物理构造形式，不得用多个源码匿名表达式改变运行时类身份

#### Scenario: Body, target, or resource proof stops

- **WHEN** 父类构造目标/访问性不明、匿名体任一方法回退或不完整、所选范围扫描不完整，或者恢复与输出预算/取消停止
- **THEN** 根类源码保持原物理构造文本，物理子类报告继续独立可查，且不发布部分匿名体
