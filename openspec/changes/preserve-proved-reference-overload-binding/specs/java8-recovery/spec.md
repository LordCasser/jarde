## ADDED Requirements

### Requirement: Proved reference invocation keeps overload binding

Java 8 调用的实参呈现引用类型与目标参数类型不同、但已证明安全上溯时，系统 SHALL 生成可重编译的实参表达式，并 SHALL 使 Java 源级重载选择与原 Methodref 指向的声明一致。转换 SHALL 只求值实参一次，保留实参与调用的来源。仅有目标 descriptor 而无类型关系证据时，系统 MUST 保留可定位的拒绝，MUST NOT 发明可能引入运行时检查的 cast。

#### Scenario: Known collection upcast selects List overload

- **WHEN** Java 8 类同时有 `call(List<String>)` 与 `call(ArrayList<String>)`，字节码将呈现为 `ArrayList` 的新对象传给前者，且所选运行环境证明 `ArrayList` 是 `List` 的子类型
- **THEN** 完整类源码 SHALL 在 Java 8 下重编译；运行 SHALL 选择 `List` 方法并输出其标记，而非更具体的 `ArrayList` 方法；原实参只构造一次

#### Scenario: Inherited overloads keep their selected declaration

- **WHEN** 同名重载分布在已证明的父类和子类，`ArrayList` 实参的物理调用指向父类的 `List` 声明
- **THEN** 完整源码 SHALL 保留父类声明的源级绑定；重编后的调用与原 class 验证运行结果 SHALL 一致

#### Scenario: Unknown hierarchy does not authorize a cast

- **WHEN** 实参类型和目标引用类型的继承关系在选定输入及受约束的平台事实中均不可证明，或当前可见的同名候选无法确定唯一源级目标
- **THEN** 系统 MUST 拒绝该调用并保留实参生产者、调用 BCI 与拒绝原因；MUST NOT 仅凭 Methodref 的目标参数类型断言安全转换

#### Scenario: Existing null and array choices remain stable

- **WHEN** 同类含 `String`、`List`、`ArrayList` 的 `null` 重载调用，以及 `int[][][]` 到 `Object[][]` 与 `int[][]` 的数组重载调用
- **THEN** 完整源码 SHALL 可重编译，所有输出 SHALL 与原 class 及固定 JADX 的验证运行一致；已证明的 cast SHALL 继续固定各自目标
