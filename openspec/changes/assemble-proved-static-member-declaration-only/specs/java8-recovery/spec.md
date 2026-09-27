## ADDED Requirements

### Requirement: Proven declaration-only static abstract member

当 Java 8 根类与唯一直接静态抽象成员类在选定制品内具有完整、准确的双向成员关系和可呈现的物理声明/方法事实时，系统 SHALL 即使根类没有构造该成员的字节码站点，也在同一根源码中输出该嵌套类声明。无 Code 的抽象方法 SHALL 保持分号声明，构造器 SHALL 使用成员简单名；原 root/child 物理事实和派生来源 SHALL 保留。关系、正文或预算证明不完整时，系统 SHALL 不发布半份嵌套源码。

#### Scenario: Single abstract static member without allocation

- **WHEN** Java 8 根类仅声明一个直接 `public static abstract class A`，成员无字段，有准确默认构造器及一个无 Code 的抽象方法，根类没有对 `A` 的构造调用
- **THEN** 根类源码包含 `A` 及其抽象方法，和同一 Runner 一起 Java 8 重编、验证运行得到与原 class 相同结果

#### Scenario: Existing construction-bearing static member

- **WHEN** 根类的唯一静态成员原已通过构造使用点证明并恢复
- **THEN** 该源码和已有来源保持原行为，不因声明型路径丢失构造表达式或重复成员声明

#### Scenario: Unsupported family or interrupted proof

- **WHEN** 根类有第二成员、双方嵌套属性冲突、选定定义缺失/歧义、child 有字段或泛型 Signature、成员恢复不完整、预算耗尽或请求取消
- **THEN** 系统 SHALL 保留可查询的物理报告与既有拒绝/停止结果，且不把未经闭合的成员声明投影为完整根源码
